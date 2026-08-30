use std::io::Read as _;

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use ed25519_dalek::SigningKey;
use omarchygs_provider_sdk::protocol::{GrantIssuer, HttpMessageSigner};
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize as _, Zeroizing};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Seeds {
    grant_seed_base64: String,
    message_seed_base64: String,
    provider_seed_base64: String,
}

#[derive(Serialize)]
struct PublicKeys {
    grant_public_key_base64: String,
    message_public_key_base64: String,
    provider_public_key_base64: String,
}

fn main() {
    let mut input = Zeroizing::new(String::new());
    std::io::stdin()
        .take(4_097)
        .read_to_string(&mut input)
        .expect("read bounded seed input");
    assert!(input.len() <= 4_096, "seed input is oversized");
    let mut seeds: Seeds = serde_json::from_str(&input).expect("decode exact seed input");
    let grant_seed = Zeroizing::new(decode(&seeds.grant_seed_base64));
    let message_seed = Zeroizing::new(decode(&seeds.message_seed_base64));
    let provider_seed = Zeroizing::new(decode(&seeds.provider_seed_base64));
    seeds.grant_seed_base64.zeroize();
    seeds.message_seed_base64.zeroize();
    seeds.provider_seed_base64.zeroize();
    drop(input);
    let grant =
        GrantIssuer::new("platform-grant-1", *grant_seed, vec![60; 32]).expect("grant signer");
    let message =
        HttpMessageSigner::new("platform-message-1", *message_seed).expect("message signer");
    let provider = SigningKey::from_bytes(&provider_seed);
    println!(
        "{}",
        serde_json::to_string(&PublicKeys {
            grant_public_key_base64: URL_SAFE_NO_PAD.encode(grant.verifying_key().as_bytes()),
            message_public_key_base64: URL_SAFE_NO_PAD.encode(message.verifying_key().as_bytes()),
            provider_public_key_base64: URL_SAFE_NO_PAD.encode(provider.verifying_key().as_bytes()),
        })
        .expect("encode public keys")
    );
}

fn decode(value: &str) -> [u8; 32] {
    URL_SAFE_NO_PAD
        .decode(value)
        .expect("seed must be unpadded base64url")
        .try_into()
        .expect("seed must be 32 bytes")
}
