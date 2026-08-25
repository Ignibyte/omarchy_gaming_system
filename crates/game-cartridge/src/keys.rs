use std::path::Path;

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};

use crate::{
    contract::SignatureAlgorithm,
    error::{CartridgeError, Result},
    io::read_bounded_regular_file,
};

const MAX_KEY_FILE_BYTES: u64 = 64 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PublisherPrivateKey {
    pub format_version: u32,
    pub algorithm: SignatureAlgorithm,
    pub key_id: String,
    pub publisher_id: String,
    pub signing_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PublisherPublicKey {
    pub format_version: u32,
    pub algorithm: SignatureAlgorithm,
    pub key_id: String,
    pub publisher_id: String,
    pub verifying_key: String,
}

pub fn generate_keypair(
    key_id: &str,
    publisher_id: &str,
) -> Result<(PublisherPrivateKey, PublisherPublicKey)> {
    if !valid_identifier(key_id) || !valid_identifier(publisher_id) {
        return Err(CartridgeError::InvalidKey);
    }
    let signing = SigningKey::generate(&mut OsRng);
    let private = PublisherPrivateKey {
        format_version: 1,
        algorithm: SignatureAlgorithm::Ed25519,
        key_id: key_id.to_owned(),
        publisher_id: publisher_id.to_owned(),
        signing_key: URL_SAFE_NO_PAD.encode(signing.to_bytes()),
    };
    let public = PublisherPublicKey {
        format_version: 1,
        algorithm: SignatureAlgorithm::Ed25519,
        key_id: key_id.to_owned(),
        publisher_id: publisher_id.to_owned(),
        verifying_key: URL_SAFE_NO_PAD.encode(signing.verifying_key().to_bytes()),
    };
    Ok((private, public))
}

impl PublisherPrivateKey {
    pub fn decode(&self) -> Result<SigningKey> {
        self.validate()?;
        let bytes = URL_SAFE_NO_PAD
            .decode(&self.signing_key)
            .map_err(|_| CartridgeError::InvalidKey)?;
        let bytes: [u8; 32] = bytes.try_into().map_err(|_| CartridgeError::InvalidKey)?;
        Ok(SigningKey::from_bytes(&bytes))
    }

    pub fn validate(&self) -> Result<()> {
        if self.format_version != 1
            || !valid_identifier(&self.key_id)
            || !valid_identifier(&self.publisher_id)
        {
            return Err(CartridgeError::InvalidKey);
        }
        Ok(())
    }

    pub fn public_key(&self) -> Result<PublisherPublicKey> {
        let signing = self.decode()?;
        Ok(PublisherPublicKey {
            format_version: self.format_version,
            algorithm: self.algorithm,
            key_id: self.key_id.clone(),
            publisher_id: self.publisher_id.clone(),
            verifying_key: URL_SAFE_NO_PAD.encode(signing.verifying_key().to_bytes()),
        })
    }
}

impl PublisherPublicKey {
    pub fn decode(&self) -> Result<VerifyingKey> {
        self.validate()?;
        let bytes = URL_SAFE_NO_PAD
            .decode(&self.verifying_key)
            .map_err(|_| CartridgeError::InvalidKey)?;
        let bytes: [u8; 32] = bytes.try_into().map_err(|_| CartridgeError::InvalidKey)?;
        VerifyingKey::from_bytes(&bytes).map_err(|_| CartridgeError::InvalidKey)
    }

    pub fn validate(&self) -> Result<()> {
        if self.format_version != 1
            || !valid_identifier(&self.key_id)
            || !valid_identifier(&self.publisher_id)
        {
            return Err(CartridgeError::InvalidKey);
        }
        Ok(())
    }
}

pub fn read_private_key(path: &Path) -> Result<PublisherPrivateKey> {
    let bytes = read_bounded_regular_file(path, MAX_KEY_FILE_BYTES)?;
    let key: PublisherPrivateKey = serde_json::from_slice(&bytes)?;
    key.decode()?;
    Ok(key)
}

pub fn read_public_key(path: &Path) -> Result<PublisherPublicKey> {
    let bytes = read_bounded_regular_file(path, MAX_KEY_FILE_BYTES)?;
    let key: PublisherPublicKey = serde_json::from_slice(&bytes)?;
    key.decode()?;
    Ok(key)
}

pub(crate) fn valid_identifier(value: &str) -> bool {
    let bytes = value.as_bytes();
    (1..=96).contains(&bytes.len())
        && bytes[0].is_ascii_lowercase()
        && bytes.iter().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'_')
        })
}
