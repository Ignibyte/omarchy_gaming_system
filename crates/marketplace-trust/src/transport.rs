//! Public-destination HTTPS transport shared by marketplace and client
//! package-channel consumers.

use std::{
    net::{IpAddr, SocketAddr},
    time::Duration,
};

use futures_util::StreamExt as _;
use omarchy_game_provider::egress::is_public_egress_ip;
use reqwest::{Client, redirect::Policy as RedirectPolicy, tls::Certificate};
use sha2::{Digest as _, Sha256};
use tokio::io::{AsyncWrite, AsyncWriteExt as _};
use url::{Host, Url};

const MAX_DNS_ANSWERS: usize = 8;
const MAX_TLS_ROOT_BYTES: usize = 32 * 1024;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);
const TOTAL_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelEgressError {
    InvalidInput,
    Denied,
    Unavailable,
    Rejected,
    Internal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelOrigin {
    canonical: String,
    host: String,
    port: u16,
}

impl ChannelOrigin {
    pub fn parse(value: &str) -> Result<Self, ChannelEgressError> {
        if value.is_empty() || value.len() > 512 || value.trim() != value {
            return Err(ChannelEgressError::InvalidInput);
        }
        let url = Url::parse(value).map_err(|_| ChannelEgressError::InvalidInput)?;
        let host = match url.host() {
            Some(Host::Domain(host)) if valid_domain(host) => host.to_owned(),
            _ => return Err(ChannelEgressError::InvalidInput),
        };
        if url.scheme() != "https"
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
            || !url.path().ends_with('/')
            || url.as_str() != value
        {
            return Err(ChannelEgressError::InvalidInput);
        }
        let port = url
            .port_or_known_default()
            .ok_or(ChannelEgressError::InvalidInput)?;
        Ok(Self {
            canonical: value.to_owned(),
            host,
            port,
        })
    }

    pub fn as_str(&self) -> &str {
        &self.canonical
    }

    fn request_url(&self, relative: &str) -> Result<Url, ChannelEgressError> {
        if !valid_relative_path(relative) {
            return Err(ChannelEgressError::InvalidInput);
        }
        let base = Url::parse(&self.canonical).map_err(|_| ChannelEgressError::Internal)?;
        let joined = base
            .join(relative)
            .map_err(|_| ChannelEgressError::InvalidInput)?;
        if joined.scheme() != "https"
            || joined.host_str() != Some(self.host.as_str())
            || joined.port_or_known_default() != Some(self.port)
            || joined.query().is_some()
            || joined.fragment().is_some()
            || !joined.as_str().starts_with(&self.canonical)
        {
            return Err(ChannelEgressError::Denied);
        }
        Ok(joined)
    }
}

pub struct GuardedChannelClient {
    client: Client,
    origin: ChannelOrigin,
}

impl GuardedChannelClient {
    pub async fn production(origin: ChannelOrigin) -> Result<Self, ChannelEgressError> {
        let sockets = production_sockets(&origin).await?;
        Self::from_sockets(origin, sockets, None)
    }

    pub async fn production_with_root(
        origin: ChannelOrigin,
        tls_root_der: &[u8],
    ) -> Result<Self, ChannelEgressError> {
        let sockets = production_sockets(&origin).await?;
        Self::from_sockets(origin, sockets, Some(tls_root_der))
    }

    /// Exact loopback transport for the repository's hostile conformance
    /// corpus. It is never selected by production configuration.
    #[doc(hidden)]
    pub fn conformance_loopback(
        origin: ChannelOrigin,
        exact_socket: SocketAddr,
        tls_root_der: &[u8],
    ) -> Result<Self, ChannelEgressError> {
        let sockets = validate_resolution(
            &origin,
            [exact_socket.ip()],
            ResolutionMode::Conformance(exact_socket),
        )?;
        Self::from_sockets(origin, sockets, Some(tls_root_der))
    }

    fn from_sockets(
        origin: ChannelOrigin,
        sockets: Vec<SocketAddr>,
        tls_root_der: Option<&[u8]>,
    ) -> Result<Self, ChannelEgressError> {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let mut builder = Client::builder()
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
            .resolve_to_addrs(&origin.host, &sockets);
        if let Some(bytes) = tls_root_der {
            if !(64..=MAX_TLS_ROOT_BYTES).contains(&bytes.len()) {
                return Err(ChannelEgressError::Denied);
            }
            let certificate =
                Certificate::from_der(bytes).map_err(|_| ChannelEgressError::Denied)?;
            builder = builder.tls_certs_only([certificate]);
        }
        let client = builder.build().map_err(|_| ChannelEgressError::Internal)?;
        Ok(Self { client, origin })
    }

    pub async fn get_bytes(
        &self,
        relative: &str,
        limit: usize,
        expected_content_type: &str,
    ) -> Result<Vec<u8>, ChannelEgressError> {
        if limit == 0 || !valid_content_type(expected_content_type) {
            return Err(ChannelEgressError::InvalidInput);
        }
        let response = self.response(relative, expected_content_type).await?;
        if response
            .content_length()
            .is_some_and(|length| length > limit as u64)
        {
            return Err(ChannelEgressError::Rejected);
        }
        let mut body =
            Vec::with_capacity(response.content_length().unwrap_or(0).min(limit as u64) as usize);
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|_| ChannelEgressError::Unavailable)?;
            if body
                .len()
                .checked_add(chunk.len())
                .is_none_or(|length| length > limit)
            {
                return Err(ChannelEgressError::Rejected);
            }
            body.extend_from_slice(&chunk);
        }
        if body.is_empty() {
            Err(ChannelEgressError::Rejected)
        } else {
            Ok(body)
        }
    }

    pub async fn download_exact<W: AsyncWrite + Unpin>(
        &self,
        relative: &str,
        expected_content_type: &str,
        expected_bytes: u64,
        expected_sha256: &str,
        output: &mut W,
    ) -> Result<(), ChannelEgressError> {
        if expected_bytes == 0
            || expected_bytes > super::MAX_PACKAGE_BYTES
            || !valid_sha256(expected_sha256)
        {
            return Err(ChannelEgressError::InvalidInput);
        }
        let response = self.response(relative, expected_content_type).await?;
        if response
            .content_length()
            .is_some_and(|length| length != expected_bytes)
        {
            return Err(ChannelEgressError::Rejected);
        }
        let mut received = 0_u64;
        let mut digest = Sha256::new();
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|_| ChannelEgressError::Unavailable)?;
            received = received
                .checked_add(chunk.len() as u64)
                .ok_or(ChannelEgressError::Rejected)?;
            if received > expected_bytes {
                return Err(ChannelEgressError::Rejected);
            }
            digest.update(&chunk);
            output
                .write_all(&chunk)
                .await
                .map_err(|_| ChannelEgressError::Internal)?;
        }
        let actual = digest
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        if received != expected_bytes || actual != expected_sha256 {
            Err(ChannelEgressError::Rejected)
        } else {
            Ok(())
        }
    }

    async fn response(
        &self,
        relative: &str,
        expected_content_type: &str,
    ) -> Result<reqwest::Response, ChannelEgressError> {
        if !valid_content_type(expected_content_type) {
            return Err(ChannelEgressError::InvalidInput);
        }
        let response = self
            .client
            .get(self.origin.request_url(relative)?)
            .header("accept", expected_content_type)
            .send()
            .await
            .map_err(|_| ChannelEgressError::Unavailable)?;
        if response.status().as_u16() != 200
            || response
                .headers()
                .get("content-type")
                .and_then(|value| value.to_str().ok())
                != Some(expected_content_type)
            || response.headers().contains_key("content-encoding")
        {
            return Err(ChannelEgressError::Rejected);
        }
        Ok(response)
    }
}

async fn production_sockets(origin: &ChannelOrigin) -> Result<Vec<SocketAddr>, ChannelEgressError> {
    let resolved = tokio::time::timeout(
        TOTAL_TIMEOUT,
        tokio::net::lookup_host((origin.host.as_str(), origin.port)),
    )
    .await
    .map_err(|_| ChannelEgressError::Unavailable)?
    .map_err(|_| ChannelEgressError::Unavailable)?;
    validate_resolution(
        origin,
        resolved.map(|socket| socket.ip()),
        ResolutionMode::Production,
    )
}

#[derive(Debug, Clone, Copy)]
enum ResolutionMode {
    Production,
    Conformance(SocketAddr),
}

fn validate_resolution(
    origin: &ChannelOrigin,
    addresses: impl IntoIterator<Item = IpAddr>,
    mode: ResolutionMode,
) -> Result<Vec<SocketAddr>, ChannelEgressError> {
    let mut sockets = Vec::new();
    for address in addresses {
        if sockets.len() >= MAX_DNS_ANSWERS {
            return Err(ChannelEgressError::Denied);
        }
        let socket = SocketAddr::new(address, origin.port);
        let allowed = match mode {
            ResolutionMode::Production => is_public_egress_ip(address),
            ResolutionMode::Conformance(exact) => address.is_loopback() && socket == exact,
        };
        if !allowed {
            return Err(ChannelEgressError::Denied);
        }
        sockets.push(socket);
    }
    sockets.sort_unstable();
    sockets.dedup();
    if sockets.is_empty() {
        Err(ChannelEgressError::Unavailable)
    } else {
        Ok(sockets)
    }
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
                && segment.len() <= 192
                && segment.bytes().all(|byte| {
                    byte.is_ascii_lowercase()
                        || byte.is_ascii_digit()
                        || matches!(byte, b'.' | b'-' | b'_' | b'+')
                })
        })
}

fn valid_content_type(value: &str) -> bool {
    matches!(
        value,
        "application/json" | "application/octet-stream" | "application/vnd.archlinux.package"
    )
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origin_paths_and_resolution_are_exact() {
        let origin =
            ChannelOrigin::parse("https://packages.example.test/v1/").expect("canonical origin");
        assert!(origin.request_url("trust.signed.json").is_ok());
        for denied in [
            "http://packages.example.test/v1/",
            "https://127.0.0.1/v1/",
            "https://packages.local/v1/",
            "https://packages.example.test/v1",
        ] {
            assert!(ChannelOrigin::parse(denied).is_err(), "{denied}");
        }
        for denied in ["/absolute", "../escape", "a//b", "a/%2e%2e/b", "a/b/"] {
            assert!(origin.request_url(denied).is_err(), "{denied}");
        }
        assert!(
            validate_resolution(
                &origin,
                [
                    "1.1.1.1".parse().expect("public"),
                    "127.0.0.1".parse().expect("loopback"),
                ],
                ResolutionMode::Production,
            )
            .is_err()
        );
    }
}
