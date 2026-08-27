use std::{env, net::SocketAddr};

use anyhow::{Context, Result, anyhow};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use omarchy_gaming_system_server::marketplace_sync::LocalCatalogConfig;

use crate::mfa::MfaCipher;

const DEFAULT_BIND_ADDRESS: &str = "127.0.0.1:8080";
const DEFAULT_DATABASE_URL: &str =
    "postgres://omarchy_gaming_system:omarchy_gaming_system@127.0.0.1:5432/omarchy_gaming_system";
pub(crate) const DEFAULT_SERVER_NAME: &str = "OmarchyGS Community";

pub struct Config {
    pub bind_address: SocketAddr,
    pub database_url: String,
    pub server_name: String,
    pub mfa_cipher: MfaCipher,
    pub provider: Option<ProviderConfig>,
    pub cartridge_distribution: Option<LocalCatalogConfig>,
}

pub struct ProviderConfig {
    pub grant_signing_seed: [u8; 32],
    pub pairwise_secret: Vec<u8>,
    pub message_signing_seed: [u8; 32],
    pub callback_authority: String,
}

impl Config {
    pub fn from_environment() -> Result<Self> {
        let bind_address = resolve_bind_address(
            env::var("OGS_BIND_ADDRESS").ok(),
            env::var("BBS_BIND_ADDRESS").ok(),
        )?;

        let database_url =
            env::var("DATABASE_URL").unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_owned());
        let server_name = parse_server_name(env::var("OGS_SERVER_NAME").ok())?;
        let encoded_mfa_key = env::var("OGS_MFA_ENCRYPTION_KEY").context(
            "OGS_MFA_ENCRYPTION_KEY is required and must be a base64url-encoded 32-byte key",
        )?;
        let mfa_cipher = parse_mfa_cipher(&encoded_mfa_key)?;
        let provider = parse_provider_config([
            env::var("OGS_PROVIDER_GRANT_SIGNING_SEED").ok(),
            env::var("OGS_PROVIDER_PAIRWISE_SECRET").ok(),
            env::var("OGS_PROVIDER_MESSAGE_SIGNING_SEED").ok(),
            env::var("OGS_PROVIDER_CALLBACK_AUTHORITY").ok(),
        ])?;
        let cartridge_distribution = LocalCatalogConfig::optional_from_environment()
            .map_err(|_| {
                anyhow!(
                    "marketplace distribution must be absent or complete: set OGS_CARTRIDGE_STORE_ROOT with either OGS_MARKETPLACE_PUBLIC_KEY or all of OGS_MARKETPLACE_TRUST_ROOT, OGS_MARKETPLACE_TRUST_BUNDLE, and OGS_MARKETPLACE_TRUST_CHANNEL_ORIGIN"
                )
            })?;

        Ok(Self {
            bind_address,
            database_url,
            server_name,
            mfa_cipher,
            provider,
            cartridge_distribution,
        })
    }
}

fn parse_server_name(value: Option<String>) -> Result<String> {
    let value = value.unwrap_or_else(|| DEFAULT_SERVER_NAME.to_owned());
    if value.is_empty()
        || value.chars().count() > 64
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        return Err(anyhow!(
            "OGS_SERVER_NAME must be 1-64 trimmed characters without control characters"
        ));
    }
    Ok(value)
}

fn parse_provider_config(values: [Option<String>; 4]) -> Result<Option<ProviderConfig>> {
    if values.iter().all(Option::is_none) {
        return Ok(None);
    }
    if values.iter().any(Option::is_none) {
        return Err(anyhow!(
            "provider configuration is all-or-none: set grant seed, pairwise secret, message seed, and callback authority"
        ));
    }
    let [
        grant_seed,
        pairwise_secret,
        message_seed,
        callback_authority,
    ] = values.map(Option::unwrap);
    let grant_signing_seed =
        decode_exact_secret("OGS_PROVIDER_GRANT_SIGNING_SEED", &grant_seed, 32)?
            .try_into()
            .map_err(|_| anyhow!("OGS_PROVIDER_GRANT_SIGNING_SEED must decode to 32 bytes"))?;
    let pairwise_secret =
        decode_exact_secret("OGS_PROVIDER_PAIRWISE_SECRET", &pairwise_secret, 32)?;
    let message_signing_seed =
        decode_exact_secret("OGS_PROVIDER_MESSAGE_SIGNING_SEED", &message_seed, 32)?
            .try_into()
            .map_err(|_| anyhow!("OGS_PROVIDER_MESSAGE_SIGNING_SEED must decode to 32 bytes"))?;
    if !valid_callback_authority(&callback_authority) {
        return Err(anyhow!(
            "OGS_PROVIDER_CALLBACK_AUTHORITY must be a lowercase DNS authority with an optional explicit port"
        ));
    }
    Ok(Some(ProviderConfig {
        grant_signing_seed,
        pairwise_secret,
        message_signing_seed,
        callback_authority,
    }))
}

fn decode_exact_secret(name: &str, encoded: &str, length: usize) -> Result<Vec<u8>> {
    let decoded = URL_SAFE_NO_PAD
        .decode(encoded)
        .with_context(|| format!("{name} must be unpadded base64url"))?;
    if decoded.len() != length {
        return Err(anyhow!("{name} must decode to {length} bytes"));
    }
    Ok(decoded)
}

fn valid_callback_authority(value: &str) -> bool {
    if value.is_empty()
        || value.len() > 259
        || value != value.to_ascii_lowercase()
        || value
            .chars()
            .any(|character| matches!(character, '/' | '?' | '#' | '@'))
    {
        return false;
    }
    let (host, port) = match value.rsplit_once(':') {
        Some((host, port))
            if !port.is_empty() && port.bytes().all(|byte| byte.is_ascii_digit()) =>
        {
            (host, Some(port))
        }
        Some(_) => return false,
        None => (value, None),
    };
    if port.is_some_and(|port| port.parse::<u16>().ok().is_none_or(|port| port == 0)) {
        return false;
    }
    host.contains('.')
        && host.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        })
}

fn parse_mfa_cipher(encoded: &str) -> Result<MfaCipher> {
    MfaCipher::from_base64url(encoded).map_err(|_| {
        anyhow!("OGS_MFA_ENCRYPTION_KEY must be a base64url-encoded 32-byte key without padding")
    })
}

fn resolve_bind_address(
    gaming_system_address: Option<String>,
    legacy_bbs_address: Option<String>,
) -> Result<SocketAddr> {
    let (variable_name, value) = if let Some(value) = gaming_system_address {
        ("OGS_BIND_ADDRESS", value)
    } else if let Some(value) = legacy_bbs_address {
        ("BBS_BIND_ADDRESS", value)
    } else {
        ("default bind address", DEFAULT_BIND_ADDRESS.to_owned())
    };

    value
        .parse()
        .with_context(|| format!("{variable_name} must be a socket address such as 127.0.0.1:8080"))
}

#[cfg(test)]
mod tests {
    use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

    use super::{
        DEFAULT_BIND_ADDRESS, DEFAULT_DATABASE_URL, DEFAULT_SERVER_NAME, parse_mfa_cipher,
        parse_provider_config, parse_server_name, resolve_bind_address,
    };

    #[test]
    fn development_defaults_are_local_only() {
        assert!(DEFAULT_BIND_ADDRESS.starts_with("127.0.0.1:"));
        assert!(DEFAULT_DATABASE_URL.contains("@127.0.0.1:"));
        assert!(DEFAULT_DATABASE_URL.contains("omarchy_gaming_system"));
    }

    #[test]
    fn public_server_name_defaults_and_rejects_ambiguous_values() {
        assert_eq!(
            parse_server_name(None).expect("default server name should be valid"),
            DEFAULT_SERVER_NAME
        );
        assert_eq!(
            parse_server_name(Some("Arcade Friends".to_owned()))
                .expect("bounded public name should be valid"),
            "Arcade Friends"
        );
        assert!(parse_server_name(Some(String::new())).is_err());
        assert!(parse_server_name(Some(" padded".to_owned())).is_err());
        assert!(parse_server_name(Some("line\nbreak".to_owned())).is_err());
        assert!(parse_server_name(Some("x".repeat(65))).is_err());
    }

    #[test]
    fn gaming_system_bind_address_precedes_legacy_and_default_values() {
        assert_eq!(
            resolve_bind_address(
                Some("127.0.0.1:9000".to_owned()),
                Some("127.0.0.1:9001".to_owned())
            )
            .expect("new bind address should parse")
            .port(),
            9000
        );
        assert_eq!(
            resolve_bind_address(None, Some("127.0.0.1:9001".to_owned()))
                .expect("legacy bind address should remain supported")
                .port(),
            9001
        );
        assert_eq!(
            resolve_bind_address(None, None)
                .expect("default bind address should parse")
                .to_string(),
            DEFAULT_BIND_ADDRESS
        );
    }

    #[test]
    fn bind_address_errors_name_the_selected_variable() {
        let error = resolve_bind_address(
            Some("not-an-address".to_owned()),
            Some("127.0.0.1:9001".to_owned()),
        )
        .expect_err("the selected new bind address should be validated");

        assert!(error.to_string().contains("OGS_BIND_ADDRESS"));
    }

    #[test]
    fn mfa_key_parser_requires_32_base64url_bytes() {
        assert!(parse_mfa_cipher(&URL_SAFE_NO_PAD.encode([0x44_u8; 32])).is_ok());
        let invalid = parse_mfa_cipher("not-base64url")
            .err()
            .expect("invalid key should fail");
        assert!(invalid.to_string().contains("OGS_MFA_ENCRYPTION_KEY"));
        assert!(parse_mfa_cipher(&URL_SAFE_NO_PAD.encode([0x44_u8; 31])).is_err());
    }

    #[test]
    fn provider_configuration_is_absent_or_complete_and_exact() {
        assert!(
            parse_provider_config([None, None, None, None])
                .expect("absent provider configuration should work")
                .is_none()
        );
        assert!(
            parse_provider_config([
                Some(URL_SAFE_NO_PAD.encode([1_u8; 32])),
                None,
                Some(URL_SAFE_NO_PAD.encode([2_u8; 32])),
                Some("callbacks.example.test".to_owned()),
            ])
            .is_err()
        );
        let complete = parse_provider_config([
            Some(URL_SAFE_NO_PAD.encode([1_u8; 32])),
            Some(URL_SAFE_NO_PAD.encode([2_u8; 32])),
            Some(URL_SAFE_NO_PAD.encode([3_u8; 32])),
            Some("callbacks.example.test:8443".to_owned()),
        ])
        .expect("complete provider configuration should parse")
        .expect("provider should be enabled");
        assert_eq!(complete.callback_authority, "callbacks.example.test:8443");
        assert!(
            parse_provider_config([
                Some(URL_SAFE_NO_PAD.encode([1_u8; 31])),
                Some(URL_SAFE_NO_PAD.encode([2_u8; 32])),
                Some(URL_SAFE_NO_PAD.encode([3_u8; 32])),
                Some("CALLBACKS.example.test".to_owned()),
            ])
            .is_err()
        );
    }
}
