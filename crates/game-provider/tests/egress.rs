use std::net::IpAddr;

#[cfg(feature = "provider-conformance")]
use std::net::{Ipv4Addr, SocketAddr};

use omarchy_game_provider::{egress::is_public_provider_ip, model::ProviderEndpoint};

#[cfg(feature = "provider-conformance")]
use omarchy_game_provider::egress::GuardedProviderClient;
#[cfg(feature = "provider-conformance")]
use omarchy_game_provider::model::ProviderQuotas;

#[test]
fn endpoint_contract_rejects_noncanonical_and_local_names() {
    for endpoint in [
        endpoint("127.0.0.1", "/omarchygs/provider/v1/"),
        endpoint("provider.local", "/omarchygs/provider/v1/"),
        endpoint("Provider.example.test", "/omarchygs/provider/v1/"),
        endpoint("provider.example.test", "/omarchygs/provider/v1"),
        endpoint("provider.example.test", "/omarchygs/../provider/"),
        endpoint("provider.example.test", "/omarchygs/provider/?target=x/"),
    ] {
        assert!(endpoint.validate().is_err(), "{endpoint:?} should reject");
    }
}

#[test]
fn public_egress_classifier_denies_metadata_private_and_special_ranges() {
    for address in [
        "169.254.169.254",
        "100.100.100.200",
        "172.16.0.1",
        "192.168.1.1",
        "198.51.100.10",
        "::1",
        "fc00::1",
        "fe80::1",
        "2001:db8::1",
        "::ffff:127.0.0.1",
    ] {
        let parsed: IpAddr = address.parse().expect("fixture address should parse");
        assert!(!is_public_provider_ip(parsed), "{address} should reject");
    }
}

#[test]
#[cfg(feature = "provider-conformance")]
fn conformance_escape_hatch_is_exact_socket_only() {
    let endpoint = endpoint("provider.example.test", "/omarchygs/provider/v1/");
    let exact = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 4443);
    let wrong_port = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 4444);
    assert!(
        GuardedProviderClient::conformance_loopback(
            ProviderEndpoint {
                port: exact.port(),
                ..endpoint.clone()
            },
            exact,
            &[vec![0x30; 128]],
            quotas(),
        )
        .is_err(),
        "invalid DER must fail even for exact conformance loopback"
    );
    assert!(
        GuardedProviderClient::conformance_loopback(
            ProviderEndpoint {
                port: exact.port(),
                ..endpoint
            },
            wrong_port,
            &[vec![0x30; 128]],
            quotas(),
        )
        .is_err(),
        "a different loopback socket must never be admitted"
    );
}

fn endpoint(host: &str, base_path: &str) -> ProviderEndpoint {
    ProviderEndpoint {
        host: host.to_owned(),
        port: 443,
        base_path: base_path.to_owned(),
    }
}

#[cfg(feature = "provider-conformance")]
fn quotas() -> ProviderQuotas {
    ProviderQuotas {
        grants_per_minute: 10,
        requests_per_minute: 10,
        callbacks_per_minute: 10,
        max_concurrent_requests: 2,
        request_body_bytes: 4096,
        response_body_bytes: 4096,
        connect_timeout_ms: 250,
        total_timeout_ms: 500,
    }
}
