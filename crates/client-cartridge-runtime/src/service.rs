use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zeroize::Zeroizing;

use omarchygs_game_cartridge::CatalogPublicKey;

use crate::{AcquireRequest, ClientCartridgeCache, CompanionError, MountRecord, acquire};

#[derive(Clone)]
pub struct CompanionState {
    cache: Arc<ClientCartridgeCache>,
    credential: Arc<Zeroizing<String>>,
    expected_host: Arc<str>,
    trusted_marketplace_key: Option<Arc<CatalogPublicKey>>,
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
        })
    }
}

pub fn router(state: CompanionState) -> Router {
    Router::new()
        .route("/v1/mounts/{server_id}", get(list_mounts))
        .route("/v1/acquisitions", post(install))
        .route("/v1/removals", post(remove))
        .layer(DefaultBodyLimit::max(4 * 1024))
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
            &trusted_marketplace_key,
        )
    })
    .await
    .map_err(|_| LocalError(CompanionError::Cache))?
    .map_err(LocalError)?;
    Ok(Json(RemovalResponse { removed }))
}

fn authorize(state: &CompanionState, headers: &HeaderMap) -> std::result::Result<(), LocalError> {
    let host = headers
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
        .ok_or(LocalError(CompanionError::Unauthorized))?;
    let authorization = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or(LocalError(CompanionError::Unauthorized))?;
    if host != state.expected_host.as_ref()
        || !constant_time_equal(authorization.as_bytes(), state.credential.as_bytes())
    {
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
}

#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct RemovalResponse {
    removed: bool,
}

struct LocalError(CompanionError);

impl IntoResponse for LocalError {
    fn into_response(self) -> Response {
        let status = match self.0 {
            CompanionError::Unauthorized => StatusCode::UNAUTHORIZED,
            CompanionError::InvalidInput => StatusCode::UNPROCESSABLE_ENTITY,
            CompanionError::AdmissionChanged => StatusCode::CONFLICT,
            CompanionError::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
            CompanionError::MarketplaceUntrusted => StatusCode::SERVICE_UNAVAILABLE,
            CompanionError::Rejected | CompanionError::Cache => StatusCode::INTERNAL_SERVER_ERROR,
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
}
