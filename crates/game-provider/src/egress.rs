//! Guarded provider endpoint resolution and bounded HTTPS transport.

use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    time::Duration,
};

use futures_util::StreamExt as _;
use http::HeaderMap;
use reqwest::{Client, redirect::Policy as RedirectPolicy, tls::Certificate};

use crate::{
    ProviderError, Result,
    model::{ProviderEndpoint, ProviderQuotas},
};

const MAX_DNS_ANSWERS: usize = 8;

/// One resolution result pinned into an exact guarded client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedDestination {
    /// Canonical registered endpoint.
    pub endpoint: ProviderEndpoint,
    /// Validated and sorted socket addresses used without a second DNS lookup.
    pub addresses: Vec<SocketAddr>,
}

/// Exact raw response. Authentication happens over these retained bytes.
#[derive(Debug)]
pub struct RawProviderResponse {
    /// HTTP status.
    pub status: u16,
    /// Exact parsed response headers.
    pub headers: HeaderMap,
    /// Body bytes stopped at the registered streaming ceiling.
    pub body: Vec<u8>,
}

/// HTTPS client bound to one exact release destination and policy snapshot.
pub struct GuardedProviderClient {
    client: Client,
    destination: ResolvedDestination,
    quotas: ProviderQuotas,
}

impl GuardedProviderClient {
    /// Resolve and construct a production client. Every DNS answer must be a
    /// globally routable unicast address.
    pub async fn production(
        endpoint: ProviderEndpoint,
        tls_roots_der: &[Vec<u8>],
        quotas: ProviderQuotas,
    ) -> Result<Self> {
        endpoint.validate()?;
        quotas.validate()?;
        let resolved = tokio::net::lookup_host((endpoint.host.as_str(), endpoint.port))
            .await
            .map_err(|_| ProviderError::Unavailable)?;
        let addresses = resolved.map(|socket| socket.ip()).collect::<Vec<_>>();
        let destination = validate_resolution(endpoint, addresses, ResolutionMode::Production)?;
        Self::from_destination(destination, tls_roots_der, quotas)
    }

    /// Build the compile-time conformance client for one exact loopback socket.
    /// No arbitrary private-network allowlist exists.
    #[cfg(any(test, feature = "provider-conformance"))]
    pub fn conformance_loopback(
        endpoint: ProviderEndpoint,
        exact_socket: SocketAddr,
        tls_roots_der: &[Vec<u8>],
        quotas: ProviderQuotas,
    ) -> Result<Self> {
        endpoint.validate()?;
        quotas.validate()?;
        let destination = validate_resolution(
            endpoint,
            [exact_socket.ip()],
            ResolutionMode::Conformance(exact_socket),
        )?;
        Self::from_destination(destination, tls_roots_der, quotas)
    }

    fn from_destination(
        destination: ResolvedDestination,
        tls_roots_der: &[Vec<u8>],
        quotas: ProviderQuotas,
    ) -> Result<Self> {
        install_crypto_provider();
        if tls_roots_der.is_empty() || tls_roots_der.len() > 8 {
            return Err(ProviderError::Denied);
        }
        let certificates = tls_roots_der
            .iter()
            .map(|der| {
                if !(64..=32 * 1024).contains(&der.len()) {
                    return Err(ProviderError::Denied);
                }
                Certificate::from_der(der).map_err(|_| ProviderError::Denied)
            })
            .collect::<Result<Vec<_>>>()?;
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
            .connect_timeout(Duration::from_millis(u64::from(quotas.connect_timeout_ms)))
            .read_timeout(Duration::from_millis(u64::from(quotas.total_timeout_ms)))
            .timeout(Duration::from_millis(u64::from(quotas.total_timeout_ms)))
            .pool_max_idle_per_host(0)
            .tls_certs_only(certificates)
            .resolve_to_addrs(&destination.endpoint.host, &destination.addresses)
            .build()
            .map_err(|_| ProviderError::Internal)?;
        Ok(Self {
            client,
            destination,
            quotas,
        })
    }

    /// Send one POST to an allowlisted operation path and stream a bounded body.
    pub async fn post(
        &self,
        operation: &str,
        headers: HeaderMap,
        body: Vec<u8>,
    ) -> Result<RawProviderResponse> {
        if body.is_empty() || body.len() > self.quotas.request_body_bytes as usize {
            return Err(ProviderError::InvalidInput);
        }
        let url = self.destination.endpoint.operation_url(operation)?;
        let response = self
            .client
            .post(url)
            .headers(headers)
            .body(body)
            .send()
            .await
            .map_err(|_| ProviderError::Unavailable)?;
        let status = response.status().as_u16();
        if (300..400).contains(&status) {
            return Err(ProviderError::ProtocolRejected);
        }
        let limit = self.quotas.response_body_bytes as usize;
        if response
            .content_length()
            .is_some_and(|length| length > limit as u64)
        {
            return Err(ProviderError::ProtocolRejected);
        }
        let headers = response.headers().clone();
        let mut body =
            Vec::with_capacity(response.content_length().unwrap_or(0).min(limit as u64) as usize);
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|_| ProviderError::Unavailable)?;
            if body
                .len()
                .checked_add(chunk.len())
                .is_none_or(|length| length > limit)
            {
                return Err(ProviderError::ProtocolRejected);
            }
            body.extend_from_slice(&chunk);
        }
        if body.is_empty() {
            return Err(ProviderError::ProtocolRejected);
        }
        Ok(RawProviderResponse {
            status,
            headers,
            body,
        })
    }

    /// Resolved addresses currently pinned into this one-release client.
    #[must_use]
    pub fn destination(&self) -> &ResolvedDestination {
        &self.destination
    }
}

fn install_crypto_provider() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}

#[derive(Debug, Clone, Copy)]
enum ResolutionMode {
    Production,
    #[cfg(any(test, feature = "provider-conformance"))]
    Conformance(SocketAddr),
}

fn validate_resolution(
    endpoint: ProviderEndpoint,
    addresses: impl IntoIterator<Item = IpAddr>,
    mode: ResolutionMode,
) -> Result<ResolvedDestination> {
    let mut sockets = Vec::new();
    for address in addresses {
        if sockets.len() >= MAX_DNS_ANSWERS {
            return Err(ProviderError::Denied);
        }
        let socket = SocketAddr::new(address, endpoint.port);
        let allowed = match mode {
            ResolutionMode::Production => is_public_provider_ip(address),
            #[cfg(any(test, feature = "provider-conformance"))]
            ResolutionMode::Conformance(exact) => address.is_loopback() && socket == exact,
        };
        if !allowed {
            return Err(ProviderError::Denied);
        }
        sockets.push(socket);
    }
    sockets.sort_unstable();
    sockets.dedup();
    if sockets.is_empty() {
        return Err(ProviderError::Unavailable);
    }
    Ok(ResolvedDestination {
        endpoint,
        addresses: sockets,
    })
}

/// Conservative globally routable unicast classification for provider egress.
/// Every private, loopback, link-local, special-use, documentation, benchmark,
/// multicast, and reserved range rejects.
#[must_use]
pub fn is_public_provider_ip(address: IpAddr) -> bool {
    match address {
        IpAddr::V4(address) => is_public_ipv4(address),
        IpAddr::V6(address) => is_public_ipv6(address),
    }
}

fn is_public_ipv4(address: Ipv4Addr) -> bool {
    let value = u32::from(address);
    ![
        ("0.0.0.0", 8),
        ("10.0.0.0", 8),
        ("100.64.0.0", 10),
        ("127.0.0.0", 8),
        ("169.254.0.0", 16),
        ("172.16.0.0", 12),
        ("192.0.0.0", 24),
        ("192.0.2.0", 24),
        ("192.88.99.0", 24),
        ("192.168.0.0", 16),
        ("198.18.0.0", 15),
        ("198.51.100.0", 24),
        ("203.0.113.0", 24),
        ("224.0.0.0", 4),
        ("240.0.0.0", 4),
    ]
    .iter()
    .any(|(network, prefix)| {
        let network = network
            .parse::<Ipv4Addr>()
            .map(u32::from)
            .unwrap_or(u32::MAX);
        prefix_matches_u32(value, network, *prefix)
    })
}

fn is_public_ipv6(address: Ipv6Addr) -> bool {
    if let Some(mapped) = address.to_ipv4_mapped() {
        return is_public_ipv4(mapped);
    }
    let value = u128::from(address);
    let global_unicast = "2000::"
        .parse::<Ipv6Addr>()
        .map(u128::from)
        .is_ok_and(|network| prefix_matches_u128(value, network, 3));
    if !global_unicast {
        return false;
    }
    ![
        ("2001::", 32),
        ("2001:1::1", 128),
        ("2001:1::2", 128),
        ("2001:1::3", 128),
        ("2001:2::", 48),
        ("2001:3::", 32),
        ("2001:4:112::", 48),
        ("2001:10::", 28),
        ("2001:20::", 28),
        ("2001:30::", 28),
        ("2001:db8::", 32),
        ("2002::", 16),
        ("2620:4f:8000::", 48),
        ("3fff::", 20),
    ]
    .iter()
    .any(|(network, prefix)| {
        let network = network
            .parse::<Ipv6Addr>()
            .map(u128::from)
            .unwrap_or(u128::MAX);
        prefix_matches_u128(value, network, *prefix)
    })
}

fn prefix_matches_u32(value: u32, network: u32, prefix: u32) -> bool {
    let mask = if prefix == 0 {
        0
    } else {
        u32::MAX << (32 - prefix)
    };
    value & mask == network & mask
}

fn prefix_matches_u128(value: u128, network: u128, prefix: u32) -> bool {
    let mask = if prefix == 0 {
        0
    } else {
        u128::MAX << (128 - prefix)
    };
    value & mask == network & mask
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_classifier_rejects_special_ipv4_and_ipv6_corpus() {
        for denied in [
            "0.0.0.0",
            "10.2.3.4",
            "100.64.1.1",
            "127.0.0.1",
            "169.254.169.254",
            "172.31.0.1",
            "192.0.2.1",
            "192.168.1.1",
            "198.18.1.1",
            "198.51.100.2",
            "203.0.113.2",
            "224.0.0.1",
            "255.255.255.255",
            "::",
            "::1",
            "::ffff:127.0.0.1",
            "64:ff9b::1",
            "64:ff9b:1::c000:201",
            "100::1",
            "2001::1",
            "2001:1::2",
            "2001:2::1",
            "2001:20::1",
            "2001:db8::1",
            "2002::1",
            "2620:4f:8000::1",
            "3fff::1",
            "4000::1",
            "5f00::1",
            "fc00::1",
            "fe80::1",
            "ff02::1",
        ] {
            let parsed = denied.parse().expect("fixture IP should parse");
            assert!(!is_public_provider_ip(parsed), "{denied} should reject");
        }
        for allowed in ["1.1.1.1", "8.8.8.8", "2606:4700:4700::1111"] {
            let parsed = allowed.parse().expect("fixture IP should parse");
            assert!(is_public_provider_ip(parsed), "{allowed} should pass");
        }
    }

    #[test]
    fn mixed_resolution_and_too_many_answers_fail_closed() {
        let endpoint = ProviderEndpoint {
            host: "provider.example.test".to_owned(),
            port: 443,
            base_path: "/omarchygs/provider/v1/".to_owned(),
        };
        assert!(
            validate_resolution(
                endpoint.clone(),
                [
                    "1.1.1.1".parse().expect("public fixture should parse"),
                    "127.0.0.1".parse().expect("private fixture should parse"),
                ],
                ResolutionMode::Production,
            )
            .is_err()
        );
        let too_many = (1_u8..=9)
            .map(|last| IpAddr::V4(Ipv4Addr::new(8, 8, 8, last)))
            .collect::<Vec<_>>();
        assert!(validate_resolution(endpoint, too_many, ResolutionMode::Production).is_err());
    }
}
