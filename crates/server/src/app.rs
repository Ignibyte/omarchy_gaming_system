use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use serde::Serialize;
use sqlx::PgPool;
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct AppState {
    pool: PgPool,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct HealthResponse {
    service: &'static str,
    version: &'static str,
    status: &'static str,
    database: &'static str,
}

pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/health", get(health))
        .with_state(AppState { pool })
        .layer(TraceLayer::new_for_http())
}

async fn health(State(state): State<AppState>) -> (StatusCode, Json<HealthResponse>) {
    match sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.pool)
        .await
    {
        Ok(1) => (StatusCode::OK, Json(health_document(true))),
        Ok(_) | Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(health_document(false)),
        ),
    }
}

fn health_document(database_ok: bool) -> HealthResponse {
    HealthResponse {
        service: "omarchy-bbs",
        version: env!("CARGO_PKG_VERSION"),
        status: if database_ok { "ok" } else { "degraded" },
        database: if database_ok { "ok" } else { "unavailable" },
    }
}

#[cfg(test)]
mod tests {
    use super::{HealthResponse, health_document};

    #[test]
    fn healthy_document_reports_service_and_database() {
        assert_eq!(
            health_document(true),
            HealthResponse {
                service: "omarchy-bbs",
                version: env!("CARGO_PKG_VERSION"),
                status: "ok",
                database: "ok",
            }
        );
    }

    #[test]
    fn degraded_document_reports_database_failure() {
        let document = health_document(false);

        assert_eq!(document.status, "degraded");
        assert_eq!(document.database, "unavailable");
    }
}
