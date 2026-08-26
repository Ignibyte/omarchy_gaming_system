//! Guarded HTTPS transport for one operator-pinned marketplace origin.

use std::{
    net::{IpAddr, SocketAddr},
    time::Duration,
};

use futures_util::StreamExt as _;
use omarchy_game_provider::egress::is_public_egress_ip;
use reqwest::{Client, redirect::Policy as RedirectPolicy, tls::Certificate};
use url::{Host, Url};

const MAX_DNS_ANSWERS: usize = 8;
const MAX_TLS_ROOT_BYTES: usize = 32 * 1024;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);
const TOTAL_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketplaceEgressError {
    InvalidInput,
    Denied,
    Unavailable,
    Rejected,
    Internal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketplaceOrigin {
    canonical: String,
    host: String,
    port: u16,
}

impl MarketplaceOrigin {
    pub fn parse(value: &str) -> Result<Self, MarketplaceEgressError> {
        if value.is_empty() || value.len() > 512 || value.trim() != value {
            return Err(MarketplaceEgressError::InvalidInput);
        }
        let url = Url::parse(value).map_err(|_| MarketplaceEgressError::InvalidInput)?;
        let host = match url.host() {
            Some(Host::Domain(host)) if valid_domain(host) => host.to_owned(),
            _ => return Err(MarketplaceEgressError::InvalidInput),
        };
        if url.scheme() != "https"
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
            || !url.path().ends_with('/')
            || url.as_str() != value
        {
            return Err(MarketplaceEgressError::InvalidInput);
        }
        let port = url
            .port_or_known_default()
            .ok_or(MarketplaceEgressError::InvalidInput)?;
        Ok(Self {
            canonical: value.to_owned(),
            host,
            port,
        })
    }

    pub fn as_str(&self) -> &str {
        &self.canonical
    }

    fn request_url(&self, relative: &str) -> Result<Url, MarketplaceEgressError> {
        if !valid_relative_path(relative) {
            return Err(MarketplaceEgressError::InvalidInput);
        }
        let base = Url::parse(&self.canonical).map_err(|_| MarketplaceEgressError::Internal)?;
        let joined = base
            .join(relative)
            .map_err(|_| MarketplaceEgressError::InvalidInput)?;
        if joined.scheme() != "https"
            || joined.host_str() != Some(self.host.as_str())
            || joined.port_or_known_default() != Some(self.port)
            || joined.query().is_some()
            || joined.fragment().is_some()
            || !joined.as_str().starts_with(&self.canonical)
        {
            return Err(MarketplaceEgressError::Denied);
        }
        Ok(joined)
    }
}

pub struct GuardedMarketplaceClient {
    client: Client,
    origin: MarketplaceOrigin,
}

impl GuardedMarketplaceClient {
    pub async fn production(
        origin: MarketplaceOrigin,
        tls_root_der: &[u8],
    ) -> Result<Self, MarketplaceEgressError> {
        let resolved = tokio::net::lookup_host((origin.host.as_str(), origin.port))
            .await
            .map_err(|_| MarketplaceEgressError::Unavailable)?;
        let addresses = resolved.map(|socket| socket.ip()).collect::<Vec<_>>();
        let sockets = validate_resolution(&origin, addresses, ResolutionMode::Production)?;
        Self::from_sockets(origin, sockets, tls_root_der)
    }

    #[cfg(any(test, feature = "marketplace-conformance"))]
    pub fn conformance_loopback(
        origin: MarketplaceOrigin,
        exact_socket: SocketAddr,
        tls_root_der: &[u8],
    ) -> Result<Self, MarketplaceEgressError> {
        let sockets = validate_resolution(
            &origin,
            [exact_socket.ip()],
            ResolutionMode::Conformance(exact_socket),
        )?;
        Self::from_sockets(origin, sockets, tls_root_der)
    }

    fn from_sockets(
        origin: MarketplaceOrigin,
        sockets: Vec<SocketAddr>,
        tls_root_der: &[u8],
    ) -> Result<Self, MarketplaceEgressError> {
        if !(64..=MAX_TLS_ROOT_BYTES).contains(&tls_root_der.len()) {
            return Err(MarketplaceEgressError::Denied);
        }
        let _ = rustls::crypto::ring::default_provider().install_default();
        let certificate =
            Certificate::from_der(tls_root_der).map_err(|_| MarketplaceEgressError::Denied)?;
        let client = Client::builder()
            .no_proxy()
            .redirect(RedirectPolicy::none())
            .referer(false)
            .https_only(true)
            .http1_only()
            .no_gzip()
            .no_brotli()
            .no_deflate()
            .no_zstd()
            .connect_timeout(CONNECT_TIMEOUT)
            .read_timeout(TOTAL_TIMEOUT)
            .timeout(TOTAL_TIMEOUT)
            .pool_max_idle_per_host(0)
            .tls_certs_only([certificate])
            .resolve_to_addrs(&origin.host, &sockets)
            .build()
            .map_err(|_| MarketplaceEgressError::Internal)?;
        Ok(Self { client, origin })
    }

    pub async fn get(
        &self,
        relative: &str,
        limit: usize,
    ) -> Result<Vec<u8>, MarketplaceEgressError> {
        if limit == 0 {
            return Err(MarketplaceEgressError::InvalidInput);
        }
        let url = self.origin.request_url(relative)?;
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|_| MarketplaceEgressError::Unavailable)?;
        if response.status().as_u16() != 200 {
            return Err(MarketplaceEgressError::Rejected);
        }
        if response
            .content_length()
            .is_some_and(|length| length > limit as u64)
        {
            return Err(MarketplaceEgressError::Rejected);
        }
        let mut body =
            Vec::with_capacity(response.content_length().unwrap_or(0).min(limit as u64) as usize);
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|_| MarketplaceEgressError::Unavailable)?;
            if body
                .len()
                .checked_add(chunk.len())
                .is_none_or(|length| length > limit)
            {
                return Err(MarketplaceEgressError::Rejected);
            }
            body.extend_from_slice(&chunk);
        }
        if body.is_empty() {
            return Err(MarketplaceEgressError::Rejected);
        }
        Ok(body)
    }
}

#[derive(Debug, Clone, Copy)]
enum ResolutionMode {
    Production,
    #[cfg(any(test, feature = "marketplace-conformance"))]
    Conformance(SocketAddr),
}

fn validate_resolution(
    origin: &MarketplaceOrigin,
    addresses: impl IntoIterator<Item = IpAddr>,
    mode: ResolutionMode,
) -> Result<Vec<SocketAddr>, MarketplaceEgressError> {
    let mut sockets = Vec::new();
    for address in addresses {
        if sockets.len() >= MAX_DNS_ANSWERS {
            return Err(MarketplaceEgressError::Denied);
        }
        let socket = SocketAddr::new(address, origin.port);
        let allowed = match mode {
            ResolutionMode::Production => is_public_egress_ip(address),
            #[cfg(any(test, feature = "marketplace-conformance"))]
            ResolutionMode::Conformance(exact) => address.is_loopback() && socket == exact,
        };
        if !allowed {
            return Err(MarketplaceEgressError::Denied);
        }
        sockets.push(socket);
    }
    sockets.sort_unstable();
    sockets.dedup();
    if sockets.is_empty() {
        return Err(MarketplaceEgressError::Unavailable);
    }
    Ok(sockets)
}

fn valid_domain(host: &str) -> bool {
    !host.is_empty()
        && host.len() <= 253
        && host == host.to_ascii_lowercase()
        && host.contains('.')
        && !host.ends_with('.')
        && !host.ends_with(".local")
        && !host.ends_with(".localhost")
        && !host.ends_with(".internal")
        && host.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        })
}

fn valid_relative_path(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= 512
        && !path.starts_with('/')
        && !path.ends_with('/')
        && !path.contains("//")
        && !path.contains('%')
        && !path.contains('?')
        && !path.contains('#')
        && !path.contains('\\')
        && path.split('/').all(|segment| {
            !segment.is_empty()
                && segment != "."
                && segment != ".."
                && segment.len() <= 128
                && segment.bytes().all(|byte| {
                    byte.is_ascii_lowercase()
                        || byte.is_ascii_digit()
                        || matches!(byte, b'.' | b'-' | b'_')
                })
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origin_and_relative_paths_are_exact() {
        let origin = MarketplaceOrigin::parse("https://market.example.test/v1/")
            .expect("canonical origin should pass");
        assert_eq!(origin.as_str(), "https://market.example.test/v1/");
        assert!(origin.request_url("snapshot.signed.json").is_ok());
        for denied in [
            "http://market.example.test/v1/",
            "https://127.0.0.1/v1/",
            "https://market.local/v1/",
            "https://market.example.test/v1",
            "https://market.example.test/v1/?x=1",
        ] {
            assert!(MarketplaceOrigin::parse(denied).is_err(), "{denied}");
        }
        for denied in ["/absolute", "../escape", "a//b", "a/%2e%2e/b", "a/b/"] {
            assert!(origin.request_url(denied).is_err(), "{denied}");
        }
    }

    #[test]
    fn resolution_rejects_private_mixed_and_excessive_answers() {
        let origin =
            MarketplaceOrigin::parse("https://market.example.test/").expect("origin should pass");
        assert!(
            validate_resolution(
                &origin,
                [
                    "1.1.1.1".parse().expect("public IP"),
                    "127.0.0.1".parse().expect("loopback IP"),
                ],
                ResolutionMode::Production,
            )
            .is_err()
        );
        let too_many = (1_u8..=9)
            .map(|last| IpAddr::from([8, 8, 8, last]))
            .collect::<Vec<_>>();
        assert!(validate_resolution(&origin, too_many, ResolutionMode::Production).is_err());
    }
}
