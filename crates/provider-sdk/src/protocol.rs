//! Pairwise identity, signed grants, provider messages, and the fixed HTTP
//! Message Signatures profile used by OmarchyGS Provider SDK protocol v1.

use std::collections::BTreeSet;

use base64::{
    Engine as _,
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
};
use data_encoding::HEXLOWER;
use ed25519_dalek::{Signature, Signer as _, SigningKey, Verifier as _, VerifyingKey};
use hmac::{Hmac, KeyInit as _, Mac as _};
use http::{HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use sha2::{Digest as _, Sha256};
use uuid::Uuid;
use zeroize::Zeroizing;

use crate::{
    ProviderError, Result,
    model::{ProviderScope, is_identifier, is_sha256_hex},
};

type HmacSha256 = Hmac<Sha256>;

/// Grant issuer identifier for provider protocol v1.
pub const PLATFORM_ISSUER: &str = "omarchygs";
/// Maximum signed grant lifetime.
pub const MAX_GRANT_LIFETIME_SECONDS: i64 = 60;
/// Maximum HTTP message signature lifetime.
pub const MAX_MESSAGE_SIGNATURE_LIFETIME_SECONDS: i64 = 30;
/// Maximum accepted future clock skew.
pub const MAX_CLOCK_SKEW_SECONDS: i64 = 5;
/// Maximum serialized grant envelope.
pub const MAX_SIGNED_GRANT_BYTES: usize = 8 * 1024;
/// Maximum v1 operation/callback JSON depth.
pub const MAX_JSON_DEPTH: usize = 12;
/// Maximum v1 operation/callback JSON values.
pub const MAX_JSON_VALUES: usize = 2_048;
/// Sole protocol version supported by this first public SDK release.
pub const PROVIDER_PROTOCOL_VERSION: u32 = 1;

const GRANT_SCHEMA: &str = "omarchygs.provider-grant/v1";
const REQUEST_SCHEMA: &str = "omarchygs.provider-request/v1";
const RESPONSE_SCHEMA: &str = "omarchygs.provider-response/v1";
const EVENT_SCHEMA: &str = "omarchygs.provider-event/v1";
const COMPATIBILITY_OFFER_SCHEMA: &str = "omarchygs.provider-compatibility-offer/v1";
const COMPATIBILITY_SELECTION_SCHEMA: &str = "omarchygs.provider-compatibility-selection/v1";
const GRANT_DOMAIN: &[u8] = b"omarchygs-provider-grant-v1\0";
const CONTENT_TYPE: &str = "application/json";
const SIGNATURE_LABEL: &str = "ogs";
const SIGNATURE_TAG: &str = "omarchygs-provider-v1";
const REQUEST_COMPONENTS: &str = "(\"@method\" \"@authority\" \"@path\" \"content-digest\" \"content-type\" \"x-ogs-provider\" \"x-ogs-release\" \"x-ogs-message-id\")";
const RESPONSE_COMPONENTS: &str = "(\"@status\" \"@method\";req \"@authority\";req \"@path\";req \"content-digest\" \"content-type\" \"x-ogs-provider\" \"x-ogs-release\" \"x-ogs-message-id\")";

/// Fixed provider protocol HTTP header carrying provider identity.
pub const HEADER_PROVIDER: &str = "x-ogs-provider";
/// Fixed provider protocol HTTP header carrying exact release identity.
pub const HEADER_RELEASE: &str = "x-ogs-release";
/// Fixed provider protocol HTTP header carrying replay/message identity.
pub const HEADER_MESSAGE_ID: &str = "x-ogs-message-id";
/// RFC 9530 body digest header.
pub const HEADER_CONTENT_DIGEST: &str = "content-digest";
/// RFC 9421 signature parameter header.
pub const HEADER_SIGNATURE_INPUT: &str = "signature-input";
/// RFC 9421 signature bytes header.
pub const HEADER_SIGNATURE: &str = "signature";

/// Exact negotiated protocol profile bound into grants and messages.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProviderCompatibility {
    /// Selected protocol version.
    pub protocol_version: u32,
    /// Exact sorted required capability set.
    pub capabilities: Vec<ProviderScope>,
}

impl ProviderCompatibility {
    /// Construct the sole exact profile accepted by SDK v1.
    #[must_use]
    pub fn current() -> Self {
        Self {
            protocol_version: PROVIDER_PROTOCOL_VERSION,
            capabilities: vec![
                ProviderScope::Launch,
                ProviderScope::Command,
                ProviderScope::Reconcile,
                ProviderScope::Event,
            ],
        }
    }

    /// Fail closed unless this is the complete current profile.
    pub fn validate(&self) -> Result<()> {
        if *self == Self::current() {
            Ok(())
        } else {
            Err(ProviderError::ProtocolRejected)
        }
    }
}

/// Platform-authenticated compatibility offer sent before provider effects.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProviderCompatibilityOffer {
    /// Protocol schema discriminator.
    pub schema: String,
    /// Exact provider.
    pub provider_id: String,
    /// Exact release.
    pub release_id: Uuid,
    /// Offer replay identity, also carried by signed HTTP headers.
    pub message_id: Uuid,
    /// Bounded supported profiles in descending preference order.
    pub supported: Vec<ProviderCompatibility>,
}

impl ProviderCompatibilityOffer {
    /// Construct the exact SDK v1 offer.
    pub fn current(provider_id: String, release_id: Uuid, message_id: Uuid) -> Result<Self> {
        let offer = Self {
            schema: COMPATIBILITY_OFFER_SCHEMA.to_owned(),
            provider_id,
            release_id,
            message_id,
            supported: vec![ProviderCompatibility::current()],
        };
        offer.validate()?;
        Ok(offer)
    }

    /// Validate all identity, ordering, uniqueness, and downgrade constraints.
    pub fn validate(&self) -> Result<()> {
        if self.schema != COMPATIBILITY_OFFER_SCHEMA
            || !is_identifier(&self.provider_id, 3, 64, b"-_")
            || self.release_id.is_nil()
            || self.message_id.is_nil()
            || self.supported.len() != 1
        {
            return Err(ProviderError::ProtocolRejected);
        }
        self.supported[0].validate()
    }

    /// Serialize exact bounded bytes for signing and transmission.
    pub fn to_bytes(&self, limit: usize) -> Result<Vec<u8>> {
        self.validate()?;
        let bytes = serde_json::to_vec(self).map_err(|_| ProviderError::Internal)?;
        if bytes.is_empty() || bytes.len() > limit {
            Err(ProviderError::InvalidInput)
        } else {
            Ok(bytes)
        }
    }
}

/// Provider-authenticated exact selection acknowledging one offer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProviderCompatibilitySelection {
    /// Protocol schema discriminator.
    pub schema: String,
    /// Exact provider.
    pub provider_id: String,
    /// Exact release.
    pub release_id: Uuid,
    /// Originating offer replay identity.
    pub offer_message_id: Uuid,
    /// Selection replay identity, also carried by signed HTTP headers.
    pub message_id: Uuid,
    /// One exact selected profile.
    pub selected: ProviderCompatibility,
}

impl ProviderCompatibilitySelection {
    /// Select the exact current profile from a valid offer.
    pub fn current(offer: &ProviderCompatibilityOffer, message_id: Uuid) -> Result<Self> {
        offer.validate()?;
        let selection = Self {
            schema: COMPATIBILITY_SELECTION_SCHEMA.to_owned(),
            provider_id: offer.provider_id.clone(),
            release_id: offer.release_id,
            offer_message_id: offer.message_id,
            message_id,
            selected: ProviderCompatibility::current(),
        };
        selection.validate_for(offer)?;
        Ok(selection)
    }

    /// Validate the provider selection against the exact authenticated offer.
    pub fn validate_for(&self, offer: &ProviderCompatibilityOffer) -> Result<()> {
        offer.validate()?;
        if self.schema != COMPATIBILITY_SELECTION_SCHEMA
            || self.provider_id != offer.provider_id
            || self.release_id != offer.release_id
            || self.offer_message_id != offer.message_id
            || self.message_id.is_nil()
            || self.message_id == offer.message_id
            || self.selected != offer.supported[0]
        {
            return Err(ProviderError::ProtocolRejected);
        }
        self.selected.validate()
    }
}

/// Ed25519 signer for short-lived grants plus pairwise persona derivation.
pub struct GrantIssuer {
    key_id: String,
    signing_key: SigningKey,
    pairwise_secret: Zeroizing<Vec<u8>>,
}

impl GrantIssuer {
    /// Construct from local secret material. Secrets are never serialized.
    pub fn new(key_id: &str, signing_seed: [u8; 32], pairwise_secret: Vec<u8>) -> Result<Self> {
        if !is_identifier(key_id, 3, 64, b"._-") || pairwise_secret.len() < 32 {
            return Err(ProviderError::InvalidInput);
        }
        Ok(Self {
            key_id: key_id.to_owned(),
            signing_key: SigningKey::from_bytes(&signing_seed),
            pairwise_secret: Zeroizing::new(pairwise_secret),
        })
    }

    /// Public key distributed to registered provider infrastructure.
    #[must_use]
    pub fn verifying_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }

    /// Derive a stable opaque pairwise persona subject for one provider/game.
    pub fn pairwise_subject(
        &self,
        provider_id: &str,
        game_key: &str,
        persona_id: Uuid,
    ) -> Result<String> {
        pairwise_subject(&self.pairwise_secret, provider_id, game_key, persona_id)
    }

    /// Sign exact validated grant claims.
    pub fn sign(&self, claims: &ProviderGrantClaims) -> Result<SignedProviderGrant> {
        claims.validate_shape()?;
        sign_grant(claims, &self.key_id, &self.signing_key)
    }
}

/// Ed25519 signer for the fixed RFC 9421 request/response profile.
pub struct HttpMessageSigner {
    key_id: String,
    signing_key: SigningKey,
}

impl HttpMessageSigner {
    /// Construct from local private seed material.
    pub fn new(key_id: &str, signing_seed: [u8; 32]) -> Result<Self> {
        if !is_identifier(key_id, 3, 64, b"._-") {
            return Err(ProviderError::InvalidInput);
        }
        Ok(Self {
            key_id: key_id.to_owned(),
            signing_key: SigningKey::from_bytes(&signing_seed),
        })
    }

    /// Public verification key.
    #[must_use]
    pub fn verifying_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }

    /// Sign an outbound request over the exact transmitted bytes.
    pub fn sign_request(
        &self,
        context: &RequestSignatureContext<'_>,
        body: &[u8],
        created_at: i64,
        nonce: &str,
    ) -> Result<SignatureHeaders> {
        let expires_at = created_at
            .checked_add(MAX_MESSAGE_SIGNATURE_LIFETIME_SECONDS)
            .ok_or(ProviderError::InvalidInput)?;
        let signature_input = signature_input(
            REQUEST_COMPONENTS,
            created_at,
            expires_at,
            nonce,
            &self.key_id,
        )?;
        let content_digest = content_digest(body);
        let base = request_signature_base(context, &content_digest, &signature_input)?;
        let signature = self.signing_key.sign(base.as_bytes());
        SignatureHeaders::new(
            context.provider_id,
            context.release_id,
            context.message_id,
            content_digest,
            signature_input,
            signature,
        )
    }

    /// Sign a response while binding its originating request context.
    pub fn sign_response(
        &self,
        status: u16,
        request: &RequestSignatureContext<'_>,
        response_message_id: Uuid,
        body: &[u8],
        created_at: i64,
        nonce: &str,
    ) -> Result<SignatureHeaders> {
        if !(100..=599).contains(&status) {
            return Err(ProviderError::InvalidInput);
        }
        let expires_at = created_at
            .checked_add(MAX_MESSAGE_SIGNATURE_LIFETIME_SECONDS)
            .ok_or(ProviderError::InvalidInput)?;
        let signature_input = signature_input(
            RESPONSE_COMPONENTS,
            created_at,
            expires_at,
            nonce,
            &self.key_id,
        )?;
        let content_digest = content_digest(body);
        let base = response_signature_base(
            status,
            request,
            response_message_id,
            &content_digest,
            &signature_input,
        )?;
        let signature = self.signing_key.sign(base.as_bytes());
        SignatureHeaders::new(
            request.provider_id,
            request.release_id,
            response_message_id,
            content_digest,
            signature_input,
            signature,
        )
    }
}

/// Exact short-lived provider grant claims.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProviderGrantClaims {
    /// Protocol schema discriminator.
    pub schema: String,
    /// Platform issuer.
    pub issuer: String,
    /// Exact provider audience.
    pub audience: String,
    /// Exact provider identity.
    pub provider_id: String,
    /// Exact provider release.
    pub release_id: Uuid,
    /// Exact game key.
    pub game_key: String,
    /// Exact rules version.
    pub rules_version: u32,
    /// Exact cartridge digest.
    pub cartridge_digest: String,
    /// Exact platform session envelope.
    pub platform_session_id: Uuid,
    /// Pairwise provider/game persona subject.
    pub subject: String,
    /// One exact capability.
    pub scope: ProviderScope,
    /// Authenticated exact compatibility selected before this operation.
    pub compatibility: ProviderCompatibility,
    /// Inclusive Unix issue time.
    pub issued_at: i64,
    /// Exclusive Unix expiry time.
    pub expires_at: i64,
    /// Unique replay identity.
    pub token_id: Uuid,
}

impl ProviderGrantClaims {
    /// Construct exact claims before signing.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        provider_id: String,
        release_id: Uuid,
        game_key: String,
        rules_version: u32,
        cartridge_digest: String,
        platform_session_id: Uuid,
        subject: String,
        scope: ProviderScope,
        compatibility: ProviderCompatibility,
        issued_at: i64,
        expires_at: i64,
        token_id: Uuid,
    ) -> Result<Self> {
        let claims = Self {
            schema: GRANT_SCHEMA.to_owned(),
            issuer: PLATFORM_ISSUER.to_owned(),
            audience: provider_id.clone(),
            provider_id,
            release_id,
            game_key,
            rules_version,
            cartridge_digest,
            platform_session_id,
            subject,
            scope,
            compatibility,
            issued_at,
            expires_at,
            token_id,
        };
        claims.validate_shape()?;
        Ok(claims)
    }

    fn validate_shape(&self) -> Result<()> {
        if self.schema != GRANT_SCHEMA
            || self.issuer != PLATFORM_ISSUER
            || self.audience != self.provider_id
            || !is_identifier(&self.provider_id, 3, 64, b"-_")
            || self.release_id.is_nil()
            || !is_identifier(&self.game_key, 3, 32, b"-_")
            || self.rules_version == 0
            || !is_sha256_hex(&self.cartridge_digest)
            || self.platform_session_id.is_nil()
            || !is_pairwise_subject(&self.subject)
            || self.scope == ProviderScope::Event
            || self.compatibility.validate().is_err()
            || self.token_id.is_nil()
            || self.expires_at <= self.issued_at
            || self.expires_at - self.issued_at > MAX_GRANT_LIFETIME_SECONDS
        {
            Err(ProviderError::InvalidInput)
        } else {
            Ok(())
        }
    }
}

/// Signed retained grant payload. Verification parses only authenticated bytes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SignedProviderGrant {
    /// Exact platform key identity.
    pub key_id: String,
    /// Unpadded base64url exact claims bytes.
    pub payload: String,
    /// Unpadded base64url Ed25519 signature.
    pub signature: String,
}

impl SignedProviderGrant {
    /// Deterministic bounded serialized bytes for persistence/transmission.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let bytes = serde_json::to_vec(self).map_err(|_| ProviderError::Internal)?;
        if bytes.len() > MAX_SIGNED_GRANT_BYTES {
            Err(ProviderError::InvalidInput)
        } else {
            Ok(bytes)
        }
    }
}

/// Expected context for grant verification.
pub struct GrantExpectation<'a> {
    /// Platform verification key ID.
    pub key_id: &'a str,
    /// Exact provider.
    pub provider_id: &'a str,
    /// Exact release.
    pub release_id: Uuid,
    /// Exact game.
    pub game_key: &'a str,
    /// Exact rules version.
    pub rules_version: u32,
    /// Exact cartridge digest.
    pub cartridge_digest: &'a str,
    /// Exact platform session.
    pub platform_session_id: Uuid,
    /// Exact pairwise subject.
    pub subject: &'a str,
    /// One required scope.
    pub scope: ProviderScope,
    /// Exact preflight selection.
    pub compatibility: &'a ProviderCompatibility,
}

/// Verify a signed grant and its complete registered context.
pub fn verify_grant(
    signed: &SignedProviderGrant,
    verifying_key: &VerifyingKey,
    expected: &GrantExpectation<'_>,
    now: i64,
) -> Result<ProviderGrantClaims> {
    if signed.key_id != expected.key_id
        || !is_identifier(&signed.key_id, 3, 64, b"._-")
        || signed.payload.len() > MAX_SIGNED_GRANT_BYTES
        || signed.signature.len() > 128
    {
        return Err(ProviderError::ProtocolRejected);
    }
    let payload = URL_SAFE_NO_PAD
        .decode(&signed.payload)
        .map_err(|_| ProviderError::ProtocolRejected)?;
    if payload.is_empty() || payload.len() > MAX_SIGNED_GRANT_BYTES {
        return Err(ProviderError::ProtocolRejected);
    }
    let signature_bytes = URL_SAFE_NO_PAD
        .decode(&signed.signature)
        .map_err(|_| ProviderError::ProtocolRejected)?;
    let signature =
        Signature::from_slice(&signature_bytes).map_err(|_| ProviderError::ProtocolRejected)?;
    let mut message = Vec::with_capacity(GRANT_DOMAIN.len() + payload.len());
    message.extend_from_slice(GRANT_DOMAIN);
    message.extend_from_slice(&payload);
    verifying_key
        .verify(&message, &signature)
        .map_err(|_| ProviderError::ProtocolRejected)?;
    let claims: ProviderGrantClaims =
        serde_json::from_slice(&payload).map_err(|_| ProviderError::ProtocolRejected)?;
    claims
        .validate_shape()
        .map_err(|_| ProviderError::ProtocolRejected)?;
    if claims.provider_id != expected.provider_id
        || claims.audience != expected.provider_id
        || claims.release_id != expected.release_id
        || claims.game_key != expected.game_key
        || claims.rules_version != expected.rules_version
        || claims.cartridge_digest != expected.cartridge_digest
        || claims.platform_session_id != expected.platform_session_id
        || claims.subject != expected.subject
        || claims.scope != expected.scope
        || claims.compatibility != *expected.compatibility
        || claims.issued_at > now.saturating_add(MAX_CLOCK_SKEW_SECONDS)
        || claims.expires_at <= now
    {
        Err(ProviderError::ProtocolRejected)
    } else {
        Ok(claims)
    }
}

/// Provider operation kind with a fixed scope and path.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderOperationKind {
    /// Create or resume a session.
    Launch,
    /// Apply one command.
    Command,
    /// Query authoritative state/receipts.
    Reconcile,
}

impl ProviderOperationKind {
    /// Required grant scope.
    #[must_use]
    pub const fn scope(self) -> ProviderScope {
        match self {
            Self::Launch => ProviderScope::Launch,
            Self::Command => ProviderScope::Command,
            Self::Reconcile => ProviderScope::Reconcile,
        }
    }

    /// Allowlisted endpoint path segment.
    #[must_use]
    pub const fn path(self) -> &'static str {
        match self {
            Self::Launch => "launch",
            Self::Command => "commands",
            Self::Reconcile => "reconcile",
        }
    }
}

/// Exact provider request body authenticated at the HTTP layer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ProviderOperationRequest {
    /// Protocol discriminator.
    pub schema: String,
    /// Exact provider.
    pub provider_id: String,
    /// Exact release.
    pub release_id: Uuid,
    /// Exact game.
    pub game_key: String,
    /// Exact rules version.
    pub rules_version: u32,
    /// Exact cartridge digest.
    pub cartridge_digest: String,
    /// Exact platform session.
    pub platform_session_id: Uuid,
    /// Pairwise subject.
    pub subject: String,
    /// HTTP replay identity.
    pub message_id: Uuid,
    /// Stable operation idempotency identity.
    pub idempotency_key: Uuid,
    /// Exact expected provider revision.
    pub expected_revision: u64,
    /// Fixed operation kind.
    pub operation: ProviderOperationKind,
    /// Exact authenticated preflight selection.
    pub compatibility: ProviderCompatibility,
    /// Bounded schema-owned operation data.
    pub payload: Value,
    /// One-scope signed platform grant.
    pub grant: SignedProviderGrant,
}

impl ProviderOperationRequest {
    /// Construct an exact provider request before serialization.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        provider_id: String,
        release_id: Uuid,
        game_key: String,
        rules_version: u32,
        cartridge_digest: String,
        platform_session_id: Uuid,
        subject: String,
        message_id: Uuid,
        idempotency_key: Uuid,
        expected_revision: u64,
        operation: ProviderOperationKind,
        compatibility: ProviderCompatibility,
        payload: Value,
        grant: SignedProviderGrant,
    ) -> Result<Self> {
        let request = Self {
            schema: REQUEST_SCHEMA.to_owned(),
            provider_id,
            release_id,
            game_key,
            rules_version,
            cartridge_digest,
            platform_session_id,
            subject,
            message_id,
            idempotency_key,
            expected_revision,
            operation,
            compatibility,
            payload,
            grant,
        };
        request.validate()?;
        Ok(request)
    }

    /// Validate structural and privacy bounds before serialization.
    pub fn validate(&self) -> Result<()> {
        if self.schema != REQUEST_SCHEMA
            || !is_identifier(&self.provider_id, 3, 64, b"-_")
            || self.release_id.is_nil()
            || !is_identifier(&self.game_key, 3, 32, b"-_")
            || self.rules_version == 0
            || !is_sha256_hex(&self.cartridge_digest)
            || self.platform_session_id.is_nil()
            || !is_pairwise_subject(&self.subject)
            || self.message_id.is_nil()
            || self.idempotency_key.is_nil()
            || self.compatibility.validate().is_err()
        {
            return Err(ProviderError::InvalidInput);
        }
        validate_json_payload(&self.payload)
    }

    /// Serialize exact bytes that will be digested, signed, and persisted.
    pub fn to_bytes(&self, limit: usize) -> Result<Vec<u8>> {
        self.validate()?;
        let bytes = serde_json::to_vec(self).map_err(|_| ProviderError::Internal)?;
        if bytes.is_empty() || bytes.len() > limit {
            Err(ProviderError::InvalidInput)
        } else {
            Ok(bytes)
        }
    }
}

/// Provider response status for the dormant platform envelope.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderSessionStatus {
    /// Provider session can accept commands.
    Active,
    /// Provider session has a terminal outcome.
    Completed,
}

/// Whether an operation was applied or rejected at the expected-revision boundary.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderOperationDisposition {
    /// The operation succeeded or reconciliation returned authoritative state.
    Applied,
    /// The provider retained a different current revision and changed no state.
    RevisionConflict,
}

/// Exact provider response body authenticated at the HTTP layer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ProviderOperationResponse {
    /// Protocol discriminator.
    pub schema: String,
    /// Exact provider.
    pub provider_id: String,
    /// Exact release.
    pub release_id: Uuid,
    /// Exact game.
    pub game_key: String,
    /// Exact rules version.
    pub rules_version: u32,
    /// Exact cartridge digest.
    pub cartridge_digest: String,
    /// Exact platform session.
    pub platform_session_id: Uuid,
    /// Pairwise subject.
    pub subject: String,
    /// Response replay identity.
    pub message_id: Uuid,
    /// Stable idempotency identity.
    pub idempotency_key: Uuid,
    /// Resulting provider revision.
    pub revision: u64,
    /// Applied or explicit revision conflict.
    pub disposition: ProviderOperationDisposition,
    /// Provider lifecycle result.
    pub status: ProviderSessionStatus,
    /// Exact compatibility selected before the originating operation.
    pub compatibility: ProviderCompatibility,
    /// Bounded schema-owned response data.
    pub payload: Value,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedProviderOperationResponseV1 {
    schema: String,
    provider_id: String,
    release_id: Uuid,
    game_key: String,
    rules_version: u32,
    cartridge_digest: String,
    platform_session_id: Uuid,
    subject: String,
    message_id: Uuid,
    idempotency_key: Uuid,
    revision: u64,
    disposition: ProviderOperationDisposition,
    status: ProviderSessionStatus,
    payload: Value,
}

impl ProviderOperationResponse {
    /// Construct one exact provider response or stable idempotency receipt.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        provider_id: String,
        release_id: Uuid,
        game_key: String,
        rules_version: u32,
        cartridge_digest: String,
        platform_session_id: Uuid,
        subject: String,
        message_id: Uuid,
        idempotency_key: Uuid,
        revision: u64,
        disposition: ProviderOperationDisposition,
        status: ProviderSessionStatus,
        compatibility: ProviderCompatibility,
        payload: Value,
    ) -> Self {
        Self {
            schema: RESPONSE_SCHEMA.to_owned(),
            provider_id,
            release_id,
            game_key,
            rules_version,
            cartridge_digest,
            platform_session_id,
            subject,
            message_id,
            idempotency_key,
            revision,
            disposition,
            status,
            compatibility,
            payload,
        }
    }

    /// Decode a locally persisted v1 receipt, adding the exact compatibility
    /// profile to receipts written before negotiation became mandatory.
    /// Never use this helper for unauthenticated network bytes.
    pub fn from_persisted_v1_bytes(
        bytes: &[u8],
        limit: usize,
        compatibility: ProviderCompatibility,
    ) -> Result<Self> {
        compatibility.validate()?;
        if let Ok(response) = parse_authenticated_json::<Self>(bytes, limit) {
            response.validate_shape()?;
            return Ok(response);
        }
        let legacy: PersistedProviderOperationResponseV1 = parse_authenticated_json(bytes, limit)?;
        let response = Self {
            schema: legacy.schema,
            provider_id: legacy.provider_id,
            release_id: legacy.release_id,
            game_key: legacy.game_key,
            rules_version: legacy.rules_version,
            cartridge_digest: legacy.cartridge_digest,
            platform_session_id: legacy.platform_session_id,
            subject: legacy.subject,
            message_id: legacy.message_id,
            idempotency_key: legacy.idempotency_key,
            revision: legacy.revision,
            disposition: legacy.disposition,
            status: legacy.status,
            compatibility,
            payload: legacy.payload,
        };
        response.validate_shape()?;
        Ok(response)
    }

    /// Validate a response against the exact operation context.
    pub fn validate_for(&self, request: &ProviderOperationRequest) -> Result<()> {
        self.validate_shape()?;
        if self.provider_id != request.provider_id
            || self.release_id != request.release_id
            || self.game_key != request.game_key
            || self.rules_version != request.rules_version
            || self.cartridge_digest != request.cartridge_digest
            || self.platform_session_id != request.platform_session_id
            || self.subject != request.subject
            || self.message_id.is_nil()
            || self.message_id == request.message_id
            || self.idempotency_key != request.idempotency_key
            || self.compatibility != request.compatibility
        {
            return Err(ProviderError::ProtocolRejected);
        }
        match self.disposition {
            ProviderOperationDisposition::Applied => match request.operation {
                ProviderOperationKind::Launch if self.revision != 0 => {
                    return Err(ProviderError::ProtocolRejected);
                }
                ProviderOperationKind::Command
                    if self.revision != request.expected_revision.saturating_add(1) =>
                {
                    return Err(ProviderError::ProtocolRejected);
                }
                ProviderOperationKind::Reconcile if self.revision < request.expected_revision => {
                    return Err(ProviderError::ProtocolRejected);
                }
                ProviderOperationKind::Launch
                | ProviderOperationKind::Command
                | ProviderOperationKind::Reconcile => {}
            },
            ProviderOperationDisposition::RevisionConflict => {
                if request.operation == ProviderOperationKind::Launch
                    || self.revision == request.expected_revision
                {
                    return Err(ProviderError::ProtocolRejected);
                }
            }
        }
        Ok(())
    }

    fn validate_shape(&self) -> Result<()> {
        if self.schema != RESPONSE_SCHEMA
            || !is_identifier(&self.provider_id, 3, 64, b"-_")
            || self.release_id.is_nil()
            || !is_identifier(&self.game_key, 3, 32, b"-_")
            || self.rules_version == 0
            || !is_sha256_hex(&self.cartridge_digest)
            || self.platform_session_id.is_nil()
            || !is_pairwise_subject(&self.subject)
            || self.message_id.is_nil()
            || self.idempotency_key.is_nil()
            || self.compatibility.validate().is_err()
        {
            return Err(ProviderError::ProtocolRejected);
        }
        validate_json_payload(&self.payload).map_err(|_| ProviderError::ProtocolRejected)
    }
}

/// Authenticated callback/event body retained only as a bounded disposition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ProviderEvent {
    /// Protocol discriminator.
    pub schema: String,
    /// Exact provider.
    pub provider_id: String,
    /// Exact release.
    pub release_id: Uuid,
    /// Exact game.
    pub game_key: String,
    /// Exact rules version.
    pub rules_version: u32,
    /// Exact cartridge digest.
    pub cartridge_digest: String,
    /// Exact platform session.
    pub platform_session_id: Uuid,
    /// Pairwise subject.
    pub subject: String,
    /// HTTP replay identity.
    pub message_id: Uuid,
    /// Stable event deduplication identity.
    pub event_id: Uuid,
    /// Monotonic provider revision.
    pub revision: u64,
    /// Fixed event kind.
    pub kind: ProviderEventKind,
    /// Exact compatibility selected for the originating provider session.
    pub compatibility: ProviderCompatibility,
    /// Bounded schema-owned event data.
    pub payload: Value,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedProviderEventV1 {
    schema: String,
    provider_id: String,
    release_id: Uuid,
    game_key: String,
    rules_version: u32,
    cartridge_digest: String,
    platform_session_id: Uuid,
    subject: String,
    message_id: Uuid,
    event_id: Uuid,
    revision: u64,
    kind: ProviderEventKind,
    payload: Value,
}

impl ProviderEvent {
    /// Construct one callback-shaped provider event.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        provider_id: String,
        release_id: Uuid,
        game_key: String,
        rules_version: u32,
        cartridge_digest: String,
        platform_session_id: Uuid,
        subject: String,
        message_id: Uuid,
        event_id: Uuid,
        revision: u64,
        kind: ProviderEventKind,
        compatibility: ProviderCompatibility,
        payload: Value,
    ) -> Self {
        Self {
            schema: EVENT_SCHEMA.to_owned(),
            provider_id,
            release_id,
            game_key,
            rules_version,
            cartridge_digest,
            platform_session_id,
            subject,
            message_id,
            event_id,
            revision,
            kind,
            compatibility,
            payload,
        }
    }

    /// Decode a locally persisted v1 outbox event, adding the exact
    /// compatibility profile to events retained before negotiation became
    /// mandatory. Never use this helper for unauthenticated network bytes.
    pub fn from_persisted_v1_bytes(
        bytes: &[u8],
        limit: usize,
        compatibility: ProviderCompatibility,
    ) -> Result<Self> {
        compatibility.validate()?;
        if let Ok(event) = parse_authenticated_json::<Self>(bytes, limit) {
            event.validate()?;
            return Ok(event);
        }
        let legacy: PersistedProviderEventV1 = parse_authenticated_json(bytes, limit)?;
        let event = Self {
            schema: legacy.schema,
            provider_id: legacy.provider_id,
            release_id: legacy.release_id,
            game_key: legacy.game_key,
            rules_version: legacy.rules_version,
            cartridge_digest: legacy.cartridge_digest,
            platform_session_id: legacy.platform_session_id,
            subject: legacy.subject,
            message_id: legacy.message_id,
            event_id: legacy.event_id,
            revision: legacy.revision,
            kind: legacy.kind,
            compatibility,
            payload: legacy.payload,
        };
        event.validate()?;
        Ok(event)
    }

    /// Validate shape before authenticated receipt admission.
    pub fn validate(&self) -> Result<()> {
        if self.schema != EVENT_SCHEMA
            || !is_identifier(&self.provider_id, 3, 64, b"-_")
            || self.release_id.is_nil()
            || !is_identifier(&self.game_key, 3, 32, b"-_")
            || self.rules_version == 0
            || !is_sha256_hex(&self.cartridge_digest)
            || self.platform_session_id.is_nil()
            || !is_pairwise_subject(&self.subject)
            || self.message_id.is_nil()
            || self.event_id.is_nil()
            || self.compatibility.validate().is_err()
        {
            return Err(ProviderError::ProtocolRejected);
        }
        validate_json_payload(&self.payload).map_err(|_| ProviderError::ProtocolRejected)
    }
}

/// Callback event kinds that Ticket 019 may project after separate policy.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderEventKind {
    /// Another participant can act.
    TurnReady,
    /// A terminal result is available for later validation.
    ResultAvailable,
    /// Provider requests an explicit reconciliation.
    ReconciliationRequired,
}

/// Exact context covered by a signed provider HTTP request.
pub struct RequestSignatureContext<'a> {
    /// Uppercase HTTP method; v1 admits only POST.
    pub method: &'a str,
    /// Canonical registered authority.
    pub authority: &'a str,
    /// Canonical absolute path with no query.
    pub path: &'a str,
    /// Exact provider.
    pub provider_id: &'a str,
    /// Exact release.
    pub release_id: Uuid,
    /// Exact message identity.
    pub message_id: Uuid,
}

impl RequestSignatureContext<'_> {
    fn validate(&self) -> Result<()> {
        if self.method != "POST"
            || !is_authority(self.authority)
            || !is_canonical_path(self.path)
            || !is_identifier(self.provider_id, 3, 64, b"-_")
            || self.release_id.is_nil()
            || self.message_id.is_nil()
        {
            Err(ProviderError::InvalidInput)
        } else {
            Ok(())
        }
    }
}

/// Parsed required signature parameters after successful verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedHttpSignature {
    /// Exact registered key ID.
    pub key_id: String,
    /// Inclusive signature creation time.
    pub created_at: i64,
    /// Exclusive signature expiry.
    pub expires_at: i64,
    /// Unique signed nonce.
    pub nonce: String,
    /// SHA-256 of exact authenticated body bytes.
    pub body_sha256: String,
}

/// Complete fixed RFC 9421/9530 header set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureHeaders {
    /// Provider identity.
    pub provider_id: String,
    /// Release identity.
    pub release_id: String,
    /// Message identity.
    pub message_id: String,
    /// RFC 9530 content digest.
    pub content_digest: String,
    /// Fixed content type.
    pub content_type: String,
    /// RFC 9421 signature parameters/components.
    pub signature_input: String,
    /// RFC 9421 signature bytes.
    pub signature: String,
}

impl SignatureHeaders {
    fn new(
        provider_id: &str,
        release_id: Uuid,
        message_id: Uuid,
        content_digest: String,
        signature_input: String,
        signature: Signature,
    ) -> Result<Self> {
        if !is_identifier(provider_id, 3, 64, b"-_") || release_id.is_nil() || message_id.is_nil() {
            return Err(ProviderError::InvalidInput);
        }
        Ok(Self {
            provider_id: provider_id.to_owned(),
            release_id: release_id.to_string(),
            message_id: message_id.to_string(),
            content_digest,
            content_type: CONTENT_TYPE.to_owned(),
            signature_input,
            signature: format!(
                "{SIGNATURE_LABEL}=:{}:",
                STANDARD.encode(signature.to_bytes())
            ),
        })
    }

    /// Convert to an HTTP header map after exact value validation.
    pub fn to_header_map(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        insert_header(&mut headers, HEADER_PROVIDER, &self.provider_id)?;
        insert_header(&mut headers, HEADER_RELEASE, &self.release_id)?;
        insert_header(&mut headers, HEADER_MESSAGE_ID, &self.message_id)?;
        insert_header(&mut headers, HEADER_CONTENT_DIGEST, &self.content_digest)?;
        insert_header(&mut headers, "content-type", &self.content_type)?;
        insert_header(&mut headers, HEADER_SIGNATURE_INPUT, &self.signature_input)?;
        insert_header(&mut headers, HEADER_SIGNATURE, &self.signature)?;
        Ok(headers)
    }

    /// Parse exactly one instance of every required header. Duplicates reject.
    pub fn from_header_map(headers: &HeaderMap) -> Result<Self> {
        Ok(Self {
            provider_id: one_header(headers, HEADER_PROVIDER)?,
            release_id: one_header(headers, HEADER_RELEASE)?,
            message_id: one_header(headers, HEADER_MESSAGE_ID)?,
            content_digest: one_header(headers, HEADER_CONTENT_DIGEST)?,
            content_type: one_header(headers, "content-type")?,
            signature_input: one_header(headers, HEADER_SIGNATURE_INPUT)?,
            signature: one_header(headers, HEADER_SIGNATURE)?,
        })
    }
}

/// Verify a fixed signed request over the exact received body.
pub fn verify_request_signature(
    headers: &SignatureHeaders,
    context: &RequestSignatureContext<'_>,
    body: &[u8],
    verifying_key: &VerifyingKey,
    expected_key_id: &str,
    now: i64,
) -> Result<VerifiedHttpSignature> {
    context
        .validate()
        .map_err(|_| ProviderError::ProtocolRejected)?;
    validate_identity_headers(
        headers,
        context.provider_id,
        context.release_id,
        context.message_id,
    )?;
    let parameters = parse_signature_input(
        &headers.signature_input,
        REQUEST_COMPONENTS,
        expected_key_id,
        now,
    )?;
    validate_content_digest(headers, body)?;
    let base = request_signature_base(context, &headers.content_digest, &headers.signature_input)
        .map_err(|_| ProviderError::ProtocolRejected)?;
    verify_signature_value(&headers.signature, base.as_bytes(), verifying_key)?;
    Ok(VerifiedHttpSignature {
        key_id: parameters.key_id,
        created_at: parameters.created_at,
        expires_at: parameters.expires_at,
        nonce: parameters.nonce,
        body_sha256: sha256_hex(body),
    })
}

/// Verify a fixed signed provider response and its originating request context.
#[allow(clippy::too_many_arguments)]
pub fn verify_response_signature(
    headers: &SignatureHeaders,
    status: u16,
    request: &RequestSignatureContext<'_>,
    response_message_id: Uuid,
    body: &[u8],
    verifying_key: &VerifyingKey,
    expected_key_id: &str,
    now: i64,
) -> Result<VerifiedHttpSignature> {
    request
        .validate()
        .map_err(|_| ProviderError::ProtocolRejected)?;
    if !(100..=599).contains(&status) {
        return Err(ProviderError::ProtocolRejected);
    }
    validate_identity_headers(
        headers,
        request.provider_id,
        request.release_id,
        response_message_id,
    )?;
    let parameters = parse_signature_input(
        &headers.signature_input,
        RESPONSE_COMPONENTS,
        expected_key_id,
        now,
    )?;
    validate_content_digest(headers, body)?;
    let base = response_signature_base(
        status,
        request,
        response_message_id,
        &headers.content_digest,
        &headers.signature_input,
    )
    .map_err(|_| ProviderError::ProtocolRejected)?;
    verify_signature_value(&headers.signature, base.as_bytes(), verifying_key)?;
    Ok(VerifiedHttpSignature {
        key_id: parameters.key_id,
        created_at: parameters.created_at,
        expires_at: parameters.expires_at,
        nonce: parameters.nonce,
        body_sha256: sha256_hex(body),
    })
}

/// Parse bounded authenticated JSON only after signature verification.
pub fn parse_authenticated_json<T: DeserializeOwned>(bytes: &[u8], limit: usize) -> Result<T> {
    if bytes.is_empty() || bytes.len() > limit {
        return Err(ProviderError::ProtocolRejected);
    }
    serde_json::from_slice(bytes).map_err(|_| ProviderError::ProtocolRejected)
}

/// SHA-256 lowercase hexadecimal digest.
#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    HEXLOWER.encode(&Sha256::digest(bytes))
}

/// Derive one opaque provider/game pairwise persona subject.
pub fn pairwise_subject(
    secret: &[u8],
    provider_id: &str,
    game_key: &str,
    persona_id: Uuid,
) -> Result<String> {
    if secret.len() < 32
        || !is_identifier(provider_id, 3, 64, b"-_")
        || !is_identifier(game_key, 3, 32, b"-_")
        || persona_id.is_nil()
    {
        return Err(ProviderError::InvalidInput);
    }
    let mut mac = HmacSha256::new_from_slice(secret).map_err(|_| ProviderError::InvalidInput)?;
    mac.update(b"omarchygs-pairwise-subject-v1\0");
    mac.update(provider_id.as_bytes());
    mac.update(&[0]);
    mac.update(game_key.as_bytes());
    mac.update(&[0]);
    mac.update(persona_id.as_bytes());
    Ok(URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes()))
}

fn sign_grant<T: Serialize>(
    value: &T,
    key_id: &str,
    signing_key: &SigningKey,
) -> Result<SignedProviderGrant> {
    if !is_identifier(key_id, 3, 64, b"._-") {
        return Err(ProviderError::InvalidInput);
    }
    let payload = serde_json::to_vec(value).map_err(|_| ProviderError::Internal)?;
    if payload.is_empty() || payload.len() > MAX_SIGNED_GRANT_BYTES {
        return Err(ProviderError::InvalidInput);
    }
    let mut message = Vec::with_capacity(GRANT_DOMAIN.len() + payload.len());
    message.extend_from_slice(GRANT_DOMAIN);
    message.extend_from_slice(&payload);
    let signature = signing_key.sign(&message);
    Ok(SignedProviderGrant {
        key_id: key_id.to_owned(),
        payload: URL_SAFE_NO_PAD.encode(payload),
        signature: URL_SAFE_NO_PAD.encode(signature.to_bytes()),
    })
}

fn signature_input(
    components: &str,
    created_at: i64,
    expires_at: i64,
    nonce: &str,
    key_id: &str,
) -> Result<String> {
    if created_at < 0
        || expires_at <= created_at
        || expires_at - created_at > MAX_MESSAGE_SIGNATURE_LIFETIME_SECONDS
        || !is_identifier(nonce, 16, 64, b"-_")
        || !is_identifier(key_id, 3, 64, b"._-")
    {
        return Err(ProviderError::InvalidInput);
    }
    Ok(format!(
        "{SIGNATURE_LABEL}={components};created={created_at};expires={expires_at};nonce=\"{nonce}\";keyid=\"{key_id}\";alg=\"ed25519\";tag=\"{SIGNATURE_TAG}\""
    ))
}

struct ParsedSignatureInput {
    created_at: i64,
    expires_at: i64,
    nonce: String,
    key_id: String,
}

fn parse_signature_input(
    input: &str,
    expected_components: &str,
    expected_key_id: &str,
    now: i64,
) -> Result<ParsedSignatureInput> {
    if input.len() > 2048 || !input.is_ascii() || input.contains(['\r', '\n']) {
        return Err(ProviderError::ProtocolRejected);
    }
    let prefix = format!("{SIGNATURE_LABEL}={expected_components};created=");
    let mut rest = input
        .strip_prefix(&prefix)
        .ok_or(ProviderError::ProtocolRejected)?;
    let (created, next) = take_integer(rest, ";expires=")?;
    rest = next;
    let (expires, next) = take_integer(rest, ";nonce=\"")?;
    rest = next;
    let (nonce, next) = take_quoted(rest, "\";keyid=\"")?;
    rest = next;
    let (key_id, tail) = take_quoted(rest, "\";alg=\"ed25519\";tag=\"")?;
    let expected_tail = format!("{SIGNATURE_TAG}\"");
    if tail != expected_tail
        || key_id != expected_key_id
        || !is_identifier(&key_id, 3, 64, b"._-")
        || !is_identifier(&nonce, 16, 64, b"-_")
        || created < 0
        || created > now.saturating_add(MAX_CLOCK_SKEW_SECONDS)
        || expires <= now
        || expires <= created
        || expires - created > MAX_MESSAGE_SIGNATURE_LIFETIME_SECONDS
    {
        return Err(ProviderError::ProtocolRejected);
    }
    Ok(ParsedSignatureInput {
        created_at: created,
        expires_at: expires,
        nonce,
        key_id,
    })
}

fn take_integer<'a>(value: &'a str, separator: &str) -> Result<(i64, &'a str)> {
    let (raw, rest) = value
        .split_once(separator)
        .ok_or(ProviderError::ProtocolRejected)?;
    if raw.is_empty() || raw.len() > 20 || !raw.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(ProviderError::ProtocolRejected);
    }
    let parsed = raw
        .parse::<i64>()
        .map_err(|_| ProviderError::ProtocolRejected)?;
    Ok((parsed, rest))
}

fn take_quoted<'a>(value: &'a str, separator: &str) -> Result<(String, &'a str)> {
    let (raw, rest) = value
        .split_once(separator)
        .ok_or(ProviderError::ProtocolRejected)?;
    if raw.is_empty() || raw.len() > 96 || raw.contains(['"', '\\', '\r', '\n']) {
        return Err(ProviderError::ProtocolRejected);
    }
    Ok((raw.to_owned(), rest))
}

fn request_signature_base(
    context: &RequestSignatureContext<'_>,
    content_digest: &str,
    signature_input: &str,
) -> Result<String> {
    context.validate()?;
    let parameters = signature_input
        .strip_prefix(&format!("{SIGNATURE_LABEL}="))
        .ok_or(ProviderError::InvalidInput)?;
    Ok(format!(
        "\"@method\": {}\n\"@authority\": {}\n\"@path\": {}\n\"content-digest\": {}\n\"content-type\": {}\n\"x-ogs-provider\": {}\n\"x-ogs-release\": {}\n\"x-ogs-message-id\": {}\n\"@signature-params\": {}",
        context.method,
        context.authority,
        context.path,
        content_digest,
        CONTENT_TYPE,
        context.provider_id,
        context.release_id,
        context.message_id,
        parameters
    ))
}

fn response_signature_base(
    status: u16,
    request: &RequestSignatureContext<'_>,
    response_message_id: Uuid,
    content_digest: &str,
    signature_input: &str,
) -> Result<String> {
    request.validate()?;
    if !(100..=599).contains(&status) || response_message_id.is_nil() {
        return Err(ProviderError::InvalidInput);
    }
    let parameters = signature_input
        .strip_prefix(&format!("{SIGNATURE_LABEL}="))
        .ok_or(ProviderError::InvalidInput)?;
    Ok(format!(
        "\"@status\": {}\n\"@method\";req: {}\n\"@authority\";req: {}\n\"@path\";req: {}\n\"content-digest\": {}\n\"content-type\": {}\n\"x-ogs-provider\": {}\n\"x-ogs-release\": {}\n\"x-ogs-message-id\": {}\n\"@signature-params\": {}",
        status,
        request.method,
        request.authority,
        request.path,
        content_digest,
        CONTENT_TYPE,
        request.provider_id,
        request.release_id,
        response_message_id,
        parameters
    ))
}

fn validate_identity_headers(
    headers: &SignatureHeaders,
    provider_id: &str,
    release_id: Uuid,
    message_id: Uuid,
) -> Result<()> {
    if headers.provider_id != provider_id
        || headers.release_id != release_id.to_string()
        || headers.message_id != message_id.to_string()
        || headers.content_type != CONTENT_TYPE
    {
        Err(ProviderError::ProtocolRejected)
    } else {
        Ok(())
    }
}

fn validate_content_digest(headers: &SignatureHeaders, body: &[u8]) -> Result<()> {
    if headers.content_digest == content_digest(body) {
        Ok(())
    } else {
        Err(ProviderError::ProtocolRejected)
    }
}

fn content_digest(body: &[u8]) -> String {
    format!("sha-256=:{}:", STANDARD.encode(Sha256::digest(body)))
}

fn verify_signature_value(value: &str, message: &[u8], verifying_key: &VerifyingKey) -> Result<()> {
    if value.len() > 128 || !value.starts_with("ogs=:") || !value.ends_with(':') {
        return Err(ProviderError::ProtocolRejected);
    }
    let encoded = &value[5..value.len() - 1];
    let bytes = STANDARD
        .decode(encoded)
        .map_err(|_| ProviderError::ProtocolRejected)?;
    let signature = Signature::from_slice(&bytes).map_err(|_| ProviderError::ProtocolRejected)?;
    verifying_key
        .verify(message, &signature)
        .map_err(|_| ProviderError::ProtocolRejected)
}

fn insert_header(headers: &mut HeaderMap, name: &'static str, value: &str) -> Result<()> {
    let name = HeaderName::from_static(name);
    let value = HeaderValue::from_str(value).map_err(|_| ProviderError::InvalidInput)?;
    headers.insert(name, value);
    Ok(())
}

fn one_header(headers: &HeaderMap, name: &'static str) -> Result<String> {
    let mut values = headers.get_all(name).iter();
    let value = values.next().ok_or(ProviderError::ProtocolRejected)?;
    if values.next().is_some() {
        return Err(ProviderError::ProtocolRejected);
    }
    value
        .to_str()
        .map(str::to_owned)
        .map_err(|_| ProviderError::ProtocolRejected)
}

fn validate_json_payload(value: &Value) -> Result<()> {
    let mut count = 0_usize;
    validate_json_value(value, 0, &mut count)
}

/// Validate bounded schema-owned provider data and reject credential-shaped
/// fields before it crosses either side of the protocol boundary.
pub fn validate_provider_payload(value: &Value) -> Result<()> {
    validate_json_payload(value)
}

fn validate_json_value(value: &Value, depth: usize, count: &mut usize) -> Result<()> {
    if depth > MAX_JSON_DEPTH {
        return Err(ProviderError::InvalidInput);
    }
    *count = count.checked_add(1).ok_or(ProviderError::InvalidInput)?;
    if *count > MAX_JSON_VALUES {
        return Err(ProviderError::InvalidInput);
    }
    match value {
        Value::Null | Value::Bool(_) => Ok(()),
        Value::Number(number) if number.is_i64() || number.is_u64() => Ok(()),
        Value::Number(_) => Err(ProviderError::InvalidInput),
        Value::String(text) => {
            if text.len() <= 4_096 && !text.chars().any(char::is_control) {
                Ok(())
            } else {
                Err(ProviderError::InvalidInput)
            }
        }
        Value::Array(values) => {
            if values.len() > 512 {
                return Err(ProviderError::InvalidInput);
            }
            for child in values {
                validate_json_value(child, depth + 1, count)?;
            }
            Ok(())
        }
        Value::Object(fields) => {
            if fields.len() > 256 {
                return Err(ProviderError::InvalidInput);
            }
            let mut names = BTreeSet::new();
            for (name, child) in fields {
                if !names.insert(name) || !is_payload_key(name) || is_sensitive_payload_key(name) {
                    return Err(ProviderError::InvalidInput);
                }
                validate_json_value(child, depth + 1, count)?;
            }
            Ok(())
        }
    }
}

fn is_payload_key(value: &str) -> bool {
    let bytes = value.as_bytes();
    (1..=64).contains(&bytes.len())
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'_')
}

fn is_sensitive_payload_key(value: &str) -> bool {
    matches!(
        value,
        "account_id"
            | "persona_id"
            | "device_token"
            | "session_token"
            | "password"
            | "credential"
            | "credentials"
            | "database_url"
            | "mfa_secret"
            | "recovery_code"
    ) || value.ends_with("_password")
        || value.ends_with("_token")
        || value.ends_with("_credential")
        || value.ends_with("_secret")
}

fn is_pairwise_subject(value: &str) -> bool {
    value.len() == 43
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn is_authority(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 260
        && value.is_ascii()
        && !value.contains(['/', '?', '#', '@', '\\', '\r', '\n'])
}

fn is_canonical_path(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 320
        && value.starts_with('/')
        && value.is_ascii()
        && !value.contains(['?', '#', '\\', '\r', '\n'])
        && !value.contains("//")
        && !value.split('/').any(|part| part == "." || part == "..")
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn grant_fixture(issuer: &GrantIssuer, now: i64) -> (ProviderGrantClaims, SignedProviderGrant) {
        let subject = issuer
            .pairwise_subject(
                "provider-one",
                "signal_siege",
                Uuid::from_u128(0x10000000000000000000000000000001),
            )
            .expect("pairwise subject should derive");
        let claims = ProviderGrantClaims::new(
            "provider-one".to_owned(),
            Uuid::from_u128(0x20000000000000000000000000000001),
            "signal_siege".to_owned(),
            1,
            "a".repeat(64),
            Uuid::from_u128(0x30000000000000000000000000000001),
            subject,
            ProviderScope::Command,
            ProviderCompatibility::current(),
            now,
            now + 60,
            Uuid::from_u128(0x40000000000000000000000000000001),
        )
        .expect("claims should validate");
        let signed = issuer.sign(&claims).expect("claims should sign");
        (claims, signed)
    }

    #[test]
    fn grant_binds_every_identity_and_expiry() {
        let issuer = GrantIssuer::new("platform-2026", [7; 32], vec![8; 32])
            .expect("issuer should construct");
        let now = 1_800_000_000;
        let (claims, signed) = grant_fixture(&issuer, now);
        let expected = GrantExpectation {
            key_id: "platform-2026",
            provider_id: &claims.provider_id,
            release_id: claims.release_id,
            game_key: &claims.game_key,
            rules_version: claims.rules_version,
            cartridge_digest: &claims.cartridge_digest,
            platform_session_id: claims.platform_session_id,
            subject: &claims.subject,
            scope: claims.scope,
            compatibility: &claims.compatibility,
        };
        assert_eq!(
            verify_grant(&signed, &issuer.verifying_key(), &expected, now)
                .expect("exact grant should verify"),
            claims
        );
        assert!(verify_grant(&signed, &issuer.verifying_key(), &expected, now + 60).is_err());
        let wrong = GrantExpectation {
            scope: ProviderScope::Launch,
            ..expected
        };
        assert!(verify_grant(&signed, &issuer.verifying_key(), &wrong, now).is_err());
    }

    #[test]
    fn pairwise_subject_changes_by_provider_game_and_persona() {
        let secret = [3_u8; 32];
        let persona = Uuid::from_u128(1);
        let first = pairwise_subject(&secret, "provider-one", "game_one", persona)
            .expect("first subject should derive");
        assert_eq!(first.len(), 43);
        assert_ne!(
            first,
            pairwise_subject(&secret, "provider-two", "game_one", persona)
                .expect("provider pair should derive")
        );
        assert_ne!(
            first,
            pairwise_subject(&secret, "provider-one", "game_two", persona)
                .expect("game pair should derive")
        );
        assert_ne!(
            first,
            pairwise_subject(&secret, "provider-one", "game_one", Uuid::from_u128(2))
                .expect("persona pair should derive")
        );
    }

    #[test]
    fn compatibility_requires_one_exact_current_profile_and_offer_binding() {
        let offer = ProviderCompatibilityOffer::current(
            "provider-one".to_owned(),
            Uuid::from_u128(20),
            Uuid::from_u128(21),
        )
        .expect("current offer should construct");
        let selection = ProviderCompatibilitySelection::current(&offer, Uuid::from_u128(22))
            .expect("current selection should construct");
        selection
            .validate_for(&offer)
            .expect("exact selection should verify");

        let mut stripped = offer.clone();
        stripped.supported[0].capabilities.pop();
        assert!(stripped.validate().is_err());

        let mut unknown = offer.clone();
        unknown.supported[0].protocol_version = 2;
        assert!(unknown.validate().is_err());

        let mut ambiguous = offer.clone();
        ambiguous.supported.push(ProviderCompatibility::current());
        assert!(ambiguous.validate().is_err());

        let wrong_offer = ProviderCompatibilityOffer::current(
            "provider-one".to_owned(),
            offer.release_id,
            Uuid::from_u128(23),
        )
        .expect("second offer should construct");
        assert!(selection.validate_for(&wrong_offer).is_err());
    }

    #[test]
    fn persisted_v1_receipts_upgrade_without_weakening_network_models() {
        let issuer = GrantIssuer::new("platform-2026", [7; 32], vec![8; 32])
            .expect("issuer should construct");
        let (claims, _) = grant_fixture(&issuer, 1_800_000_000);
        let response = ProviderOperationResponse::new(
            claims.provider_id.clone(),
            claims.release_id,
            claims.game_key.clone(),
            claims.rules_version,
            claims.cartridge_digest.clone(),
            claims.platform_session_id,
            claims.subject.clone(),
            Uuid::from_u128(31),
            Uuid::from_u128(32),
            1,
            ProviderOperationDisposition::Applied,
            ProviderSessionStatus::Active,
            ProviderCompatibility::current(),
            json!({"turn": 1}),
        );
        let mut legacy_response = serde_json::to_value(&response).expect("response should encode");
        legacy_response
            .as_object_mut()
            .expect("response should be an object")
            .remove("compatibility");
        let legacy_response =
            serde_json::to_vec(&legacy_response).expect("legacy response should encode");
        assert!(
            parse_authenticated_json::<ProviderOperationResponse>(&legacy_response, 65_536)
                .is_err()
        );
        assert_eq!(
            ProviderOperationResponse::from_persisted_v1_bytes(
                &legacy_response,
                65_536,
                ProviderCompatibility::current(),
            )
            .expect("persisted response should upgrade"),
            response
        );

        let event = ProviderEvent::new(
            claims.provider_id,
            claims.release_id,
            claims.game_key,
            claims.rules_version,
            claims.cartridge_digest,
            claims.platform_session_id,
            claims.subject,
            Uuid::from_u128(33),
            Uuid::from_u128(34),
            1,
            ProviderEventKind::TurnReady,
            ProviderCompatibility::current(),
            json!({"turn": 1}),
        );
        let mut legacy_event = serde_json::to_value(&event).expect("event should encode");
        legacy_event
            .as_object_mut()
            .expect("event should be an object")
            .remove("compatibility");
        let legacy_event = serde_json::to_vec(&legacy_event).expect("legacy event should encode");
        assert!(parse_authenticated_json::<ProviderEvent>(&legacy_event, 65_536).is_err());
        assert_eq!(
            ProviderEvent::from_persisted_v1_bytes(
                &legacy_event,
                65_536,
                ProviderCompatibility::current(),
            )
            .expect("persisted event should upgrade"),
            event
        );
    }

    #[test]
    fn request_signature_binds_body_and_context() {
        let signer =
            HttpMessageSigner::new("platform-2026", [5; 32]).expect("signer should construct");
        let context = RequestSignatureContext {
            method: "POST",
            authority: "provider.example.test",
            path: "/omarchygs/provider/v1/commands",
            provider_id: "provider-one",
            release_id: Uuid::from_u128(1),
            message_id: Uuid::from_u128(2),
        };
        let body = br#"{"command":"advance"}"#;
        let headers = signer
            .sign_request(&context, body, 1_800_000_000, "nonce-0000000001")
            .expect("request should sign");
        let verified = verify_request_signature(
            &headers,
            &context,
            body,
            &signer.verifying_key(),
            "platform-2026",
            1_800_000_001,
        )
        .expect("request should verify");
        assert_eq!(verified.body_sha256, sha256_hex(body));
        assert!(
            verify_request_signature(
                &headers,
                &context,
                b"{}",
                &signer.verifying_key(),
                "platform-2026",
                1_800_000_001,
            )
            .is_err()
        );
        let wrong_path = RequestSignatureContext {
            path: "/omarchygs/provider/v1/admin",
            ..context
        };
        assert!(
            verify_request_signature(
                &headers,
                &wrong_path,
                body,
                &signer.verifying_key(),
                "platform-2026",
                1_800_000_001,
            )
            .is_err()
        );
    }

    #[test]
    fn response_signature_binds_status_request_and_response_identity() {
        let signer =
            HttpMessageSigner::new("provider-key", [6; 32]).expect("signer should construct");
        let request = RequestSignatureContext {
            method: "POST",
            authority: "provider.example.test",
            path: "/omarchygs/provider/v1/reconcile",
            provider_id: "provider-one",
            release_id: Uuid::from_u128(10),
            message_id: Uuid::from_u128(11),
        };
        let response_message_id = Uuid::from_u128(12);
        let body = br#"{"revision":1}"#;
        let headers = signer
            .sign_response(
                200,
                &request,
                response_message_id,
                body,
                1_800_000_000,
                "nonce-0000000002",
            )
            .expect("response should sign");
        verify_response_signature(
            &headers,
            200,
            &request,
            response_message_id,
            body,
            &signer.verifying_key(),
            "provider-key",
            1_800_000_001,
        )
        .expect("response should verify");
        assert!(
            verify_response_signature(
                &headers,
                201,
                &request,
                response_message_id,
                body,
                &signer.verifying_key(),
                "provider-key",
                1_800_000_001,
            )
            .is_err()
        );
    }

    #[test]
    fn payload_rejects_sensitive_fields_float_depth_and_controls() {
        assert!(validate_json_payload(&json!({"action": "advance", "count": 2})).is_ok());
        for invalid in [
            json!({"account_id": "hidden"}),
            json!({"device_token": "hidden"}),
            json!({"password": "hidden"}),
            json!({"ratio": 0.5}),
            json!({"text": "bad\ntext"}),
        ] {
            assert!(
                validate_json_payload(&invalid).is_err(),
                "{invalid} should reject"
            );
        }
    }
}
