use std::{
    net::{Ipv4Addr, SocketAddr, TcpListener},
    time::{Duration, SystemTime},
};

use anyhow::{Context as _, Result, anyhow};
use ed25519_dalek::VerifyingKey;
use omarchygs_provider_sdk::{
    ProviderScope,
    protocol::{
        GrantIssuer, HttpMessageSigner, ProviderCompatibility, ProviderCompatibilityOffer,
        ProviderCompatibilitySelection, ProviderGrantClaims, ProviderOperationDisposition,
        ProviderOperationKind, ProviderOperationRequest, ProviderOperationResponse,
        ProviderSessionStatus, RequestSignatureContext, SignatureHeaders, parse_authenticated_json,
        validate_provider_payload, verify_response_signature,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use url::Url;
use uuid::Uuid;

use crate::CallbackSink;

const BASE_PATH: &str = "/omarchygs/provider/v1/";
const BODY_LIMIT: usize = 65_536;
const MAX_CONTINUATION_COMMANDS: usize = 64;

/// Complete, sorted v1 case inventory. Adding or omitting a case changes the
/// public receipt contract.
pub const REQUIRED_CASES: [&str; 15] = [
    "callback_deduplication",
    "callback_recovery",
    "changed_intent",
    "compatibility",
    "context_mismatch",
    "digest_mismatch",
    "malformed_input",
    "outage_recovery",
    "oversized_input",
    "reconcile",
    "request_replay",
    "signature_mismatch",
    "stale_revision",
    "timeout_unknown_outcome",
    "valid_flow",
];

/// Bounded, game-specific payloads used by the otherwise fixed conformance
/// corpus. This profile changes no transport, authentication, replay, fault,
/// callback, or receipt assertion.
#[derive(Debug, Clone, PartialEq)]
pub struct ConformanceGameplayProfile {
    launch_payload: Value,
    timeout_command_payload: Value,
    continuation_command_payloads: Vec<Value>,
    final_status: ProviderSessionStatus,
}

impl ConformanceGameplayProfile {
    /// Create a persistent or terminal gameplay path for the fixed corpus.
    /// The timeout command must leave the session active so the continuation
    /// commands can execute, and at least one continuation command is required.
    pub fn new(
        launch_payload: Value,
        timeout_command_payload: Value,
        continuation_command_payloads: Vec<Value>,
        final_status: ProviderSessionStatus,
    ) -> Result<Self> {
        let profile = Self {
            launch_payload,
            timeout_command_payload,
            continuation_command_payloads,
            final_status,
        };
        profile.validate()?;
        Ok(profile)
    }

    fn relay_forge() -> Self {
        Self {
            launch_payload: json!({"player_count": 1}),
            timeout_command_payload: json!({"command": {"action": "mine"}}),
            continuation_command_payloads: ["mine", "charge", "forge"]
                .into_iter()
                .map(|action| json!({"command": {"action": action}}))
                .collect(),
            final_status: ProviderSessionStatus::Completed,
        }
    }

    fn validate(&self) -> Result<()> {
        if self.continuation_command_payloads.is_empty()
            || self.continuation_command_payloads.len() > MAX_CONTINUATION_COMMANDS
        {
            return Err(anyhow!(
                "conformance gameplay continuation is out of bounds"
            ));
        }
        for payload in std::iter::once(&self.launch_payload)
            .chain(std::iter::once(&self.timeout_command_payload))
            .chain(self.continuation_command_payloads.iter())
        {
            validate_provider_payload(payload)
                .map_err(|_| anyhow!("conformance gameplay payload rejected"))?;
            if serde_json::to_vec(payload)
                .context("encode conformance gameplay payload")?
                .len()
                > BODY_LIMIT
            {
                return Err(anyhow!("conformance gameplay payload exceeded bound"));
            }
        }
        Ok(())
    }
}

/// One exact local target and ephemeral platform test authority.
pub struct ConformanceTarget {
    endpoint: Url,
    socket_override: SocketAddr,
    authority: String,
    root_der: Vec<u8>,
    provider_id: String,
    release_id: Uuid,
    game_key: String,
    rules_version: u32,
    cartridge_digest: String,
    subject: String,
    provider_message_key_id: String,
    provider_message_key: VerifyingKey,
    grant_issuer: GrantIssuer,
    platform_signer: HttpMessageSigner,
    normal_timeout: Duration,
    unknown_outcome_timeout: Duration,
    gameplay: ConformanceGameplayProfile,
}

impl ConformanceTarget {
    /// Construct a loopback-only target. Seeds create local test authority;
    /// they are never serialized into the receipt.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        endpoint: Url,
        socket_override: SocketAddr,
        authority: String,
        root_der: Vec<u8>,
        provider_id: String,
        release_id: Uuid,
        game_key: String,
        rules_version: u32,
        cartridge_digest: String,
        subject: String,
        provider_message_key_id: String,
        provider_message_key: VerifyingKey,
        platform_grant_key_id: &str,
        platform_grant_seed: [u8; 32],
        pairwise_secret: Vec<u8>,
        platform_message_key_id: &str,
        platform_message_seed: [u8; 32],
        normal_timeout: Duration,
        unknown_outcome_timeout: Duration,
    ) -> Result<Self> {
        if endpoint.scheme() != "https"
            || endpoint.username() != ""
            || endpoint.password().is_some()
            || endpoint.path() != BASE_PATH
            || endpoint.query().is_some()
            || endpoint.fragment().is_some()
            || release_id.is_nil()
            || !(64..=4096).contains(&root_der.len())
            || normal_timeout <= unknown_outcome_timeout
            || unknown_outcome_timeout < Duration::from_millis(10)
        {
            return Err(anyhow!(
                "conformance target requires exact loopback TLS identity"
            ));
        }
        validate_exact_transport(&endpoint, socket_override, Some(&authority))?;
        Ok(Self {
            endpoint,
            socket_override,
            authority,
            root_der,
            provider_id,
            release_id,
            game_key,
            rules_version,
            cartridge_digest,
            subject,
            provider_message_key_id,
            provider_message_key,
            grant_issuer: GrantIssuer::new(
                platform_grant_key_id,
                platform_grant_seed,
                pairwise_secret,
            )
            .map_err(|_| anyhow!("platform grant test authority rejected"))?,
            platform_signer: HttpMessageSigner::new(platform_message_key_id, platform_message_seed)
                .map_err(|_| anyhow!("platform message test authority rejected"))?,
            normal_timeout,
            unknown_outcome_timeout,
            gameplay: ConformanceGameplayProfile::relay_forge(),
        })
    }

    /// Replace only the bounded game payload path. The default remains the
    /// original Relay Forge sequence for backward compatibility.
    pub fn with_gameplay_profile(mut self, gameplay: ConformanceGameplayProfile) -> Result<Self> {
        gameplay.validate()?;
        self.gameplay = gameplay;
        Ok(self)
    }

    #[must_use]
    pub fn platform_grant_key(&self) -> VerifyingKey {
        self.grant_issuer.verifying_key()
    }

    #[must_use]
    pub fn platform_message_key(&self) -> VerifyingKey {
        self.platform_signer.verifying_key()
    }

    /// Public provider response key used by the paired local callback sink.
    #[must_use]
    pub fn provider_message_key(&self) -> VerifyingKey {
        self.provider_message_key
    }
}

/// One required case. Receipts intentionally retain no endpoint, payload,
/// subject, key, body, or timing detail.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ConformanceCase {
    pub id: String,
    pub passed: bool,
}

/// Canonical bounded conformance result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ConformanceReceipt {
    pub format: String,
    pub sdk_version: u32,
    pub provider_id: String,
    pub release_id: Uuid,
    pub cases: Vec<ConformanceCase>,
}

impl ConformanceReceipt {
    pub fn validate(&self) -> Result<()> {
        let ids = self
            .cases
            .iter()
            .map(|case| case.id.as_str())
            .collect::<Vec<_>>();
        if self.format != "omarchygs.provider-conformance-receipt/v1"
            || self.sdk_version != 1
            || self.provider_id.is_empty()
            || self.release_id.is_nil()
            || ids != REQUIRED_CASES
            || self.cases.iter().any(|case| !case.passed)
        {
            return Err(anyhow!("conformance receipt is incomplete"));
        }
        Ok(())
    }
}

struct PreparedOperation {
    request: ProviderOperationRequest,
    body: Vec<u8>,
    headers: http::HeaderMap,
    path: String,
}

struct WireResponse {
    status: reqwest::StatusCode,
    headers: http::HeaderMap,
    body: Vec<u8>,
}

/// Execute the entire finite public corpus against one live provider and one
/// authenticated callback sink.
pub async fn run_conformance(
    target: &ConformanceTarget,
    callback: &CallbackSink,
) -> Result<ConformanceReceipt> {
    let client = exact_client(
        &target.endpoint,
        target.socket_override,
        &target.root_der,
        target.normal_timeout,
    )?;
    compatibility(target, &client).await?;

    let session_id = Uuid::new_v4();
    let launch_id = Uuid::new_v4();
    let launch = prepare_operation(
        target,
        session_id,
        launch_id,
        0,
        ProviderOperationKind::Launch,
        target.gameplay.launch_payload.clone(),
    )?;
    let launched = send_operation(target, &client, &launch, target.normal_timeout).await?;
    require_state(
        &launched,
        0,
        ProviderOperationDisposition::Applied,
        ProviderSessionStatus::Active,
    )?;
    let replay = send_operation(target, &client, &launch, target.normal_timeout).await?;
    if replay != launched {
        return Err(anyhow!(
            "exact request replay did not return stable response bytes"
        ));
    }

    let changed = prepare_operation(
        target,
        session_id,
        launch_id,
        0,
        ProviderOperationKind::Launch,
        json!({"conformance_changed_intent": true}),
    )?;
    expect_status(
        &post(
            target,
            &client,
            &changed.path,
            changed.headers.clone(),
            changed.body.clone(),
            target.normal_timeout,
        )
        .await?,
        reqwest::StatusCode::CONFLICT,
        "changed intent",
    )?;

    let stale = prepare_operation(
        target,
        session_id,
        Uuid::new_v4(),
        9,
        ProviderOperationKind::Command,
        target.gameplay.timeout_command_payload.clone(),
    )?;
    let stale_response = send_operation(target, &client, &stale, target.normal_timeout).await?;
    require_state(
        &stale_response,
        0,
        ProviderOperationDisposition::RevisionConflict,
        ProviderSessionStatus::Active,
    )?;

    authentication_faults(target, &client).await?;
    outage_and_recovery(target).await?;

    let timeout_command = prepare_operation(
        target,
        session_id,
        Uuid::new_v4(),
        0,
        ProviderOperationKind::Command,
        target.gameplay.timeout_command_payload.clone(),
    )?;
    if post(
        target,
        &client,
        &timeout_command.path,
        timeout_command.headers.clone(),
        timeout_command.body.clone(),
        target.unknown_outcome_timeout,
    )
    .await
    .is_ok()
    {
        return Err(anyhow!(
            "commit-timeout case unexpectedly returned before deadline"
        ));
    }
    let recovered =
        send_operation(target, &client, &timeout_command, target.normal_timeout).await?;
    require_state(
        &recovered,
        1,
        ProviderOperationDisposition::Applied,
        ProviderSessionStatus::Active,
    )?;
    let stable = send_operation(target, &client, &timeout_command, target.normal_timeout).await?;
    if stable != recovered {
        return Err(anyhow!(
            "unknown-outcome retry did not resolve stable receipt"
        ));
    }

    let mut revision = 1;
    let continuation_count = target.gameplay.continuation_command_payloads.len();
    for (index, payload) in target
        .gameplay
        .continuation_command_payloads
        .iter()
        .enumerate()
    {
        let operation = prepare_operation(
            target,
            session_id,
            Uuid::new_v4(),
            revision,
            ProviderOperationKind::Command,
            payload.clone(),
        )?;
        let response = send_operation(target, &client, &operation, target.normal_timeout).await?;
        revision += 1;
        require_state(
            &response,
            revision,
            ProviderOperationDisposition::Applied,
            if index + 1 == continuation_count {
                target.gameplay.final_status
            } else {
                ProviderSessionStatus::Active
            },
        )?;
    }

    let observed = callback
        .wait_for_duplicate(session_id, revision, Duration::from_secs(8))
        .await?;
    if observed.deliveries < 2 {
        return Err(anyhow!("callback retry/deduplication facts mismatch"));
    }

    let reconcile = prepare_operation(
        target,
        session_id,
        Uuid::new_v4(),
        revision,
        ProviderOperationKind::Reconcile,
        json!({}),
    )?;
    let reconciled = send_operation(target, &client, &reconcile, target.normal_timeout).await?;
    require_state(
        &reconciled,
        revision,
        ProviderOperationDisposition::Applied,
        target.gameplay.final_status,
    )?;

    let receipt = ConformanceReceipt {
        format: "omarchygs.provider-conformance-receipt/v1".to_owned(),
        sdk_version: 1,
        provider_id: target.provider_id.clone(),
        release_id: target.release_id,
        cases: REQUIRED_CASES
            .into_iter()
            .map(|id| ConformanceCase {
                id: id.to_owned(),
                passed: true,
            })
            .collect(),
    };
    receipt.validate()?;
    Ok(receipt)
}

async fn compatibility(target: &ConformanceTarget, client: &reqwest::Client) -> Result<()> {
    let message_id = Uuid::new_v4();
    let offer = ProviderCompatibilityOffer::current(
        target.provider_id.clone(),
        target.release_id,
        message_id,
    )
    .map_err(|_| anyhow!("build compatibility offer"))?;
    let body = offer
        .to_bytes(BODY_LIMIT)
        .map_err(|_| anyhow!("encode compatibility offer"))?;
    let path = format!("{BASE_PATH}compatibility");
    let context = context(target, &path, message_id);
    let headers = target
        .platform_signer
        .sign_request(
            &context,
            &body,
            unix_seconds()?,
            &format!("conformance-{message_id}"),
        )
        .map_err(|_| anyhow!("sign compatibility offer"))?
        .to_header_map()
        .map_err(|_| anyhow!("compatibility headers"))?;
    let wire = post(target, client, &path, headers, body, target.normal_timeout).await?;
    expect_status(&wire, reqwest::StatusCode::OK, "compatibility")?;
    let response_id = response_message_id(&wire.headers)?;
    verify_response_signature(
        &SignatureHeaders::from_header_map(&wire.headers)
            .map_err(|_| anyhow!("compatibility response headers"))?,
        wire.status.as_u16(),
        &context,
        response_id,
        &wire.body,
        &target.provider_message_key,
        &target.provider_message_key_id,
        unix_seconds()?,
    )
    .map_err(|_| anyhow!("compatibility response signature rejected"))?;
    let selection: ProviderCompatibilitySelection =
        parse_authenticated_json(&wire.body, BODY_LIMIT)
            .map_err(|_| anyhow!("compatibility response rejected"))?;
    selection
        .validate_for(&offer)
        .map_err(|_| anyhow!("compatibility selection mismatch"))
}

async fn authentication_faults(target: &ConformanceTarget, client: &reqwest::Client) -> Result<()> {
    let session = Uuid::new_v4();
    let valid = prepare_operation(
        target,
        session,
        Uuid::new_v4(),
        0,
        ProviderOperationKind::Launch,
        target.gameplay.launch_payload.clone(),
    )?;

    let wrong = HttpMessageSigner::new("wrong-message-key", [99; 32])
        .map_err(|_| anyhow!("wrong signer"))?;
    let wrong_headers = wrong
        .sign_request(
            &context(target, &valid.path, valid.request.message_id),
            &valid.body,
            unix_seconds()?,
            &format!("wrong-signature-{}", Uuid::new_v4()),
        )
        .map_err(|_| anyhow!("wrong signature"))?
        .to_header_map()
        .map_err(|_| anyhow!("wrong headers"))?;
    expect_status(
        &post(
            target,
            client,
            &valid.path,
            wrong_headers,
            valid.body.clone(),
            target.normal_timeout,
        )
        .await?,
        reqwest::StatusCode::UNAUTHORIZED,
        "signature mismatch",
    )?;

    let mut changed_body = valid.body.clone();
    changed_body.push(b' ');
    expect_status(
        &post(
            target,
            client,
            &valid.path,
            valid.headers.clone(),
            changed_body,
            target.normal_timeout,
        )
        .await?,
        reqwest::StatusCode::UNAUTHORIZED,
        "digest mismatch",
    )?;

    let wrong_context = RequestSignatureContext {
        method: "POST",
        authority: &target.authority,
        path: "/omarchygs/provider/v1/reconcile",
        provider_id: &target.provider_id,
        release_id: target.release_id,
        message_id: valid.request.message_id,
    };
    let wrong_context_headers = target
        .platform_signer
        .sign_request(
            &wrong_context,
            &valid.body,
            unix_seconds()?,
            &format!("wrong-context-{}", Uuid::new_v4()),
        )
        .map_err(|_| anyhow!("wrong context signature"))?
        .to_header_map()
        .map_err(|_| anyhow!("wrong context headers"))?;
    expect_status(
        &post(
            target,
            client,
            &valid.path,
            wrong_context_headers,
            valid.body.clone(),
            target.normal_timeout,
        )
        .await?,
        reqwest::StatusCode::UNAUTHORIZED,
        "context mismatch",
    )?;

    let malformed = b"{".to_vec();
    let malformed_id = Uuid::new_v4();
    let malformed_headers = target
        .platform_signer
        .sign_request(
            &context(target, &valid.path, malformed_id),
            &malformed,
            unix_seconds()?,
            &format!("malformed-{malformed_id}"),
        )
        .map_err(|_| anyhow!("malformed signature"))?
        .to_header_map()
        .map_err(|_| anyhow!("malformed headers"))?;
    expect_status(
        &post(
            target,
            client,
            &valid.path,
            malformed_headers,
            malformed,
            target.normal_timeout,
        )
        .await?,
        reqwest::StatusCode::UNAUTHORIZED,
        "malformed input",
    )?;

    let oversized = vec![b'x'; BODY_LIMIT + 1];
    let oversized_id = Uuid::new_v4();
    let oversized_headers = target
        .platform_signer
        .sign_request(
            &context(target, &valid.path, oversized_id),
            &oversized,
            unix_seconds()?,
            &format!("oversized-{oversized_id}"),
        )
        .map_err(|_| anyhow!("oversized signature"))?
        .to_header_map()
        .map_err(|_| anyhow!("oversized headers"))?;
    expect_status(
        &post(
            target,
            client,
            &valid.path,
            oversized_headers,
            oversized,
            target.normal_timeout,
        )
        .await?,
        reqwest::StatusCode::PAYLOAD_TOO_LARGE,
        "oversized input",
    )
}

async fn outage_and_recovery(target: &ConformanceTarget) -> Result<()> {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).context("reserve outage socket")?;
    let outage = listener.local_addr().context("outage socket")?;
    drop(listener);
    let mut outage_endpoint = target.endpoint.clone();
    outage_endpoint
        .set_port(Some(outage.port()))
        .map_err(|()| anyhow!("set exact outage port"))?;
    let unavailable = exact_client(
        &outage_endpoint,
        outage,
        &target.root_der,
        Duration::from_millis(150),
    )?;
    let outage_url = outage_endpoint
        .join("compatibility")
        .context("build outage URL")?;
    if unavailable
        .post(outage_url)
        .timeout(Duration::from_millis(150))
        .body(b"{}".to_vec())
        .send()
        .await
        .is_ok()
    {
        return Err(anyhow!("transport outage unexpectedly succeeded"));
    }
    let recovered = exact_client(
        &target.endpoint,
        target.socket_override,
        &target.root_der,
        target.normal_timeout,
    )?;
    compatibility(target, &recovered)
        .await
        .context("recovery after transport outage")
}

fn prepare_operation(
    target: &ConformanceTarget,
    platform_session_id: Uuid,
    idempotency_key: Uuid,
    expected_revision: u64,
    operation: ProviderOperationKind,
    payload: Value,
) -> Result<PreparedOperation> {
    let now = unix_seconds()?;
    let message_id = Uuid::new_v4();
    let claims = ProviderGrantClaims::new(
        target.provider_id.clone(),
        target.release_id,
        target.game_key.clone(),
        target.rules_version,
        target.cartridge_digest.clone(),
        platform_session_id,
        target.subject.clone(),
        scope(operation),
        ProviderCompatibility::current(),
        now,
        now + 60,
        Uuid::new_v4(),
    )
    .map_err(|_| anyhow!("build operation grant"))?;
    let grant = target
        .grant_issuer
        .sign(&claims)
        .map_err(|_| anyhow!("sign operation grant"))?;
    let request = ProviderOperationRequest::new(
        target.provider_id.clone(),
        target.release_id,
        target.game_key.clone(),
        target.rules_version,
        target.cartridge_digest.clone(),
        platform_session_id,
        target.subject.clone(),
        message_id,
        idempotency_key,
        expected_revision,
        operation,
        ProviderCompatibility::current(),
        payload,
        grant,
    )
    .map_err(|_| anyhow!("build provider operation"))?;
    let body = request
        .to_bytes(BODY_LIMIT)
        .map_err(|_| anyhow!("encode provider operation"))?;
    let path = format!("{BASE_PATH}{}", operation.path());
    let headers = target
        .platform_signer
        .sign_request(
            &context(target, &path, message_id),
            &body,
            now,
            &format!("conformance-{message_id}"),
        )
        .map_err(|_| anyhow!("sign provider operation"))?
        .to_header_map()
        .map_err(|_| anyhow!("provider operation headers"))?;
    Ok(PreparedOperation {
        request,
        body,
        headers,
        path,
    })
}

async fn send_operation(
    target: &ConformanceTarget,
    client: &reqwest::Client,
    operation: &PreparedOperation,
    timeout: Duration,
) -> Result<Vec<u8>> {
    let wire = post(
        target,
        client,
        &operation.path,
        operation.headers.clone(),
        operation.body.clone(),
        timeout,
    )
    .await?;
    expect_status(&wire, reqwest::StatusCode::OK, "provider operation")?;
    let message_id = response_message_id(&wire.headers)?;
    let request_context = context(target, &operation.path, operation.request.message_id);
    verify_response_signature(
        &SignatureHeaders::from_header_map(&wire.headers)
            .map_err(|_| anyhow!("response headers rejected"))?,
        wire.status.as_u16(),
        &request_context,
        message_id,
        &wire.body,
        &target.provider_message_key,
        &target.provider_message_key_id,
        unix_seconds()?,
    )
    .map_err(|_| anyhow!("response signature rejected"))?;
    let response: ProviderOperationResponse = parse_authenticated_json(&wire.body, BODY_LIMIT)
        .map_err(|_| anyhow!("response body rejected"))?;
    response
        .validate_for(&operation.request)
        .map_err(|_| anyhow!("response context rejected"))?;
    Ok(wire.body)
}

fn require_state(
    body: &[u8],
    revision: u64,
    disposition: ProviderOperationDisposition,
    status: ProviderSessionStatus,
) -> Result<()> {
    let response: ProviderOperationResponse = parse_authenticated_json(body, BODY_LIMIT)
        .map_err(|_| anyhow!("response body rejected"))?;
    if response.revision == revision
        && response.disposition == disposition
        && response.status == status
    {
        Ok(())
    } else {
        Err(anyhow!("provider state response mismatch"))
    }
}

fn scope(operation: ProviderOperationKind) -> ProviderScope {
    operation.scope()
}

fn context<'a>(
    target: &'a ConformanceTarget,
    path: &'a str,
    message_id: Uuid,
) -> RequestSignatureContext<'a> {
    RequestSignatureContext {
        method: "POST",
        authority: &target.authority,
        path,
        provider_id: &target.provider_id,
        release_id: target.release_id,
        message_id,
    }
}

fn exact_client(
    endpoint: &Url,
    socket: SocketAddr,
    root_der: &[u8],
    timeout: Duration,
) -> Result<reqwest::Client> {
    validate_exact_transport(endpoint, socket, None)?;
    let host = endpoint
        .host_str()
        .ok_or_else(|| anyhow!("endpoint host missing"))?;
    let root = reqwest::Certificate::from_der(root_der).context("provider TLS root DER")?;
    reqwest::Client::builder()
        .https_only(true)
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .tls_certs_only([root])
        .resolve(host, socket)
        .connect_timeout(timeout)
        .timeout(timeout)
        .build()
        .context("build exact conformance client")
}

fn validate_exact_transport(
    endpoint: &Url,
    socket: SocketAddr,
    expected_authority: Option<&str>,
) -> Result<()> {
    let Some(url::Host::Domain(host)) = endpoint.host() else {
        return Err(anyhow!("conformance endpoint requires a DNS hostname"));
    };
    let Some(port) = endpoint.port_or_known_default() else {
        return Err(anyhow!("conformance endpoint requires an exact port"));
    };
    let authority = if port == 443 {
        host.to_owned()
    } else {
        format!("{host}:{port}")
    };
    if !socket.ip().is_loopback()
        || socket.port() != port
        || expected_authority.is_some_and(|expected| expected != authority)
    {
        return Err(anyhow!(
            "conformance endpoint, authority, and loopback socket must match exactly"
        ));
    }
    Ok(())
}

async fn post(
    target: &ConformanceTarget,
    client: &reqwest::Client,
    path: &str,
    headers: http::HeaderMap,
    body: Vec<u8>,
    timeout: Duration,
) -> Result<WireResponse> {
    let url = target
        .endpoint
        .join(path.trim_start_matches(BASE_PATH))
        .context("build exact provider URL")?;
    let response = client
        .post(url)
        .headers(headers)
        .timeout(timeout)
        .body(body)
        .send()
        .await
        .context("provider transport unavailable")?;
    let status = response.status();
    let headers = response.headers().clone();
    let body = response
        .bytes()
        .await
        .context("read provider response")?
        .to_vec();
    if body.len() > BODY_LIMIT {
        return Err(anyhow!("provider response exceeded bound"));
    }
    Ok(WireResponse {
        status,
        headers,
        body,
    })
}

fn expect_status(
    response: &WireResponse,
    expected: reqwest::StatusCode,
    label: &str,
) -> Result<()> {
    if response.status == expected {
        Ok(())
    } else {
        Err(anyhow!(
            "{label} returned unexpected status {}",
            response.status.as_u16()
        ))
    }
}

fn response_message_id(headers: &http::HeaderMap) -> Result<Uuid> {
    SignatureHeaders::from_header_map(headers)
        .map_err(|_| anyhow!("response signature headers rejected"))?
        .message_id
        .parse()
        .map_err(|_| anyhow!("response message ID rejected"))
}

fn unix_seconds() -> Result<i64> {
    i64::try_from(
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .context("system clock before Unix epoch")?
            .as_secs(),
    )
    .context("system clock overflow")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn receipt_requires_exact_sorted_complete_inventory() {
        let mut receipt = ConformanceReceipt {
            format: "omarchygs.provider-conformance-receipt/v1".to_owned(),
            sdk_version: 1,
            provider_id: "relay-labs".to_owned(),
            release_id: Uuid::new_v4(),
            cases: REQUIRED_CASES
                .into_iter()
                .map(|id| ConformanceCase {
                    id: id.to_owned(),
                    passed: true,
                })
                .collect(),
        };
        receipt.validate().expect("complete receipt");
        receipt.cases.swap(0, 1);
        assert!(receipt.validate().is_err());
    }

    #[test]
    fn exact_transport_binds_domain_authority_and_socket_port() {
        let endpoint = Url::parse("https://provider.test:4443/omarchygs/provider/v1/")
            .expect("valid endpoint");
        let exact = SocketAddr::from((Ipv4Addr::LOCALHOST, 4443));
        validate_exact_transport(&endpoint, exact, Some("provider.test:4443"))
            .expect("exact target");

        assert!(
            validate_exact_transport(
                &endpoint,
                SocketAddr::from((Ipv4Addr::LOCALHOST, 4444)),
                Some("provider.test:4443"),
            )
            .is_err()
        );
        assert!(validate_exact_transport(&endpoint, exact, Some("other.test:4443")).is_err());

        let literal =
            Url::parse("https://127.0.0.1:4443/omarchygs/provider/v1/").expect("valid IP endpoint");
        assert!(validate_exact_transport(&literal, exact, Some("127.0.0.1:4443")).is_err());
    }

    #[test]
    fn gameplay_profile_accepts_a_persistent_bounded_game_day() {
        let profile = ConformanceGameplayProfile::new(
            json!({"command": {"action": "enter"}}),
            json!({"command": {"action": "status"}}),
            vec![
                json!({"command": {"action": "enter_dungeon"}}),
                json!({"command": {"action": "sleep"}}),
            ],
            ProviderSessionStatus::Active,
        )
        .expect("persistent game profile");

        assert_eq!(profile.final_status, ProviderSessionStatus::Active);
        assert_eq!(profile.continuation_command_payloads.len(), 2);
    }

    #[test]
    fn gameplay_profile_rejects_unbounded_or_sensitive_payloads() {
        assert!(
            ConformanceGameplayProfile::new(
                json!({"command": {"action": "enter"}}),
                json!({"command": {"action": "status"}}),
                Vec::new(),
                ProviderSessionStatus::Active,
            )
            .is_err()
        );
        assert!(
            ConformanceGameplayProfile::new(
                json!({"account_id": "forbidden"}),
                json!({"command": {"action": "status"}}),
                vec![json!({"command": {"action": "sleep"}})],
                ProviderSessionStatus::Active,
            )
            .is_err()
        );
        assert!(
            ConformanceGameplayProfile::new(
                json!({"command": {"action": "enter"}}),
                json!({"command": {"action": "status"}}),
                vec![json!({"command": {"action": "status"}}); MAX_CONTINUATION_COMMANDS + 1],
                ProviderSessionStatus::Active,
            )
            .is_err()
        );
    }
}
