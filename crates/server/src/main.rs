mod accounts;
mod app;
mod challenges;
mod config;
mod connections;
mod credentials;
mod games;
mod inboxes;
mod mfa;
mod personas;
mod provider_games;
mod registration_invites;
mod reports;
mod server_discovery;
mod sessions;
mod sync;

#[cfg(test)]
mod server_modules;
#[cfg(not(test))]
pub(crate) use omarchy_gaming_system_server::server_modules;

#[cfg(test)]
mod cartridge_catalog_api_tests;
#[cfg(test)]
mod challenge_api_tests;
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
mod provider_game_api_tests;
#[cfg(test)]
mod registration_api_tests;
#[cfg(test)]
mod report_api_tests;
#[cfg(test)]
mod server_discovery_api_tests;
#[cfg(test)]
mod server_module_tests;
#[cfg(test)]
mod session_api_tests;
#[cfg(test)]
mod signal_siege_api_tests;
#[cfg(test)]
mod sync_api_tests;

use std::{future::Future, sync::Arc};

use anyhow::{Context, Result, anyhow};
use config::Config;
use omarchy_game_runtime::{GameDefinition, GameRegistry};
use omarchy_game_signal_siege::{SignalSiege, SignalSiegeVersus};
use omarchy_gaming_system_server::cartridge_distribution::CartridgeDistributionRuntime;
use sqlx::{PgPool, migrate::Migrator, postgres::PgPoolOptions};
use tokio::{net::TcpListener, signal};
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

static MIGRATOR: Migrator = sqlx::migrate!("../../migrations");

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let config = Config::from_environment()?;
    let game_registry = production_game_registry()?;
    let pool = connect_database(&config.database_url).await?;
    MIGRATOR
        .run(&pool)
        .await
        .context("failed to run database migrations")?;
    let provider_runtime = config
        .provider
        .map(|provider| provider_games::ProviderRuntime::production(pool.clone(), provider))
        .transpose()
        .map_err(|error| anyhow!("invalid provider runtime: {}", error.code()))?;
    let cartridge_distribution = CartridgeDistributionRuntime::from_configs(
        &pool,
        config.cartridge_distribution.as_ref(),
        config.operator_custom.as_ref(),
    )
    .await
    .map_err(|error| anyhow!("invalid cartridge distribution runtime: {}", error.code()))?;
    let (module_service, module_emitter) = match config.module {
        Some(module) => {
            let emitter = server_modules::ModuleEmitter::configured(&module);
            let service = optional_module_service(
                server_modules::ServerModuleService::production(pool.clone(), module).await,
            )?;
            (service, Some(emitter))
        }
        None => (None, None),
    };
    let module_shutdown = module_service
        .as_ref()
        .map(server_modules::ServerModuleService::shutdown_trigger);

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
        app::router_with_application_runtimes(
            pool,
            config.mfa_cipher,
            sync_hub,
            game_registry,
            app::ApplicationRuntimes {
                provider: provider_runtime,
                cartridge_distribution,
                module_emitter,
            },
            Arc::from(config.server_name),
        ),
    )
    .with_graceful_shutdown(fan_out_shutdown(shutdown_signal(), move || {
        if let Some(trigger) = module_shutdown {
            trigger.request();
        }
    }))
    .await;
    sync_listener.abort();
    if let Some(service) = module_service {
        service.shutdown().await;
    }
    server_result.context("HTTP server stopped unexpectedly")
}

fn optional_module_service<T>(
    result: std::result::Result<T, server_modules::ModuleError>,
) -> Result<Option<T>> {
    match result {
        Ok(service) => Ok(Some(service)),
        Err(server_modules::ModuleError::Denied) => {
            warn!(
                "configured server module is inactive; core service will continue without module execution"
            );
            Ok(None)
        }
        Err(error) => Err(anyhow!("invalid server module runtime: {}", error.code())),
    }
}

async fn fan_out_shutdown<F, G>(signal: F, stop_module: G)
where
    F: Future<Output = ()>,
    G: FnOnce(),
{
    signal.await;
    stop_module();
}

pub(crate) fn production_game_registry() -> Result<GameRegistry> {
    GameRegistry::new([
        Arc::new(SignalSiege) as Arc<dyn GameDefinition>,
        Arc::new(SignalSiegeVersus) as Arc<dyn GameDefinition>,
    ])
    .map_err(|error| anyhow!("invalid production game registry: {error:?}"))
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

#[cfg(test)]
mod shutdown_tests {
    use std::time::Duration;

    use tokio::sync::oneshot;

    use super::*;

    #[tokio::test]
    async fn module_shutdown_is_signalled_when_http_graceful_shutdown_begins() {
        let (signal_sender, signal_receiver) = oneshot::channel();
        let (module_sender, mut module_receiver) = oneshot::channel();
        let fanout = tokio::spawn(fan_out_shutdown(
            async move {
                let _ = signal_receiver.await;
            },
            move || {
                let _ = module_sender.send(());
            },
        ));

        assert!(
            tokio::time::timeout(Duration::from_millis(10), &mut module_receiver)
                .await
                .is_err()
        );
        signal_sender.send(()).expect("signal should be observed");
        fanout.await.expect("fanout task should finish");
        tokio::time::timeout(Duration::from_secs(1), module_receiver)
            .await
            .expect("module shutdown should be prompt")
            .expect("module shutdown sender should fire");
    }

    #[test]
    fn inactive_module_does_not_prevent_core_startup() {
        assert!(
            optional_module_service::<()>(Err(server_modules::ModuleError::Denied))
                .expect("inactive module should be optional")
                .is_none()
        );
        assert!(
            optional_module_service::<()>(Err(server_modules::ModuleError::Unavailable)).is_err()
        );
    }
}
