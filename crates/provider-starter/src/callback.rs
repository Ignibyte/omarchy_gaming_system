use std::{net::SocketAddr, sync::Arc, time::Duration};

use anyhow::{Context as _, Result as AnyResult, anyhow};
use omarchygs_provider_sdk::protocol::{
    HttpMessageSigner, ProviderEvent, RequestSignatureContext, parse_authenticated_json,
};
use sqlx::FromRow;
use tokio::task::JoinHandle;
use url::Url;
use uuid::Uuid;

use crate::{GameIdentity, store::StarterStore};

const MAX_CALLBACK_BYTES: usize = 65_536;
const MAX_CALLBACK_ATTEMPTS: i32 = 8;

/// One exact platform callback target. Socket override is test-only and is
/// rejected unless the crate is built with `conformance`.
#[derive(Debug, Clone)]
pub struct CallbackConfig {
    url: Url,
    root_der: Vec<u8>,
    #[cfg(feature = "conformance")]
    socket_override: Option<SocketAddr>,
}

impl CallbackConfig {
    /// Construct an exact HTTPS callback target for one release.
    pub fn new(
        url: Url,
        root_der: Vec<u8>,
        release_id: Uuid,
        socket_override: Option<SocketAddr>,
    ) -> AnyResult<Self> {
        if url.scheme() != "https"
            || url.username() != ""
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
            || url.path() != format!("/v1/provider-events/{release_id}")
            || url.host_str().is_none()
            || !(64..=4096).contains(&root_der.len())
        {
            return Err(anyhow!(
                "callback target must be one exact bounded HTTPS release URL"
            ));
        }
        if socket_override.is_some() && !cfg!(feature = "conformance") {
            return Err(anyhow!(
                "callback socket override requires conformance feature"
            ));
        }
        #[cfg(feature = "conformance")]
        if let Some(socket) = socket_override
            && (!socket.ip().is_loopback()
                || !matches!(url.host(), Some(url::Host::Domain(_)))
                || url.port_or_known_default() != Some(socket.port()))
        {
            return Err(anyhow!(
                "callback socket override must match one loopback DNS endpoint"
            ));
        }
        Ok(Self {
            url,
            root_der,
            #[cfg(feature = "conformance")]
            socket_override,
        })
    }
}

#[derive(FromRow)]
struct OutboxRow {
    event_id: Uuid,
    message_id: Uuid,
    body: Vec<u8>,
    attempt_count: i32,
}

pub(crate) fn spawn_callback_worker(
    store: StarterStore,
    signer: Arc<HttpMessageSigner>,
    identity: GameIdentity,
    release_id: Uuid,
    config: CallbackConfig,
) -> AnyResult<JoinHandle<()>> {
    let authority = config
        .url
        .host_str()
        .map(|host| match config.url.port() {
            Some(port) if port != 443 => format!("{host}:{port}"),
            _ => host.to_owned(),
        })
        .ok_or_else(|| anyhow!("callback target requires authority"))?;
    let certificate = reqwest::Certificate::from_der(&config.root_der)
        .context("callback TLS root must be DER")?;
    let builder = reqwest::Client::builder()
        .https_only(true)
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .tls_certs_only([certificate])
        .connect_timeout(Duration::from_secs(2))
        .timeout(Duration::from_secs(5));
    #[cfg(feature = "conformance")]
    let builder = if let Some(socket) = config.socket_override {
        builder.resolve(
            config
                .url
                .host_str()
                .ok_or_else(|| anyhow!("callback target requires host"))?,
            socket,
        )
    } else {
        builder
    };
    let client = builder.build().context("build callback client")?;
    let path = config.url.path().to_owned();
    Ok(tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(100));
        loop {
            interval.tick().await;
            if let Err(error) = deliver_one(
                &store,
                &signer,
                &identity,
                release_id,
                &client,
                &config.url,
                &authority,
                &path,
            )
            .await
            {
                tracing::warn!(error = %error, "provider starter callback delivery deferred");
            }
        }
    }))
}

#[allow(clippy::too_many_arguments)]
async fn deliver_one(
    store: &StarterStore,
    signer: &HttpMessageSigner,
    identity: &GameIdentity,
    release_id: Uuid,
    client: &reqwest::Client,
    callback_url: &Url,
    authority: &str,
    path: &str,
) -> AnyResult<()> {
    let row = sqlx::query_as::<_, OutboxRow>(
        r#"
        SELECT event_id, message_id, body, attempt_count
        FROM provider_starter_event_outbox
        WHERE status = 'pending' AND next_attempt_at <= clock_timestamp()
        ORDER BY next_attempt_at, created_at, event_id
        LIMIT 1
        "#,
    )
    .fetch_optional(store.pool())
    .await?;
    let Some(row) = row else {
        return Ok(());
    };
    let event: ProviderEvent = parse_authenticated_json(&row.body, MAX_CALLBACK_BYTES)
        .map_err(|_| anyhow!("persisted callback is invalid"))?;
    event
        .validate()
        .map_err(|_| anyhow!("persisted callback is invalid"))?;
    if event.provider_id != identity.provider_id
        || event.release_id != release_id
        || event.game_key != identity.game_key
        || event.rules_version != identity.rules_version
        || event.cartridge_digest != identity.cartridge_digest
        || event.event_id != row.event_id
        || event.message_id != row.message_id
    {
        return Err(anyhow!("persisted callback identity mismatch"));
    }
    let context = RequestSignatureContext {
        method: "POST",
        authority,
        path,
        provider_id: &identity.provider_id,
        release_id,
        message_id: row.message_id,
    };
    let headers = signer
        .sign_request(
            &context,
            &row.body,
            unix_seconds()?,
            &format!("starter-callback-{}", Uuid::new_v4()),
        )
        .map_err(|_| anyhow!("callback signing failed"))?;
    let status = match client
        .post(callback_url.clone())
        .headers(
            headers
                .to_header_map()
                .map_err(|_| anyhow!("callback headers failed"))?,
        )
        .body(row.body)
        .send()
        .await
    {
        Ok(response) => Some(response.status()),
        Err(error) => {
            tracing::warn!(error = %error, event_id = %row.event_id, "callback transport failed");
            None
        }
    };
    let delivered = status.is_some_and(|status| {
        status == reqwest::StatusCode::NO_CONTENT || status == reqwest::StatusCode::ACCEPTED
    });
    let next_attempt = row
        .attempt_count
        .checked_add(1)
        .ok_or_else(|| anyhow!("callback attempt overflow"))?;
    let failed = !delivered && next_attempt >= MAX_CALLBACK_ATTEMPTS;
    let exponent = u32::try_from(row.attempt_count.clamp(0, 7))?;
    let retry_ms = 250_i64
        .checked_mul(1_i64.checked_shl(exponent).unwrap_or(128))
        .unwrap_or(30_000)
        .min(30_000);
    let changed = sqlx::query(
        r#"
        UPDATE provider_starter_event_outbox
        SET attempt_count = $2,
            status = CASE WHEN $3 THEN 'delivered' WHEN $4 THEN 'failed' ELSE 'pending' END,
            delivered_at = CASE WHEN $3 THEN clock_timestamp() ELSE NULL END,
            next_attempt_at = CASE
                WHEN $3 OR $4 THEN next_attempt_at
                ELSE clock_timestamp() + ($5::bigint * interval '1 millisecond')
            END,
            updated_at = clock_timestamp()
        WHERE event_id = $1 AND status = 'pending' AND attempt_count = $6
        "#,
    )
    .bind(row.event_id)
    .bind(next_attempt)
    .bind(delivered)
    .bind(failed)
    .bind(retry_ms)
    .bind(row.attempt_count)
    .execute(store.pool())
    .await?;
    if changed.rows_affected() != 1 {
        return Err(anyhow!("callback attempt lost concurrency race"));
    }
    Ok(())
}

fn unix_seconds() -> AnyResult<i64> {
    let duration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .context("system clock before Unix epoch")?;
    i64::try_from(duration.as_secs()).context("system clock overflow")
}

#[cfg(all(test, feature = "conformance"))]
mod tests {
    use std::net::Ipv4Addr;

    use super::*;

    #[test]
    fn conformance_override_binds_domain_and_url_port() {
        let release_id = Uuid::new_v4();
        let url = Url::parse(&format!(
            "https://platform.test:4443/v1/provider-events/{release_id}"
        ))
        .expect("valid callback URL");
        let root = vec![1; 64];
        assert!(
            CallbackConfig::new(
                url.clone(),
                root.clone(),
                release_id,
                Some(SocketAddr::from((Ipv4Addr::LOCALHOST, 4443))),
            )
            .is_ok()
        );
        assert!(
            CallbackConfig::new(
                url,
                root.clone(),
                release_id,
                Some(SocketAddr::from((Ipv4Addr::LOCALHOST, 4444))),
            )
            .is_err()
        );
        assert!(
            CallbackConfig::new(
                Url::parse(&format!(
                    "https://127.0.0.1:4443/v1/provider-events/{release_id}"
                ))
                .expect("valid IP callback URL"),
                root,
                release_id,
                Some(SocketAddr::from((Ipv4Addr::LOCALHOST, 4443))),
            )
            .is_err()
        );
    }
}
