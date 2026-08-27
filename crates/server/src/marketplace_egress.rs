//! Server adapter for the shared guarded marketplace/channel HTTPS transport.

#[cfg(any(test, feature = "marketplace-conformance"))]
use std::net::SocketAddr;

use omarchygs_marketplace_trust::{ChannelEgressError, ChannelOrigin, GuardedChannelClient};

pub type MarketplaceEgressError = ChannelEgressError;
pub type MarketplaceOrigin = ChannelOrigin;

pub struct GuardedMarketplaceClient(GuardedChannelClient);

impl GuardedMarketplaceClient {
    pub async fn production(
        origin: MarketplaceOrigin,
        tls_root_der: &[u8],
    ) -> Result<Self, MarketplaceEgressError> {
        GuardedChannelClient::production_with_root(origin, tls_root_der)
            .await
            .map(Self)
    }

    #[cfg(any(test, feature = "marketplace-conformance"))]
    pub fn conformance_loopback(
        origin: MarketplaceOrigin,
        exact_socket: SocketAddr,
        tls_root_der: &[u8],
    ) -> Result<Self, MarketplaceEgressError> {
        GuardedChannelClient::conformance_loopback(origin, exact_socket, tls_root_der).map(Self)
    }

    pub async fn get(
        &self,
        relative: &str,
        limit: usize,
    ) -> Result<Vec<u8>, MarketplaceEgressError> {
        let content_type = if relative.ends_with(".json") {
            "application/json"
        } else {
            "application/octet-stream"
        };
        self.0.get_bytes(relative, limit, content_type).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origin_contract_is_the_shared_channel_contract() {
        let origin = MarketplaceOrigin::parse("https://market.example.test/v1/")
            .expect("canonical origin should pass");
        assert_eq!(origin.as_str(), "https://market.example.test/v1/");
        for denied in [
            "http://market.example.test/v1/",
            "https://127.0.0.1/v1/",
            "https://market.local/v1/",
            "https://market.example.test/v1",
            "https://market.example.test/v1/?x=1",
        ] {
            assert!(MarketplaceOrigin::parse(denied).is_err(), "{denied}");
        }
    }
}
