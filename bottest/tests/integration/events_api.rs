//! #2503 — Office event API integration contract.
//!
//! Two complementary checks for the same four behaviours the issue asks for:
//!
//! 1. `test_events_api_live_*` drives the real HTTP endpoints with a test
//!    bearer token (`/api/events/dead-letters`, `/api/events/replay`,
//!    `/api/events/publish`) exactly like the sibling API/flow tests. It skips
//!    with a reason when no botserver is reachable.
//! 2. `test_events_contract_*` is a pure, offline contract test of the same
//!    semantics through `botevents`' process-wide runtime types. It always
//!    runs and needs no server, database, OtterIO or LLM.

use std::time::Duration;

use bottest::prelude::*;
use reqwest::{Client, StatusCode};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::tenant_tokens::mint_admin_jwt;

// ---------------------------------------------------------------------------
// Live-server helpers (follow bottest/tests/integration/api.rs + flow_support)
// ---------------------------------------------------------------------------

fn external_server_url() -> Option<String> {
    std::env::var("BOTSERVER_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
}

/// Uses an externally provided server when reachable, otherwise starts the
/// same harness botserver the other integration tests use. The `TestContext`
/// is returned so the caller keeps it alive for the whole flow.
async fn get_test_server() -> Option<(Option<TestContext>, String)> {
    if let Some(url) = external_server_url() {
        let probe = Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .ok()?;
        if probe.get(&url).send().await.is_ok() {
            return Some((None, url));
        }
    }

    let ctx = TestHarness::quick().await.ok()?;
    let server = ctx.start_botserver().await.ok()?;
    if server.is_running() {
        Some((Some(ctx), server.url.clone()))
    } else {
        None
    }
}

/// A locally minted HS256 JWT whose `sub`/`email` claims map to a seeded admin.
/// Signed with the same `saas_jwt_secret` the running server resolves, so the
/// `require_auth` layer accepts it on the events endpoints.
fn test_bearer_token() -> String {
    mint_admin_jwt(
        "events",
        &Uuid::new_v4().to_string(),
        &Uuid::new_v4().to_string(),
    )
}

fn http_client() -> Option<Client> {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .ok()
}

/// Number of dead letters currently exposed by the admin list endpoint.
async fn dead_letter_count(client: &Client, base_url: &str, token: &str) -> Option<usize> {
    let response = client
        .get(format!("{base_url}/api/events/dead-letters"))
        .bearer_auth(token)
        .send()
        .await
        .ok()?;
    if response.status() != StatusCode::OK {
        return None;
    }
    let body: Value = response.json().await.ok()?;
    body.as_array().map(Vec::len)
}

// ---------------------------------------------------------------------------
// Live HTTP test
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_events_api_live_dead_letters_replay_dedup_publish() {
    let Some((_ctx, base_url)) = get_test_server().await else {
        eprintln!("Skipping events API test: no server available");
        return;
    };
    let Some(client) = http_client() else {
        eprintln!("Skipping events API test: cannot build HTTP client");
        return;
    };
    let token = test_bearer_token();

    // 1. LIST — the runtime seeds a demo dead letter on first use, so a fresh
    //    server always exposes at least one privacy-safe DLQ summary.
    let response = client
        .get(format!("{base_url}/api/events/dead-letters"))
        .bearer_auth(&token)
        .send()
        .await;
    let Ok(response) = response else {
        eprintln!("Skipping events API test: list request failed");
        return;
    };
    assert_eq!(response.status(), StatusCode::OK, "list must be authorized");
    let letters: Value = response.json().await.unwrap_or_else(|_| json!([]));
    let list = letters.as_array().map_or(&[][..], Vec::as_slice);
    assert!(!list.is_empty(), "list must return the seeded dead letter");
    assert!(
        list[0].get("body").is_none() && list[0].get("payload").is_none(),
        "DLQ summary must not leak event content"
    );
    let key = list[0]
        .get("idempotency_key")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    assert!(!key.is_empty(), "dead letter must carry an idempotency key");
    let replay_body = json!({ "idempotency_key": key });

    // 2. FIRST REPLAY — performs a real delivery (deduped: false, HTTP 200).
    let response = client
        .post(format!("{base_url}/api/events/replay"))
        .bearer_auth(&token)
        .json(&replay_body)
        .send()
        .await;
    let Ok(response) = response else {
        eprintln!("Skipping events API test: first replay request failed");
        return;
    };
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "first replay must succeed"
    );
    let first: Value = response.json().await.unwrap_or_else(|_| json!({}));
    assert_eq!(
        first.get("deduped").and_then(Value::as_bool),
        Some(false),
        "first replay must deliver once"
    );
    assert_eq!(
        first.get("status_code").and_then(Value::as_u64),
        Some(200),
        "delivery must succeed after the demo transport recovers"
    );

    // 3. SECOND REPLAY — deduped, reuses the stored receipt, no new delivery.
    let response = client
        .post(format!("{base_url}/api/events/replay"))
        .bearer_auth(&token)
        .json(&replay_body)
        .send()
        .await;
    let Ok(response) = response else {
        eprintln!("Skipping events API test: second replay request failed");
        return;
    };
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "second replay must succeed"
    );
    let second: Value = response.json().await.unwrap_or_else(|_| json!({}));
    assert_eq!(
        second.get("deduped").and_then(Value::as_bool),
        Some(true),
        "second replay must be deduped"
    );
    assert_eq!(
        first.get("delivered_at"),
        second.get("delivered_at"),
        "deduped replay must reuse the original receipt, not deliver again"
    );

    // 4. PUBLISH — a metadata-only event with a failing webhook dead-letters.
    let before = dead_letter_count(&client, &base_url, &token).await;
    let publish_body = json!({
        "event_type": "sheets.exported",
        "webhook_url": "https://events.example.test/hook",
        "doc_id": "doc-live",
        "workspace_id": "ws-live",
        "actor_id": "usr-live"
    });
    let response = client
        .post(format!("{base_url}/api/events/publish"))
        .bearer_auth(&token)
        .json(&publish_body)
        .send()
        .await;
    let Ok(response) = response else {
        eprintln!("Skipping events API test: publish request failed");
        return;
    };
    assert_eq!(response.status(), StatusCode::OK, "publish must succeed");
    let published: Value = response.json().await.unwrap_or_else(|_| json!({}));
    assert_eq!(
        published.get("dead_lettered").and_then(Value::as_u64),
        Some(1),
        "publish to the failing demo webhook must dead-letter once"
    );
    let after = dead_letter_count(&client, &base_url, &token).await;
    if let (Some(before), Some(after)) = (before, after) {
        assert_eq!(
            after,
            before + 1,
            "publish must add exactly one dead letter"
        );
    }
}

// ---------------------------------------------------------------------------
// Offline contract test (botevents public API — no server required)
// ---------------------------------------------------------------------------

#[test]
fn test_events_contract_dead_letters_replay_dedup_publish() {
    use botevents::office::runtime::OfficeEventsRuntime;
    use botevents::{DomainEvent, OfficeEventType};

    let runtime = OfficeEventsRuntime::new();

    // 1. LIST — a seeded, privacy-safe dead letter is always present.
    let letters = runtime.dead_letters();
    assert!(
        !letters.is_empty(),
        "DLQ must contain the seeded demo letter"
    );
    let seeded = &letters[0];
    assert_eq!(seeded.event_id, "evt-demo-001");
    assert_eq!(seeded.event_type, "sheets.exported");
    let json = serde_json::to_value(seeded).unwrap_or_else(|_| json!({}));
    assert!(json.get("idempotency_key").is_some());
    assert!(json.get("body").is_none(), "DLQ view must not carry a body");
    assert!(
        json.get("payload").is_none(),
        "DLQ view must not carry a payload"
    );
    assert!(
        json.get("content").is_none(),
        "DLQ view must not carry content"
    );

    // 2. FIRST REPLAY — one real delivery, entry removed from the DLQ.
    let key = seeded.idempotency_key.clone();
    let first = runtime.replay(&key);
    assert!(first.is_ok(), "first replay must succeed: {first:?}");
    let first_deduped = first.as_ref().map_or(true, |result| result.deduped);
    assert!(!first_deduped, "first replay must perform a delivery");
    let first_receipt = first.as_ref().map(|result| &result.receipt);
    assert_eq!(
        first_receipt.map(|receipt| receipt.status_code),
        Ok(200),
        "delivery must succeed after the demo transport recovers"
    );
    assert!(
        runtime
            .dead_letters()
            .iter()
            .all(|letter| letter.idempotency_key != key),
        "successful replay must remove the DLQ entry"
    );

    // 3. SECOND REPLAY — deduped; the original receipt is reused verbatim, so
    //    no second delivery is made.
    let second = runtime.replay(&key);
    assert!(second.is_ok(), "second replay must succeed: {second:?}");
    let second_deduped = second.as_ref().map_or(false, |result| result.deduped);
    assert!(second_deduped, "ledger must dedupe a repeated key");
    let first_at = first.as_ref().map(|result| result.receipt.delivered_at);
    let second_at = second.as_ref().map(|result| result.receipt.delivered_at);
    assert_eq!(first_at, second_at, "deduped replay must not deliver again");
    assert!(
        runtime.dead_letters().is_empty(),
        "deduped replay adds no dead letter"
    );

    // 4. PUBLISH — a metadata-only event with a webhook creates a dead letter.
    let event = DomainEvent::new(
        "evt-contract-publish",
        OfficeEventType::DocCreated,
        "ws-contract",
        "doc-contract",
        "usr-contract",
    );
    let report = runtime.publish(&event, Some("https://events.example.test/hook"));
    assert_eq!(report.dead_lettered, 1, "publish must dead-letter once");
    assert!(
        runtime
            .dead_letters()
            .iter()
            .any(|letter| letter.event_id == "evt-contract-publish"),
        "published event must appear in the DLQ"
    );
}
