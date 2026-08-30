use std::{
    fs::{File, Metadata},
    io::Read as _,
    net::SocketAddr,
    os::unix::fs::MetadataExt as _,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Context as _, Result, anyhow};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use ed25519_dalek::VerifyingKey;
use omarchygs_provider_conformance::{
    CallbackSink, CallbackSinkConfig, ConformanceTarget, run_conformance,
};
use rustix::{
    fs::{CWD, Mode, OFlags, ResolveFlags, openat2},
    process::geteuid,
};
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;
use zeroize::{Zeroize as _, Zeroizing};

const MAX_CONFIG_BYTES: usize = 64 * 1024;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConformanceCliConfig {
    authority: String,
    callback_authority: String,
    callback_bind_address: SocketAddr,
    callback_certificate_pem: PathBuf,
    callback_path: String,
    callback_private_key_pem: PathBuf,
    cartridge_digest: String,
    endpoint: String,
    game_key: String,
    normal_timeout_ms: u64,
    pairwise_secret_base64: String,
    platform_grant_key_id: String,
    platform_grant_seed_base64: String,
    platform_message_key_id: String,
    platform_message_seed_base64: String,
    provider_id: String,
    provider_message_key_id: String,
    provider_message_public_key_base64: String,
    provider_root_der_base64: String,
    provider_socket_override: SocketAddr,
    release_id: Uuid,
    rules_version: u32,
    subject: String,
    unknown_outcome_timeout_ms: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|_| anyhow!("install rustls crypto provider"))?;
    let path = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or_else(|| {
            anyhow!("usage: omarchygs-provider-conformance /absolute/private/config.json")
        })?;
    let bytes = Zeroizing::new(read_private_file(&path, MAX_CONFIG_BYTES)?);
    let mut config: ConformanceCliConfig =
        serde_json::from_slice(&bytes).context("decode exact conformance config")?;
    if serde_json::to_vec(&config).context("encode exact conformance config")? != bytes.as_slice() {
        return Err(anyhow!(
            "conformance config must use canonical compact field order"
        ));
    }
    drop(bytes);
    let grant_seed = Zeroizing::new(decode_32(&config.platform_grant_seed_base64, "grant seed")?);
    let message_seed = Zeroizing::new(decode_32(
        &config.platform_message_seed_base64,
        "message seed",
    )?);
    let pairwise = Zeroizing::new(decode_bounded(
        &config.pairwise_secret_base64,
        32,
        128,
        "pairwise secret",
    )?);
    config.platform_grant_seed_base64.zeroize();
    config.platform_message_seed_base64.zeroize();
    config.pairwise_secret_base64.zeroize();
    let provider_key = VerifyingKey::from_bytes(&decode_32(
        &config.provider_message_public_key_base64,
        "provider message key",
    )?)
    .map_err(|_| anyhow!("provider message key rejected"))?;
    let target = ConformanceTarget::new(
        Url::parse(&config.endpoint).context("provider endpoint")?,
        config.provider_socket_override,
        config.authority,
        decode_bounded(&config.provider_root_der_base64, 64, 4096, "provider root")?,
        config.provider_id.clone(),
        config.release_id,
        config.game_key.clone(),
        config.rules_version,
        config.cartridge_digest.clone(),
        config.subject.clone(),
        config.provider_message_key_id.clone(),
        provider_key,
        &config.platform_grant_key_id,
        *grant_seed,
        pairwise.to_vec(),
        &config.platform_message_key_id,
        *message_seed,
        Duration::from_millis(config.normal_timeout_ms),
        Duration::from_millis(config.unknown_outcome_timeout_ms),
    )?;
    let callback = CallbackSink::start(CallbackSinkConfig {
        bind_address: config.callback_bind_address,
        authority: config.callback_authority,
        path: config.callback_path,
        provider_id: config.provider_id,
        release_id: config.release_id,
        provider_message_key_id: config.provider_message_key_id,
        game_key: config.game_key,
        rules_version: config.rules_version,
        cartridge_digest: config.cartridge_digest,
        subject: config.subject,
        provider_message_key: target.provider_message_key(),
        certificate_pem: config.callback_certificate_pem,
        private_key_pem: config.callback_private_key_pem,
    })
    .await?;
    let receipt = run_conformance(&target, &callback).await;
    let stopped = callback.stop().await;
    let receipt = receipt?;
    stopped?;
    println!(
        "{}",
        serde_json::to_string(&receipt).context("encode conformance receipt")?
    );
    Ok(())
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
        return Err(anyhow!("conformance config path must be absolute"));
    }
    let link = std::fs::symlink_metadata(path).context("inspect conformance config")?;
    if !trusted_metadata(&link, &link, limit) || link.file_type().is_symlink() {
        return Err(anyhow!(
            "conformance config must be one private regular file"
        ));
    }
    let mut file = File::from(
        openat2(
            CWD,
            path,
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
            ResolveFlags::NO_SYMLINKS | ResolveFlags::NO_MAGICLINKS,
        )
        .context("open conformance config without symlinks")?,
    );
    let opened = file
        .metadata()
        .context("inspect opened conformance config")?;
    if !trusted_metadata(&opened, &link, limit) {
        return Err(anyhow!("conformance config changed during open"));
    }
    let mut bytes = Vec::with_capacity(usize::try_from(opened.len())?);
    (&mut file)
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .context("read conformance config")?;
    let final_metadata = file.metadata().context("reinspect conformance config")?;
    if bytes.is_empty() || bytes.len() > limit || !trusted_metadata(&final_metadata, &opened, limit)
    {
        return Err(anyhow!("conformance config changed while reading"));
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
