use std::{sync::Arc, time::SystemTime};

use axum::{
    body::Body,
    http::{Request, StatusCode, header::HOST},
};
use http_body_util::BodyExt as _;
use omarchygs_provider_sdk::{
    ProviderError, ProviderScope, Result,
    protocol::{
        GrantIssuer, HttpMessageSigner, ProviderCompatibility, ProviderGrantClaims,
        ProviderOperationKind, ProviderOperationRequest, ProviderSessionStatus,
        RequestSignatureContext,
    },
};
use omarchygs_provider_starter::{
    CallbackConfig, GameIdentity, GameState, GameTransition, ProviderGame, ProviderStarter,
    ProviderStarterConfig, StarterLimits,
};
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt as _;
use url::Url;
use uuid::Uuid;

const PROVIDER: &str = "starter-test";
const GAME: &str = "counter-game";
const AUTHORITY: &str = "provider.example.test";
const RELEASE: Uuid = Uuid::from_u128(0x45454545454545454545454545454545);

#[derive(Clone)]
struct CounterGame {
    identity: GameIdentity,
}

impl CounterGame {
    fn new() -> Self {
        Self {
            identity: GameIdentity {
                provider_id: PROVIDER.to_owned(),
                game_key: GAME.to_owned(),
                rules_version: 1,
                cartridge_digest: "d".repeat(64),
            },
        }
    }
}

impl ProviderGame for CounterGame {
    fn identity(&self) -> &GameIdentity {
        &self.identity
    }

    fn launch(&self, payload: &Value) -> Result<GameState> {
        if payload != &json!({"start": 0}) {
            return Err(ProviderError::InvalidInput);
        }
        Ok(GameState {
            status: ProviderSessionStatus::Active,
            state: json!({"count": 0}),
        })
    }

    fn command(&self, current: &GameState, payload: &Value) -> Result<GameTransition> {
        if payload != &json!({"increment": 1}) {
            return Err(ProviderError::InvalidInput);
        }
        let count = current.state["count"]
            .as_u64()
            .ok_or(ProviderError::Internal)?;
        Ok(GameTransition {
            status: ProviderSessionStatus::Active,
            state: json!({"count": count + 1}),
        })
    }

    fn view(&self, current: &GameState) -> Result<Value> {
        Ok(current.state.clone())
    }

    fn event(&self, _current: &GameState) -> Result<Option<omarchygs_provider_starter::GameEvent>> {
        Ok(None)
    }
}

struct TestAuthority {
    grants: GrantIssuer,
    messages: HttpMessageSigner,
}

impl TestAuthority {
    fn new() -> Self {
        Self {
            grants: GrantIssuer::new("platform-grant-1", [11; 32], vec![12; 32])
                .expect("grant issuer"),
            messages: HttpMessageSigner::new("platform-message-1", [13; 32])
                .expect("message signer"),
        }
    }
}

#[sqlx::test]
#[ignore = "requires PostgreSQL; run scripts/test-provider-conformance.sh"]
async fn durable_whole_operation_receipts_survive_restart(pool: PgPool) {
    let authority = TestAuthority::new();
    let game = CounterGame::new();
    let starter = Arc::new(
        ProviderStarter::from_pool(game.clone(), starter_config(&authority), pool.clone())
            .await
            .expect("starter"),
    );
    let session_id = Uuid::new_v4();
    let idempotency_key = Uuid::new_v4();
    let launch = operation(
        &authority,
        session_id,
        idempotency_key,
        0,
        ProviderOperationKind::Launch,
        json!({"start": 0}),
    );
    let (status, first) = send(&starter, &launch).await;
    assert_eq!(status, StatusCode::OK);
    let (status, replay) = send(&starter, &launch).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(first, replay);

    let changed = operation(
        &authority,
        session_id,
        idempotency_key,
        0,
        ProviderOperationKind::Launch,
        json!({"start": 1}),
    );
    assert_eq!(send(&starter, &changed).await.0, StatusCode::CONFLICT);

    drop(starter);
    let restarted = Arc::new(
        ProviderStarter::from_pool(game.clone(), starter_config(&authority), pool.clone())
            .await
            .expect("restarted starter"),
    );
    let (status, after_restart) = send(&restarted, &launch).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(first, after_restart);

    let stale = operation(
        &authority,
        session_id,
        Uuid::new_v4(),
        4,
        ProviderOperationKind::Command,
        json!({"increment": 1}),
    );
    let (status, stale_body) = send(&restarted, &stale).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        String::from_utf8(stale_body)
            .expect("response utf8")
            .contains("revision_conflict")
    );

    let sessions: i64 = sqlx::query_scalar("SELECT count(*) FROM provider_starter_sessions")
        .fetch_one(&pool)
        .await
        .expect("session count");
    let receipts: i64 =
        sqlx::query_scalar("SELECT count(*) FROM provider_starter_operation_receipts")
            .fetch_one(&pool)
            .await
            .expect("receipt count");
    assert_eq!(sessions, 1);
    assert_eq!(receipts, 2);
}

fn starter_config(authority: &TestAuthority) -> ProviderStarterConfig {
    ProviderStarterConfig::new(
        RELEASE,
        AUTHORITY.to_owned(),
        "platform-grant-1".to_owned(),
        authority.grants.verifying_key(),
        "platform-message-1".to_owned(),
        authority.messages.verifying_key(),
        HttpMessageSigner::new("provider-message-1", [14; 32]).expect("provider signer"),
        CallbackConfig::new(
            Url::parse(&format!(
                "https://callback.example.test/v1/provider-events/{RELEASE}"
            ))
            .expect("callback URL"),
            vec![1; 64],
            RELEASE,
            None,
        )
        .expect("callback config"),
        StarterLimits::default(),
    )
    .expect("starter config")
}

fn operation(
    authority: &TestAuthority,
    session_id: Uuid,
    idempotency_key: Uuid,
    expected_revision: u64,
    kind: ProviderOperationKind,
    payload: Value,
) -> (String, axum::http::HeaderMap, Vec<u8>) {
    let now = i64::try_from(
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("clock")
            .as_secs(),
    )
    .expect("time");
    let message_id = Uuid::new_v4();
    let subject = "A".repeat(43);
    let claims = ProviderGrantClaims::new(
        PROVIDER.to_owned(),
        RELEASE,
        GAME.to_owned(),
        1,
        "d".repeat(64),
        session_id,
        subject.clone(),
        scope(kind),
        ProviderCompatibility::current(),
        now,
        now + 60,
        Uuid::new_v4(),
    )
    .expect("claims");
    let request = ProviderOperationRequest::new(
        PROVIDER.to_owned(),
        RELEASE,
        GAME.to_owned(),
        1,
        "d".repeat(64),
        session_id,
        subject,
        message_id,
        idempotency_key,
        expected_revision,
        kind,
        ProviderCompatibility::current(),
        payload,
        authority.grants.sign(&claims).expect("grant"),
    )
    .expect("request");
    let body = request.to_bytes(65_536).expect("body");
    let path = format!("/omarchygs/provider/v1/{}", kind.path());
    let context = RequestSignatureContext {
        method: "POST",
        authority: AUTHORITY,
        path: &path,
        provider_id: PROVIDER,
        release_id: RELEASE,
        message_id,
    };
    let mut headers = authority
        .messages
        .sign_request(&context, &body, now, &format!("starter-test-{message_id}"))
        .expect("signature")
        .to_header_map()
        .expect("headers");
    headers.insert(HOST, AUTHORITY.parse().expect("host"));
    (path, headers, body)
}

fn scope(kind: ProviderOperationKind) -> ProviderScope {
    kind.scope()
}

async fn send(
    starter: &Arc<ProviderStarter<CounterGame>>,
    operation: &(String, axum::http::HeaderMap, Vec<u8>),
) -> (StatusCode, Vec<u8>) {
    let request = Request::builder()
        .method("POST")
        .uri(&operation.0)
        .body(Body::from(operation.2.clone()))
        .expect("HTTP request");
    let (mut parts, body) = request.into_parts();
    parts.headers = operation.1.clone();
    let response = starter
        .router()
        .oneshot(Request::from_parts(parts, body))
        .await
        .expect("router");
    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes()
        .to_vec();
    (status, bytes)
}
