use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use omarchy_gaming_system_server::operator_custom::OperatorCustomAuthority;

pub(crate) const PROTOCOL_VERSION: u16 = 1;

const BASE_CAPABILITIES: [&str; 12] = [
    "accounts.invite-registration.v1",
    "auth.device-sessions.v1",
    "auth.totp.v1",
    "games.cartridge-catalog.v1",
    "games.challenges.v1",
    "games.sessions.v1",
    "identity.personas.v1",
    "social.connections.v1",
    "social.private-inbox.v1",
    "social.reporting.v1",
    "sync.cursor.v1",
    "sync.websocket-hints.v1",
];

pub(crate) const CUSTOM_MODULE_WARNING: &str =
    "This server runs operator-custom code not reviewed or supported by OmarchyGS.";
pub(crate) const CUSTOM_MODULE_SUPPORT_BOUNDARY: &str =
    "Security, privacy, availability, and support are the server operator's responsibility.";

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub(crate) struct OperatorCustomModulesDisclosure {
    format: &'static str,
    server_id: String,
    active_count: u8,
    behavior_capabilities: Vec<&'static str>,
    warning: &'static str,
    support_boundary: &'static str,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub(crate) struct DiscoveryDocument {
    service: &'static str,
    server_id: String,
    server_name: String,
    protocol_version: u16,
    capabilities: Vec<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    operator_custom: Option<OperatorCustomAuthority>,
    #[serde(skip_serializing_if = "Option::is_none")]
    operator_custom_modules: Option<OperatorCustomModulesDisclosure>,
}

pub(crate) async fn document(
    pool: &PgPool,
    server_name: &str,
    provider_enabled: bool,
    cartridge_runtime_enabled: bool,
    operator_custom: Option<&OperatorCustomAuthority>,
) -> Result<DiscoveryDocument, sqlx::Error> {
    let server_id =
        sqlx::query_scalar::<_, Uuid>("SELECT id FROM server_identity WHERE singleton = TRUE")
            .fetch_one(pool)
            .await?;

    let (active_count, moderation_labels): (i64, bool) = sqlx::query_as(
        r#"
        SELECT count(*), COALESCE(
            bool_or('moderation_add_label' = ANY(a.granted_capabilities)),
            FALSE
        )
        FROM server_module_instances i
        JOIN server_module_releases r ON r.release_id = i.release_id
        LEFT JOIN server_module_admissions a
          ON a.admission_id = i.current_admission_id
         AND a.lifecycle_revision = i.current_admission_revision
        WHERE r.provenance_class = 'operator_custom'
          AND i.lifecycle IN ('active', 'degraded')
        "#,
    )
    .fetch_one(pool)
    .await?;
    let operator_custom_modules = if active_count == 0 {
        None
    } else {
        let active_count = u8::try_from(active_count)
            .ok()
            .filter(|count| (1..=8).contains(count))
            .ok_or_else(|| {
                sqlx::Error::Protocol("custom module disclosure count invalid".into())
            })?;
        Some(OperatorCustomModulesDisclosure {
            format: "omarchygs.operator-custom-modules-disclosure/v1",
            server_id: server_id.to_string(),
            active_count,
            behavior_capabilities: if moderation_labels {
                vec!["moderation_labels"]
            } else {
                Vec::new()
            },
            warning: CUSTOM_MODULE_WARNING,
            support_boundary: CUSTOM_MODULE_SUPPORT_BOUNDARY,
        })
    };

    Ok(document_for(
        server_id,
        server_name,
        provider_enabled,
        cartridge_runtime_enabled,
        operator_custom,
        operator_custom_modules,
    ))
}

fn document_for(
    server_id: Uuid,
    server_name: &str,
    provider_enabled: bool,
    cartridge_runtime_enabled: bool,
    operator_custom: Option<&OperatorCustomAuthority>,
    operator_custom_modules: Option<OperatorCustomModulesDisclosure>,
) -> DiscoveryDocument {
    let mut capabilities = BASE_CAPABILITIES.to_vec();
    if provider_enabled {
        capabilities.push("games.registered-provider.v1");
        capabilities.sort_unstable();
    }
    if cartridge_runtime_enabled {
        capabilities.push("games.cartridge-acquisition.v1");
        capabilities.push("games.session-cartridge.v1");
        capabilities.push("games.session-cartridge-acquisition.v1");
        capabilities.sort_unstable();
    }
    if operator_custom.is_some() {
        capabilities.push("games.operator-custom-cartridges.v1");
        capabilities.sort_unstable();
    }
    if operator_custom_modules.is_some() {
        capabilities.push("server.operator-custom-modules.v1");
        capabilities.sort_unstable();
    }

    DiscoveryDocument {
        service: "omarchy-gaming-system",
        server_id: server_id.to_string(),
        server_name: server_name.to_owned(),
        protocol_version: PROTOCOL_VERSION,
        capabilities,
        operator_custom: operator_custom.cloned(),
        operator_custom_modules,
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::{BASE_CAPABILITIES, document_for};

    #[test]
    fn base_capabilities_are_unique_and_lexically_ordered() {
        assert!(BASE_CAPABILITIES.windows(2).all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn provider_capability_is_truthful_and_ordered() {
        let server_id = Uuid::from_u128(1);
        let base = document_for(server_id, "Test Community", false, false, None, None);
        let provider = document_for(server_id, "Test Community", true, false, None, None);

        assert!(!base.capabilities.contains(&"games.registered-provider.v1"));
        assert!(
            provider
                .capabilities
                .contains(&"games.registered-provider.v1")
        );
        assert!(
            provider
                .capabilities
                .windows(2)
                .all(|pair| pair[0] < pair[1])
        );
    }

    #[test]
    fn acquisition_capability_is_truthful_and_ordered() {
        let document = document_for(
            Uuid::from_u128(1),
            "Test Community",
            false,
            true,
            None,
            None,
        );
        assert!(
            document
                .capabilities
                .contains(&"games.cartridge-acquisition.v1")
        );
        assert!(
            document
                .capabilities
                .contains(&"games.session-cartridge.v1")
        );
        assert!(
            document
                .capabilities
                .contains(&"games.session-cartridge-acquisition.v1")
        );
        assert!(
            document
                .capabilities
                .windows(2)
                .all(|pair| pair[0] < pair[1])
        );
    }
}
