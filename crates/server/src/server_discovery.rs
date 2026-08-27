use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

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

#[derive(Debug, Serialize, PartialEq, Eq)]
pub(crate) struct DiscoveryDocument {
    service: &'static str,
    server_id: String,
    server_name: String,
    protocol_version: u16,
    capabilities: Vec<&'static str>,
}

pub(crate) async fn document(
    pool: &PgPool,
    server_name: &str,
    provider_enabled: bool,
    cartridge_runtime_enabled: bool,
) -> Result<DiscoveryDocument, sqlx::Error> {
    let server_id =
        sqlx::query_scalar::<_, Uuid>("SELECT id FROM server_identity WHERE singleton = TRUE")
            .fetch_one(pool)
            .await?;

    Ok(document_for(
        server_id,
        server_name,
        provider_enabled,
        cartridge_runtime_enabled,
    ))
}

fn document_for(
    server_id: Uuid,
    server_name: &str,
    provider_enabled: bool,
    cartridge_runtime_enabled: bool,
) -> DiscoveryDocument {
    let mut capabilities = BASE_CAPABILITIES.to_vec();
    if provider_enabled {
        capabilities.push("games.registered-provider.v1");
        capabilities.sort_unstable();
    }
    if cartridge_runtime_enabled {
        capabilities.push("games.cartridge-acquisition.v1");
        capabilities.push("games.session-cartridge.v1");
        capabilities.sort_unstable();
    }

    DiscoveryDocument {
        service: "omarchy-gaming-system",
        server_id: server_id.to_string(),
        server_name: server_name.to_owned(),
        protocol_version: PROTOCOL_VERSION,
        capabilities,
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
        let base = document_for(server_id, "Test Community", false, false);
        let provider = document_for(server_id, "Test Community", true, false);

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
        let document = document_for(Uuid::from_u128(1), "Test Community", false, true);
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
                .windows(2)
                .all(|pair| pair[0] < pair[1])
        );
    }
}
