//! PostgreSQL-backed provider registry, lifecycle, grants, quota, leases, and audit.

use ed25519_dalek::VerifyingKey;
use serde::Serialize;
use serde_json::{Value, json};
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use tracing::error;
use uuid::Uuid;

use crate::{
    ProviderError, Result,
    model::{
        ActiveSessionPolicy, LifecycleStatus, OperationalKeyInput, OperationalKeyKind,
        OperatorCommand, ProviderEndpoint, ProviderQuotas, ProviderScope, RegisterReleaseInput,
        ReleasePolicy, SessionAdmission,
    },
    protocol::{GrantIssuer, ProviderGrantClaims, SignedProviderGrant, sha256_hex},
};

/// Durable quota families.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuotaKind {
    /// Short-lived grant issuance.
    Grant,
    /// Outbound broker request.
    Request,
    /// Inbound callback-shaped event.
    Callback,
}

impl QuotaKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Grant => "grant",
            Self::Request => "request",
            Self::Callback => "callback",
        }
    }
}

/// Safe operator command receipt.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct OperatorReceipt {
    /// Stable applied command name.
    pub command: &'static str,
    /// Provider affected by the command.
    pub provider_id: String,
    /// Release affected when applicable.
    pub release_id: Option<Uuid>,
    /// Current configuration revision when applicable.
    pub config_revision: Option<u64>,
    /// Resulting lifecycle when applicable.
    pub status: Option<LifecycleStatus>,
}

/// One active immutable operational key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisteredOperationalKey {
    /// Exact registered key ID.
    pub key_id: String,
    /// Exact public bytes.
    pub public_material: Vec<u8>,
}

impl RegisteredOperationalKey {
    /// Parse an Ed25519 message verification key.
    pub fn verifying_key(&self) -> Result<VerifyingKey> {
        let bytes: [u8; 32] = self
            .public_material
            .as_slice()
            .try_into()
            .map_err(|_| ProviderError::Internal)?;
        VerifyingKey::from_bytes(&bytes).map_err(|_| ProviderError::Internal)
    }
}

/// Complete active registered material for one admitted operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderSecurityMaterial {
    /// Exact policy snapshot.
    pub policy: ReleasePolicy,
    /// Active provider message verification keys.
    pub message_keys: Vec<RegisteredOperationalKey>,
    /// Active registered TLS root DER certificates.
    pub tls_roots_der: Vec<Vec<u8>>,
}

/// Local-only grant request. It cannot carry account or device credentials.
pub struct IssueGrantRequest {
    /// Exact release.
    pub release_id: Uuid,
    /// Owned persona used only for local pairwise derivation.
    pub persona_id: Uuid,
    /// Exact platform session envelope.
    pub platform_session_id: Uuid,
    /// One exact non-event scope.
    pub scope: ProviderScope,
    /// New or proven existing provider session.
    pub session: SessionAdmission,
}

/// Signed and durably recorded grant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssuedGrant {
    /// Authenticated exact claims.
    pub claims: ProviderGrantClaims,
    /// Signed envelope.
    pub signed: SignedProviderGrant,
    /// Exact serialized signed bytes stored in PostgreSQL.
    pub bytes: Vec<u8>,
}

/// One expiring cross-process request concurrency lease.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConcurrencyLease {
    /// Exact release.
    pub release_id: Uuid,
    /// Unique lease identity.
    pub lease_id: Uuid,
}

/// PostgreSQL-backed provider security foundation.
#[derive(Clone)]
pub struct ProviderRegistry {
    pool: PgPool,
}

impl ProviderRegistry {
    /// Bind the registry to the platform PostgreSQL pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Apply one operator-only command transactionally with immutable audit.
    pub async fn apply_operator_command(
        &self,
        command: &OperatorCommand,
    ) -> Result<OperatorReceipt> {
        command.validate()?;
        match command {
            OperatorCommand::RegisterRelease {
                actor,
                reason,
                registration,
            } => self.register_release(actor, reason, registration).await,
            OperatorCommand::RotateKey {
                actor,
                reason,
                release_id,
                key_kind,
                key,
            } => {
                self.rotate_key(actor, reason, *release_id, *key_kind, key)
                    .await
            }
            OperatorCommand::SetProviderStatus {
                actor,
                reason,
                provider_id,
                status,
            } => {
                self.set_provider_status(actor, reason, provider_id, *status)
                    .await
            }
            OperatorCommand::SetReleaseStatus {
                actor,
                reason,
                release_id,
                status,
            } => {
                self.set_release_status(actor, reason, *release_id, *status)
                    .await
            }
            OperatorCommand::SetScopeStatus {
                actor,
                reason,
                release_id,
                scope,
                status,
            } => {
                self.set_scope_status(actor, reason, *release_id, *scope, *status)
                    .await
            }
            OperatorCommand::SetKeyStatus {
                actor,
                reason,
                release_id,
                key_kind,
                key_id,
                status,
            } => {
                self.set_key_status(actor, reason, *release_id, *key_kind, key_id, *status)
                    .await
            }
            OperatorCommand::UpdateQuotas {
                actor,
                reason,
                release_id,
                quotas,
            } => self.update_quotas(actor, reason, *release_id, quotas).await,
        }
    }

    /// Load exact policy without admitting an operation.
    pub async fn load_policy(&self, release_id: Uuid) -> Result<ReleasePolicy> {
        if release_id.is_nil() {
            return Err(ProviderError::InvalidInput);
        }
        let row = sqlx::query_as::<_, ReleasePolicyRow>(RELEASE_POLICY_QUERY)
            .bind(release_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|error| map_database_error(error, "load provider release policy"))?
            .ok_or(ProviderError::NotFound)?;
        row.try_into()
    }

    /// Re-evaluate lifecycle/scope/key policy and return exact active material.
    pub async fn admit(
        &self,
        release_id: Uuid,
        scope: ProviderScope,
        session: SessionAdmission,
    ) -> Result<ProviderSecurityMaterial> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|error| map_database_error(error, "begin provider admission"))?;
        let material = load_material_locked(&mut transaction, release_id).await?;
        let scope_status = load_scope_status(&mut transaction, release_id, scope).await?;
        evaluate_admission(&material, scope_status, scope, session)?;
        transaction
            .commit()
            .await
            .map_err(|error| map_database_error(error, "commit provider admission"))?;
        Ok(material)
    }

    /// Issue, sign, quota-charge, and durably record one short-lived grant.
    pub async fn issue_grant(
        &self,
        issuer: &GrantIssuer,
        request: &IssueGrantRequest,
    ) -> Result<IssuedGrant> {
        if request.release_id.is_nil()
            || request.persona_id.is_nil()
            || request.platform_session_id.is_nil()
            || request.scope == ProviderScope::Event
        {
            return Err(ProviderError::InvalidInput);
        }
        match self.issue_grant_inner(issuer, request).await {
            Ok(grant) => Ok(grant),
            Err(error) => {
                self.audit_known_release_failure(
                    request.release_id,
                    "grant_denied",
                    error.code(),
                    Some(request.platform_session_id),
                )
                .await?;
                Err(error)
            }
        }
    }

    async fn issue_grant_inner(
        &self,
        issuer: &GrantIssuer,
        request: &IssueGrantRequest,
    ) -> Result<IssuedGrant> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|error| map_database_error(error, "begin provider grant"))?;
        let material = load_material_locked(&mut transaction, request.release_id).await?;
        let scope_status =
            load_scope_status(&mut transaction, request.release_id, request.scope).await?;
        evaluate_admission(&material, scope_status, request.scope, request.session)?;
        let now = database_unix_seconds(&mut transaction).await?;
        charge_quota_locked(
            &mut transaction,
            request.release_id,
            QuotaKind::Grant,
            material.policy.quotas.grants_per_minute,
        )
        .await?;
        let subject = issuer.pairwise_subject(
            &material.policy.provider_id,
            &material.policy.game_key,
            request.persona_id,
        )?;
        let token_id = Uuid::new_v4();
        let expires_at = now.checked_add(60).ok_or(ProviderError::Internal)?;
        let claims = ProviderGrantClaims::new(
            material.policy.provider_id.clone(),
            material.policy.release_id,
            material.policy.game_key.clone(),
            material.policy.rules_version,
            material.policy.cartridge_digest.clone(),
            request.platform_session_id,
            subject,
            request.scope,
            now,
            expires_at,
            token_id,
        )?;
        let signed = issuer.sign(&claims)?;
        let bytes = signed.to_bytes()?;
        let claims_bytes = serde_json::to_vec(&claims).map_err(|_| ProviderError::Internal)?;
        sqlx::query(
            r#"
            INSERT INTO provider_grants (
                token_id,
                release_id,
                platform_session_id,
                pairwise_subject,
                scope,
                claims_sha256,
                signed_grant,
                issued_at,
                expires_at
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7,
                to_timestamp($8), to_timestamp($9)
            )
            "#,
        )
        .bind(token_id)
        .bind(request.release_id)
        .bind(request.platform_session_id)
        .bind(&claims.subject)
        .bind(request.scope.as_str())
        .bind(sha256_hex(&claims_bytes))
        .bind(&bytes)
        .bind(now)
        .bind(expires_at)
        .execute(&mut *transaction)
        .await
        .map_err(|error| map_database_error(error, "insert provider grant"))?;
        insert_audit(
            &mut transaction,
            &material.policy.provider_id,
            Some(request.release_id),
            "broker",
            "grant_issuer",
            "grant_issued",
            "allowed",
            "grant_issued",
            Some(request.platform_session_id),
            json!({"scope": request.scope.as_str(), "token_id": token_id}),
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|error| map_database_error(error, "commit provider grant"))?;
        Ok(IssuedGrant {
            claims,
            signed,
            bytes,
        })
    }

    /// Atomically re-admit, charge request quota, and acquire an expiring
    /// cross-process concurrency lease.
    pub async fn begin_request(
        &self,
        release_id: Uuid,
        scope: ProviderScope,
        session: SessionAdmission,
        correlation_id: Uuid,
    ) -> Result<(ProviderSecurityMaterial, ConcurrencyLease)> {
        if correlation_id.is_nil() || scope == ProviderScope::Event {
            return Err(ProviderError::InvalidInput);
        }
        match self
            .begin_request_inner(release_id, scope, session, correlation_id)
            .await
        {
            Ok(value) => Ok(value),
            Err(error) => {
                self.audit_known_release_failure(
                    release_id,
                    "request_denied",
                    error.code(),
                    Some(correlation_id),
                )
                .await?;
                Err(error)
            }
        }
    }

    async fn begin_request_inner(
        &self,
        release_id: Uuid,
        scope: ProviderScope,
        session: SessionAdmission,
        correlation_id: Uuid,
    ) -> Result<(ProviderSecurityMaterial, ConcurrencyLease)> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|error| map_database_error(error, "begin provider request admission"))?;
        let material = load_material_locked(&mut transaction, release_id).await?;
        let scope_status = load_scope_status(&mut transaction, release_id, scope).await?;
        evaluate_admission(&material, scope_status, scope, session)?;
        charge_quota_locked(
            &mut transaction,
            release_id,
            QuotaKind::Request,
            material.policy.quotas.requests_per_minute,
        )
        .await?;
        sqlx::query(
            "DELETE FROM provider_concurrency_leases WHERE release_id = $1 AND expires_at <= clock_timestamp()",
        )
        .bind(release_id)
        .execute(&mut *transaction)
        .await
        .map_err(|error| map_database_error(error, "expire provider request leases"))?;
        let active: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM provider_concurrency_leases WHERE release_id = $1",
        )
        .bind(release_id)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|error| map_database_error(error, "count provider request leases"))?;
        if active >= i64::from(material.policy.quotas.max_concurrent_requests) {
            return Err(ProviderError::QuotaExceeded);
        }
        let lease_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO provider_concurrency_leases (lease_id, release_id, expires_at)
            VALUES (
                $1,
                $2,
                clock_timestamp() + ($3::BIGINT * INTERVAL '1 millisecond')
            )
            "#,
        )
        .bind(lease_id)
        .bind(release_id)
        .bind(i64::from(material.policy.quotas.total_timeout_ms) + 1_000)
        .execute(&mut *transaction)
        .await
        .map_err(|error| map_database_error(error, "insert provider request lease"))?;
        insert_audit(
            &mut transaction,
            &material.policy.provider_id,
            Some(release_id),
            "broker",
            "provider_broker",
            "request_admitted",
            "allowed",
            "request_admitted",
            Some(correlation_id),
            json!({"scope": scope.as_str(), "lease_id": lease_id}),
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|error| map_database_error(error, "commit provider request admission"))?;
        Ok((
            material,
            ConcurrencyLease {
                release_id,
                lease_id,
            },
        ))
    }

    /// Release one exact request lease. Absence is idempotent after expiry.
    pub async fn release_request(&self, lease: ConcurrencyLease) -> Result<()> {
        sqlx::query(
            "DELETE FROM provider_concurrency_leases WHERE release_id = $1 AND lease_id = $2",
        )
        .bind(lease.release_id)
        .bind(lease.lease_id)
        .execute(&self.pool)
        .await
        .map_err(|error| map_database_error(error, "release provider request lease"))?;
        Ok(())
    }

    /// Charge callback quota after resolving the exact release identity.
    pub async fn admit_callback(
        &self,
        release_id: Uuid,
        correlation_id: Uuid,
    ) -> Result<ProviderSecurityMaterial> {
        if correlation_id.is_nil() {
            return Err(ProviderError::InvalidInput);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|error| map_database_error(error, "begin provider callback admission"))?;
        let material = load_material_locked(&mut transaction, release_id).await?;
        let scope_status =
            load_scope_status(&mut transaction, release_id, ProviderScope::Event).await?;
        evaluate_admission(
            &material,
            scope_status,
            ProviderScope::Event,
            SessionAdmission::Existing,
        )?;
        charge_quota_locked(
            &mut transaction,
            release_id,
            QuotaKind::Callback,
            material.policy.quotas.callbacks_per_minute,
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|error| map_database_error(error, "commit provider callback admission"))?;
        Ok(material)
    }

    /// Append one bounded non-secret broker/provider security event.
    #[allow(clippy::too_many_arguments)]
    pub async fn record_security_event(
        &self,
        release_id: Uuid,
        actor_type: &'static str,
        actor_id: &'static str,
        event_type: &'static str,
        outcome: &'static str,
        reason_code: &'static str,
        correlation_id: Option<Uuid>,
        safe_details: Value,
    ) -> Result<()> {
        validate_safe_audit_values(
            actor_type,
            actor_id,
            event_type,
            outcome,
            reason_code,
            &safe_details,
        )?;
        let provider_id: String =
            sqlx::query_scalar("SELECT provider_id FROM provider_releases WHERE release_id = $1")
                .bind(release_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|error| map_database_error(error, "resolve provider audit identity"))?
                .ok_or(ProviderError::NotFound)?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|error| map_database_error(error, "begin provider audit"))?;
        insert_audit(
            &mut transaction,
            &provider_id,
            Some(release_id),
            actor_type,
            actor_id,
            event_type,
            outcome,
            reason_code,
            correlation_id,
            safe_details,
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|error| map_database_error(error, "commit provider audit"))?;
        Ok(())
    }

    async fn audit_known_release_failure(
        &self,
        release_id: Uuid,
        event_type: &'static str,
        reason_code: &'static str,
        correlation_id: Option<Uuid>,
    ) -> Result<()> {
        match self
            .record_security_event(
                release_id,
                "broker",
                "provider_broker",
                event_type,
                "denied",
                reason_code,
                correlation_id,
                json!({}),
            )
            .await
        {
            Ok(()) | Err(ProviderError::NotFound) => Ok(()),
            Err(_) => Err(ProviderError::Internal),
        }
    }

    pub(crate) fn pool(&self) -> &PgPool {
        &self.pool
    }

    async fn register_release(
        &self,
        actor: &str,
        reason: &str,
        registration: &RegisterReleaseInput,
    ) -> Result<OperatorReceipt> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|error| map_database_error(error, "begin provider registration"))?;
        sqlx::query(
            r#"
            INSERT INTO provider_registrations (provider_id, display_name)
            VALUES ($1, $2)
            ON CONFLICT (provider_id) DO NOTHING
            "#,
        )
        .bind(&registration.provider_id)
        .bind(&registration.display_name)
        .execute(&mut *transaction)
        .await
        .map_err(|error| map_database_error(error, "insert provider registration"))?;
        let provider: ProviderRow = sqlx::query_as(
            r#"
            SELECT provider_id, display_name, status
            FROM provider_registrations
            WHERE provider_id = $1
            FOR UPDATE
            "#,
        )
        .bind(&registration.provider_id)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|error| map_database_error(error, "lock provider registration"))?;
        if provider.display_name != registration.display_name || provider.status == "revoked" {
            return Err(ProviderError::Conflict);
        }
        let endpoint = &registration.endpoint;
        sqlx::query(
            r#"
            INSERT INTO provider_releases (
                release_id, provider_id, game_key, rules_version,
                cartridge_digest, endpoint_host, endpoint_port,
                endpoint_base_path, active_session_policy,
                grant_limit_per_minute, request_limit_per_minute,
                callback_limit_per_minute, max_concurrent_requests,
                request_body_limit_bytes, response_body_limit_bytes,
                connect_timeout_ms, total_timeout_ms
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9,
                $10, $11, $12, $13, $14, $15, $16, $17
            )
            "#,
        )
        .bind(registration.release_id)
        .bind(&registration.provider_id)
        .bind(&registration.game_key)
        .bind(i64::from(registration.rules_version))
        .bind(&registration.cartridge_digest)
        .bind(&endpoint.host)
        .bind(i32::from(endpoint.port))
        .bind(&endpoint.base_path)
        .bind(registration.active_session_policy.as_str())
        .bind(i64::from(registration.quotas.grants_per_minute))
        .bind(i64::from(registration.quotas.requests_per_minute))
        .bind(i64::from(registration.quotas.callbacks_per_minute))
        .bind(i64::from(registration.quotas.max_concurrent_requests))
        .bind(i64::from(registration.quotas.request_body_bytes))
        .bind(i64::from(registration.quotas.response_body_bytes))
        .bind(i64::from(registration.quotas.connect_timeout_ms))
        .bind(i64::from(registration.quotas.total_timeout_ms))
        .execute(&mut *transaction)
        .await
        .map_err(|error| map_conflict(error, "insert provider release"))?;
        for scope in &registration.scopes {
            sqlx::query("INSERT INTO provider_release_scopes (release_id, scope) VALUES ($1, $2)")
                .bind(registration.release_id)
                .bind(scope.as_str())
                .execute(&mut *transaction)
                .await
                .map_err(|error| map_conflict(error, "insert provider release scope"))?;
        }
        for key in &registration.message_keys {
            insert_key(
                &mut transaction,
                registration.release_id,
                OperationalKeyKind::MessageEd25519,
                key,
            )
            .await?;
        }
        for key in &registration.tls_roots {
            insert_key(
                &mut transaction,
                registration.release_id,
                OperationalKeyKind::TlsRootDer,
                key,
            )
            .await?;
        }
        insert_audit(
            &mut transaction,
            &registration.provider_id,
            Some(registration.release_id),
            "operator",
            actor,
            "release_registered",
            "recorded",
            "operator_registration",
            None,
            json!({
                "reason": reason,
                "config_revision": 1,
                "message_key_ids": registration.message_keys.iter().map(|key| &key.key_id).collect::<Vec<_>>(),
                "tls_root_ids": registration.tls_roots.iter().map(|key| &key.key_id).collect::<Vec<_>>(),
                "scopes": registration.scopes.iter().map(|scope| scope.as_str()).collect::<Vec<_>>()
            }),
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|error| map_database_error(error, "commit provider registration"))?;
        Ok(OperatorReceipt {
            command: "register_release",
            provider_id: registration.provider_id.clone(),
            release_id: Some(registration.release_id),
            config_revision: Some(1),
            status: Some(LifecycleStatus::Active),
        })
    }

    async fn rotate_key(
        &self,
        actor: &str,
        reason: &str,
        release_id: Uuid,
        kind: OperationalKeyKind,
        key: &OperationalKeyInput,
    ) -> Result<OperatorReceipt> {
        let mut transaction = self.pool.begin().await.map_err(|error| {
            map_database_error(error, "begin provider operational key rotation")
        })?;
        let row = lock_release_row(&mut transaction, release_id).await?;
        if row.provider_status == "revoked" || row.release_status == "revoked" {
            return Err(ProviderError::Denied);
        }
        insert_key(&mut transaction, release_id, kind, key).await?;
        let revision = advance_config_revision(&mut transaction, release_id).await?;
        insert_audit(
            &mut transaction,
            &row.provider_id,
            Some(release_id),
            "operator",
            actor,
            "key_rotated",
            "recorded",
            "operator_key_rotation",
            None,
            json!({
                "reason": reason,
                "key_kind": kind.as_str(),
                "key_id": key.key_id,
                "config_revision": revision
            }),
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|error| map_database_error(error, "commit provider key rotation"))?;
        Ok(OperatorReceipt {
            command: "rotate_key",
            provider_id: row.provider_id,
            release_id: Some(release_id),
            config_revision: Some(revision),
            status: None,
        })
    }

    async fn set_provider_status(
        &self,
        actor: &str,
        reason: &str,
        provider_id: &str,
        status: LifecycleStatus,
    ) -> Result<OperatorReceipt> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|error| map_database_error(error, "begin provider lifecycle update"))?;
        let current: String = sqlx::query_scalar(
            "SELECT status FROM provider_registrations WHERE provider_id = $1 FOR UPDATE",
        )
        .bind(provider_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|error| map_database_error(error, "lock provider lifecycle"))?
        .ok_or(ProviderError::NotFound)?;
        validate_transition(&current, status)?;
        sqlx::query(
            r#"
            UPDATE provider_registrations
            SET status = $2,
                revoked_at = CASE WHEN $2 = 'revoked' THEN clock_timestamp() ELSE NULL END,
                updated_at = clock_timestamp()
            WHERE provider_id = $1
            "#,
        )
        .bind(provider_id)
        .bind(status.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(|error| map_database_error(error, "update provider lifecycle"))?;
        insert_audit(
            &mut transaction,
            provider_id,
            None,
            "operator",
            actor,
            "provider_status_changed",
            "recorded",
            "operator_lifecycle_change",
            None,
            json!({"reason": reason, "from": current, "to": status.as_str()}),
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|error| map_database_error(error, "commit provider lifecycle update"))?;
        Ok(OperatorReceipt {
            command: "set_provider_status",
            provider_id: provider_id.to_owned(),
            release_id: None,
            config_revision: None,
            status: Some(status),
        })
    }

    async fn set_release_status(
        &self,
        actor: &str,
        reason: &str,
        release_id: Uuid,
        status: LifecycleStatus,
    ) -> Result<OperatorReceipt> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|error| map_database_error(error, "begin provider release lifecycle"))?;
        let row = lock_release_row(&mut transaction, release_id).await?;
        validate_transition(&row.release_status, status)?;
        let revision: i64 = sqlx::query_scalar(
            r#"
            UPDATE provider_releases
            SET status = $2,
                revoked_at = CASE WHEN $2 = 'revoked' THEN clock_timestamp() ELSE NULL END,
                config_revision = config_revision + 1,
                updated_at = clock_timestamp()
            WHERE release_id = $1
            RETURNING config_revision
            "#,
        )
        .bind(release_id)
        .bind(status.as_str())
        .fetch_one(&mut *transaction)
        .await
        .map_err(|error| map_database_error(error, "update provider release lifecycle"))?;
        let revision = u64::try_from(revision).map_err(|_| ProviderError::Internal)?;
        insert_audit(
            &mut transaction,
            &row.provider_id,
            Some(release_id),
            "operator",
            actor,
            "release_status_changed",
            "recorded",
            "operator_lifecycle_change",
            None,
            json!({"reason": reason, "from": row.release_status, "to": status.as_str(), "config_revision": revision}),
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|error| map_database_error(error, "commit provider release lifecycle"))?;
        Ok(OperatorReceipt {
            command: "set_release_status",
            provider_id: row.provider_id,
            release_id: Some(release_id),
            config_revision: Some(revision),
            status: Some(status),
        })
    }

    async fn set_scope_status(
        &self,
        actor: &str,
        reason: &str,
        release_id: Uuid,
        scope: ProviderScope,
        status: LifecycleStatus,
    ) -> Result<OperatorReceipt> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|error| map_database_error(error, "begin provider scope lifecycle"))?;
        let row = lock_release_row(&mut transaction, release_id).await?;
        let current: String = sqlx::query_scalar(
            "SELECT status FROM provider_release_scopes WHERE release_id = $1 AND scope = $2 FOR UPDATE",
        )
        .bind(release_id)
        .bind(scope.as_str())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|error| map_database_error(error, "lock provider scope lifecycle"))?
        .ok_or(ProviderError::NotFound)?;
        validate_transition(&current, status)?;
        sqlx::query(
            r#"
            UPDATE provider_release_scopes
            SET status = $3,
                revoked_at = CASE WHEN $3 = 'revoked' THEN clock_timestamp() ELSE NULL END,
                updated_at = clock_timestamp()
            WHERE release_id = $1 AND scope = $2
            "#,
        )
        .bind(release_id)
        .bind(scope.as_str())
        .bind(status.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(|error| map_database_error(error, "update provider scope lifecycle"))?;
        let revision = advance_config_revision(&mut transaction, release_id).await?;
        insert_audit(
            &mut transaction,
            &row.provider_id,
            Some(release_id),
            "operator",
            actor,
            "scope_status_changed",
            "recorded",
            "operator_scope_change",
            None,
            json!({"reason": reason, "scope": scope.as_str(), "from": current, "to": status.as_str(), "config_revision": revision}),
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|error| map_database_error(error, "commit provider scope lifecycle"))?;
        Ok(OperatorReceipt {
            command: "set_scope_status",
            provider_id: row.provider_id,
            release_id: Some(release_id),
            config_revision: Some(revision),
            status: Some(status),
        })
    }

    #[allow(clippy::too_many_arguments)]
    async fn set_key_status(
        &self,
        actor: &str,
        reason: &str,
        release_id: Uuid,
        kind: OperationalKeyKind,
        key_id: &str,
        status: LifecycleStatus,
    ) -> Result<OperatorReceipt> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|error| map_database_error(error, "begin provider key lifecycle"))?;
        let row = lock_release_row(&mut transaction, release_id).await?;
        let current: String = sqlx::query_scalar(
            r#"
            SELECT status
            FROM provider_release_keys
            WHERE release_id = $1 AND key_kind = $2 AND key_id = $3
            FOR UPDATE
            "#,
        )
        .bind(release_id)
        .bind(kind.as_str())
        .bind(key_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|error| map_database_error(error, "lock provider key lifecycle"))?
        .ok_or(ProviderError::NotFound)?;
        validate_transition(&current, status)?;
        sqlx::query(
            r#"
            UPDATE provider_release_keys
            SET status = $4,
                revoked_at = CASE WHEN $4 = 'revoked' THEN clock_timestamp() ELSE NULL END,
                updated_at = clock_timestamp()
            WHERE release_id = $1 AND key_kind = $2 AND key_id = $3
            "#,
        )
        .bind(release_id)
        .bind(kind.as_str())
        .bind(key_id)
        .bind(status.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(|error| map_database_error(error, "update provider key lifecycle"))?;
        let revision = advance_config_revision(&mut transaction, release_id).await?;
        insert_audit(
            &mut transaction,
            &row.provider_id,
            Some(release_id),
            "operator",
            actor,
            "key_status_changed",
            "recorded",
            "operator_key_change",
            None,
            json!({"reason": reason, "key_kind": kind.as_str(), "key_id": key_id, "from": current, "to": status.as_str(), "config_revision": revision}),
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|error| map_database_error(error, "commit provider key lifecycle"))?;
        Ok(OperatorReceipt {
            command: "set_key_status",
            provider_id: row.provider_id,
            release_id: Some(release_id),
            config_revision: Some(revision),
            status: Some(status),
        })
    }

    async fn update_quotas(
        &self,
        actor: &str,
        reason: &str,
        release_id: Uuid,
        quotas: &ProviderQuotas,
    ) -> Result<OperatorReceipt> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|error| map_database_error(error, "begin provider quota update"))?;
        let row = lock_release_row(&mut transaction, release_id).await?;
        if row.release_status == "revoked" || row.provider_status == "revoked" {
            return Err(ProviderError::Denied);
        }
        let revision: i64 = sqlx::query_scalar(
            r#"
            UPDATE provider_releases
            SET grant_limit_per_minute = $2,
                request_limit_per_minute = $3,
                callback_limit_per_minute = $4,
                max_concurrent_requests = $5,
                request_body_limit_bytes = $6,
                response_body_limit_bytes = $7,
                connect_timeout_ms = $8,
                total_timeout_ms = $9,
                config_revision = config_revision + 1,
                updated_at = clock_timestamp()
            WHERE release_id = $1
            RETURNING config_revision
            "#,
        )
        .bind(release_id)
        .bind(i64::from(quotas.grants_per_minute))
        .bind(i64::from(quotas.requests_per_minute))
        .bind(i64::from(quotas.callbacks_per_minute))
        .bind(i64::from(quotas.max_concurrent_requests))
        .bind(i64::from(quotas.request_body_bytes))
        .bind(i64::from(quotas.response_body_bytes))
        .bind(i64::from(quotas.connect_timeout_ms))
        .bind(i64::from(quotas.total_timeout_ms))
        .fetch_one(&mut *transaction)
        .await
        .map_err(|error| map_database_error(error, "update provider quotas"))?;
        let revision = u64::try_from(revision).map_err(|_| ProviderError::Internal)?;
        insert_audit(
            &mut transaction,
            &row.provider_id,
            Some(release_id),
            "operator",
            actor,
            "quotas_changed",
            "recorded",
            "operator_quota_change",
            None,
            json!({"reason": reason, "config_revision": revision, "quotas": quotas}),
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|error| map_database_error(error, "commit provider quota update"))?;
        Ok(OperatorReceipt {
            command: "update_quotas",
            provider_id: row.provider_id,
            release_id: Some(release_id),
            config_revision: Some(revision),
            status: None,
        })
    }
}

const RELEASE_POLICY_QUERY: &str = r#"
    SELECT
        p.provider_id,
        p.status AS provider_status,
        r.release_id,
        r.game_key,
        r.rules_version,
        r.cartridge_digest,
        r.endpoint_host,
        r.endpoint_port,
        r.endpoint_base_path,
        r.status AS release_status,
        r.active_session_policy,
        r.config_revision,
        r.grant_limit_per_minute,
        r.request_limit_per_minute,
        r.callback_limit_per_minute,
        r.max_concurrent_requests,
        r.request_body_limit_bytes,
        r.response_body_limit_bytes,
        r.connect_timeout_ms,
        r.total_timeout_ms
    FROM provider_releases r
    JOIN provider_registrations p ON p.provider_id = r.provider_id
    WHERE r.release_id = $1
"#;

#[derive(Debug, FromRow)]
struct ReleasePolicyRow {
    provider_id: String,
    provider_status: String,
    release_id: Uuid,
    game_key: String,
    rules_version: i64,
    cartridge_digest: String,
    endpoint_host: String,
    endpoint_port: i32,
    endpoint_base_path: String,
    release_status: String,
    active_session_policy: String,
    config_revision: i64,
    grant_limit_per_minute: i32,
    request_limit_per_minute: i32,
    callback_limit_per_minute: i32,
    max_concurrent_requests: i32,
    request_body_limit_bytes: i32,
    response_body_limit_bytes: i32,
    connect_timeout_ms: i32,
    total_timeout_ms: i32,
}

impl TryFrom<ReleasePolicyRow> for ReleasePolicy {
    type Error = ProviderError;

    fn try_from(row: ReleasePolicyRow) -> Result<Self> {
        let rules_version =
            u32::try_from(row.rules_version).map_err(|_| ProviderError::Internal)?;
        let endpoint_port =
            u16::try_from(row.endpoint_port).map_err(|_| ProviderError::Internal)?;
        let config_revision =
            u64::try_from(row.config_revision).map_err(|_| ProviderError::Internal)?;
        let quotas = ProviderQuotas {
            grants_per_minute: u32::try_from(row.grant_limit_per_minute)
                .map_err(|_| ProviderError::Internal)?,
            requests_per_minute: u32::try_from(row.request_limit_per_minute)
                .map_err(|_| ProviderError::Internal)?,
            callbacks_per_minute: u32::try_from(row.callback_limit_per_minute)
                .map_err(|_| ProviderError::Internal)?,
            max_concurrent_requests: u16::try_from(row.max_concurrent_requests)
                .map_err(|_| ProviderError::Internal)?,
            request_body_bytes: u32::try_from(row.request_body_limit_bytes)
                .map_err(|_| ProviderError::Internal)?,
            response_body_bytes: u32::try_from(row.response_body_limit_bytes)
                .map_err(|_| ProviderError::Internal)?,
            connect_timeout_ms: u32::try_from(row.connect_timeout_ms)
                .map_err(|_| ProviderError::Internal)?,
            total_timeout_ms: u32::try_from(row.total_timeout_ms)
                .map_err(|_| ProviderError::Internal)?,
        };
        quotas.validate().map_err(|_| ProviderError::Internal)?;
        let endpoint = ProviderEndpoint {
            host: row.endpoint_host,
            port: endpoint_port,
            base_path: row.endpoint_base_path,
        };
        endpoint.validate().map_err(|_| ProviderError::Internal)?;
        Ok(Self {
            provider_id: row.provider_id,
            provider_status: LifecycleStatus::parse(&row.provider_status)?,
            release_id: row.release_id,
            game_key: row.game_key,
            rules_version,
            cartridge_digest: row.cartridge_digest,
            endpoint,
            release_status: LifecycleStatus::parse(&row.release_status)?,
            active_session_policy: ActiveSessionPolicy::parse(&row.active_session_policy)?,
            config_revision,
            quotas,
        })
    }
}

#[derive(Debug, FromRow)]
struct LockedReleaseRow {
    provider_id: String,
    provider_status: String,
    release_status: String,
}

#[derive(Debug, FromRow)]
struct ProviderRow {
    #[allow(dead_code)]
    provider_id: String,
    display_name: String,
    status: String,
}

#[derive(Debug, FromRow)]
struct KeyRow {
    key_id: String,
    public_material: Vec<u8>,
}

async fn load_material_locked(
    transaction: &mut Transaction<'_, Postgres>,
    release_id: Uuid,
) -> Result<ProviderSecurityMaterial> {
    let provider_id: String =
        sqlx::query_scalar("SELECT provider_id FROM provider_releases WHERE release_id = $1")
            .bind(release_id)
            .fetch_optional(&mut **transaction)
            .await
            .map_err(|error| map_database_error(error, "resolve provider release root"))?
            .ok_or(ProviderError::NotFound)?;
    sqlx::query("SELECT provider_id FROM provider_registrations WHERE provider_id = $1 FOR UPDATE")
        .bind(&provider_id)
        .fetch_one(&mut **transaction)
        .await
        .map_err(|error| map_database_error(error, "lock provider root"))?;
    sqlx::query("SELECT release_id FROM provider_releases WHERE release_id = $1 FOR UPDATE")
        .bind(release_id)
        .fetch_one(&mut **transaction)
        .await
        .map_err(|error| map_database_error(error, "lock provider release"))?;
    let row = sqlx::query_as::<_, ReleasePolicyRow>(RELEASE_POLICY_QUERY)
        .bind(release_id)
        .fetch_one(&mut **transaction)
        .await
        .map_err(|error| map_database_error(error, "lock provider release"))?;
    let policy = ReleasePolicy::try_from(row)?;
    let message_rows =
        load_active_keys(transaction, release_id, OperationalKeyKind::MessageEd25519).await?;
    let tls_rows =
        load_active_keys(transaction, release_id, OperationalKeyKind::TlsRootDer).await?;
    if message_rows.is_empty() || tls_rows.is_empty() {
        return Err(ProviderError::Denied);
    }
    Ok(ProviderSecurityMaterial {
        policy,
        message_keys: message_rows
            .into_iter()
            .map(|row| RegisteredOperationalKey {
                key_id: row.key_id,
                public_material: row.public_material,
            })
            .collect(),
        tls_roots_der: tls_rows
            .into_iter()
            .map(|row| row.public_material)
            .collect(),
    })
}

async fn load_active_keys(
    transaction: &mut Transaction<'_, Postgres>,
    release_id: Uuid,
    kind: OperationalKeyKind,
) -> Result<Vec<KeyRow>> {
    sqlx::query_as::<_, KeyRow>(
        r#"
        SELECT key_id, public_material
        FROM provider_release_keys
        WHERE release_id = $1
          AND key_kind = $2
          AND status = 'active'
          AND valid_from <= clock_timestamp()
          AND (valid_until IS NULL OR valid_until > clock_timestamp())
        ORDER BY key_id
        "#,
    )
    .bind(release_id)
    .bind(kind.as_str())
    .fetch_all(&mut **transaction)
    .await
    .map_err(|error| map_database_error(error, "load active provider keys"))
}

async fn load_scope_status(
    transaction: &mut Transaction<'_, Postgres>,
    release_id: Uuid,
    scope: ProviderScope,
) -> Result<LifecycleStatus> {
    let status: String = sqlx::query_scalar(
        "SELECT status FROM provider_release_scopes WHERE release_id = $1 AND scope = $2 FOR UPDATE",
    )
    .bind(release_id)
    .bind(scope.as_str())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|error| map_database_error(error, "load provider scope"))?
    .ok_or(ProviderError::Denied)?;
    LifecycleStatus::parse(&status)
}

/// Pure lifecycle matrix used independently by PostgreSQL and unit tests.
pub fn evaluate_admission(
    material: &ProviderSecurityMaterial,
    scope_status: LifecycleStatus,
    scope: ProviderScope,
    session: SessionAdmission,
) -> Result<()> {
    if scope_status != LifecycleStatus::Active
        || material.message_keys.is_empty()
        || material.tls_roots_der.is_empty()
        || material.policy.provider_status == LifecycleStatus::Revoked
        || material.policy.release_status == LifecycleStatus::Revoked
    {
        return Err(ProviderError::Denied);
    }
    let suspended = material.policy.provider_status == LifecycleStatus::Suspended
        || material.policy.release_status == LifecycleStatus::Suspended;
    if !suspended {
        return Ok(());
    }
    if session == SessionAdmission::New
        || matches!(scope, ProviderScope::Launch | ProviderScope::Event)
    {
        return Err(ProviderError::Denied);
    }
    match material.policy.active_session_policy {
        ActiveSessionPolicy::Terminate => Err(ProviderError::Denied),
        ActiveSessionPolicy::ReadOnly if scope == ProviderScope::Reconcile => Ok(()),
        ActiveSessionPolicy::Continue
            if matches!(scope, ProviderScope::Command | ProviderScope::Reconcile) =>
        {
            Ok(())
        }
        ActiveSessionPolicy::ReadOnly | ActiveSessionPolicy::Continue => Err(ProviderError::Denied),
    }
}

async fn lock_release_row(
    transaction: &mut Transaction<'_, Postgres>,
    release_id: Uuid,
) -> Result<LockedReleaseRow> {
    let provider_id: String =
        sqlx::query_scalar("SELECT provider_id FROM provider_releases WHERE release_id = $1")
            .bind(release_id)
            .fetch_optional(&mut **transaction)
            .await
            .map_err(|error| map_database_error(error, "resolve provider release"))?
            .ok_or(ProviderError::NotFound)?;
    let provider_status: String = sqlx::query_scalar(
        "SELECT status FROM provider_registrations WHERE provider_id = $1 FOR UPDATE",
    )
    .bind(&provider_id)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|error| map_database_error(error, "lock provider root"))?;
    let release_status: String =
        sqlx::query_scalar("SELECT status FROM provider_releases WHERE release_id = $1 FOR UPDATE")
            .bind(release_id)
            .fetch_one(&mut **transaction)
            .await
            .map_err(|error| map_database_error(error, "lock provider release"))?;
    Ok(LockedReleaseRow {
        provider_id,
        provider_status,
        release_status,
    })
}

async fn insert_key(
    transaction: &mut Transaction<'_, Postgres>,
    release_id: Uuid,
    kind: OperationalKeyKind,
    key: &OperationalKeyInput,
) -> Result<()> {
    let material = key.decode(kind)?;
    let digest = sha256_hex(&material);
    sqlx::query(
        r#"
        INSERT INTO provider_release_keys (
            release_id, key_kind, key_id, public_material,
            material_sha256, valid_from, valid_until
        )
        VALUES (
            $1, $2, $3, $4, $5, to_timestamp($6),
            CASE WHEN $7::BIGINT IS NULL THEN NULL ELSE to_timestamp($7) END
        )
        "#,
    )
    .bind(release_id)
    .bind(kind.as_str())
    .bind(&key.key_id)
    .bind(material)
    .bind(digest)
    .bind(key.valid_from)
    .bind(key.valid_until)
    .execute(&mut **transaction)
    .await
    .map_err(|error| map_conflict(error, "insert provider operational key"))?;
    Ok(())
}

async fn advance_config_revision(
    transaction: &mut Transaction<'_, Postgres>,
    release_id: Uuid,
) -> Result<u64> {
    let revision: i64 = sqlx::query_scalar(
        r#"
        UPDATE provider_releases
        SET config_revision = config_revision + 1,
            updated_at = clock_timestamp()
        WHERE release_id = $1
        RETURNING config_revision
        "#,
    )
    .bind(release_id)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|error| map_database_error(error, "advance provider config revision"))?;
    u64::try_from(revision).map_err(|_| ProviderError::Internal)
}

async fn database_unix_seconds(transaction: &mut Transaction<'_, Postgres>) -> Result<i64> {
    sqlx::query_scalar("SELECT EXTRACT(EPOCH FROM clock_timestamp())::BIGINT")
        .fetch_one(&mut **transaction)
        .await
        .map_err(|error| map_database_error(error, "read provider database clock"))
}

async fn charge_quota_locked(
    transaction: &mut Transaction<'_, Postgres>,
    release_id: Uuid,
    kind: QuotaKind,
    limit: u32,
) -> Result<()> {
    let used = sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO provider_quota_windows (
            release_id, quota_kind, window_started_at, used
        )
        VALUES ($1, $2, date_trunc('minute', clock_timestamp()), 1)
        ON CONFLICT (release_id, quota_kind, window_started_at)
        DO UPDATE
        SET used = provider_quota_windows.used + 1,
            updated_at = clock_timestamp()
        WHERE provider_quota_windows.used < $3
        RETURNING used
        "#,
    )
    .bind(release_id)
    .bind(kind.as_str())
    .bind(i64::from(limit))
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|error| map_database_error(error, "charge provider quota"))?;
    if used.is_some() {
        Ok(())
    } else {
        Err(ProviderError::QuotaExceeded)
    }
}

#[allow(clippy::too_many_arguments)]
async fn insert_audit(
    transaction: &mut Transaction<'_, Postgres>,
    provider_id: &str,
    release_id: Option<Uuid>,
    actor_type: &str,
    actor_id: &str,
    event_type: &str,
    outcome: &str,
    reason_code: &str,
    correlation_id: Option<Uuid>,
    safe_details: Value,
) -> Result<()> {
    validate_safe_audit_values(
        actor_type,
        actor_id,
        event_type,
        outcome,
        reason_code,
        &safe_details,
    )?;
    sqlx::query(
        r#"
        INSERT INTO provider_security_audit_events (
            provider_id, release_id, actor_type, actor_id,
            event_type, outcome, reason_code, correlation_id, safe_details
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
    )
    .bind(provider_id)
    .bind(release_id)
    .bind(actor_type)
    .bind(actor_id)
    .bind(event_type)
    .bind(outcome)
    .bind(reason_code)
    .bind(correlation_id)
    .bind(safe_details)
    .execute(&mut **transaction)
    .await
    .map_err(|error| map_database_error(error, "insert provider security audit"))?;
    Ok(())
}

fn validate_safe_audit_values(
    actor_type: &str,
    actor_id: &str,
    event_type: &str,
    outcome: &str,
    reason_code: &str,
    safe_details: &Value,
) -> Result<()> {
    let actor_ok = matches!(actor_type, "operator" | "broker" | "provider")
        && (1..=96).contains(&actor_id.len())
        && !actor_id.chars().any(char::is_control);
    let code_ok = |value: &str, min: usize| {
        (min..=64).contains(&value.len())
            && value
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    };
    let details = serde_json::to_vec(safe_details).map_err(|_| ProviderError::InvalidInput)?;
    if actor_ok
        && code_ok(event_type, 3)
        && matches!(outcome, "allowed" | "denied" | "failed" | "recorded")
        && code_ok(reason_code, 2)
        && safe_details.is_object()
        && details.len() <= 4_096
    {
        Ok(())
    } else {
        Err(ProviderError::InvalidInput)
    }
}

fn validate_transition(current: &str, requested: LifecycleStatus) -> Result<()> {
    let current = LifecycleStatus::parse(current)?;
    if current == LifecycleStatus::Revoked && requested != LifecycleStatus::Revoked {
        Err(ProviderError::Conflict)
    } else {
        Ok(())
    }
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
    error!(operation, error = %error_value, "provider database operation failed");
    ProviderError::Internal
}

#[cfg(test)]
mod tests {
    use base64::{Engine as _, engine::general_purpose::STANDARD};

    use super::*;

    fn material(
        provider: LifecycleStatus,
        release: LifecycleStatus,
        policy: ActiveSessionPolicy,
    ) -> ProviderSecurityMaterial {
        ProviderSecurityMaterial {
            policy: ReleasePolicy {
                provider_id: "provider-one".to_owned(),
                provider_status: provider,
                release_id: Uuid::from_u128(1),
                game_key: "signal_siege".to_owned(),
                rules_version: 1,
                cartridge_digest: "a".repeat(64),
                endpoint: ProviderEndpoint {
                    host: "provider.example.test".to_owned(),
                    port: 443,
                    base_path: "/omarchygs/provider/v1/".to_owned(),
                },
                release_status: release,
                active_session_policy: policy,
                config_revision: 1,
                quotas: ProviderQuotas {
                    grants_per_minute: 10,
                    requests_per_minute: 10,
                    callbacks_per_minute: 10,
                    max_concurrent_requests: 2,
                    request_body_bytes: 8192,
                    response_body_bytes: 65_536,
                    connect_timeout_ms: 500,
                    total_timeout_ms: 2_000,
                },
            },
            message_keys: vec![RegisteredOperationalKey {
                key_id: "provider-key".to_owned(),
                public_material: vec![1; 32],
            }],
            tls_roots_der: vec![vec![2; 128]],
        }
    }

    #[test]
    fn lifecycle_matrix_stops_new_launch_and_honors_existing_policy() {
        let active = material(
            LifecycleStatus::Active,
            LifecycleStatus::Active,
            ActiveSessionPolicy::Continue,
        );
        assert!(
            evaluate_admission(
                &active,
                LifecycleStatus::Active,
                ProviderScope::Launch,
                SessionAdmission::New,
            )
            .is_ok()
        );
        for policy in [
            ActiveSessionPolicy::Terminate,
            ActiveSessionPolicy::ReadOnly,
            ActiveSessionPolicy::Continue,
        ] {
            let suspended = material(LifecycleStatus::Suspended, LifecycleStatus::Active, policy);
            assert!(
                evaluate_admission(
                    &suspended,
                    LifecycleStatus::Active,
                    ProviderScope::Launch,
                    SessionAdmission::New,
                )
                .is_err()
            );
            assert_eq!(
                evaluate_admission(
                    &suspended,
                    LifecycleStatus::Active,
                    ProviderScope::Reconcile,
                    SessionAdmission::Existing,
                )
                .is_ok(),
                policy != ActiveSessionPolicy::Terminate
            );
            assert_eq!(
                evaluate_admission(
                    &suspended,
                    LifecycleStatus::Active,
                    ProviderScope::Command,
                    SessionAdmission::Existing,
                )
                .is_ok(),
                policy == ActiveSessionPolicy::Continue
            );
        }
        let revoked = material(
            LifecycleStatus::Active,
            LifecycleStatus::Revoked,
            ActiveSessionPolicy::Continue,
        );
        assert!(
            evaluate_admission(
                &revoked,
                LifecycleStatus::Active,
                ProviderScope::Reconcile,
                SessionAdmission::Existing,
            )
            .is_err()
        );
    }

    #[test]
    fn disabled_scope_or_missing_key_fails_closed() {
        let mut active = material(
            LifecycleStatus::Active,
            LifecycleStatus::Active,
            ActiveSessionPolicy::Continue,
        );
        assert!(
            evaluate_admission(
                &active,
                LifecycleStatus::Suspended,
                ProviderScope::Command,
                SessionAdmission::Existing,
            )
            .is_err()
        );
        active.message_keys.clear();
        assert!(
            evaluate_admission(
                &active,
                LifecycleStatus::Active,
                ProviderScope::Command,
                SessionAdmission::Existing,
            )
            .is_err()
        );
    }

    #[test]
    fn public_material_is_standard_base64_not_secret_input() {
        let key = OperationalKeyInput {
            key_id: "provider-key".to_owned(),
            public_material_base64: STANDARD.encode([7_u8; 32]),
            valid_from: 1,
            valid_until: None,
        };
        assert_eq!(
            key.decode(OperationalKeyKind::MessageEd25519)
                .expect("public key should decode"),
            vec![7_u8; 32]
        );
    }
}
