use ed25519_dalek::SigningKey;
use http::HeaderValue;
use omarchy_game_provider::{
    model::ProviderScope,
    protocol::{
        GrantExpectation, GrantIssuer, HttpMessageSigner, ProviderGrantClaims,
        ProviderOperationKind, ProviderOperationRequest, RequestSignatureContext, SignatureHeaders,
        verify_grant, verify_request_signature,
    },
};
use serde_json::json;
use uuid::Uuid;

const NOW: i64 = 1_800_000_000;

#[test]
fn public_contract_rejects_duplicate_and_malformed_signature_headers() {
    let signer = HttpMessageSigner::new("platform-message-1", [41; 32])
        .expect("message signer should construct");
    let context = request_context();
    let body = br#"{"action":"advance"}"#;
    let headers = signer
        .sign_request(&context, body, NOW, "nonce-contract-0001")
        .expect("request should sign");
    let mut map = headers.to_header_map().expect("headers should encode");
    map.append("x-ogs-provider", HeaderValue::from_static("provider-one"));
    assert!(SignatureHeaders::from_header_map(&map).is_err());

    let mut malformed = headers;
    malformed.signature_input.push_str(";created=1800000000");
    assert!(
        verify_request_signature(
            &malformed,
            &context,
            body,
            &signer.verifying_key(),
            "platform-message-1",
            NOW,
        )
        .is_err()
    );
}

#[test]
fn public_contract_rejects_expired_future_tampered_and_mismatched_messages() {
    let signer = HttpMessageSigner::new("platform-message-1", [42; 32])
        .expect("message signer should construct");
    let context = request_context();
    let body = br#"{"action":"advance"}"#;
    let headers = signer
        .sign_request(&context, body, NOW, "nonce-contract-0002")
        .expect("request should sign");
    assert!(
        verify_request_signature(
            &headers,
            &context,
            body,
            &signer.verifying_key(),
            "platform-message-1",
            NOW + 30,
        )
        .is_err()
    );
    assert!(
        verify_request_signature(
            &headers,
            &context,
            b"{\"action\":\"other\"}",
            &signer.verifying_key(),
            "platform-message-1",
            NOW,
        )
        .is_err()
    );
    let wrong_context = RequestSignatureContext {
        path: "/omarchygs/provider/v1/reconcile",
        ..context
    };
    assert!(
        verify_request_signature(
            &headers,
            &wrong_context,
            body,
            &signer.verifying_key(),
            "platform-message-1",
            NOW,
        )
        .is_err()
    );
    let future = signer
        .sign_request(&context, body, NOW + 6, "nonce-contract-0003")
        .expect("future request should serialize");
    assert!(
        verify_request_signature(
            &future,
            &context,
            body,
            &signer.verifying_key(),
            "platform-message-1",
            NOW,
        )
        .is_err()
    );
}

#[test]
fn grant_verification_binds_every_registered_identity_and_exact_bytes() {
    let issuer = GrantIssuer::new("platform-grant-1", [43; 32], vec![44; 32])
        .expect("grant issuer should construct");
    let claims = grant_claims(&issuer);
    let signed = issuer.sign(&claims).expect("claims should sign");
    let expected = expectation(&claims);
    verify_grant(&signed, &issuer.verifying_key(), &expected, NOW)
        .expect("exact grant should verify");

    let wrong_release = GrantExpectation {
        release_id: Uuid::from_u128(99),
        ..expected
    };
    assert!(verify_grant(&signed, &issuer.verifying_key(), &wrong_release, NOW).is_err());
    let mut tampered = signed.clone();
    tampered.payload.push('a');
    assert!(
        verify_grant(
            &tampered,
            &issuer.verifying_key(),
            &expectation(&claims),
            NOW
        )
        .is_err()
    );
    let wrong_key = SigningKey::from_bytes(&[45; 32]);
    assert!(
        verify_grant(
            &signed,
            &wrong_key.verifying_key(),
            &expectation(&claims),
            NOW
        )
        .is_err()
    );
}

#[test]
fn serialized_operation_exposes_pairwise_identity_but_no_local_identity_or_credential() {
    let issuer = GrantIssuer::new("platform-grant-1", [46; 32], vec![47; 32])
        .expect("grant issuer should construct");
    let persona_id = Uuid::from_u128(0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa);
    let subject = issuer
        .pairwise_subject("provider-one", "signal_siege", persona_id)
        .expect("pairwise subject should derive");
    let mut claims = grant_claims(&issuer);
    claims.subject.clone_from(&subject);
    let grant = issuer.sign(&claims).expect("claims should sign");
    let request = ProviderOperationRequest::new(
        claims.provider_id.clone(),
        claims.release_id,
        claims.game_key.clone(),
        claims.rules_version,
        claims.cartridge_digest.clone(),
        claims.platform_session_id,
        subject.clone(),
        Uuid::from_u128(10),
        Uuid::from_u128(11),
        0,
        ProviderOperationKind::Command,
        json!({"action": "advance"}),
        grant,
    )
    .expect("operation should validate");
    let serialized = String::from_utf8(request.to_bytes(16 * 1024).expect("request bytes"))
        .expect("request should be UTF-8 JSON");
    assert!(serialized.contains(&subject));
    for forbidden in [
        persona_id.to_string(),
        "account_id".to_owned(),
        "device_token".to_owned(),
        "database_url".to_owned(),
        "password".to_owned(),
    ] {
        assert!(
            !serialized.contains(&forbidden),
            "{forbidden} must not cross boundary"
        );
    }
}

fn request_context() -> RequestSignatureContext<'static> {
    RequestSignatureContext {
        method: "POST",
        authority: "provider.example.test",
        path: "/omarchygs/provider/v1/commands",
        provider_id: "provider-one",
        release_id: Uuid::from_u128(1),
        message_id: Uuid::from_u128(2),
    }
}

fn grant_claims(issuer: &GrantIssuer) -> ProviderGrantClaims {
    let subject = issuer
        .pairwise_subject("provider-one", "signal_siege", Uuid::from_u128(3))
        .expect("pairwise subject should derive");
    ProviderGrantClaims::new(
        "provider-one".to_owned(),
        Uuid::from_u128(4),
        "signal_siege".to_owned(),
        1,
        "d".repeat(64),
        Uuid::from_u128(5),
        subject,
        ProviderScope::Command,
        NOW,
        NOW + 60,
        Uuid::from_u128(6),
    )
    .expect("grant claims should validate")
}

fn expectation(claims: &ProviderGrantClaims) -> GrantExpectation<'_> {
    GrantExpectation {
        key_id: "platform-grant-1",
        provider_id: &claims.provider_id,
        release_id: claims.release_id,
        game_key: &claims.game_key,
        rules_version: claims.rules_version,
        cartridge_digest: &claims.cartridge_digest,
        platform_session_id: claims.platform_session_id,
        subject: &claims.subject,
        scope: claims.scope,
    }
}
