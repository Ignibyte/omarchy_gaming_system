use std::{
    fs::{File, Metadata},
    io::Read as _,
    net::SocketAddr,
    os::unix::fs::MetadataExt as _,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use anyhow::{Context as _, Result, anyhow};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use ed25519_dalek::VerifyingKey;
use omarchygs_provider_sdk::protocol::{HttpMessageSigner, ProviderOperationKind};
use omarchygs_provider_starter::{
    CallbackConfig, ProviderStarter, ProviderStarterConfig, StarterLimits,
};
use relay_forge_provider::RelayForge;
use rustix::{
    fs::{CWD, Mode, OFlags, ResolveFlags, openat2},
    process::geteuid,
};
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;
use zeroize::{Zeroize as _, Zeroizing};

const MAX_CONFIG_BYTES: usize = 64 * 1024;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    authority: String,
    bind_address: SocketAddr,
    callback_sidecar_socket: Option<SocketAddr>,
    callback_socket_override: Option<SocketAddr>,
    callback_tls_root_der_base64: String,
    callback_url: String,
    cartridge_digest: String,
    command_response_delay_ms: u64,
    database_url: String,
    platform_grant_key_id: String,
    platform_grant_public_key_base64: String,
    platform_message_key_id: String,
    platform_message_public_key_base64: String,
    provider_message_key_id: String,
    provider_message_signing_seed_base64: String,
    release_id: Uuid,
    tls_certificate: PathBuf,
    tls_private_key: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|_| anyhow!("install rustls crypto provider"))?;
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                tracing_subscriber::EnvFilter::new("relay_forge_provider=info")
            }),
        )
        .init();
    let path = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("usage: relay-forge-provider /absolute/private/config.json"))?;
    let bytes = Zeroizing::new(read_private_file(&path, MAX_CONFIG_BYTES)?);
    let mut config: Config =
        serde_json::from_slice(&bytes).context("decode exact provider config")?;
    if serde_json::to_vec(&config).context("encode exact provider config")? != bytes.as_slice() {
        return Err(anyhow!(
            "provider config must use canonical compact field order"
        ));
    }
    drop(bytes);
    let provider_seed = Zeroizing::new(decode_32(
        &config.provider_message_signing_seed_base64,
        "provider signing seed",
    )?);
    config.provider_message_signing_seed_base64.zeroize();
    let signer = HttpMessageSigner::new(&config.provider_message_key_id, *provider_seed)
        .map_err(|_| anyhow!("provider signer configuration rejected"))?;
    if config.callback_sidecar_socket.is_some() && config.callback_socket_override.is_some() {
        return Err(anyhow!(
            "sidecar and conformance callback sockets are mutually exclusive"
        ));
    }
    let callback_url = Url::parse(&config.callback_url).context("callback URL")?;
    let callback_root = decode_bounded(
        &config.callback_tls_root_der_base64,
        64,
        4096,
        "callback root",
    )?;
    let callback = match config.callback_sidecar_socket {
        Some(socket) => {
            CallbackConfig::sidecar(callback_url, callback_root, config.release_id, socket)?
        }
        None => CallbackConfig::new(
            callback_url,
            callback_root,
            config.release_id,
            config.callback_socket_override,
        )?,
    };
    let limits = StarterLimits {
        request_body_bytes: 65_536,
        operation_response_delay_after_commit: if config.command_response_delay_ms == 0 {
            None
        } else {
            Some((
                ProviderOperationKind::Command,
                Duration::from_millis(config.command_response_delay_ms),
            ))
        },
    };
    let starter_config = ProviderStarterConfig::new(
        config.release_id,
        config.authority,
        config.platform_grant_key_id,
        decode_key(
            &config.platform_grant_public_key_base64,
            "platform grant key",
        )?,
        config.platform_message_key_id,
        decode_key(
            &config.platform_message_public_key_base64,
            "platform message key",
        )?,
        signer,
        callback,
        limits,
    )
    .map_err(|_| anyhow!("provider starter configuration rejected"))?;
    let game = RelayForge::new(config.cartridge_digest);
    let starter = ProviderStarter::connect(game, starter_config, &config.database_url, 8).await;
    config.database_url.zeroize();
    let starter =
        Arc::new(starter.map_err(|_| anyhow!("provider starter database initialization failed"))?);
    starter
        .serve_tls(
            config.bind_address,
            &config.tls_certificate,
            &config.tls_private_key,
        )
        .await
}

fn decode_key(value: &str, label: &str) -> Result<VerifyingKey> {
    VerifyingKey::from_bytes(&decode_32(value, label)?).map_err(|_| anyhow!("{label} is invalid"))
}

fn decode_32(value: &str, label: &str) -> Result<[u8; 32]> {
    decode_bounded(value, 32, 32, label)?
        .try_into()
        .map_err(|_| anyhow!("{label} must be 32 bytes"))
}

fn decode_bounded(value: &str, minimum: usize, maximum: usize, label: &str) -> Result<Vec<u8>> {
    if value.len() > maximum.saturating_mul(2) {
        return Err(anyhow!("{label} is oversized"));
    }
    let bytes = URL_SAFE_NO_PAD
        .decode(value)
        .with_context(|| format!("{label} must be unpadded base64url"))?;
    if !(minimum..=maximum).contains(&bytes.len()) {
        return Err(anyhow!("{label} has invalid size"));
    }
    Ok(bytes)
}

fn read_private_file(path: &Path, limit: usize) -> Result<Vec<u8>> {
    if !path.is_absolute() {
        return Err(anyhow!("provider config path must be absolute"));
    }
    let link = std::fs::symlink_metadata(path).context("inspect provider config")?;
    if !trusted_metadata(&link, &link, limit) || link.file_type().is_symlink() {
        return Err(anyhow!("provider config must be one private regular file"));
    }
    let mut file = File::from(
        openat2(
            CWD,
            path,
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
            ResolveFlags::NO_SYMLINKS | ResolveFlags::NO_MAGICLINKS,
        )
        .context("open provider config without symlinks")?,
    );
    let opened = file.metadata().context("inspect opened provider config")?;
    if !trusted_metadata(&opened, &link, limit) {
        return Err(anyhow!("provider config changed during open"));
    }
    let mut bytes = Vec::with_capacity(usize::try_from(opened.len())?);
    (&mut file)
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .context("read provider config")?;
    let final_metadata = file.metadata().context("reinspect provider config")?;
    if bytes.is_empty() || bytes.len() > limit || !trusted_metadata(&final_metadata, &opened, limit)
    {
        return Err(anyhow!("provider config changed while reading"));
    }
    Ok(bytes)
}

fn trusted_metadata(current: &Metadata, expected: &Metadata, limit: usize) -> bool {
    current.is_file()
        && current.len() > 0
        && current.len() <= limit as u64
        && current.dev() == expected.dev()
        && current.ino() == expected.ino()
        && current.len() == expected.len()
        && current.uid() == geteuid().as_raw()
        && current.uid() == expected.uid()
        && current.gid() == expected.gid()
        && current.mode() == expected.mode()
        && current.mode() & 0o777 == 0o600
        && current.nlink() == 1
        && current.nlink() == expected.nlink()
        && current.mtime() == expected.mtime()
        && current.mtime_nsec() == expected.mtime_nsec()
        && current.ctime() == expected.ctime()
        && current.ctime_nsec() == expected.ctime_nsec()
}
