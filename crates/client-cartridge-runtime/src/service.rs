use std::{
    collections::{BTreeMap, VecDeque},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use axum::{
    Json, Router,
    body::Bytes,
    extract::{DefaultBodyLimit, Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zeroize::Zeroizing;

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use omarchygs_game_cartridge::CatalogPublicKey;
use omarchygs_game_cartridge_renderer::{PreparedNavigation, RenderPlan, valid_asset_token};
use rand_core::{OsRng, RngCore as _};

use crate::{
    AcquireRequest, ClientCartridgeCache, CompanionError, MountRecord, RenderRequest,
    SessionAcquireRequest, acquire, acquire_session, compile_mounted_render_plan,
};

const MAX_RENDER_REQUEST_BYTES: usize = 512 * 1024;
const MAX_CACHED_PLANS: usize = 16;
const MAX_PLAN_ASSETS: usize = 80;
const MAX_CACHED_ASSET_BYTES: usize = 64 * 1024 * 1024;
const PLAN_LIFETIME: Duration = Duration::from_secs(5 * 60);

#[derive(Clone)]
pub struct CompanionState {
    cache: Arc<ClientCartridgeCache>,
    credential: Arc<Zeroizing<String>>,
    expected_host: Arc<str>,
    trusted_marketplace_key: Option<Arc<CatalogPublicKey>>,
    render_assets: Arc<Mutex<RenderAssetCache>>,
}

impl CompanionState {
    pub fn new(
        cache: Arc<ClientCartridgeCache>,
        credential: Zeroizing<String>,
        expected_host: String,
        trusted_marketplace_key: Option<CatalogPublicKey>,
    ) -> std::result::Result<Self, CompanionError> {
        if credential.len() < 40
            || credential.len() > 128
            || expected_host.len() < 9
            || expected_host.len() > 64
        {
            return Err(CompanionError::InvalidInput);
        }
        Ok(Self {
            cache,
            credential: Arc::new(credential),
            expected_host: Arc::from(expected_host),
            trusted_marketplace_key: trusted_marketplace_key.map(Arc::new),
            render_assets: Arc::new(Mutex::new(RenderAssetCache::default())),
        })
    }
}

pub fn router(state: CompanionState) -> Router {
    Router::new()
        .route("/v1/mounts/{server_id}", get(list_mounts))
        .route(
            "/v1/acquisitions",
            post(install).layer(DefaultBodyLimit::max(4 * 1024)),
        )
        .route(
            "/v1/session-acquisitions",
            post(install_session).layer(DefaultBodyLimit::max(4 * 1024)),
        )
        .route(
            "/v1/removals",
            post(remove).layer(DefaultBodyLimit::max(4 * 1024)),
        )
        .route(
            "/v1/render-plans",
            post(render_plan).layer(DefaultBodyLimit::max(MAX_RENDER_REQUEST_BYTES)),
        )
        .route("/v1/render-assets/{capability}/{token}", get(render_asset))
        .layer(middleware::map_response(no_store))
        .with_state(state)
}

async fn list_mounts(
    State(state): State<CompanionState>,
    Path(server_id): Path<String>,
    headers: HeaderMap,
) -> std::result::Result<Json<MountList>, LocalError> {
    authorize(&state, &headers)?;
    let server_id = exact_uuid(&server_id)?;
    let trusted_marketplace_key = state
        .trusted_marketplace_key
        .clone()
        .ok_or(LocalError(CompanionError::MarketplaceUntrusted))?;
    let cache = state.cache.clone();
    let mounts =
        tokio::task::spawn_blocking(move || cache.mounts(server_id, &trusted_marketplace_key))
            .await
            .map_err(|_| LocalError(CompanionError::Cache))?
            .map_err(LocalError)?;
    Ok(Json(MountList { mounts }))
}

async fn install(
    State(state): State<CompanionState>,
    headers: HeaderMap,
    Json(request): Json<AcquireRequest>,
) -> std::result::Result<Json<MountResponse>, LocalError> {
    authorize(&state, &headers)?;
    let trusted_marketplace_key = state
        .trusted_marketplace_key
        .as_deref()
        .ok_or(LocalError(CompanionError::MarketplaceUntrusted))?;
    let acquired = acquire(request, trusted_marketplace_key)
        .await
        .map_err(LocalError)?;
    let cache = state.cache.clone();
    let mount =
        tokio::task::spawn_blocking(move || cache.install(&acquired.verified, acquired.mount))
            .await
            .map_err(|_| LocalError(CompanionError::Cache))?
            .map_err(LocalError)?;
    Ok(Json(MountResponse { mount }))
}

async fn install_session(
    State(state): State<CompanionState>,
    headers: HeaderMap,
    Json(request): Json<SessionAcquireRequest>,
) -> std::result::Result<Json<MountResponse>, LocalError> {
    authorize(&state, &headers)?;
    let trusted_marketplace_key = state
        .trusted_marketplace_key
        .as_deref()
        .ok_or(LocalError(CompanionError::MarketplaceUntrusted))?;
    let acquired = acquire_session(request, trusted_marketplace_key)
        .await
        .map_err(LocalError)?;
    let cache = state.cache.clone();
    let mount =
        tokio::task::spawn_blocking(move || cache.install(&acquired.verified, acquired.mount))
            .await
            .map_err(|_| LocalError(CompanionError::Cache))?
            .map_err(LocalError)?;
    Ok(Json(MountResponse { mount }))
}

async fn remove(
    State(state): State<CompanionState>,
    headers: HeaderMap,
    Json(request): Json<RemoveRequest>,
) -> std::result::Result<Json<RemovalResponse>, LocalError> {
    authorize(&state, &headers)?;
    let server_id = exact_uuid(&request.server_id)?;
    let trusted_marketplace_key = state
        .trusted_marketplace_key
        .clone()
        .ok_or(LocalError(CompanionError::MarketplaceUntrusted))?;
    let cache = state.cache.clone();
    let removed = tokio::task::spawn_blocking(move || {
        cache.remove(
            server_id,
            &request.game_key,
            &request.archive_sha256,
            request.admission_revision,
            &trusted_marketplace_key,
        )
    })
    .await
    .map_err(|_| LocalError(CompanionError::Cache))?
    .map_err(LocalError)?;
    Ok(Json(RemovalResponse { removed }))
}

async fn render_plan(
    State(state): State<CompanionState>,
    headers: HeaderMap,
    Json(request): Json<RenderRequest>,
) -> std::result::Result<Json<RenderPlanResponse>, LocalError> {
    authorize(&state, &headers)?;
    let trusted_marketplace_key = state
        .trusted_marketplace_key
        .clone()
        .ok_or(LocalError(CompanionError::MarketplaceUntrusted))?;
    let cache = state.cache.clone();
    let prepared = tokio::task::spawn_blocking(move || {
        compile_mounted_render_plan(&cache, &request, &trusted_marketplace_key)
    })
    .await
    .map_err(|_| LocalError(CompanionError::Cache))?
    .map_err(LocalError)?;
    let capability = state
        .render_assets
        .lock()
        .map_err(|_| LocalError(CompanionError::Cache))?
        .insert(prepared.assets)
        .map_err(LocalError)?;
    Ok(Json(RenderPlanResponse {
        format: "omarchygs.session-cartridge-render/v2",
        screen_id: prepared.screen_id,
        entry_screen_id: prepared.entry_screen_id,
        navigation: prepared.navigation,
        plan: prepared.plan,
        asset_base_url: format!(
            "http://{}/v1/render-assets/{capability}",
            state.expected_host
        ),
    }))
}

async fn render_asset(
    State(state): State<CompanionState>,
    Path((capability, token)): Path<(String, String)>,
    headers: HeaderMap,
) -> std::result::Result<Response, LocalError> {
    authorize_host(&state, &headers)?;
    if !valid_capability(&capability) || !valid_asset_token(&token) {
        return Err(LocalError(CompanionError::Unauthorized));
    }
    let bytes = state
        .render_assets
        .lock()
        .map_err(|_| LocalError(CompanionError::Cache))?
        .get(&capability, &token)
        .ok_or(LocalError(CompanionError::Unauthorized))?;
    let content_type = if token.ends_with(".png") {
        "image/png"
    } else if token.ends_with(".wav") {
        "audio/wav"
    } else {
        return Err(LocalError(CompanionError::Unauthorized));
    };
    Ok(([(header::CONTENT_TYPE, content_type)], bytes).into_response())
}

fn authorize(state: &CompanionState, headers: &HeaderMap) -> std::result::Result<(), LocalError> {
    authorize_host(state, headers)?;
    let authorization = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or(LocalError(CompanionError::Unauthorized))?;
    if !constant_time_equal(authorization.as_bytes(), state.credential.as_bytes()) {
        return Err(LocalError(CompanionError::Unauthorized));
    }
    Ok(())
}

fn authorize_host(
    state: &CompanionState,
    headers: &HeaderMap,
) -> std::result::Result<(), LocalError> {
    let host = headers
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
        .ok_or(LocalError(CompanionError::Unauthorized))?;
    if host != state.expected_host.as_ref() {
        return Err(LocalError(CompanionError::Unauthorized));
    }
    Ok(())
}

fn constant_time_equal(left: &[u8], right: &[u8]) -> bool {
    let mut difference = left.len() ^ right.len();
    let maximum = left.len().max(right.len());
    for index in 0..maximum {
        let left_byte = left.get(index).copied().unwrap_or(0);
        let right_byte = right.get(index).copied().unwrap_or(0);
        difference |= usize::from(left_byte ^ right_byte);
    }
    difference == 0
}

fn exact_uuid(value: &str) -> std::result::Result<Uuid, LocalError> {
    let parsed = Uuid::try_parse(value).map_err(|_| LocalError(CompanionError::InvalidInput))?;
    if parsed.is_nil() || parsed.to_string() != value {
        Err(LocalError(CompanionError::InvalidInput))
    } else {
        Ok(parsed)
    }
}

async fn no_store(mut response: Response) -> Response {
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response.headers_mut().insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    response
}

#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct MountList {
    mounts: Vec<MountRecord>,
}

#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct MountResponse {
    mount: MountRecord,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RemoveRequest {
    server_id: String,
    game_key: String,
    archive_sha256: String,
    #[serde(default)]
    admission_revision: Option<u64>,
}

#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct RemovalResponse {
    removed: bool,
}

#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct RenderPlanResponse {
    format: &'static str,
    screen_id: String,
    entry_screen_id: String,
    navigation: Vec<PreparedNavigation>,
    plan: RenderPlan,
    asset_base_url: String,
}

#[derive(Default)]
struct RenderAssetCache {
    plans: VecDeque<CachedRenderAssets>,
    total_bytes: usize,
}

struct CachedRenderAssets {
    capability: String,
    created_at: Instant,
    bytes: usize,
    assets: BTreeMap<String, Bytes>,
}

impl RenderAssetCache {
    fn insert(&mut self, assets: BTreeMap<String, Vec<u8>>) -> crate::Result<String> {
        if assets.len() > MAX_PLAN_ASSETS || !assets.keys().all(|token| valid_asset_token(token)) {
            return Err(CompanionError::Render);
        }
        let bytes = assets.values().try_fold(0usize, |total, asset| {
            total.checked_add(asset.len()).ok_or(CompanionError::Render)
        })?;
        if bytes > MAX_CACHED_ASSET_BYTES {
            return Err(CompanionError::Render);
        }
        let assets = assets
            .into_iter()
            .map(|(token, bytes)| (token, Bytes::from(bytes)))
            .collect();
        self.prune_expired();
        while self.plans.len() >= MAX_CACHED_PLANS
            || self.total_bytes.saturating_add(bytes) > MAX_CACHED_ASSET_BYTES
        {
            self.evict_oldest();
        }
        let capability = loop {
            let mut random = [0_u8; 32];
            OsRng.fill_bytes(&mut random);
            let candidate = URL_SAFE_NO_PAD.encode(random);
            if !self.plans.iter().any(|plan| plan.capability == candidate) {
                break candidate;
            }
        };
        self.total_bytes += bytes;
        self.plans.push_back(CachedRenderAssets {
            capability: capability.clone(),
            created_at: Instant::now(),
            bytes,
            assets,
        });
        Ok(capability)
    }

    fn get(&mut self, capability: &str, token: &str) -> Option<Bytes> {
        self.prune_expired();
        self.plans
            .iter()
            .find(|plan| plan.capability == capability)
            .and_then(|plan| plan.assets.get(token))
            .cloned()
    }

    fn prune_expired(&mut self) {
        while self
            .plans
            .front()
            .is_some_and(|plan| plan.created_at.elapsed() >= PLAN_LIFETIME)
        {
            self.evict_oldest();
        }
    }

    fn evict_oldest(&mut self) {
        if let Some(plan) = self.plans.pop_front() {
            self.total_bytes = self.total_bytes.saturating_sub(plan.bytes);
        }
    }
}

fn valid_capability(value: &str) -> bool {
    value.len() == 43
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

struct LocalError(CompanionError);

impl IntoResponse for LocalError {
    fn into_response(self) -> Response {
        let status = match self.0 {
            CompanionError::Unauthorized => StatusCode::UNAUTHORIZED,
            CompanionError::InvalidInput => StatusCode::UNPROCESSABLE_ENTITY,
            CompanionError::AdmissionChanged => StatusCode::CONFLICT,
            CompanionError::MountMissing => StatusCode::NOT_FOUND,
            CompanionError::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
            CompanionError::MarketplaceUntrusted => StatusCode::SERVICE_UNAVAILABLE,
            CompanionError::Rejected | CompanionError::Cache | CompanionError::Render => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };
        (
            status,
            Json(ErrorEnvelope {
                error: ErrorBody {
                    code: self.0.code(),
                },
            }),
        )
            .into_response()
    }
}

#[derive(Serialize)]
struct ErrorEnvelope {
    error: ErrorBody,
}

#[derive(Serialize)]
struct ErrorBody {
    code: &'static str,
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use omarchygs_game_cartridge::generate_catalog_keypair;
    use tower::ServiceExt as _;

    use super::*;

    #[tokio::test]
    async fn loopback_api_requires_exact_host_and_credential() {
        let temp = tempfile::tempdir().expect("temp should create");
        let cache = Arc::new(
            ClientCartridgeCache::open(&temp.path().join("cache")).expect("cache should open"),
        );
        let (_, marketplace_public) =
            generate_catalog_keypair("marketplace-primary-v1", "marketplace")
                .expect("key should generate");
        let state = CompanionState::new(
            cache,
            Zeroizing::new("A".repeat(43)),
            "127.0.0.1:32123".to_owned(),
            Some(marketplace_public),
        )
        .expect("state should construct");
        let app = router(state);
        let path = "/v1/mounts/00000000-0000-0000-0000-000000000001";

        let missing = app
            .clone()
            .oneshot(
                Request::get(path)
                    .header("host", "127.0.0.1:32123")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");
        assert_eq!(missing.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(missing.headers().get("cache-control").unwrap(), "no-store");

        let wrong_host = app
            .clone()
            .oneshot(
                Request::get(path)
                    .header("host", "localhost:32123")
                    .header("authorization", format!("Bearer {}", "A".repeat(43)))
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");
        assert_eq!(wrong_host.status(), StatusCode::UNAUTHORIZED);

        let accepted = app
            .oneshot(
                Request::get(path)
                    .header("host", "127.0.0.1:32123")
                    .header("authorization", format!("Bearer {}", "A".repeat(43)))
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");
        assert_eq!(accepted.status(), StatusCode::OK);
        assert_eq!(accepted.headers().get("cache-control").unwrap(), "no-store");
    }

    #[tokio::test]
    async fn acquisition_fails_closed_without_client_marketplace_trust() {
        let temp = tempfile::tempdir().expect("temp should create");
        let cache = Arc::new(
            ClientCartridgeCache::open(&temp.path().join("cache")).expect("cache should open"),
        );
        let state = CompanionState::new(
            cache,
            Zeroizing::new("A".repeat(43)),
            "127.0.0.1:32123".to_owned(),
            None,
        )
        .expect("state should construct");
        let response = router(state)
            .oneshot(
                Request::post("/v1/acquisitions")
                    .header("host", "127.0.0.1:32123")
                    .header("authorization", format!("Bearer {}", "A".repeat(43)))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({
                            "server_origin": "https://games.example.test",
                            "server_id": "00000000-0000-0000-0000-000000000001",
                            "device_bearer": "ogs1_AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
                            "game_key": "test-game",
                            "archive_sha256": "a".repeat(64),
                            "admission_revision": 1
                        })
                        .to_string(),
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(response.headers().get("cache-control").unwrap(), "no-store");
    }

    #[tokio::test]
    async fn render_assets_require_exact_host_capability_and_digest_token() {
        let temp = tempfile::tempdir().expect("temp should create");
        let cache = Arc::new(
            ClientCartridgeCache::open(&temp.path().join("cache")).expect("cache should open"),
        );
        let (_, marketplace_public) =
            generate_catalog_keypair("marketplace-primary-v1", "marketplace")
                .expect("key should generate");
        let state = CompanionState::new(
            cache,
            Zeroizing::new("A".repeat(43)),
            "127.0.0.1:32123".to_owned(),
            Some(marketplace_public),
        )
        .expect("state should construct");
        let token = format!("{}.png", "a".repeat(64));
        let capability = state
            .render_assets
            .lock()
            .expect("cache should lock")
            .insert(BTreeMap::from([(token.clone(), b"png-bytes".to_vec())]))
            .expect("asset should cache");
        let app = router(state);
        let path = format!("/v1/render-assets/{capability}/{token}");

        let wrong_host = app
            .clone()
            .oneshot(
                Request::get(&path)
                    .header("host", "localhost:32123")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");
        assert_eq!(wrong_host.status(), StatusCode::UNAUTHORIZED);

        let wrong_capability = app
            .clone()
            .oneshot(
                Request::get(format!("/v1/render-assets/{}/{}", "B".repeat(43), token))
                    .header("host", "127.0.0.1:32123")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");
        assert_eq!(wrong_capability.status(), StatusCode::UNAUTHORIZED);

        let accepted = app
            .oneshot(
                Request::get(&path)
                    .header("host", "127.0.0.1:32123")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");
        assert_eq!(accepted.status(), StatusCode::OK);
        assert_eq!(accepted.headers().get("content-type").unwrap(), "image/png");
        assert_eq!(
            accepted.headers().get("x-content-type-options").unwrap(),
            "nosniff"
        );
    }

    #[test]
    fn render_asset_cache_is_count_and_memory_bounded() {
        let mut cache = RenderAssetCache::default();
        let token = format!("{}.png", "a".repeat(64));
        let first = cache
            .insert(BTreeMap::from([(token.clone(), vec![1])]))
            .expect("first plan should cache");
        for byte in 2..=MAX_CACHED_PLANS as u8 + 1 {
            cache
                .insert(BTreeMap::from([(token.clone(), vec![byte])]))
                .expect("bounded plan should cache");
        }
        assert_eq!(cache.plans.len(), MAX_CACHED_PLANS);
        assert!(cache.get(&first, &token).is_none());
        assert!(
            cache
                .insert(BTreeMap::from([(
                    token,
                    vec![0; MAX_CACHED_ASSET_BYTES + 1],
                )]))
                .is_err()
        );
    }
}
