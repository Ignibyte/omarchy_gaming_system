use std::{env, fs, path::Path};

use anyhow::{Context as _, Result, anyhow};
use omarchygs_marketplace_trust::{
    MarketplaceTrustPayload, PublicChannelBootstrap, generate_trust_root_keypair,
    read_public_channel_bootstrap, read_trust_root_private_key, read_trust_root_public_key,
    sign_marketplace_trust, signed_trust_bytes, verify_marketplace_trust_bytes,
    write_new_private_key, write_new_public_channel_bootstrap, write_new_public_key,
    write_new_signed_trust,
};

fn main() -> Result<()> {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    match arguments.as_slice() {
        [command, key_id, channel_id, private_path, public_path] if command == "generate-root" => {
            require_absolute(private_path)?;
            let (private, public) = generate_trust_root_keypair(key_id, channel_id)
                .map_err(|_| anyhow!("marketplace_channel_invalid_input"))?;
            write_new_private_key(Path::new(private_path), &private)
                .map_err(|_| anyhow!("marketplace_channel_write_failed"))?;
            write_new_public_key(Path::new(public_path), &public)
                .map_err(|_| anyhow!("marketplace_channel_write_failed"))?;
            println!(
                "root_public_sha256={}",
                sha256(&serde_json::to_vec(&public)?)
            );
        }
        [command, payload_path, private_path, output_path] if command == "sign" => {
            require_absolute(private_path)?;
            let payload_bytes =
                fs::read(payload_path).context("marketplace_channel_read_failed")?;
            let payload: MarketplaceTrustPayload = serde_json::from_slice(&payload_bytes)
                .context("marketplace_channel_invalid_input")?;
            if serde_json::to_vec(&payload)? != payload_bytes {
                return Err(anyhow!("marketplace_channel_invalid_input"));
            }
            let private = read_trust_root_private_key(Path::new(private_path))
                .map_err(|_| anyhow!("marketplace_channel_invalid_key"))?;
            let signed = sign_marketplace_trust(&payload, &private)
                .map_err(|_| anyhow!("marketplace_channel_invalid_input"))?;
            write_new_signed_trust(Path::new(output_path), &signed)
                .map_err(|_| anyhow!("marketplace_channel_write_failed"))?;
            println!("trust_sha256={}", sha256(&signed_trust_bytes(&signed)?));
        }
        [
            command,
            signed_path,
            public_path,
            channel_id,
            channel_origin,
            now,
        ] if command == "verify" => {
            let now = now
                .parse::<u64>()
                .map_err(|_| anyhow!("marketplace_channel_invalid_input"))?;
            let bytes = fs::read(signed_path).context("marketplace_channel_read_failed")?;
            let public = read_trust_root_public_key(Path::new(public_path))
                .map_err(|_| anyhow!("marketplace_channel_invalid_key"))?;
            let trust =
                verify_marketplace_trust_bytes(&bytes, &public, channel_id, channel_origin, now)
                    .map_err(|_| anyhow!("marketplace_channel_rejected"))?;
            println!(
                "channel_id={} bundle_version={} trust_sha256={}",
                trust.payload().channel_id,
                trust.payload().bundle_version,
                sha256(trust.signed_bytes())
            );
        }
        [
            command,
            public_path,
            channel_origin,
            manifest_path,
            minimum_bundle_version,
            minimum_current_snapshot_version,
            platform,
            architecture,
            package_version,
            output_path,
        ] if command == "bootstrap" => {
            let minimum_bundle_version = minimum_bundle_version
                .parse::<u64>()
                .map_err(|_| anyhow!("marketplace_channel_invalid_input"))?;
            let minimum_current_snapshot_version = minimum_current_snapshot_version
                .parse::<u64>()
                .map_err(|_| anyhow!("marketplace_channel_invalid_input"))?;
            let root = read_trust_root_public_key(Path::new(public_path))
                .map_err(|_| anyhow!("marketplace_channel_invalid_key"))?;
            let bootstrap = PublicChannelBootstrap {
                format: "omarchygs.public-channel-bootstrap/v1".to_owned(),
                channel_id: root.channel_id.clone(),
                channel_origin: channel_origin.clone(),
                manifest_path: manifest_path.clone(),
                minimum_bundle_version,
                minimum_current_snapshot_version,
                platform: platform.clone(),
                architecture: architecture.clone(),
                installed_package_version: package_version.clone(),
                root,
            };
            write_new_public_channel_bootstrap(Path::new(output_path), &bootstrap)
                .map_err(|_| anyhow!("marketplace_channel_write_failed"))?;
            println!(
                "bootstrap_sha256={}",
                sha256(&serde_json::to_vec(&bootstrap)?)
            );
        }
        [command, bootstrap_path] if command == "verify-bootstrap" => {
            let bootstrap = read_public_channel_bootstrap(Path::new(bootstrap_path))
                .map_err(|_| anyhow!("marketplace_channel_invalid_input"))?;
            println!(
                "channel_id={} bootstrap_sha256={}",
                bootstrap.channel_id,
                sha256(&serde_json::to_vec(&bootstrap)?)
            );
        }
        _ => {
            return Err(anyhow!(
                "usage: omarchygs-marketplace-channel generate-root KEY_ID CHANNEL_ID PRIVATE PUBLIC | sign PAYLOAD PRIVATE OUTPUT | verify SIGNED PUBLIC CHANNEL_ID CHANNEL_ORIGIN NOW_UNIX | bootstrap PUBLIC CHANNEL_ORIGIN MANIFEST_PATH MINIMUM_BUNDLE_VERSION MINIMUM_CURRENT_SNAPSHOT_VERSION PLATFORM ARCHITECTURE PACKAGE_VERSION OUTPUT | verify-bootstrap BOOTSTRAP"
            ));
        }
    }
    Ok(())
}

fn sha256(bytes: &[u8]) -> String {
    use sha2::{Digest as _, Sha256};
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn require_absolute(path: &str) -> Result<()> {
    if Path::new(path).is_absolute() {
        Ok(())
    } else {
        Err(anyhow!("marketplace_channel_invalid_input"))
    }
}
