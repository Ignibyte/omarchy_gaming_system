mod accounts;
mod app;
mod config;
mod connections;
mod credentials;
mod games;
mod inboxes;
mod mfa;
mod personas;
mod sessions;
mod sync;

#[cfg(test)]
mod connection_api_tests;
#[cfg(test)]
mod game_api_tests;
#[cfg(test)]
mod inbox_api_tests;
#[cfg(test)]
mod mfa_api_tests;
#[cfg(test)]
mod persona_api_tests;
#[cfg(test)]
mod session_api_tests;
#[cfg(test)]
mod sync_api_tests;

use anyhow::{Context, Result};
use config::Config;
use omarchy_game_runtime::GameRegistry;
use sqlx::{PgPool, migrate::Migrator, postgres::PgPoolOptions};
use tokio::{net::TcpListener, signal};
use tracing::info;
use tracing_subscriber::EnvFilter;

static MIGRATOR: Migrator = sqlx::migrate!("../../migrations");

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let config = Config::from_environment()?;
    let pool = connect_database(&config.database_url).await?;
    MIGRATOR
        .run(&pool)
        .await
        .context("failed to run database migrations")?;

    let listener = TcpListener::bind(config.bind_address)
        .await
        .with_context(|| format!("failed to bind server to {}", config.bind_address))?;

    let sync_hub = sync::SyncHub::new();
    let sync_listener = sync::start_postgres_listener(&pool, sync_hub.clone())
        .await
        .context("failed to start persona sync listener")?;

    info!(address = %config.bind_address, "Omarchy Gaming System server listening");

    let server_result = axum::serve(
        listener,
        app::router_with_runtime(pool, config.mfa_cipher, sync_hub, GameRegistry::empty()),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await;
    sync_listener.abort();
    server_result.context("HTTP server stopped unexpectedly")
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("omarchy_gaming_system_server=info,tower_http=info"));

    tracing_subscriber::fmt().with_env_filter(filter).init();
}

async fn connect_database(database_url: &str) -> Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .context("failed to connect to PostgreSQL")
}

async fn shutdown_signal() {
    let interrupt = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = interrupt => {},
        () = terminate => {},
    }

    info!("shutdown signal received");
}
