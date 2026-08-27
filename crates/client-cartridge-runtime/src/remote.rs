use std::time::Duration;

use futures_util::StreamExt as _;
use omarchygs_game_cartridge::{
    AcquisitionServerAdmission, CatalogPublicKey, MAX_ACQUISITION_DOCUMENT_BYTES,
    VerifiedAcquisition, rich_2d_host_profile, supported_sdk_identity, verify_acquisition_bytes,
};
use reqwest::{Client, Response, StatusCode, header};
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;
use zeroize::Zeroizing;

use crate::{CompanionError, MountRecord, Result};

const MAX_DISCOVERY_BYTES: usize = 16 * 1024;
const MAX_CATALOG_BYTES: usize = 256 * 1024;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AcquireRequest {
    pub server_origin: String,
    pub server_id: String,
    pub device_bearer: String,
    pub game_key: String,
    pub archive_sha256: String,
    pub admission_revision: u64,
}

pub struct RemoteAcquisition {
    pub verified: VerifiedAcquisition,
    pub mount: MountRecord,
}

pub async fn acquire(
    mut request: AcquireRequest,
    trusted_marketplace_key: &CatalogPublicKey,
) -> Result<RemoteAcquisition> {
    let origin = selected_origin(&request.server_origin)?;
    let server_id = exact_uuid(&request.server_id)?;
    if !valid_identifier(&request.game_key)
        || !valid_sha256(&request.archive_sha256)
        || request.admission_revision == 0
        || !valid_bearer(&request.device_bearer)
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
    let sdk = supported_sdk_identity().map_err(|_| CompanionError::Rejected)?;
    let verified = verify_acquisition_bytes(
        &bytes,
        &expected,
        trusted_marketplace_key,
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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DiscoveryDocument {
    service: String,
    server_id: String,
    #[allow(dead_code)]
    server_name: String,
    protocol_version: u16,
    capabilities: Vec<String>,
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
    pub marketplace: MarketplaceProvenance,
    pub server_admission: ServerAdmission,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

impl CatalogRelease {
    fn admission(&self, server_id: Uuid) -> AcquisitionServerAdmission {
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
        MarketplaceSnapshotPayload, RELEASE_ARCHIVE_PATH, RELEASE_ATTESTATION_PATH,
        RELEASE_CONFORMANCE_PATH, create_release, export_sdk, generate_catalog_keypair,
        generate_keypair, rich_2d_host_profile, sign_catalog_policy, sign_marketplace_snapshot,
        verify_release_directory,
    };
    use serde_json::{Value, json};
    use tokio::net::TcpListener;

    use super::*;
    use crate::ClientCartridgeCache;

    const REVISION: &str = "1111111111111111111111111111111111111111";
    const BUILDER_DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const SERVER_ID: &str = "11111111-1111-4111-8111-111111111111";
    const BEARER: &str = "ogs1_AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

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
                "welcome": "Welcome to Door Legends. One choice opens the way.",
                "status": "A weathered brass door waits in the dark.",
                "enter_label": "Enter the brass door"
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
        assert!(prepared.assets.is_empty());
        render_request.server_origin = "https://other.example.test".to_owned();
        assert!(matches!(
            crate::compile_mounted_render_plan(
                &cache,
                &render_request,
                &fixture.marketplace_public,
            ),
            Err(CompanionError::AdmissionChanged)
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
    }

    struct RemoteFixture {
        _temp: tempfile::TempDir,
        acquisition: Arc<Vec<u8>>,
        catalog: Value,
        marketplace_public: CatalogPublicKey,
        admission: AcquisitionServerAdmission,
    }

    impl RemoteFixture {
        fn new() -> Self {
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
            let (marketplace_private, marketplace_public) =
                generate_catalog_keypair("marketplace-primary-v1", "marketplace").unwrap();
            let policy = sign_catalog_policy(
                &release,
                &marketplace_private,
                1,
                CatalogStatus::Active,
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
                publisher_key: publisher_public,
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
                        "lifecycle_status": "active"
                    },
                    "server_admission": {"revision": admission.admission_revision}
                }]
            });
            Self {
                _temp: temp,
                acquisition: Arc::new(acquisition),
                catalog,
                marketplace_public,
                admission,
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
            }
        }

        async fn spawn(&self, change_final_catalog: bool) -> (String, tokio::task::JoinHandle<()>) {
            let state = FixtureState {
                acquisition: self.acquisition.clone(),
                catalog: self.catalog.clone(),
                catalog_calls: Arc::new(AtomicUsize::new(0)),
                change_final_catalog,
                game_key: self.admission.game_key.clone(),
                digest: self.admission.archive_sha256.clone(),
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
    }

    #[derive(Clone)]
    struct FixtureState {
        acquisition: Arc<Vec<u8>>,
        catalog: Value,
        catalog_calls: Arc<AtomicUsize>,
        change_final_catalog: bool,
        game_key: String,
        digest: String,
    }

    async fn discovery() -> Json<Value> {
        Json(json!({
            "service": "omarchy-gaming-system",
            "server_id": SERVER_ID,
            "server_name": "Remote fixture",
            "protocol_version": 1,
            "capabilities": [
                "games.cartridge-acquisition.v1",
                "games.cartridge-catalog.v1"
            ]
        }))
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

    fn authorized(headers: &HeaderMap) -> bool {
        headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value == format!("Bearer {BEARER}"))
    }
}
