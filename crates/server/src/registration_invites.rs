//! Shared registration-invitation code generation and digest contract.

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rand_core::{OsRng, RngCore as _};
use sha2::{Digest as _, Sha256};

pub const INVITE_CODE_PREFIX: &str = "ogsi_";
pub const INVITE_CODE_BYTES: usize = 32;
pub const INVITE_CODE_LENGTH: usize = 48;

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, PartialEq, Eq)]
pub enum InviteCodeError {
    Random,
}

#[cfg_attr(not(test), allow(dead_code))]
pub fn generate() -> Result<(String, [u8; 32]), InviteCodeError> {
    let mut secret = [0_u8; INVITE_CODE_BYTES];
    OsRng
        .try_fill_bytes(&mut secret)
        .map_err(|_| InviteCodeError::Random)?;
    let code = format!("{INVITE_CODE_PREFIX}{}", URL_SAFE_NO_PAD.encode(secret));
    let digest = digest(&code).ok_or(InviteCodeError::Random)?;
    Ok((code, digest))
}

pub fn digest(code: &str) -> Option<[u8; 32]> {
    let encoded = code.strip_prefix(INVITE_CODE_PREFIX)?;
    if code.len() != INVITE_CODE_LENGTH || encoded.len() != 43 {
        return None;
    }
    let decoded = URL_SAFE_NO_PAD.decode(encoded).ok()?;
    if decoded.len() != INVITE_CODE_BYTES || URL_SAFE_NO_PAD.encode(&decoded) != encoded {
        return None;
    }
    Some(Sha256::digest(code.as_bytes()).into())
}

#[cfg(test)]
mod tests {
    use super::{INVITE_CODE_BYTES, INVITE_CODE_LENGTH, INVITE_CODE_PREFIX, digest, generate};

    #[test]
    fn generated_codes_are_canonical_random_and_digestible() {
        let (first, first_digest) = generate().expect("first code should generate");
        let (second, second_digest) = generate().expect("second code should generate");

        assert_eq!(first.len(), INVITE_CODE_LENGTH);
        assert!(first.starts_with(INVITE_CODE_PREFIX));
        assert_eq!(digest(&first), Some(first_digest));
        assert_ne!(first, second);
        assert_ne!(first_digest, second_digest);
        assert_eq!(first_digest.len(), INVITE_CODE_BYTES);
    }

    #[test]
    fn malformed_or_noncanonical_codes_are_rejected() {
        let (valid, _) = generate().expect("code should generate");
        for invalid in [
            "",
            "ogsi_short",
            &valid.to_ascii_uppercase(),
            &format!(" {valid}"),
            &format!("{valid}="),
            &format!("ogs1_{}", &valid[5..]),
        ] {
            assert_eq!(digest(invalid), None, "expected {invalid:?} to fail");
        }
    }
}
