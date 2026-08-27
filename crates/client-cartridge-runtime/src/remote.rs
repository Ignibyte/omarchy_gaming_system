use std::time::Duration;

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use futures_util::StreamExt as _;
use omarchygs_game_cartridge::{
    ACQUISITION_FORMAT, ACQUISITION_FORMAT_V2, AcquisitionServerAdmission, CartridgeAcquisition,
    CatalogPublicKey, MAX_ACQUISITION_DOCUMENT_BYTES, MAX_MARKETPLACE_SNAPSHOT_BYTES,
    OPERATOR_CUSTOM_WARNING, VerifiedAcquisition, VerifiedOperatorCustomAcquisition,
    operator_custom_key_sha256, rich_2d_host_profile, supported_sdk_identity,
    verify_acquisition_bytes_with_policy_key, verify_marketplace_snapshot_bytes,
    verify_operator_custom_acquisition_bytes,
};
use reqwest::{Client, Response, StatusCode, header};
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;
use zeroize::Zeroizing;

use crate::{
    ClientTrustSnapshot, CompanionError, MountRecord, OperatorCustomMountRecord,
    OperatorCustomTrust, Result,
};

const MAX_DISCOVERY_BYTES: usize = 16 * 1024;
const MAX_CATALOG_BYTES: usize = 256 * 1024;
const MAX_SESSION_BYTES: usize = 512 * 1024;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AcquireRequest {
    pub server_origin: String,
    pub server_id: String,
    pub device_bearer: String,
    pub game_key: String,
    pub archive_sha256: String,
    pub admission_revision: u64,
    #[serde(default)]
    pub provenance_class: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionAcquireRequest {
    pub server_origin: String,
    pub server_id: String,
    pub device_bearer: String,
    pub persona_id: String,
    pub game_session_id: String,
    #[serde(default)]
    pub provenance_class: Option<String>,
}

pub struct RemoteAcquisition {
    pub verified: VerifiedAcquisition,
    pub mount: MountRecord,
}

pub struct RemoteOperatorCustomAcquisition {
    pub verified: VerifiedOperatorCustomAcquisition,
    pub mount: OperatorCustomMountRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct OperatorCustomDiscovery {
    pub operator_name: String,
    pub authority_id: String,
    pub key_id: String,
    pub key_sha256: String,
    pub public_key: CatalogPublicKey,
}

pub trait AcquisitionTrust {
    fn authorize_key(&self, key: &CatalogPublicKey, snapshot_version: u64) -> Result<()>;
    fn authorize_current_policy_key(
        &self,
        key: &CatalogPublicKey,
        snapshot_version: u64,
    ) -> Result<()>;
}

impl AcquisitionTrust for CatalogPublicKey {
    fn authorize_key(&self, key: &CatalogPublicKey, snapshot_version: u64) -> Result<()> {
        if snapshot_version > 0 && self == key {
            Ok(())
        } else {
            Err(CompanionError::MarketplaceUntrusted)
        }
    }

    fn authorize_current_policy_key(
        &self,
        key: &CatalogPublicKey,
        snapshot_version: u64,
    ) -> Result<()> {
        self.authorize_key(key, snapshot_version)
    }
}

impl AcquisitionTrust for ClientTrustSnapshot {
    fn authorize_key(&self, key: &CatalogPublicKey, snapshot_version: u64) -> Result<()> {
        ClientTrustSnapshot::authorize_key(self, key, snapshot_version)
    }

    fn authorize_current_policy_key(
        &self,
        key: &CatalogPublicKey,
        snapshot_version: u64,
    ) -> Result<()> {
        self.authorize_current_key(key, snapshot_version)
    }
}

pub async fn acquire(
    mut request: AcquireRequest,
    trusted_marketplace: &impl AcquisitionTrust,
) -> Result<RemoteAcquisition> {
    let origin = selected_origin(&request.server_origin)?;
    let server_id = exact_uuid(&request.server_id)?;
    if !valid_identifier(&request.game_key)
        || !valid_sha256(&request.archive_sha256)
        || request.admission_revision == 0
        || !valid_bearer(&request.device_bearer)
        || request
            .provenance_class
            .as_deref()
            .is_some_and(|value| value != "marketplace_vetted")
    {
        return Err(CompanionError::InvalidInput);
    }
    let bearer = Zeroizing::new(std::mem::take(&mut request.device_bearer));
    let client = remote_client()?;
    let discovery_url = origin
        .join("/.well-known/omarchygs")
        .map_err(|_| CompanionError::InvalidInput)?;
    let discovery: DiscoveryDocument = get_json(
        &client,
        discovery_url,
        None,
        MAX_DISCOVERY_BYTES,
        CompanionError::Rejected,
    )
    .await?;
    if discovery.service != "omarchy-gaming-system"
        || discovery.server_id != server_id.to_string()
        || discovery.protocol_version != 1
        || !sorted_capabilities(&discovery.capabilities)
        || !discovery
            .capabilities
            .iter()
            .any(|value| value == "games.cartridge-acquisition.v1")
    {
        return Err(CompanionError::Rejected);
    }
    let initial = fetch_catalog(&client, &origin, &bearer).await?;
    let selected = select_release(
        &initial,
        &request.game_key,
        &request.archive_sha256,
        request.admission_revision,
    )?;
    let expected = selected.admission(server_id);
    let acquisition_url = origin
        .join(&format!(
            "/v1/cartridges/{}/{}/acquisition",
            request.game_key, request.archive_sha256
        ))
        .map_err(|_| CompanionError::InvalidInput)?;
    let bytes = get_bytes(
        &client,
        acquisition_url,
        Some(&bearer),
        MAX_ACQUISITION_DOCUMENT_BYTES,
        CompanionError::AdmissionChanged,
    )
    .await?;
    let (evidence_key, policy_key) = authorized_acquisition_keys(&bytes, trusted_marketplace)?;
    let sdk = supported_sdk_identity().map_err(|_| CompanionError::Rejected)?;
    let verified = verify_acquisition_bytes_with_policy_key(
        &bytes,
        &expected,
        &evidence_key,
        &policy_key,
        &sdk,
        &rich_2d_host_profile(),
    )
    .map_err(|_| CompanionError::Rejected)?;
    let final_catalog = fetch_catalog(&client, &origin, &bearer).await?;
    let final_release = select_release(
        &final_catalog,
        &request.game_key,
        &request.archive_sha256,
        request.admission_revision,
    )?;
    if final_release != selected {
        return Err(CompanionError::AdmissionChanged);
    }
    let mount = MountRecord::from_verified(
        origin.origin().ascii_serialization(),
        server_id,
        selected,
        &verified,
    )?;
    Ok(RemoteAcquisition { verified, mount })
}

pub async fn acquire_session(
    mut request: SessionAcquireRequest,
    trusted_marketplace: &impl AcquisitionTrust,
) -> Result<RemoteAcquisition> {
    let origin = selected_origin(&request.server_origin)?;
    let server_id = exact_uuid(&request.server_id)?;
    let persona_id = exact_uuid(&request.persona_id)?;
    let session_id = exact_uuid(&request.game_session_id)?;
    if !valid_bearer(&request.device_bearer)
        || request
            .provenance_class
            .as_deref()
            .is_some_and(|value| value != "marketplace_vetted")
    {
        return Err(CompanionError::InvalidInput);
    }
    let bearer = Zeroizing::new(std::mem::take(&mut request.device_bearer));
    let client = remote_client()?;
    require_discovery_capability(
        &client,
        &origin,
        server_id,
        "games.session-cartridge-acquisition.v1",
    )
    .await?;
    let initial = fetch_session(&client, &origin, &bearer, persona_id, session_id).await?;
    let presentation = initial.exact_presentation(session_id)?.clone();
    let expected = presentation.admission(server_id);
    let acquisition_url = origin
        .join(&format!(
            "/v1/personas/{persona_id}/game-sessions/{session_id}/cartridge-acquisition"
        ))
        .map_err(|_| CompanionError::InvalidInput)?;
    let bytes = get_bytes(
        &client,
        acquisition_url,
        Some(&bearer),
        MAX_ACQUISITION_DOCUMENT_BYTES,
        CompanionError::AdmissionChanged,
    )
    .await?;
    let (evidence_key, policy_key) = authorized_acquisition_keys(&bytes, trusted_marketplace)?;
    let sdk = supported_sdk_identity().map_err(|_| CompanionError::Rejected)?;
    let verified = verify_acquisition_bytes_with_policy_key(
        &bytes,
        &expected,
        &evidence_key,
        &policy_key,
        &sdk,
        &rich_2d_host_profile(),
    )
    .map_err(|_| CompanionError::Rejected)?;
    let final_session = fetch_session(&client, &origin, &bearer, persona_id, session_id).await?;
    if final_session.exact_presentation(session_id)? != &presentation {
        return Err(CompanionError::AdmissionChanged);
    }
    let mount = MountRecord::from_session_verified(
        origin.origin().ascii_serialization(),
        server_id,
        &expected,
        &verified,
    )?;
    Ok(RemoteAcquisition { verified, mount })
}

pub async fn discover_operator_custom(
    server_origin: &str,
    expected_server_id: &str,
) -> Result<OperatorCustomDiscovery> {
    let origin = selected_origin(server_origin)?;
    let server_id = exact_uuid(expected_server_id)?;
    let discovery = fetch_discovery(&remote_client()?, &origin).await?;
    validate_discovery(&discovery, server_id, "games.operator-custom-cartridges.v1")?;
    let custom = discovery
        .operator_custom
        .ok_or(CompanionError::OperatorCustomUntrusted)?;
    validate_operator_discovery(&custom)?;
    Ok(custom)
}

pub async fn acquire_operator_custom(
    mut request: AcquireRequest,
    trusted_operator: &OperatorCustomTrust,
) -> Result<RemoteOperatorCustomAcquisition> {
    let origin = selected_origin(&request.server_origin)?;
    let canonical_origin = origin.origin().ascii_serialization();
    let server_id = exact_uuid(&request.server_id)?;
    if !valid_identifier(&request.game_key)
        || !valid_sha256(&request.archive_sha256)
        || request.admission_revision == 0
        || !valid_bearer(&request.device_bearer)
        || !trusted_operator.matches(&canonical_origin, server_id)
        || request
            .provenance_class
            .as_deref()
            .is_some_and(|value| value != "operator_custom")
    {
        return Err(CompanionError::InvalidInput);
    }
    let bearer = Zeroizing::new(std::mem::take(&mut request.device_bearer));
    let client = remote_client()?;
    let discovery = fetch_discovery(&client, &origin).await?;
    validate_discovery(&discovery, server_id, "games.operator-custom-cartridges.v1")?;
    let advertised = discovery
        .operator_custom
        .ok_or(CompanionError::OperatorCustomUntrusted)?;
    validate_operator_discovery(&advertised)?;
    if !trusted_operator.matches_discovery(&advertised) {
        return Err(CompanionError::OperatorCustomUntrusted);
    }
    let initial = fetch_catalog(&client, &origin, &bearer).await?;
    let selected = select_release(
        &initial,
        &request.game_key,
        &request.archive_sha256,
        request.admission_revision,
    )?;
    validate_selected_custom(selected, trusted_operator)?;
    let expected = selected.admission(server_id);
    let acquisition_url = origin
        .join(&format!(
            "/v1/cartridges/{}/{}/acquisition",
            request.game_key, request.archive_sha256
        ))
        .map_err(|_| CompanionError::InvalidInput)?;
    let bytes = get_bytes(
        &client,
        acquisition_url,
        Some(&bearer),
        MAX_ACQUISITION_DOCUMENT_BYTES,
        CompanionError::AdmissionChanged,
    )
    .await?;
    let sdk = supported_sdk_identity().map_err(|_| CompanionError::Rejected)?;
    let verified = verify_operator_custom_acquisition_bytes(
        &bytes,
        &expected,
        &trusted_operator.public_key,
        &sdk,
        &rich_2d_host_profile(),
    )
    .map_err(|_| CompanionError::Rejected)?;
    let final_catalog = fetch_catalog(&client, &origin, &bearer).await?;
    let final_release = select_release(
        &final_catalog,
        &request.game_key,
        &request.archive_sha256,
        request.admission_revision,
    )?;
    if final_release != selected {
        return Err(CompanionError::AdmissionChanged);
    }
    validate_selected_custom(final_release, trusted_operator)?;
    let mount =
        OperatorCustomMountRecord::from_verified(canonical_origin, server_id, selected, &verified)?;
    Ok(RemoteOperatorCustomAcquisition { verified, mount })
}

pub async fn acquire_operator_custom_session(
    mut request: SessionAcquireRequest,
    trusted_operator: &OperatorCustomTrust,
) -> Result<RemoteOperatorCustomAcquisition> {
    let origin = selected_origin(&request.server_origin)?;
    let canonical_origin = origin.origin().ascii_serialization();
    let server_id = exact_uuid(&request.server_id)?;
    let persona_id = exact_uuid(&request.persona_id)?;
    let session_id = exact_uuid(&request.game_session_id)?;
    if !valid_bearer(&request.device_bearer)
        || !trusted_operator.matches(&canonical_origin, server_id)
        || request
            .provenance_class
            .as_deref()
            .is_some_and(|value| value != "operator_custom")
    {
        return Err(CompanionError::InvalidInput);
    }
    let bearer = Zeroizing::new(std::mem::take(&mut request.device_bearer));
    let client = remote_client()?;
    let discovery = fetch_discovery(&client, &origin).await?;
    validate_discovery(&discovery, server_id, "games.operator-custom-cartridges.v1")?;
    if !discovery
        .operator_custom
        .as_ref()
        .is_some_and(|value| trusted_operator.matches_discovery(value))
    {
        return Err(CompanionError::OperatorCustomUntrusted);
    }
    let initial = fetch_session(&client, &origin, &bearer, persona_id, session_id).await?;
    let presentation = initial.exact_presentation(session_id)?.clone();
    validate_session_custom(&presentation, trusted_operator)?;
    let expected = presentation.admission(server_id);
    let acquisition_url = origin
        .join(&format!(
            "/v1/personas/{persona_id}/game-sessions/{session_id}/cartridge-acquisition"
        ))
        .map_err(|_| CompanionError::InvalidInput)?;
    let bytes = get_bytes(
        &client,
        acquisition_url,
        Some(&bearer),
        MAX_ACQUISITION_DOCUMENT_BYTES,
        CompanionError::AdmissionChanged,
    )
    .await?;
    let sdk = supported_sdk_identity().map_err(|_| CompanionError::Rejected)?;
    let verified = verify_operator_custom_acquisition_bytes(
        &bytes,
        &expected,
        &trusted_operator.public_key,
        &sdk,
        &rich_2d_host_profile(),
    )
    .map_err(|_| CompanionError::Rejected)?;
    let final_session = fetch_session(&client, &origin, &bearer, persona_id, session_id).await?;
    if final_session.exact_presentation(session_id)? != &presentation {
        return Err(CompanionError::AdmissionChanged);
    }
    let mount = OperatorCustomMountRecord::from_session_verified(
        canonical_origin,
        server_id,
        &expected,
        presentation
            .operator_custom
            .as_ref()
            .ok_or(CompanionError::Rejected)?,
        &verified,
    )?;
    Ok(RemoteOperatorCustomAcquisition { verified, mount })
}

fn authorized_acquisition_keys(
    bytes: &[u8],
    trust: &impl AcquisitionTrust,
) -> Result<(CatalogPublicKey, CatalogPublicKey)> {
    if bytes.is_empty() || bytes.len() > MAX_ACQUISITION_DOCUMENT_BYTES {
        return Err(CompanionError::Rejected);
    }
    let document: CartridgeAcquisition =
        serde_json::from_slice(bytes).map_err(|_| CompanionError::Rejected)?;
    if serde_json::to_vec(&document).map_err(|_| CompanionError::Rejected)? != bytes {
        return Err(CompanionError::Rejected);
    }
    let snapshot = authenticated_snapshot(
        &document.signed_marketplace_snapshot,
        &document.marketplace_key,
    )?;
    trust
        .authorize_key(&document.marketplace_key, snapshot.snapshot_version)
        .map_err(|_| CompanionError::Rejected)?;
    let policy_key = match document.format.as_str() {
        ACQUISITION_FORMAT
            if document.policy_marketplace_key.is_none()
                && document.signed_policy_marketplace_snapshot.is_none() =>
        {
            trust
                .authorize_current_policy_key(&document.marketplace_key, snapshot.snapshot_version)
                .map_err(|_| CompanionError::Rejected)?;
            document.marketplace_key.clone()
        }
        ACQUISITION_FORMAT_V2 => {
            let policy_key = document
                .policy_marketplace_key
                .clone()
                .ok_or(CompanionError::Rejected)?;
            let policy_snapshot = authenticated_snapshot(
                document
                    .signed_policy_marketplace_snapshot
                    .as_deref()
                    .ok_or(CompanionError::Rejected)?,
                &policy_key,
            )?;
            trust
                .authorize_current_policy_key(&policy_key, policy_snapshot.snapshot_version)
                .map_err(|_| CompanionError::Rejected)?;
            policy_key
        }
        _ => return Err(CompanionError::Rejected),
    };
    Ok((document.marketplace_key, policy_key))
}

fn authenticated_snapshot(
    encoded: &str,
    key: &CatalogPublicKey,
) -> Result<omarchygs_game_cartridge::MarketplaceSnapshotPayload> {
    if encoded.is_empty()
        || encoded.len() > MAX_MARKETPLACE_SNAPSHOT_BYTES.saturating_mul(4).div_ceil(3) + 4
    {
        return Err(CompanionError::Rejected);
    }
    let snapshot_bytes = URL_SAFE_NO_PAD
        .decode(encoded)
        .map_err(|_| CompanionError::Rejected)?;
    if snapshot_bytes.is_empty() || snapshot_bytes.len() > MAX_MARKETPLACE_SNAPSHOT_BYTES {
        return Err(CompanionError::Rejected);
    }
    verify_marketplace_snapshot_bytes(&snapshot_bytes, key).map_err(|_| CompanionError::Rejected)
}

async fn require_discovery_capability(
    client: &Client,
    origin: &Url,
    server_id: Uuid,
    capability: &str,
) -> Result<()> {
    let discovery = fetch_discovery(client, origin).await?;
    validate_discovery(&discovery, server_id, capability)
}

async fn fetch_discovery(client: &Client, origin: &Url) -> Result<DiscoveryDocument> {
    let discovery_url = origin
        .join("/.well-known/omarchygs")
        .map_err(|_| CompanionError::InvalidInput)?;
    get_json(
        client,
        discovery_url,
        None,
        MAX_DISCOVERY_BYTES,
        CompanionError::Rejected,
    )
    .await
}

fn validate_discovery(
    discovery: &DiscoveryDocument,
    server_id: Uuid,
    capability: &str,
) -> Result<()> {
    if discovery.service != "omarchy-gaming-system"
        || discovery.server_id != server_id.to_string()
        || discovery.protocol_version != 1
        || !sorted_capabilities(&discovery.capabilities)
        || !discovery
            .capabilities
            .iter()
            .any(|value| value == capability)
    {
        Err(CompanionError::Rejected)
    } else {
        Ok(())
    }
}

fn validate_operator_discovery(value: &OperatorCustomDiscovery) -> Result<()> {
    if !valid_text(&value.operator_name, 128)
        || !valid_identifier(&value.authority_id)
        || !valid_identifier(&value.key_id)
        || value.public_key.authority_id != value.authority_id
        || value.public_key.key_id != value.key_id
        || operator_custom_key_sha256(&value.public_key).map_err(|_| CompanionError::Rejected)?
            != value.key_sha256
    {
        Err(CompanionError::Rejected)
    } else {
        Ok(())
    }
}

fn remote_client() -> Result<Client> {
    let _ = rustls::crypto::ring::default_provider().install_default();
    Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(30))
        .user_agent("OmarchyGS-Cartridge-Companion/0.1")
        .build()
        .map_err(|_| CompanionError::Unavailable)
}

async fn fetch_catalog(
    client: &Client,
    origin: &Url,
    bearer: &Zeroizing<String>,
) -> Result<CatalogDocument> {
    let url = origin
        .join("/v1/cartridges")
        .map_err(|_| CompanionError::InvalidInput)?;
    get_json(
        client,
        url,
        Some(bearer),
        MAX_CATALOG_BYTES,
        CompanionError::AdmissionChanged,
    )
    .await
}

async fn fetch_session(
    client: &Client,
    origin: &Url,
    bearer: &Zeroizing<String>,
    persona_id: Uuid,
    session_id: Uuid,
) -> Result<SessionDocument> {
    let url = origin
        .join(&format!(
            "/v1/personas/{persona_id}/game-sessions/{session_id}"
        ))
        .map_err(|_| CompanionError::InvalidInput)?;
    get_json(
        client,
        url,
        Some(bearer),
        MAX_SESSION_BYTES,
        CompanionError::AdmissionChanged,
    )
    .await
}

async fn get_json<T: for<'de> Deserialize<'de>>(
    client: &Client,
    url: Url,
    bearer: Option<&Zeroizing<String>>,
    maximum: usize,
    status_error: CompanionError,
) -> Result<T> {
    let bytes = get_bytes(client, url, bearer, maximum, status_error).await?;
    serde_json::from_slice(&bytes).map_err(|_| CompanionError::Rejected)
}

async fn get_bytes(
    client: &Client,
    url: Url,
    bearer: Option<&Zeroizing<String>>,
    maximum: usize,
    status_error: CompanionError,
) -> Result<Vec<u8>> {
    let expected = url.clone();
    let mut request = client
        .get(url)
        .header(header::ACCEPT, "application/json")
        .header(header::ACCEPT_ENCODING, "identity");
    if let Some(bearer) = bearer {
        request = request.bearer_auth(bearer.as_str());
    }
    let response = request
        .send()
        .await
        .map_err(|_| CompanionError::Unavailable)?;
    validate_response(&response, &expected, maximum, status_error)?;
    let mut stream = response.bytes_stream();
    let mut bytes = Vec::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|_| CompanionError::Unavailable)?;
        if bytes.len().saturating_add(chunk.len()) > maximum {
            return Err(CompanionError::Rejected);
        }
        bytes.extend_from_slice(&chunk);
    }
    if bytes.is_empty() {
        Err(CompanionError::Rejected)
    } else {
        Ok(bytes)
    }
}

fn validate_response(
    response: &Response,
    expected: &Url,
    maximum: usize,
    status_error: CompanionError,
) -> Result<()> {
    if response.url() != expected {
        return Err(CompanionError::Rejected);
    }
    if response.status() != StatusCode::OK {
        return Err(status_error);
    }
    if response
        .content_length()
        .is_some_and(|length| length > maximum as u64)
    {
        return Err(CompanionError::Rejected);
    }
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");
    if content_type != "application/json" {
        return Err(CompanionError::Rejected);
    }
    Ok(())
}

pub(crate) fn selected_origin(value: &str) -> Result<Url> {
    let parsed = Url::parse(value).map_err(|_| CompanionError::InvalidInput)?;
    if parsed.username() != ""
        || parsed.password().is_some()
        || parsed.query().is_some()
        || parsed.fragment().is_some()
        || parsed.path() != "/"
        || parsed.host_str().is_none()
    {
        return Err(CompanionError::InvalidInput);
    }
    let loopback = parsed.host_str().is_some_and(|host| {
        host.eq_ignore_ascii_case("localhost")
            || host
                .parse::<std::net::IpAddr>()
                .is_ok_and(|address| address.is_loopback())
    });
    if parsed.scheme() != "https" && !(parsed.scheme() == "http" && loopback) {
        return Err(CompanionError::InvalidInput);
    }
    let canonical = parsed.origin().ascii_serialization();
    if value != canonical && value != format!("{canonical}/") {
        return Err(CompanionError::InvalidInput);
    }
    Url::parse(&canonical).map_err(|_| CompanionError::InvalidInput)
}

fn exact_uuid(value: &str) -> Result<Uuid> {
    let parsed = Uuid::try_parse(value).map_err(|_| CompanionError::InvalidInput)?;
    if parsed.is_nil() || parsed.to_string() != value {
        Err(CompanionError::InvalidInput)
    } else {
        Ok(parsed)
    }
}

fn select_release<'a>(
    document: &'a CatalogDocument,
    game_key: &str,
    digest: &str,
    revision: u64,
) -> Result<&'a CatalogRelease> {
    if document.cartridges.len() > 128 {
        return Err(CompanionError::Rejected);
    }
    document
        .cartridges
        .iter()
        .find(|release| {
            release.game_key == game_key
                && release.archive_sha256 == digest
                && release.server_admission.revision == revision
        })
        .ok_or(CompanionError::AdmissionChanged)
}

fn validate_selected_custom(release: &CatalogRelease, trusted: &OperatorCustomTrust) -> Result<()> {
    let custom = release
        .operator_custom
        .as_ref()
        .ok_or(CompanionError::AdmissionChanged)?;
    if release.marketplace.is_some()
        || custom.provenance_class != "operator_custom"
        || custom.operator_name != trusted.operator_name
        || custom.authority_id != trusted.public_key.authority_id
        || custom.key_id != trusted.public_key.key_id
        || custom.key_sha256 != trusted.key_sha256
        || custom.warning != OPERATOR_CUSTOM_WARNING
        || release
            .warning
            .as_deref()
            .is_none_or(|warning| !warning.starts_with(OPERATOR_CUSTOM_WARNING))
        || custom.policy_version == 0
        || !matches!(custom.lifecycle_status.as_str(), "active" | "deprecated")
    {
        Err(CompanionError::AdmissionChanged)
    } else {
        Ok(())
    }
}

fn validate_session_custom(
    presentation: &SessionPresentation,
    trusted: &OperatorCustomTrust,
) -> Result<()> {
    let custom = presentation
        .operator_custom
        .as_ref()
        .ok_or(CompanionError::AdmissionChanged)?;
    if custom.provenance_class != "operator_custom"
        || custom.operator_name != trusted.operator_name
        || custom.authority_id != trusted.public_key.authority_id
        || custom.key_id != trusted.public_key.key_id
        || custom.key_sha256 != trusted.key_sha256
        || custom.warning != OPERATOR_CUSTOM_WARNING
    {
        Err(CompanionError::AdmissionChanged)
    } else {
        Ok(())
    }
}

fn sorted_capabilities(values: &[String]) -> bool {
    values.len() <= 32
        && values.iter().all(|value| {
            !value.is_empty()
                && value.len() <= 64
                && value.bytes().all(|byte| {
                    byte.is_ascii_lowercase()
                        || byte.is_ascii_digit()
                        || matches!(byte, b'.' | b'-')
                })
        })
        && values.windows(2).all(|pair| pair[0] < pair[1])
}

fn valid_identifier(value: &str) -> bool {
    let bytes = value.as_bytes();
    (1..=96).contains(&bytes.len())
        && bytes[0].is_ascii_lowercase()
        && bytes.iter().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'_')
        })
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_bearer(value: &str) -> bool {
    let Some(encoded) = value
        .strip_prefix("ogs1_")
        .or_else(|| value.strip_prefix("bbs1_"))
    else {
        return false;
    };
    encoded.len() == 43
        && encoded
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn valid_text(value: &str, maximum: usize) -> bool {
    !value.is_empty()
        && value.trim() == value
        && value.chars().count() <= maximum
        && !value.chars().any(char::is_control)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DiscoveryDocument {
    service: String,
    server_id: String,
    #[allow(dead_code)]
    server_name: String,
    protocol_version: u16,
    capabilities: Vec<String>,
    #[serde(default)]
    operator_custom: Option<OperatorCustomDiscovery>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)]
struct SessionDocument {
    id: String,
    game_key: String,
    game_version: u32,
    revision: i64,
    status: String,
    state: Option<serde_json::Value>,
    authority: String,
    provider_release_id: Option<String>,
    availability: Option<String>,
    presentation: Option<SessionPresentation>,
    result: Option<serde_json::Value>,
    participants: Vec<serde_json::Value>,
    completed_at: Option<String>,
    created_at: String,
    updated_at: String,
}

impl SessionDocument {
    fn exact_presentation(&self, expected_session_id: Uuid) -> Result<&SessionPresentation> {
        if self.id != expected_session_id.to_string() {
            return Err(CompanionError::AdmissionChanged);
        }
        let presentation = self
            .presentation
            .as_ref()
            .ok_or(CompanionError::AdmissionChanged)?;
        presentation.validate()?;
        Ok(presentation)
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct SessionPresentation {
    format: String,
    publisher_id: String,
    game_key: String,
    rules_version: u32,
    cartridge_version: u32,
    archive_sha256: String,
    signed_identity_sha256: String,
    admission_revision: u64,
    lifecycle_status: String,
    active_session_policy: String,
    #[serde(default)]
    operator_custom: Option<SessionOperatorCustomProvenance>,
    #[serde(default)]
    warning: Option<String>,
}

impl SessionPresentation {
    fn validate(&self) -> Result<()> {
        let warning_matches = match (self.lifecycle_status.as_str(), &self.operator_custom) {
            ("active" | "retired", None) => self.warning.is_none(),
            ("deprecated", None) => self
                .warning
                .as_ref()
                .is_some_and(|warning| valid_text(warning, 512)),
            ("active" | "deprecated" | "retired", Some(custom)) => self
                .warning
                .as_ref()
                .is_some_and(|warning| warning.starts_with(&custom.warning)),
            _ => false,
        };
        if self.format != "omarchygs.session-cartridge/v1"
            || !valid_identifier(&self.publisher_id)
            || !valid_identifier(&self.game_key)
            || self.rules_version == 0
            || self.cartridge_version == 0
            || !valid_sha256(&self.archive_sha256)
            || !valid_sha256(&self.signed_identity_sha256)
            || self.admission_revision == 0
            || self.active_session_policy != "continue"
            || !warning_matches
        {
            Err(CompanionError::AdmissionChanged)
        } else {
            Ok(())
        }
    }

    pub(crate) fn admission(&self, server_id: Uuid) -> AcquisitionServerAdmission {
        AcquisitionServerAdmission {
            server_id: server_id.to_string(),
            game_key: self.game_key.clone(),
            publisher_id: self.publisher_id.clone(),
            rules_version: self.rules_version,
            cartridge_version: self.cartridge_version,
            archive_sha256: self.archive_sha256.clone(),
            signed_identity_sha256: self.signed_identity_sha256.clone(),
            admission_revision: self.admission_revision,
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct SessionOperatorCustomProvenance {
    pub(crate) provenance_class: String,
    pub(crate) operator_name: String,
    pub(crate) authority_id: String,
    pub(crate) key_id: String,
    pub(crate) key_sha256: String,
    pub(crate) warning: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CatalogDocument {
    cartridges: Vec<CatalogRelease>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct CatalogRelease {
    pub game_key: String,
    pub publisher_id: String,
    pub rules_version: u32,
    pub cartridge_version: u32,
    pub display_name: String,
    pub archive_sha256: String,
    pub signed_identity_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marketplace: Option<MarketplaceProvenance>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operator_custom: Option<OperatorCustomProvenance>,
    pub server_admission: ServerAdmission,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

impl CatalogRelease {
    pub(crate) fn admission(&self, server_id: Uuid) -> AcquisitionServerAdmission {
        AcquisitionServerAdmission {
            server_id: server_id.to_string(),
            game_key: self.game_key.clone(),
            publisher_id: self.publisher_id.clone(),
            rules_version: self.rules_version,
            cartridge_version: self.cartridge_version,
            archive_sha256: self.archive_sha256.clone(),
            signed_identity_sha256: self.signed_identity_sha256.clone(),
            admission_revision: self.server_admission.revision,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct MarketplaceProvenance {
    pub provenance_class: String,
    pub marketplace_id: String,
    pub marketplace_name: String,
    pub reviewed_by: String,
    pub review_summary: String,
    pub policy_version: u64,
    pub lifecycle_status: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct OperatorCustomProvenance {
    pub provenance_class: String,
    pub operator_name: String,
    pub authority_id: String,
    pub key_id: String,
    pub key_sha256: String,
    pub warning: String,
    pub policy_version: u64,
    pub lifecycle_status: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ServerAdmission {
    pub revision: u64,
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::Path,
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
        time::{SystemTime, UNIX_EPOCH},
    };

    use axum::{
        Json, Router,
        body::Body,
        extract::{Path as AxumPath, State},
        http::{HeaderMap, StatusCode, header},
        response::{IntoResponse, Response},
        routing::get,
    };
    use omarchygs_game_cartridge::{
        AcquisitionServerAdmission, CartridgeAcquisition, CatalogStatus, MarketplaceReleaseEntry,
        MarketplaceSnapshotPayload, OperatorCustomAcquisition, RELEASE_ARCHIVE_PATH,
        RELEASE_ATTESTATION_PATH, RELEASE_CONFORMANCE_PATH, create_release, export_sdk,
        generate_catalog_keypair, generate_keypair, operator_custom_key_sha256,
        rich_2d_host_profile, sign_catalog_policy, sign_marketplace_snapshot,
        sign_operator_custom_release, signed_operator_custom_release_bytes,
        verify_release_directory,
    };
    use omarchygs_marketplace_trust::{
        MarketplaceKeyStatus, MarketplaceTrustKey, MarketplaceTrustPayload, catalog_key_sha256,
        generate_trust_root_keypair, sign_marketplace_trust, signed_trust_bytes,
        verify_marketplace_trust_bytes,
    };
    use serde_json::{Value, json};
    use tokio::net::TcpListener;

    use super::*;
    use crate::{ClientCartridgeCache, ClientTrustSnapshot};

    const REVISION: &str = "1111111111111111111111111111111111111111";
    const BUILDER_DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const SERVER_ID: &str = "11111111-1111-4111-8111-111111111111";
    const PERSONA_ID: &str = "22222222-2222-4222-8222-222222222222";
    const SESSION_ID: &str = "33333333-3333-4333-8333-333333333333";
    const BEARER: &str = "ogs1_AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

    fn channel_trust(
        current_snapshot_version: u64,
        keys: Vec<MarketplaceTrustKey>,
    ) -> ClientTrustSnapshot {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be valid")
            .as_secs();
        let authority_id = keys
            .last()
            .expect("channel trust needs an active key")
            .key
            .authority_id
            .clone();
        let (root_private, root_public) =
            generate_trust_root_keypair("root-1", "official").expect("root should generate");
        let payload = MarketplaceTrustPayload {
            format: "omarchygs.marketplace-trust-channel/v2".to_owned(),
            channel_id: "official".to_owned(),
            channel_name: "Official OmarchyGS".to_owned(),
            channel_origin: "https://packages.example.test/v1/".to_owned(),
            marketplace_origin: "https://market.example.test/v1/".to_owned(),
            marketplace_authority_id: authority_id,
            bundle_version: current_snapshot_version,
            current_snapshot_version,
            not_before_unix: now - 10,
            expires_at_unix: now + 3600,
            keys,
            packages: Vec::new(),
        };
        let signed = sign_marketplace_trust(&payload, &root_private).expect("trust should sign");
        let trust = verify_marketplace_trust_bytes(
            &signed_trust_bytes(&signed).expect("trust bytes"),
            &root_public,
            "official",
            "https://packages.example.test/v1/",
            now,
        )
        .expect("trust should verify");
        ClientTrustSnapshot::Channel(Arc::new(trust))
    }

    #[tokio::test]
    async fn exact_remote_acquisition_mounts_and_catalog_change_fails_closed() {
        let fixture = RemoteFixture::new();
        let (origin, server) = fixture.spawn(false).await;
        let acquired = acquire(fixture.request(origin.clone()), &fixture.marketplace_public)
            .await
            .expect("exact acquisition");
        let cache_temp = tempfile::tempdir().expect("cache temp should create");
        let cache = ClientCartridgeCache::open(&cache_temp.path().join("cache"))
            .expect("cache should open");
        let mounted = cache
            .install(&acquired.verified, acquired.mount)
            .expect("verified acquisition should mount");
        assert_eq!(mounted.archive_sha256, fixture.admission.archive_sha256);
        let mut render_request: crate::RenderRequest = serde_json::from_value(json!({
            "server_origin": origin,
            "server_id": SERVER_ID,
            "game_key": mounted.game_key,
            "archive_sha256": mounted.archive_sha256,
            "admission_revision": mounted.admission_revision,
            "lifecycle_status": "active",
            "active_session_policy": "continue",
            "view": {
                "chronicle_label": "Read the chronicle",
                "welcome": "Welcome to Door Legends. One choice opens the way.",
                "status": "A weathered brass door waits in the dark.",
                "enter_label": "Enter the brass door",
                "lobby_label": "Return to the lobby"
            },
            "preferences": {
                "scale": 1.0,
                "high_contrast": false,
                "reduced_motion": false,
                "muted_audio": false
            }
        }))
        .expect("render request should parse");
        let prepared = crate::compile_mounted_render_plan(
            &cache,
            &render_request,
            &fixture.marketplace_public,
        )
        .expect("mounted Door Legends cartridge should compile");
        let plan = serde_json::to_value(&prepared.plan).expect("plan should serialize");
        assert_eq!(plan["origin"]["archive_sha256"], mounted.archive_sha256);
        assert_eq!(plan["nodes"][2]["kind"], "button");
        assert_eq!(plan["nodes"][2]["action"], "enter");
        assert_eq!(prepared.screen_id, "lobby");
        assert_eq!(prepared.entry_screen_id, "lobby");
        assert_eq!(
            prepared.navigation,
            vec![omarchygs_game_cartridge_renderer::PreparedNavigation {
                action: "navigate.chronicle".to_owned(),
                target_screen: "chronicle".to_owned(),
            }]
        );
        assert!(prepared.assets.is_empty());
        render_request.screen_id = Some("chronicle".to_owned());
        let chronicle = crate::compile_mounted_render_plan(
            &cache,
            &render_request,
            &fixture.marketplace_public,
        )
        .expect("signed secondary screen should compile");
        assert_eq!(chronicle.screen_id, "chronicle");
        assert_eq!(chronicle.entry_screen_id, "lobby");
        assert_eq!(chronicle.navigation[0].target_screen, "lobby");

        let (_, replacement_key) =
            generate_catalog_keypair("marketplace-primary-v2", "marketplace").unwrap();
        let rotated = channel_trust(
            2,
            vec![
                MarketplaceTrustKey {
                    key_sha256: catalog_key_sha256(&fixture.marketplace_public).unwrap(),
                    key: fixture.marketplace_public.clone(),
                    status: MarketplaceKeyStatus::Retired,
                    first_snapshot_version: 1,
                    last_snapshot_version: Some(1),
                },
                MarketplaceTrustKey {
                    key_sha256: catalog_key_sha256(&replacement_key).unwrap(),
                    key: replacement_key,
                    status: MarketplaceKeyStatus::Active,
                    first_snapshot_version: 2,
                    last_snapshot_version: None,
                },
            ],
        );
        assert!(matches!(
            crate::render::compile_mounted_render_plan_with_trust(
                &cache,
                &render_request,
                &rotated,
            ),
            Err(CompanionError::MarketplaceUntrusted)
        ));

        render_request.server_origin = "https://other.example.test".to_owned();
        assert!(matches!(
            crate::compile_mounted_render_plan(
                &cache,
                &render_request,
                &fixture.marketplace_public,
            ),
            Err(CompanionError::MountMissing)
        ));
        assert_eq!(
            cache
                .mounts(
                    Uuid::parse_str(SERVER_ID).unwrap(),
                    &fixture.marketplace_public,
                )
                .expect("mount should read"),
            vec![mounted]
        );
        server.abort();

        let exact_current = channel_trust(
            1,
            vec![MarketplaceTrustKey {
                key_sha256: catalog_key_sha256(&fixture.marketplace_public).unwrap(),
                key: fixture.marketplace_public.clone(),
                status: MarketplaceKeyStatus::Active,
                first_snapshot_version: 1,
                last_snapshot_version: None,
            }],
        );
        let (channel_origin, channel_server) = fixture.spawn(false).await;
        acquire(fixture.request(channel_origin), &exact_current)
            .await
            .expect("the exact declared current snapshot should acquire");
        channel_server.abort();

        let stale_current = channel_trust(
            2,
            vec![MarketplaceTrustKey {
                key_sha256: catalog_key_sha256(&fixture.marketplace_public).unwrap(),
                key: fixture.marketplace_public.clone(),
                status: MarketplaceKeyStatus::Active,
                first_snapshot_version: 1,
                last_snapshot_version: None,
            }],
        );
        let (stale_origin, stale_server) = fixture.spawn(false).await;
        let error = match acquire(fixture.request(stale_origin), &stale_current).await {
            Ok(_) => panic!("an older active-key policy snapshot must be rejected"),
            Err(error) => error,
        };
        assert_eq!(error.code(), "companion_server_rejected");
        stale_server.abort();

        let (_, substituted_marketplace_key) =
            generate_catalog_keypair("marketplace-primary-v1", "marketplace").unwrap();
        assert_ne!(substituted_marketplace_key, fixture.marketplace_public);
        let (substituted_origin, substituted_server) = fixture.spawn(false).await;
        let error = match acquire(
            fixture.request(substituted_origin),
            &substituted_marketplace_key,
        )
        .await
        {
            Ok(_) => panic!("server-selected marketplace key must fail"),
            Err(error) => error,
        };
        assert_eq!(error.code(), "companion_server_rejected");
        substituted_server.abort();

        let (changed_origin, changed_server) = fixture.spawn(true).await;
        let error =
            match acquire(fixture.request(changed_origin), &fixture.marketplace_public).await {
                Ok(_) => panic!("changed final admission must fail"),
                Err(error) => error,
            };
        assert_eq!(error.code(), "companion_admission_changed");
        changed_server.abort();
    }

    #[tokio::test]
    async fn historical_session_acquisition_uses_the_exact_session_pin() {
        let fixture = RemoteFixture::with_status(CatalogStatus::Retired);
        let (origin, server) = fixture.spawn_session(false).await;
        let acquired = acquire_session(
            SessionAcquireRequest {
                server_origin: origin,
                server_id: SERVER_ID.to_owned(),
                device_bearer: BEARER.to_owned(),
                persona_id: PERSONA_ID.to_owned(),
                game_session_id: SESSION_ID.to_owned(),
                provenance_class: None,
            },
            &fixture.marketplace_public,
        )
        .await
        .expect("historical session acquisition should verify");
        assert_eq!(acquired.mount.game_key, fixture.admission.game_key);
        assert_eq!(
            acquired.mount.archive_sha256,
            fixture.admission.archive_sha256
        );
        assert_eq!(
            acquired.mount.admission_revision,
            fixture.admission.admission_revision
        );
        assert_eq!(acquired.mount.lifecycle_status, "retired");
        let cache_root = tempfile::tempdir().unwrap();
        let cache = ClientCartridgeCache::open(&cache_root.path().join("cache")).unwrap();
        cache
            .install_session(&acquired.verified, acquired.mount.clone())
            .expect("an active session may refresh a retired exact cartridge");
        server.abort();

        let (changed_origin, changed_server) = fixture.spawn_session(true).await;
        let error = match acquire_session(
            SessionAcquireRequest {
                server_origin: changed_origin,
                server_id: SERVER_ID.to_owned(),
                device_bearer: BEARER.to_owned(),
                persona_id: PERSONA_ID.to_owned(),
                game_session_id: SESSION_ID.to_owned(),
                provenance_class: None,
            },
            &fixture.marketplace_public,
        )
        .await
        {
            Ok(_) => panic!("a changed final session pin must fail closed"),
            Err(error) => error,
        };
        assert_eq!(error.code(), "companion_admission_changed");
        changed_server.abort();
    }

    #[tokio::test]
    async fn operator_custom_remote_acquisition_requires_exact_pin_and_rechecks_catalog() {
        let fixture = RemoteFixture::new();
        let (origin, server) = fixture.spawn_custom(false).await;
        let discovery = discover_operator_custom(&origin, SERVER_ID)
            .await
            .expect("custom discovery should validate");
        assert_eq!(discovery, fixture.operator_discovery);
        let cache_temp = tempfile::tempdir().expect("cache temp should create");
        let cache = ClientCartridgeCache::open(&cache_temp.path().join("cache"))
            .expect("cache should open");
        let trust = cache
            .trust_operator_custom(
                &origin,
                Uuid::parse_str(SERVER_ID).unwrap(),
                &discovery,
                &discovery.key_sha256,
            )
            .expect("explicit exact operator pin should persist");
        let acquired = acquire_operator_custom(fixture.custom_request(origin.clone()), &trust)
            .await
            .expect("custom acquisition should verify");
        let mounted = cache
            .install_operator_custom(&acquired.verified, acquired.mount)
            .expect("verified custom acquisition should mount");
        assert_eq!(mounted.operator_custom.provenance_class, "operator_custom");
        assert_eq!(mounted.operator_custom.key_sha256, discovery.key_sha256);
        assert!(mounted.warning.starts_with(OPERATOR_CUSTOM_WARNING));
        let render_request: crate::RenderRequest = serde_json::from_value(json!({
            "server_origin": origin,
            "server_id": SERVER_ID,
            "game_key": mounted.game_key,
            "archive_sha256": mounted.archive_sha256,
            "admission_revision": mounted.admission_revision,
            "lifecycle_status": "active",
            "active_session_policy": "continue",
            "provenance_class": "operator_custom",
            "view": {
                "chronicle_label": "Read the chronicle",
                "welcome": "Welcome to Door Legends. One choice opens the way.",
                "status": "A weathered brass door waits in the dark.",
                "enter_label": "Enter the brass door",
                "lobby_label": "Return to the lobby"
            }
        }))
        .expect("custom render request should parse");
        crate::render::compile_operator_custom_render_plan(&cache, &render_request)
            .expect("pinned custom mount should compile through trusted renderer");
        server.abort();

        let (changed_origin, changed_server) = fixture.spawn_custom(true).await;
        let changed_discovery = discover_operator_custom(&changed_origin, SERVER_ID)
            .await
            .expect("changed fixture discovery should remain valid");
        let changed_cache = ClientCartridgeCache::open(&cache_temp.path().join("changed-cache"))
            .expect("changed cache should open");
        let changed_trust = changed_cache
            .trust_operator_custom(
                &changed_origin,
                Uuid::parse_str(SERVER_ID).unwrap(),
                &changed_discovery,
                &changed_discovery.key_sha256,
            )
            .expect("changed origin receives its own explicit pin");
        let error =
            match acquire_operator_custom(fixture.custom_request(changed_origin), &changed_trust)
                .await
            {
                Ok(_) => panic!("changed final custom catalog must fail closed"),
                Err(error) => error,
            };
        assert_eq!(error.code(), "companion_admission_changed");
        changed_server.abort();
    }

    #[test]
    fn selected_origin_accepts_only_canonical_https_or_loopback_http() {
        assert!(selected_origin("https://games.example.test").is_ok());
        assert!(selected_origin("http://127.0.0.1:8080").is_ok());
        for rejected in [
            "http://games.example.test",
            "https://user@games.example.test",
            "https://games.example.test/path",
            "https://games.example.test?next=elsewhere",
            "HTTPS://games.example.test",
        ] {
            assert!(selected_origin(rejected).is_err(), "accepted {rejected}");
        }
        let retired = SessionPresentation {
            format: "omarchygs.session-cartridge/v1".to_owned(),
            publisher_id: "publisher".to_owned(),
            game_key: "game".to_owned(),
            rules_version: 1,
            cartridge_version: 1,
            archive_sha256: "a".repeat(64),
            signed_identity_sha256: "b".repeat(64),
            admission_revision: 1,
            lifecycle_status: "retired".to_owned(),
            active_session_policy: "continue".to_owned(),
            operator_custom: None,
            warning: None,
        };
        assert!(
            retired.validate().is_ok(),
            "retired releases remain acquirable only for continuing sessions"
        );
    }

    struct RemoteFixture {
        _temp: tempfile::TempDir,
        acquisition: Arc<Vec<u8>>,
        custom_acquisition: Arc<Vec<u8>>,
        catalog: Value,
        custom_catalog: Value,
        marketplace_public: CatalogPublicKey,
        operator_discovery: OperatorCustomDiscovery,
        admission: AcquisitionServerAdmission,
        lifecycle_status: String,
    }

    impl RemoteFixture {
        fn new() -> Self {
            Self::with_status(CatalogStatus::Active)
        }

        fn with_status(status: CatalogStatus) -> Self {
            let lifecycle_status = match status {
                CatalogStatus::Active => "active",
                CatalogStatus::Deprecated => "deprecated",
                CatalogStatus::Suspended => "suspended",
                CatalogStatus::Revoked => "revoked",
                CatalogStatus::Retired => "retired",
            }
            .to_owned();
            let temp = tempfile::tempdir().unwrap();
            let sdk = temp.path().join("sdk");
            let release_root = temp.path().join("release");
            fs::create_dir(&sdk).unwrap();
            fs::create_dir(&release_root).unwrap();
            export_sdk(&sdk).unwrap();
            let (publisher_private, publisher_public) =
                generate_keypair("publisher-primary-v1", "ignibyte").unwrap();
            create_release(
                &Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../examples/first-party-door-legends/cartridge"),
                &publisher_private,
                &sdk,
                REVISION,
                BUILDER_DIGEST,
                &rich_2d_host_profile(),
                &release_root,
            )
            .unwrap();
            let release = verify_release_directory(
                &release_root,
                &publisher_public,
                &sdk,
                &rich_2d_host_profile(),
            )
            .unwrap();
            let (operator_private, operator_public) =
                generate_catalog_keypair("custom-primary-v1", "test-community").unwrap();
            let signed_operator = sign_operator_custom_release(
                &release,
                &publisher_public,
                &operator_private,
                SERVER_ID,
                "Test Community Operator",
            )
            .unwrap();
            let signed_operator = signed_operator_custom_release_bytes(&signed_operator).unwrap();
            let custom_policy = sign_catalog_policy(
                &release,
                &operator_private,
                1,
                status,
                "Local operator policy.",
            )
            .unwrap();
            let custom_policy = serde_json::to_vec(&custom_policy).unwrap();
            let (marketplace_private, marketplace_public) =
                generate_catalog_keypair("marketplace-primary-v1", "marketplace").unwrap();
            let policy = sign_catalog_policy(
                &release,
                &marketplace_private,
                1,
                status,
                "Reviewed exact release.",
            )
            .unwrap();
            let entry = MarketplaceReleaseEntry {
                release_path: "releases/door-legends/1/".to_owned(),
                game_key: release.payload().game_key.clone(),
                publisher_id: release.payload().publisher_id.clone(),
                rules_version: release.payload().rules_version,
                cartridge_version: release.payload().cartridge_version,
                archive_sha256: release.payload().archive_sha256.clone(),
                signed_identity_sha256: release.payload().signed_identity_sha256.clone(),
                publisher_key: publisher_public.clone(),
                reviewed_by: "review-team".to_owned(),
                review_summary: "Bounded first-party review passed.".to_owned(),
                policy,
            };
            let snapshot_payload = MarketplaceSnapshotPayload {
                format: "omarchygs.marketplace-snapshot/v1".to_owned(),
                snapshot_version: 1,
                authority_id: marketplace_public.authority_id.clone(),
                marketplace_name: "Test Marketplace".to_owned(),
                releases: vec![entry],
            };
            let snapshot = serde_json::to_vec(
                &sign_marketplace_snapshot(&snapshot_payload, &marketplace_private).unwrap(),
            )
            .unwrap();
            let admission = AcquisitionServerAdmission {
                server_id: SERVER_ID.to_owned(),
                game_key: release.payload().game_key.clone(),
                publisher_id: release.payload().publisher_id.clone(),
                rules_version: release.payload().rules_version,
                cartridge_version: release.payload().cartridge_version,
                archive_sha256: release.payload().archive_sha256.clone(),
                signed_identity_sha256: release.payload().signed_identity_sha256.clone(),
                admission_revision: 1,
            };
            let acquisition = CartridgeAcquisition::from_verified_bytes(
                admission.clone(),
                marketplace_public.clone(),
                &snapshot,
                &fs::read(release_root.join(RELEASE_ARCHIVE_PATH)).unwrap(),
                &fs::read(release_root.join(RELEASE_CONFORMANCE_PATH)).unwrap(),
                &fs::read(release_root.join(RELEASE_ATTESTATION_PATH)).unwrap(),
            )
            .unwrap()
            .to_bounded_json()
            .unwrap();
            let custom_acquisition = OperatorCustomAcquisition::from_verified_bytes(
                admission.clone(),
                operator_public.clone(),
                &signed_operator,
                &custom_policy,
                &fs::read(release_root.join(RELEASE_ARCHIVE_PATH)).unwrap(),
                &fs::read(release_root.join(RELEASE_CONFORMANCE_PATH)).unwrap(),
                &fs::read(release_root.join(RELEASE_ATTESTATION_PATH)).unwrap(),
            )
            .unwrap()
            .to_bounded_json()
            .unwrap();
            let catalog = json!({
                "cartridges": [{
                    "game_key": admission.game_key,
                    "publisher_id": admission.publisher_id,
                    "rules_version": admission.rules_version,
                    "cartridge_version": admission.cartridge_version,
                    "display_name": release.cartridge().manifest().display_name,
                    "archive_sha256": admission.archive_sha256,
                    "signed_identity_sha256": admission.signed_identity_sha256,
                    "marketplace": {
                        "provenance_class": "marketplace_vetted",
                        "marketplace_id": snapshot_payload.authority_id,
                        "marketplace_name": snapshot_payload.marketplace_name,
                        "reviewed_by": "review-team",
                        "review_summary": "Bounded first-party review passed.",
                        "policy_version": 1,
                        "lifecycle_status": lifecycle_status
                    },
                    "server_admission": {"revision": admission.admission_revision}
                }]
            });
            let key_sha256 = operator_custom_key_sha256(&operator_public).unwrap();
            let operator_discovery = OperatorCustomDiscovery {
                operator_name: "Test Community Operator".to_owned(),
                authority_id: operator_public.authority_id.clone(),
                key_id: operator_public.key_id.clone(),
                key_sha256: key_sha256.clone(),
                public_key: operator_public,
            };
            let custom_catalog = json!({
                "cartridges": [{
                    "game_key": admission.game_key,
                    "publisher_id": admission.publisher_id,
                    "rules_version": admission.rules_version,
                    "cartridge_version": admission.cartridge_version,
                    "display_name": release.cartridge().manifest().display_name,
                    "archive_sha256": admission.archive_sha256,
                    "signed_identity_sha256": admission.signed_identity_sha256,
                    "operator_custom": {
                        "provenance_class": "operator_custom",
                        "operator_name": operator_discovery.operator_name,
                        "authority_id": operator_discovery.authority_id,
                        "key_id": operator_discovery.key_id,
                        "key_sha256": key_sha256,
                        "warning": OPERATOR_CUSTOM_WARNING,
                        "policy_version": 1,
                        "lifecycle_status": lifecycle_status
                    },
                    "server_admission": {"revision": admission.admission_revision},
                    "warning": OPERATOR_CUSTOM_WARNING
                }]
            });
            Self {
                _temp: temp,
                acquisition: Arc::new(acquisition),
                custom_acquisition: Arc::new(custom_acquisition),
                catalog,
                custom_catalog,
                marketplace_public,
                operator_discovery,
                admission,
                lifecycle_status,
            }
        }

        fn request(&self, server_origin: String) -> AcquireRequest {
            AcquireRequest {
                server_origin,
                server_id: SERVER_ID.to_owned(),
                device_bearer: BEARER.to_owned(),
                game_key: self.admission.game_key.clone(),
                archive_sha256: self.admission.archive_sha256.clone(),
                admission_revision: self.admission.admission_revision,
                provenance_class: None,
            }
        }

        fn custom_request(&self, server_origin: String) -> AcquireRequest {
            AcquireRequest {
                server_origin,
                server_id: SERVER_ID.to_owned(),
                device_bearer: BEARER.to_owned(),
                game_key: self.admission.game_key.clone(),
                archive_sha256: self.admission.archive_sha256.clone(),
                admission_revision: self.admission.admission_revision,
                provenance_class: Some("operator_custom".to_owned()),
            }
        }

        async fn spawn(&self, change_final_catalog: bool) -> (String, tokio::task::JoinHandle<()>) {
            let state = FixtureState {
                acquisition: self.acquisition.clone(),
                catalog: self.catalog.clone(),
                operator_custom: None,
                catalog_calls: Arc::new(AtomicUsize::new(0)),
                change_final_catalog,
                session_calls: Arc::new(AtomicUsize::new(0)),
                change_final_session: false,
                game_key: self.admission.game_key.clone(),
                digest: self.admission.archive_sha256.clone(),
                admission: self.admission.clone(),
                lifecycle_status: self.lifecycle_status.clone(),
            };
            let app = Router::new()
                .route("/.well-known/omarchygs", get(discovery))
                .route("/v1/cartridges", get(catalog))
                .route(
                    "/v1/cartridges/{game_key}/{digest}/acquisition",
                    get(acquisition),
                )
                .with_state(state);
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let address = listener.local_addr().unwrap();
            let handle = tokio::spawn(async move {
                axum::serve(listener, app).await.unwrap();
            });
            (format!("http://{address}"), handle)
        }

        async fn spawn_custom(
            &self,
            change_final_catalog: bool,
        ) -> (String, tokio::task::JoinHandle<()>) {
            let state = FixtureState {
                acquisition: self.custom_acquisition.clone(),
                catalog: self.custom_catalog.clone(),
                operator_custom: Some(self.operator_discovery.clone()),
                catalog_calls: Arc::new(AtomicUsize::new(0)),
                change_final_catalog,
                session_calls: Arc::new(AtomicUsize::new(0)),
                change_final_session: false,
                game_key: self.admission.game_key.clone(),
                digest: self.admission.archive_sha256.clone(),
                admission: self.admission.clone(),
                lifecycle_status: self.lifecycle_status.clone(),
            };
            let app = Router::new()
                .route("/.well-known/omarchygs", get(discovery))
                .route("/v1/cartridges", get(catalog))
                .route(
                    "/v1/cartridges/{game_key}/{digest}/acquisition",
                    get(acquisition),
                )
                .with_state(state);
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let address = listener.local_addr().unwrap();
            let handle = tokio::spawn(async move {
                axum::serve(listener, app).await.unwrap();
            });
            (format!("http://{address}"), handle)
        }

        async fn spawn_session(
            &self,
            change_final_session: bool,
        ) -> (String, tokio::task::JoinHandle<()>) {
            let state = FixtureState {
                acquisition: self.acquisition.clone(),
                catalog: self.catalog.clone(),
                operator_custom: None,
                catalog_calls: Arc::new(AtomicUsize::new(0)),
                change_final_catalog: false,
                session_calls: Arc::new(AtomicUsize::new(0)),
                change_final_session,
                game_key: self.admission.game_key.clone(),
                digest: self.admission.archive_sha256.clone(),
                admission: self.admission.clone(),
                lifecycle_status: self.lifecycle_status.clone(),
            };
            let app = Router::new()
                .route("/.well-known/omarchygs", get(discovery))
                .route(
                    "/v1/personas/{persona_id}/game-sessions/{session_id}",
                    get(session),
                )
                .route(
                    "/v1/personas/{persona_id}/game-sessions/{session_id}/cartridge-acquisition",
                    get(session_acquisition),
                )
                .with_state(state);
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let address = listener.local_addr().unwrap();
            let handle = tokio::spawn(async move {
                axum::serve(listener, app).await.unwrap();
            });
            (format!("http://{address}"), handle)
        }
    }

    #[derive(Clone)]
    struct FixtureState {
        acquisition: Arc<Vec<u8>>,
        catalog: Value,
        operator_custom: Option<OperatorCustomDiscovery>,
        catalog_calls: Arc<AtomicUsize>,
        change_final_catalog: bool,
        session_calls: Arc<AtomicUsize>,
        change_final_session: bool,
        game_key: String,
        digest: String,
        admission: AcquisitionServerAdmission,
        lifecycle_status: String,
    }

    async fn discovery(State(state): State<FixtureState>) -> Json<Value> {
        let mut document = json!({
            "service": "omarchy-gaming-system",
            "server_id": SERVER_ID,
            "server_name": "Remote fixture",
            "protocol_version": 1,
            "capabilities": [
                "games.cartridge-acquisition.v1",
                "games.cartridge-catalog.v1",
                "games.session-cartridge-acquisition.v1"
            ]
        });
        if let Some(operator_custom) = state.operator_custom {
            document["capabilities"] = json!([
                "games.cartridge-acquisition.v1",
                "games.cartridge-catalog.v1",
                "games.operator-custom-cartridges.v1",
                "games.session-cartridge-acquisition.v1"
            ]);
            document["operator_custom"] = serde_json::to_value(operator_custom).unwrap();
        }
        Json(document)
    }

    async fn catalog(State(state): State<FixtureState>, headers: HeaderMap) -> Response {
        if !authorized(&headers) {
            return StatusCode::UNAUTHORIZED.into_response();
        }
        let call = state.catalog_calls.fetch_add(1, Ordering::SeqCst);
        let mut catalog = state.catalog.clone();
        if state.change_final_catalog && call > 0 {
            catalog["cartridges"][0]["server_admission"]["revision"] = json!(2);
        }
        Json(catalog).into_response()
    }

    async fn acquisition(
        State(state): State<FixtureState>,
        AxumPath((game_key, digest)): AxumPath<(String, String)>,
        headers: HeaderMap,
    ) -> Response {
        if !authorized(&headers) || game_key != state.game_key || digest != state.digest {
            return StatusCode::NOT_FOUND.into_response();
        }
        (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json")],
            Body::from(state.acquisition.as_ref().clone()),
        )
            .into_response()
    }

    async fn session(
        State(state): State<FixtureState>,
        AxumPath((persona_id, session_id)): AxumPath<(String, String)>,
        headers: HeaderMap,
    ) -> Response {
        if !authorized(&headers) || persona_id != PERSONA_ID || session_id != SESSION_ID {
            return StatusCode::NOT_FOUND.into_response();
        }
        let call = state.session_calls.fetch_add(1, Ordering::SeqCst);
        let mut admission_revision = state.admission.admission_revision;
        if state.change_final_session && call > 0 {
            admission_revision += 1;
        }
        Json(json!({
            "id": SESSION_ID,
            "game_key": state.admission.game_key,
            "game_version": state.admission.rules_version,
            "revision": 1,
            "status": "active",
            "state": {},
            "authority": "local",
            "provider_release_id": null,
            "availability": null,
            "presentation": {
                "format": "omarchygs.session-cartridge/v1",
                "publisher_id": state.admission.publisher_id,
                "game_key": state.admission.game_key,
                "rules_version": state.admission.rules_version,
                "cartridge_version": state.admission.cartridge_version,
                "archive_sha256": state.admission.archive_sha256,
                "signed_identity_sha256": state.admission.signed_identity_sha256,
                "admission_revision": admission_revision,
                "lifecycle_status": state.lifecycle_status,
                "active_session_policy": "continue"
            },
            "result": null,
            "participants": [],
            "completed_at": null,
            "created_at": "2026-08-26T00:00:00Z",
            "updated_at": "2026-08-26T00:00:00Z"
        }))
        .into_response()
    }

    async fn session_acquisition(
        State(state): State<FixtureState>,
        AxumPath((persona_id, session_id)): AxumPath<(String, String)>,
        headers: HeaderMap,
    ) -> Response {
        if !authorized(&headers) || persona_id != PERSONA_ID || session_id != SESSION_ID {
            return StatusCode::NOT_FOUND.into_response();
        }
        (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json")],
            Body::from(state.acquisition.as_ref().clone()),
        )
            .into_response()
    }

    fn authorized(headers: &HeaderMap) -> bool {
        headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value == format!("Bearer {BEARER}"))
    }
}
