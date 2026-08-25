//! Durable provider operation, guarded transport, response, callback, and
//! reconciliation orchestration.

use std::time::SystemTime;

#[cfg(feature = "provider-conformance")]
use std::net::SocketAddr;

use ed25519_dalek::VerifyingKey;
use serde::Serialize;
use serde_json::{Value, json};
use sqlx::{FromRow, Postgres, Transaction};
use tracing::error;
use uuid::Uuid;

use crate::{
    ProviderError, Result,
    egress::GuardedProviderClient,
    model::SessionAdmission,
    protocol::{
        GrantIssuer, HttpMessageSigner, ProviderEvent, ProviderOperationKind,
        ProviderOperationRequest, ProviderOperationResponse, RequestSignatureContext,
        SignatureHeaders, parse_authenticated_json, sha256_hex, validate_provider_payload,
        verify_request_signature, verify_response_signature,
    },
    registry::{ConcurrencyLease, ProviderRegistry, ProviderSecurityMaterial},
};

/// One semantic provider operation. No account/device credential can be supplied.
pub struct BrokerOperationInput {
    /// Exact registered release.
    pub release_id: Uuid,
    /// Local owned persona used only for pairwise derivation.
    pub persona_id: Uuid,
    /// Exact platform session envelope.
    pub platform_session_id: Uuid,
    /// Stable caller idempotency identity.
    pub idempotency_key: Uuid,
    /// Expected provider revision.
    pub expected_revision: u64,
    /// Launch, command, or reconciliation.
    pub operation: ProviderOperationKind,
    /// New or proven existing session.
    pub session: SessionAdmission,
    /// Bounded schema-owned operation data.
    pub payload: Value,
}

impl BrokerOperationInput {
    fn validate(&self) -> Result<()> {
        if self.release_id.is_nil()
            || self.persona_id.is_nil()
            || self.platform_session_id.is_nil()
            || self.idempotency_key.is_nil()
            || (self.operation == ProviderOperationKind::Launch
                && self.session != SessionAdmission::New)
            || (self.operation != ProviderOperationKind::Launch
                && self.session != SessionAdmission::Existing)
        {
            return Err(ProviderError::InvalidInput);
        }
        validate_provider_payload(&self.payload)
    }
}

/// First accepted or exact duplicate callback disposition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallbackDisposition {
    /// First authenticated event identity.
    Accepted,
    /// Exact authenticated replay of an already retained event.
    Duplicate,
}

enum TransportProfile {
    Production,
    #[cfg(feature = "provider-conformance")]
    Conformance(SocketAddr),
}

/// OmarchyGS-only broker for one dormant provider protocol foundation.
pub struct ProviderBroker {
    registry: ProviderRegistry,
    grant_issuer: GrantIssuer,
    message_signer: HttpMessageSigner,
    transport: TransportProfile,
}

impl ProviderBroker {
    /// Construct the production public-network-only broker.
    #[must_use]
    pub fn new(
        registry: ProviderRegistry,
        grant_issuer: GrantIssuer,
        message_signer: HttpMessageSigner,
    ) -> Self {
        Self {
            registry,
            grant_issuer,
            message_signer,
            transport: TransportProfile::Production,
        }
    }

    /// Construct the compile-time-gated exact loopback conformance broker.
    #[cfg(feature = "provider-conformance")]
    #[must_use]
    pub fn conformance_loopback(
        registry: ProviderRegistry,
        grant_issuer: GrantIssuer,
        message_signer: HttpMessageSigner,
        exact_socket: SocketAddr,
    ) -> Self {
        Self {
            registry,
            grant_issuer,
            message_signer,
            transport: TransportProfile::Conformance(exact_socket),
        }
    }

    /// Execute or exactly replay one durable provider operation.
    pub async fn execute(&self, input: &BrokerOperationInput) -> Result<ProviderOperationResponse> {
        input.validate()?;
        let initial_policy = self.registry.load_policy(input.release_id).await?;
        let subject = self.grant_issuer.pairwise_subject(
            &initial_policy.provider_id,
            &initial_policy.game_key,
            input.persona_id,
        )?;
        let intent = OperationIntent {
            schema: "omarchygs.provider-operation-intent/v1",
            provider_id: &initial_policy.provider_id,
            release_id: input.release_id,
            game_key: &initial_policy.game_key,
            rules_version: initial_policy.rules_version,
            cartridge_digest: &initial_policy.cartridge_digest,
            platform_session_id: input.platform_session_id,
            subject: &subject,
            idempotency_key: input.idempotency_key,
            expected_revision: input.expected_revision,
            operation: input.operation,
            payload: &input.payload,
        };
        let intent_bytes = serde_json::to_vec(&intent).map_err(|_| ProviderError::Internal)?;
        if intent_bytes.is_empty()
            || intent_bytes.len() > initial_policy.quotas.request_body_bytes as usize
        {
            return Err(ProviderError::InvalidInput);
        }
        let intent_digest = sha256_hex(&intent_bytes);
        if let Some(response) = self
            .prepare_operation(
                input,
                &intent_bytes,
                &intent_digest,
                initial_policy.quotas.total_timeout_ms,
            )
            .await?
        {
            return Ok(response);
        }

        let grant = self
            .registry
            .issue_grant(
                &self.grant_issuer,
                &crate::registry::IssueGrantRequest {
                    release_id: input.release_id,
                    persona_id: input.persona_id,
                    platform_session_id: input.platform_session_id,
                    scope: input.operation.scope(),
                    session: input.session,
                },
            )
            .await?;
        let message_id = Uuid::new_v4();
        let request = ProviderOperationRequest::new(
            grant.claims.provider_id.clone(),
            grant.claims.release_id,
            grant.claims.game_key.clone(),
            grant.claims.rules_version,
            grant.claims.cartridge_digest.clone(),
            grant.claims.platform_session_id,
            grant.claims.subject.clone(),
            message_id,
            input.idempotency_key,
            input.expected_revision,
            input.operation,
            input.payload.clone(),
            grant.signed,
        )?;
        let request_limit = initial_policy.quotas.request_body_bytes as usize;
        let request_bytes = request.to_bytes(request_limit)?;
        let correlation_id = message_id;
        let (material, lease) = self
            .registry
            .begin_request(
                input.release_id,
                input.operation.scope(),
                input.session,
                correlation_id,
            )
            .await?;
        self.execute_attempt(
            input,
            &intent_digest,
            &request,
            request_bytes,
            material,
            lease,
            grant.claims.token_id,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    async fn execute_attempt(
        &self,
        input: &BrokerOperationInput,
        intent_digest: &str,
        request: &ProviderOperationRequest,
        request_bytes: Vec<u8>,
        material: ProviderSecurityMaterial,
        lease: ConcurrencyLease,
        grant_token_id: Uuid,
    ) -> Result<ProviderOperationResponse> {
        let request_digest = sha256_hex(&request_bytes);
        let attempt_number = match self
            .create_attempt(
                input,
                intent_digest,
                request.message_id,
                grant_token_id,
                &request_digest,
                &request_bytes,
                material.policy.quotas.total_timeout_ms,
            )
            .await
        {
            Ok(number) => number,
            Err(error) => {
                self.registry.release_request(lease).await?;
                return Err(error);
            }
        };
        let attempt_result = self
            .perform_attempt(
                input,
                intent_digest,
                attempt_number,
                request,
                request_bytes,
                &material,
            )
            .await;
        let release_result = self.registry.release_request(lease).await;
        match attempt_result {
            Ok(response) => {
                release_result?;
                Ok(response)
            }
            Err(error) => {
                let finish_result = self
                    .finish_failed_attempt(input, attempt_number, &error)
                    .await;
                let audit_result = self
                    .audit_transport_failure(input.release_id, request.message_id, &error)
                    .await;
                if release_result.is_err() || finish_result.is_err() || audit_result.is_err() {
                    return Err(ProviderError::Internal);
                }
                Err(error)
            }
        }
    }

    async fn perform_attempt(
        &self,
        input: &BrokerOperationInput,
        intent_digest: &str,
        attempt_number: i32,
        request: &ProviderOperationRequest,
        request_bytes: Vec<u8>,
        material: &ProviderSecurityMaterial,
    ) -> Result<ProviderOperationResponse> {
        let operation_url = material
            .policy
            .endpoint
            .operation_url(request.operation.path())?;
        let authority = material.policy.endpoint.authority();
        let path = operation_url.path().to_owned();
        let context = RequestSignatureContext {
            method: "POST",
            authority: &authority,
            path: &path,
            provider_id: &material.policy.provider_id,
            release_id: material.policy.release_id,
            message_id: request.message_id,
        };
        let now = current_unix_seconds()?;
        let headers = self.message_signer.sign_request(
            &context,
            &request_bytes,
            now,
            &format!("n-{}", Uuid::new_v4()),
        )?;
        let client = self.guarded_client(material).await?;
        let raw = client
            .post(
                request.operation.path(),
                headers.to_header_map()?,
                request_bytes,
            )
            .await;
        let raw = raw?;
        if raw.status != 200 {
            return Err(ProviderError::ProtocolRejected);
        }
        let response_headers = SignatureHeaders::from_header_map(&raw.headers)?;
        let response_message_id = response_headers
            .message_id
            .parse::<Uuid>()
            .map_err(|_| ProviderError::ProtocolRejected)?;
        let verified = verify_response_with_active_keys(
            material,
            &response_headers,
            raw.status,
            &context,
            response_message_id,
            &raw.body,
            current_unix_seconds()?,
        )?;
        let response: ProviderOperationResponse = parse_authenticated_json(
            &raw.body,
            material.policy.quotas.response_body_bytes as usize,
        )?;
        response.validate_for(request)?;
        if response.message_id != response_message_id {
            return Err(ProviderError::ProtocolRejected);
        }
        self.complete_operation(
            input,
            intent_digest,
            attempt_number,
            &response,
            &raw.body,
            &verified.body_sha256,
        )
        .await?;
        self.registry
            .record_security_event(
                input.release_id,
                "broker",
                "provider_broker",
                "response_authenticated",
                "recorded",
                "response_authenticated",
                Some(request.message_id),
                json!({
                    "provider_message_id": response.message_id,
                    "provider_revision": response.revision,
                    "key_id": verified.key_id
                }),
            )
            .await?;
        Ok(response)
    }

    async fn guarded_client(
        &self,
        material: &ProviderSecurityMaterial,
    ) -> Result<GuardedProviderClient> {
        match self.transport {
            TransportProfile::Production => {
                GuardedProviderClient::production(
                    material.policy.endpoint.clone(),
                    &material.tls_roots_der,
                    material.policy.quotas.clone(),
                )
                .await
            }
            #[cfg(feature = "provider-conformance")]
            TransportProfile::Conformance(socket) => GuardedProviderClient::conformance_loopback(
                material.policy.endpoint.clone(),
                socket,
                &material.tls_roots_der,
                material.policy.quotas.clone(),
            ),
        }
    }

    /// Authenticate and deduplicate a callback-shaped provider request. This
    /// method records no result, achievement, notification, or gameplay state.
    pub async fn ingest_callback(
        &self,
        release_id: Uuid,
        context: &RequestSignatureContext<'_>,
        headers: &SignatureHeaders,
        body: &[u8],
        now: i64,
    ) -> Result<(CallbackDisposition, ProviderEvent)> {
        if release_id.is_nil() || context.release_id != release_id {
            return Err(ProviderError::InvalidInput);
        }
        let material = self
            .registry
            .admit_callback(release_id, context.message_id)
            .await?;
        let authenticated = (|| {
            if context.provider_id != material.policy.provider_id {
                return Err(ProviderError::ProtocolRejected);
            }
            let verified = verify_request_with_active_keys(&material, headers, context, body, now)?;
            let event: ProviderEvent = parse_authenticated_json(
                body,
                material.policy.quotas.response_body_bytes as usize,
            )?;
            event.validate()?;
            if event.provider_id != material.policy.provider_id
                || event.release_id != release_id
                || event.game_key != material.policy.game_key
                || event.rules_version != material.policy.rules_version
                || event.cartridge_digest != material.policy.cartridge_digest
                || event.message_id != context.message_id
            {
                return Err(ProviderError::ProtocolRejected);
            }
            Ok((verified, event))
        })();
        let (verified, event) = match authenticated {
            Ok(value) => value,
            Err(error) => {
                self.registry
                    .record_security_event(
                        release_id,
                        "provider",
                        "registered_provider",
                        "callback_rejected",
                        "denied",
                        error.code(),
                        Some(context.message_id),
                        json!({}),
                    )
                    .await?;
                return Err(error);
            }
        };
        let disposition = self
            .record_callback_receipt(&event, &verified.body_sha256)
            .await?;
        self.registry
            .record_security_event(
                release_id,
                "provider",
                "registered_provider",
                "callback_authenticated",
                "recorded",
                match disposition {
                    CallbackDisposition::Accepted => "callback_accepted",
                    CallbackDisposition::Duplicate => "callback_duplicate",
                },
                Some(event.message_id),
                json!({
                    "event_id": event.event_id,
                    "provider_revision": event.revision,
                    "key_id": verified.key_id
                }),
            )
            .await?;
        Ok((disposition, event))
    }

    async fn prepare_operation(
        &self,
        input: &BrokerOperationInput,
        intent_bytes: &[u8],
        intent_digest: &str,
        total_timeout_ms: u32,
    ) -> Result<Option<ProviderOperationResponse>> {
        let mut transaction =
            self.registry.pool().begin().await.map_err(|error| {
                map_database_error(error, "begin provider operation preparation")
            })?;
        let existing = sqlx::query_as::<_, OperationRow>(
            r#"
            SELECT
                intent_sha256,
                intent_body,
                status,
                response_body,
                EXTRACT(EPOCH FROM updated_at)::BIGINT AS updated_unix
            FROM provider_operations
            WHERE release_id = $1
              AND platform_session_id = $2
              AND idempotency_key = $3
            FOR UPDATE
            "#,
        )
        .bind(input.release_id)
        .bind(input.platform_session_id)
        .bind(input.idempotency_key)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|error| map_database_error(error, "load provider operation receipt"))?;
        if let Some(existing) = existing {
            if existing.intent_sha256 != intent_digest || existing.intent_body != intent_bytes {
                return Err(ProviderError::Conflict);
            }
            if existing.status == "completed" {
                let response_body = existing.response_body.ok_or(ProviderError::Internal)?;
                let response = parse_authenticated_json(&response_body, 524_288)?;
                transaction.commit().await.map_err(|error| {
                    map_database_error(error, "commit provider operation replay")
                })?;
                return Ok(Some(response));
            }
            if existing.status == "in_flight" {
                let now = current_unix_seconds()?;
                let timeout = i64::from(total_timeout_ms) / 1_000 + 2;
                if now.saturating_sub(existing.updated_unix) <= timeout {
                    return Err(ProviderError::Conflict);
                }
                sqlx::query(
                    r#"
                    UPDATE provider_operations
                    SET status = 'unknown',
                        last_error_code = 'provider_attempt_abandoned',
                        updated_at = clock_timestamp()
                    WHERE release_id = $1
                      AND platform_session_id = $2
                      AND idempotency_key = $3
                    "#,
                )
                .bind(input.release_id)
                .bind(input.platform_session_id)
                .bind(input.idempotency_key)
                .execute(&mut *transaction)
                .await
                .map_err(|error| {
                    map_database_error(error, "expire provider in-flight operation")
                })?;
            }
        } else {
            sqlx::query(
                r#"
                INSERT INTO provider_operations (
                    release_id,
                    platform_session_id,
                    idempotency_key,
                    scope,
                    expected_revision,
                    intent_sha256,
                    intent_body
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
            )
            .bind(input.release_id)
            .bind(input.platform_session_id)
            .bind(input.idempotency_key)
            .bind(input.operation.scope().as_str())
            .bind(i64::try_from(input.expected_revision).map_err(|_| ProviderError::InvalidInput)?)
            .bind(intent_digest)
            .bind(intent_bytes)
            .execute(&mut *transaction)
            .await
            .map_err(|error| map_conflict(error, "insert provider operation receipt"))?;
        }
        transaction
            .commit()
            .await
            .map_err(|error| map_database_error(error, "commit provider operation preparation"))?;
        Ok(None)
    }

    #[allow(clippy::too_many_arguments)]
    async fn create_attempt(
        &self,
        input: &BrokerOperationInput,
        intent_digest: &str,
        message_id: Uuid,
        grant_token_id: Uuid,
        request_digest: &str,
        request_body: &[u8],
        total_timeout_ms: u32,
    ) -> Result<i32> {
        let mut transaction = self
            .registry
            .pool()
            .begin()
            .await
            .map_err(|error| map_database_error(error, "begin provider operation attempt"))?;
        let row: AttemptRootRow = sqlx::query_as(
            r#"
            SELECT
                intent_sha256,
                status,
                attempt_count,
                EXTRACT(EPOCH FROM updated_at)::BIGINT AS updated_unix
            FROM provider_operations
            WHERE release_id = $1
              AND platform_session_id = $2
              AND idempotency_key = $3
            FOR UPDATE
            "#,
        )
        .bind(input.release_id)
        .bind(input.platform_session_id)
        .bind(input.idempotency_key)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|error| map_database_error(error, "lock provider operation attempt"))?
        .ok_or(ProviderError::Internal)?;
        if row.intent_sha256 != intent_digest || row.status == "completed" {
            return Err(ProviderError::Conflict);
        }
        if row.status == "in_flight" {
            let timeout = i64::from(total_timeout_ms) / 1_000 + 2;
            if current_unix_seconds()?.saturating_sub(row.updated_unix) <= timeout {
                return Err(ProviderError::Conflict);
            }
        }
        let attempt_number = row
            .attempt_count
            .checked_add(1)
            .ok_or(ProviderError::Internal)?;
        sqlx::query(
            r#"
            UPDATE provider_operations
            SET status = 'in_flight',
                attempt_count = $4,
                last_error_code = NULL,
                updated_at = clock_timestamp()
            WHERE release_id = $1
              AND platform_session_id = $2
              AND idempotency_key = $3
            "#,
        )
        .bind(input.release_id)
        .bind(input.platform_session_id)
        .bind(input.idempotency_key)
        .bind(attempt_number)
        .execute(&mut *transaction)
        .await
        .map_err(|error| map_database_error(error, "update provider operation attempt"))?;
        sqlx::query(
            r#"
            INSERT INTO provider_operation_attempts (
                release_id,
                platform_session_id,
                idempotency_key,
                attempt_number,
                message_id,
                grant_token_id,
                request_sha256,
                request_body
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(input.release_id)
        .bind(input.platform_session_id)
        .bind(input.idempotency_key)
        .bind(attempt_number)
        .bind(message_id)
        .bind(grant_token_id)
        .bind(request_digest)
        .bind(request_body)
        .execute(&mut *transaction)
        .await
        .map_err(|error| map_conflict(error, "insert provider operation attempt"))?;
        transaction
            .commit()
            .await
            .map_err(|error| map_database_error(error, "commit provider operation attempt"))?;
        Ok(attempt_number)
    }

    async fn finish_failed_attempt(
        &self,
        input: &BrokerOperationInput,
        attempt_number: i32,
        provider_error: &ProviderError,
    ) -> Result<()> {
        let (attempt_status, operation_status) =
            if matches!(provider_error, ProviderError::Unavailable) {
                ("unknown", "unknown")
            } else {
                ("failed", "failed")
            };
        let mut transaction = self
            .registry
            .pool()
            .begin()
            .await
            .map_err(|error| map_database_error(error, "begin provider failed attempt"))?;
        sqlx::query(
            r#"
            UPDATE provider_operation_attempts
            SET status = $5,
                error_code = $6,
                updated_at = clock_timestamp()
            WHERE release_id = $1
              AND platform_session_id = $2
              AND idempotency_key = $3
              AND attempt_number = $4
              AND status = 'in_flight'
            "#,
        )
        .bind(input.release_id)
        .bind(input.platform_session_id)
        .bind(input.idempotency_key)
        .bind(attempt_number)
        .bind(attempt_status)
        .bind(provider_error.code())
        .execute(&mut *transaction)
        .await
        .map_err(|error| map_database_error(error, "mark provider attempt failed"))?;
        sqlx::query(
            r#"
            UPDATE provider_operations
            SET status = $4,
                last_error_code = $5,
                updated_at = clock_timestamp()
            WHERE release_id = $1
              AND platform_session_id = $2
              AND idempotency_key = $3
              AND status <> 'completed'
            "#,
        )
        .bind(input.release_id)
        .bind(input.platform_session_id)
        .bind(input.idempotency_key)
        .bind(operation_status)
        .bind(provider_error.code())
        .execute(&mut *transaction)
        .await
        .map_err(|error| map_database_error(error, "mark provider operation failed"))?;
        transaction
            .commit()
            .await
            .map_err(|error| map_database_error(error, "commit provider failed attempt"))?;
        Ok(())
    }

    async fn complete_operation(
        &self,
        input: &BrokerOperationInput,
        intent_digest: &str,
        attempt_number: i32,
        response: &ProviderOperationResponse,
        response_body: &[u8],
        response_digest: &str,
    ) -> Result<()> {
        let mut transaction =
            self.registry.pool().begin().await.map_err(|error| {
                map_database_error(error, "begin provider operation completion")
            })?;
        let stored_intent: String = sqlx::query_scalar(
            r#"
            SELECT intent_sha256
            FROM provider_operations
            WHERE release_id = $1
              AND platform_session_id = $2
              AND idempotency_key = $3
            FOR UPDATE
            "#,
        )
        .bind(input.release_id)
        .bind(input.platform_session_id)
        .bind(input.idempotency_key)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|error| map_database_error(error, "lock provider operation completion"))?;
        if stored_intent != intent_digest {
            return Err(ProviderError::Conflict);
        }
        record_message_receipt(
            &mut transaction,
            input.release_id,
            "response",
            response.message_id,
            input.platform_session_id,
            None,
            response_digest,
            response.revision,
        )
        .await?;
        sqlx::query(
            r#"
            UPDATE provider_operation_attempts
            SET status = 'completed',
                error_code = NULL,
                updated_at = clock_timestamp()
            WHERE release_id = $1
              AND platform_session_id = $2
              AND idempotency_key = $3
              AND attempt_number = $4
            "#,
        )
        .bind(input.release_id)
        .bind(input.platform_session_id)
        .bind(input.idempotency_key)
        .bind(attempt_number)
        .execute(&mut *transaction)
        .await
        .map_err(|error| map_database_error(error, "complete provider operation attempt"))?;
        sqlx::query(
            r#"
            UPDATE provider_operations
            SET status = 'completed',
                provider_revision = $4,
                response_sha256 = $5,
                response_body = $6,
                last_error_code = NULL,
                updated_at = clock_timestamp()
            WHERE release_id = $1
              AND platform_session_id = $2
              AND idempotency_key = $3
            "#,
        )
        .bind(input.release_id)
        .bind(input.platform_session_id)
        .bind(input.idempotency_key)
        .bind(i64::try_from(response.revision).map_err(|_| ProviderError::ProtocolRejected)?)
        .bind(response_digest)
        .bind(response_body)
        .execute(&mut *transaction)
        .await
        .map_err(|error| map_database_error(error, "complete provider operation receipt"))?;
        transaction
            .commit()
            .await
            .map_err(|error| map_database_error(error, "commit provider operation completion"))?;
        Ok(())
    }

    async fn record_callback_receipt(
        &self,
        event: &ProviderEvent,
        body_digest: &str,
    ) -> Result<CallbackDisposition> {
        let mut transaction = self
            .registry
            .pool()
            .begin()
            .await
            .map_err(|error| map_database_error(error, "begin provider callback receipt"))?;
        sqlx::query("SELECT release_id FROM provider_releases WHERE release_id = $1 FOR UPDATE")
            .bind(event.release_id)
            .fetch_one(&mut *transaction)
            .await
            .map_err(|error| map_database_error(error, "lock provider callback receipt root"))?;
        let existing = sqlx::query_as::<_, MessageReceiptRow>(
            r#"
            SELECT message_id, event_id, authenticated_sha256, platform_session_id
            FROM provider_message_receipts
            WHERE release_id = $1
              AND direction = 'callback'
              AND (message_id = $2 OR event_id = $3)
            FOR UPDATE
            "#,
        )
        .bind(event.release_id)
        .bind(event.message_id)
        .bind(event.event_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|error| map_database_error(error, "load provider callback receipt"))?;
        if let Some(existing) = existing {
            if existing.message_id == event.message_id
                && existing.event_id == Some(event.event_id)
                && existing.authenticated_sha256 == body_digest
                && existing.platform_session_id == event.platform_session_id
            {
                transaction
                    .commit()
                    .await
                    .map_err(|error| map_database_error(error, "commit callback duplicate"))?;
                return Ok(CallbackDisposition::Duplicate);
            }
            return Err(ProviderError::Conflict);
        }
        record_message_receipt(
            &mut transaction,
            event.release_id,
            "callback",
            event.message_id,
            event.platform_session_id,
            Some(event.event_id),
            body_digest,
            event.revision,
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|error| map_database_error(error, "commit provider callback receipt"))?;
        Ok(CallbackDisposition::Accepted)
    }

    async fn audit_transport_failure(
        &self,
        release_id: Uuid,
        correlation_id: Uuid,
        provider_error: &ProviderError,
    ) -> Result<()> {
        self.registry
            .record_security_event(
                release_id,
                "broker",
                "provider_broker",
                "provider_request_failed",
                "failed",
                provider_error.code(),
                Some(correlation_id),
                json!({}),
            )
            .await
    }
}

#[derive(Serialize)]
struct OperationIntent<'a> {
    schema: &'static str,
    provider_id: &'a str,
    release_id: Uuid,
    game_key: &'a str,
    rules_version: u32,
    cartridge_digest: &'a str,
    platform_session_id: Uuid,
    subject: &'a str,
    idempotency_key: Uuid,
    expected_revision: u64,
    operation: ProviderOperationKind,
    payload: &'a Value,
}

#[derive(FromRow)]
struct OperationRow {
    intent_sha256: String,
    intent_body: Vec<u8>,
    status: String,
    response_body: Option<Vec<u8>>,
    updated_unix: i64,
}

#[derive(FromRow)]
struct AttemptRootRow {
    intent_sha256: String,
    status: String,
    attempt_count: i32,
    updated_unix: i64,
}

#[derive(FromRow)]
struct MessageReceiptRow {
    message_id: Uuid,
    event_id: Option<Uuid>,
    authenticated_sha256: String,
    platform_session_id: Uuid,
}

#[allow(clippy::too_many_arguments)]
async fn record_message_receipt(
    transaction: &mut Transaction<'_, Postgres>,
    release_id: Uuid,
    direction: &str,
    message_id: Uuid,
    platform_session_id: Uuid,
    event_id: Option<Uuid>,
    digest: &str,
    revision: u64,
) -> Result<()> {
    let existing = sqlx::query_as::<_, MessageReceiptRow>(
        r#"
        SELECT message_id, event_id, authenticated_sha256, platform_session_id
        FROM provider_message_receipts
        WHERE release_id = $1 AND direction = $2 AND message_id = $3
        FOR UPDATE
        "#,
    )
    .bind(release_id)
    .bind(direction)
    .bind(message_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|error| map_database_error(error, "load provider message receipt"))?;
    if let Some(existing) = existing {
        if existing.event_id == event_id
            && existing.authenticated_sha256 == digest
            && existing.platform_session_id == platform_session_id
        {
            return Ok(());
        }
        return Err(ProviderError::Conflict);
    }
    sqlx::query(
        r#"
        INSERT INTO provider_message_receipts (
            release_id,
            direction,
            message_id,
            platform_session_id,
            event_id,
            authenticated_sha256,
            disposition,
            provider_revision
        )
        VALUES ($1, $2, $3, $4, $5, $6, 'accepted', $7)
        "#,
    )
    .bind(release_id)
    .bind(direction)
    .bind(message_id)
    .bind(platform_session_id)
    .bind(event_id)
    .bind(digest)
    .bind(i64::try_from(revision).map_err(|_| ProviderError::ProtocolRejected)?)
    .execute(&mut **transaction)
    .await
    .map_err(|error| map_conflict(error, "insert provider message receipt"))?;
    Ok(())
}

fn verify_response_with_active_keys(
    material: &ProviderSecurityMaterial,
    headers: &SignatureHeaders,
    status: u16,
    context: &RequestSignatureContext<'_>,
    message_id: Uuid,
    body: &[u8],
    now: i64,
) -> Result<crate::protocol::VerifiedHttpSignature> {
    verify_with_keys(&material.message_keys, |key, verifying_key| {
        verify_response_signature(
            headers,
            status,
            context,
            message_id,
            body,
            verifying_key,
            key,
            now,
        )
    })
}

fn verify_request_with_active_keys(
    material: &ProviderSecurityMaterial,
    headers: &SignatureHeaders,
    context: &RequestSignatureContext<'_>,
    body: &[u8],
    now: i64,
) -> Result<crate::protocol::VerifiedHttpSignature> {
    verify_with_keys(&material.message_keys, |key, verifying_key| {
        verify_request_signature(headers, context, body, verifying_key, key, now)
    })
}

fn verify_with_keys<T>(
    keys: &[crate::registry::RegisteredOperationalKey],
    verify: impl Fn(&str, &VerifyingKey) -> Result<T>,
) -> Result<T> {
    let mut verified = None;
    for key in keys {
        let verifying_key = key.verifying_key()?;
        if let Ok(value) = verify(&key.key_id, &verifying_key) {
            if verified.is_some() {
                return Err(ProviderError::ProtocolRejected);
            }
            verified = Some(value);
        }
    }
    verified.ok_or(ProviderError::ProtocolRejected)
}

fn current_unix_seconds() -> Result<i64> {
    let seconds = SystemTime::UNIX_EPOCH
        .elapsed()
        .map_err(|_| ProviderError::Internal)?
        .as_secs();
    i64::try_from(seconds).map_err(|_| ProviderError::Internal)
}

fn map_conflict(error: sqlx::Error, operation: &'static str) -> ProviderError {
    if error
        .as_database_error()
        .is_some_and(sqlx::error::DatabaseError::is_unique_violation)
    {
        ProviderError::Conflict
    } else {
        map_database_error(error, operation)
    }
}

fn map_database_error(error_value: sqlx::Error, operation: &'static str) -> ProviderError {
    error!(operation, error = %error_value, "provider broker database operation failed");
    ProviderError::Internal
}
