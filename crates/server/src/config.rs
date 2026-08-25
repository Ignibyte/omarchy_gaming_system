use std::{env, net::SocketAddr};

use anyhow::{Context, Result, anyhow};

use crate::mfa::MfaCipher;

const DEFAULT_BIND_ADDRESS: &str = "127.0.0.1:8080";
const DEFAULT_DATABASE_URL: &str =
    "postgres://omarchy_gaming_system:omarchy_gaming_system@127.0.0.1:5432/omarchy_gaming_system";

#[derive(Clone)]
pub struct Config {
    pub bind_address: SocketAddr,
    pub database_url: String,
    pub mfa_cipher: MfaCipher,
}

impl Config {
    pub fn from_environment() -> Result<Self> {
        let bind_address = resolve_bind_address(
            env::var("OGS_BIND_ADDRESS").ok(),
            env::var("BBS_BIND_ADDRESS").ok(),
        )?;

        let database_url =
            env::var("DATABASE_URL").unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_owned());
        let encoded_mfa_key = env::var("OGS_MFA_ENCRYPTION_KEY").context(
            "OGS_MFA_ENCRYPTION_KEY is required and must be a base64url-encoded 32-byte key",
        )?;
        let mfa_cipher = parse_mfa_cipher(&encoded_mfa_key)?;

        Ok(Self {
            bind_address,
            database_url,
            mfa_cipher,
        })
    }
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
        DEFAULT_BIND_ADDRESS, DEFAULT_DATABASE_URL, parse_mfa_cipher, resolve_bind_address,
    };

    #[test]
    fn development_defaults_are_local_only() {
        assert!(DEFAULT_BIND_ADDRESS.starts_with("127.0.0.1:"));
        assert!(DEFAULT_DATABASE_URL.contains("@127.0.0.1:"));
        assert!(DEFAULT_DATABASE_URL.contains("omarchy_gaming_system"));
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
}
