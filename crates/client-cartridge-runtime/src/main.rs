use std::{
    env, fs,
    fs::OpenOptions,
    io::Write as _,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    os::unix::{fs::MetadataExt as _, fs::OpenOptionsExt as _},
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Context as _, Result, anyhow};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use omarchygs_client_cartridge_runtime::{
    ClientCartridgeCache, ClientMarketplaceTrust, ClientTrustStore, CompanionState, router,
};
use omarchygs_game_cartridge::read_catalog_public_key;
use omarchygs_marketplace_trust::read_public_channel_bootstrap;
use rand_core::{OsRng, RngCore as _};
use serde::Serialize;
use tokio::net::TcpListener;
use zeroize::Zeroizing;

#[tokio::main]
async fn main() -> Result<()> {
    let arguments = Arguments::parse()?;
    let cache = Arc::new(
        ClientCartridgeCache::open(&arguments.cache_root).map_err(|error| anyhow!(error.code()))?,
    );
    let marketplace_trust = match (
        arguments.marketplace_public_key_file.as_deref(),
        arguments.marketplace_trust_bootstrap_file.as_deref(),
    ) {
        (Some(path), None) => ClientMarketplaceTrust::Manual(Arc::new(
            read_catalog_public_key(path).context("companion_marketplace_key_invalid")?,
        )),
        (None, Some(path)) => {
            let bootstrap = read_public_channel_bootstrap(path)
                .map_err(|_| anyhow!("companion_marketplace_bootstrap_invalid"))?;
            ClientMarketplaceTrust::Channel(Arc::new(
                ClientTrustStore::open(&arguments.cache_root, bootstrap)
                    .map_err(|error| anyhow!(error.code()))?,
            ))
        }
        (None, None) => ClientMarketplaceTrust::None,
        (Some(_), Some(_)) => return Err(anyhow!("companion_invalid_arguments")),
    };
    let listener = TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0))
        .await
        .context("companion_bind_failed")?;
    let address = listener.local_addr().context("companion_bind_failed")?;
    let mut random = [0_u8; 32];
    OsRng.fill_bytes(&mut random);
    let credential = Zeroizing::new(URL_SAFE_NO_PAD.encode(random));
    random.fill(0);
    let endpoint = format!("http://{address}");
    let startup = StartupDocument {
        format: "omarchygs.cartridge-companion-startup/v1",
        endpoint: endpoint.clone(),
        credential: credential.to_string(),
        pid: std::process::id(),
    };
    write_startup_document(&arguments.startup_file, &startup)?;
    let state =
        CompanionState::new_with_trust(cache, credential, address.to_string(), marketplace_trust)
            .map_err(|error| anyhow!(error.code()))?;
    let result = axum::serve(listener, router(state))
        .with_graceful_shutdown(shutdown_signal())
        .await;
    let _ = fs::remove_file(&arguments.startup_file);
    result.context("companion_serve_failed")
}

struct Arguments {
    startup_file: PathBuf,
    cache_root: PathBuf,
    marketplace_public_key_file: Option<PathBuf>,
    marketplace_trust_bootstrap_file: Option<PathBuf>,
}

impl Arguments {
    fn parse() -> Result<Self> {
        let mut startup_file = None;
        let mut cache_root = None;
        let mut marketplace_public_key_file = None;
        let mut marketplace_trust_bootstrap_file = None;
        for value in env::args_os().skip(1) {
            let value = value
                .into_string()
                .map_err(|_| anyhow!("companion_invalid_arguments"))?;
            if let Some(path) = value.strip_prefix("--startup-file=") {
                if startup_file.replace(PathBuf::from(path)).is_some() {
                    return Err(anyhow!("companion_invalid_arguments"));
                }
            } else if let Some(path) = value.strip_prefix("--cache-root=") {
                if cache_root.replace(PathBuf::from(path)).is_some() {
                    return Err(anyhow!("companion_invalid_arguments"));
                }
            } else if let Some(path) = value.strip_prefix("--marketplace-public-key-file=") {
                if marketplace_public_key_file
                    .replace(PathBuf::from(path))
                    .is_some()
                {
                    return Err(anyhow!("companion_invalid_arguments"));
                }
            } else if let Some(path) = value.strip_prefix("--marketplace-trust-bootstrap-file=") {
                if marketplace_trust_bootstrap_file
                    .replace(PathBuf::from(path))
                    .is_some()
                {
                    return Err(anyhow!("companion_invalid_arguments"));
                }
            } else {
                return Err(anyhow!("companion_invalid_arguments"));
            }
        }
        let startup_file = startup_file.ok_or_else(|| anyhow!("companion_invalid_arguments"))?;
        let cache_root = cache_root.ok_or_else(|| anyhow!("companion_invalid_arguments"))?;
        if !startup_file.is_absolute()
            || !cache_root.is_absolute()
            || marketplace_public_key_file
                .as_ref()
                .is_some_and(|path| !path.is_absolute())
            || marketplace_trust_bootstrap_file
                .as_ref()
                .is_some_and(|path| !path.is_absolute())
        {
            return Err(anyhow!("companion_invalid_arguments"));
        }
        Ok(Self {
            startup_file,
            cache_root,
            marketplace_public_key_file,
            marketplace_trust_bootstrap_file,
        })
    }
}

#[derive(Serialize)]
struct StartupDocument {
    format: &'static str,
    endpoint: String,
    credential: String,
    pid: u32,
}

fn write_startup_document(path: &Path, document: &StartupDocument) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("companion_invalid_startup_path"))?;
    validate_private_directory(parent)?;
    let bytes = serde_json::to_vec(document).context("companion_startup_encode_failed")?;
    if bytes.len() > 1024 {
        return Err(anyhow!("companion_startup_encode_failed"));
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .context("companion_startup_create_failed")?;
    file.write_all(&bytes)
        .context("companion_startup_write_failed")?;
    file.sync_all().context("companion_startup_write_failed")?;
    Ok(())
}

fn validate_private_directory(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path).context("companion_invalid_startup_path")?;
    if !metadata.is_dir()
        || metadata.file_type().is_symlink()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.mode() & 0o777 != 0o700
    {
        return Err(anyhow!("companion_invalid_startup_path"));
    }
    Ok(())
}

async fn shutdown_signal() {
    let interrupt = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    let terminate = async {
        if let Ok(mut signal) =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            signal.recv().await;
        }
    };
    tokio::select! {
        () = interrupt => {},
        () = terminate => {},
    }
}
