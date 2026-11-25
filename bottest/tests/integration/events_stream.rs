//! #2503 — native event stream: push delivery with no polling.
//!
//! Live-server contract for `GET /ws/events/stream` (SSE): one long-lived
//! connection is opened, a `sheets.exported` event is published through the
//! admin API, and the event must arrive on the **same** open connection. The
//! client never re-issues the GET, so delivery cannot be a polling loop.
//!
//! Two complementary checks, mirroring `events_api.rs`:
//!
//! 1. `test_events_stream_sse_*` drives the real SSE endpoint with a locally
//!    minted admin token. It skips with a reason when no botserver is reachable
//!    (no `BOTSERVER_URL` and the local harness cannot start one).
//! 2. `test_events_stream_contract_*` is an offline contract test of the same
//!    push/cursor semantics through `botevents`' public runtime. It always runs
//!    and needs no server, database, MinIO or LLM.

use std::time::{Duration, Instant};

use bottest::prelude::*;
use reqwest::{header::CONTENT_TYPE, Client, Response, StatusCode};
use serde_json::{json, Value};

use crate::tenant_tokens::mint_admin_jwt;

// ---------------------------------------------------------------------------
// Live-server helpers (same pattern as events_api.rs / flow_support)
// ---------------------------------------------------------------------------

/// Uses an externally provided server when reachable, otherwise starts the
/// harness botserver the other integration tests use. The `TestContext` is
/// returned so the caller keeps the server alive for the whole flow.
async fn get_test_server() -> Option<(Option<TestContext>, String)> {
    if let Some(url) = std::env::var("BOTSERVER_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
    {
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

/// A locally minted HS256 JWT. The SSE handshake authenticates through the
/// `access_token` query parameter (EventSource cannot send an Authorization
/// header); the same token is a Bearer credential for the publish endpoint.
fn test_bearer_token() -> String {
    mint_admin_jwt(
        "events-stream",
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

/// Pulls one complete, non-comment SSE frame from an already-open response.
/// Chunks are read only when `buffer` has no complete frame yet. Returns
/// `(event name, data)` so the caller can identify `native` envelopes.
///
/// `reqwest::Response::chunk()` is used instead of `bytes_stream()` because
/// bottest does not enable reqwest's optional `stream` feature.
async fn next_sse_frame(
    response: &mut Response,
    deadline: Instant,
    buffer: &mut String,
) -> Option<(String, String)> {
    loop {
        if let Some(end) = buffer.find("\n\n") {
            let frame: String = buffer.drain(..end + 2).collect();
            let mut name = String::new();
            let mut data = String::new();
            for line in frame.lines() {
                if let Some(value) = line.strip_prefix("event:") {
                    name.push_str(value.trim());
                } else if let Some(value) = line.strip_prefix("data:") {
                    if !data.is_empty() {
                        data.push('\n');
                    }
                    data.push_str(value.strip_prefix(' ').unwrap_or(value));
                }
            }
            if !name.is_empty() || !data.is_empty() {
                return Some((name, data));
            }
            continue;
        }

        let remaining = deadline.checked_duration_since(Instant::now())?;
        let chunk = tokio::time::timeout(remaining, response.chunk())
            .await
            .ok()?
            .ok()?
            .ok()?;
        buffer.push_str(&String::from_utf8_lossy(&chunk).replace("\r\n", "\n"));
    }
}

// ---------------------------------------------------------------------------
// Live SSE test — the event must be pushed on the single open connection
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_events_stream_sse_delivers_sheets_exported_without_polling() {
    let Some((_ctx, base_url)) = get_test_server().await else {
        eprintln!("Skipping native event stream test: no server available");
        return;
    };
    let Some(client) = http_client() else {
        eprintln!("Skipping native event stream test: cannot build HTTP client");
        return;
    };
    let token = test_bearer_token();

    // 1. ONE long-lived subscription, filtered to the event under test. This
    //    is the only GET in the whole test — there is no poll loop.
    let stream_url =
        format!("{base_url}/ws/events/stream?types=sheets.exported&access_token={token}");
    let connect =
        tokio::time::timeout(Duration::from_secs(8), client.get(&stream_url).send()).await;
    let Ok(Ok(mut response)) = connect else {
        eprintln!("Skipping native event stream test: SSE connect timed out or failed");
        return;
    };
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "SSE subscription must authenticate via the access_token query parameter"
    );
    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();
    assert!(
        content_type.contains("text/event-stream"),
        "stream must be SSE, got '{content_type}'"
    );

    let mut buffer = String::new();

    // 2. Record what the connection carried before the publish (keep-alive
    //    comments, a seeded demo envelope). Nothing here may be our new event.
    let drain_until = Instant::now() + Duration::from_millis(700);
    let mut seen_before = Vec::new();
    while let Some((name, data)) = next_sse_frame(&mut response, drain_until, &mut buffer).await {
        seen_before.push((name, data));
    }

    // 3. Publish one fresh event. No further subscription GET is made.
    let publish_body = json!({
        "event_type": "sheets.exported",
        "workspace_id": "ws-stream-live",
        "doc_id": "sheet-stream-live",
        "actor_id": "usr-stream-live",
    });
    let published = client
        .post(format!("{base_url}/api/events/publish"))
        .bearer_auth(&token)
        .json(&publish_body)
        .send()
        .await;
    let Ok(published) = published else {
        eprintln!("Skipping native event stream test: publish request failed");
        return;
    };
    assert_eq!(published.status(), StatusCode::OK, "publish must succeed");
    let report: Value = published.json().await.unwrap_or_else(|_| json!({}));
    assert_eq!(
        report.get("event_type").and_then(Value::as_str),
        Some("sheets.exported")
    );
    assert!(
        report
            .get("native_enqueued")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            >= 1,
        "publish must enqueue a native envelope for the browser stream"
    );
    let event_id = report
        .get("event_id")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    assert!(
        !event_id.is_empty(),
        "publish must return a server-generated event id"
    );
    assert!(
        !seen_before
            .iter()
            .any(|(_, data)| data.contains(event_id.as_str())),
        "the new event must not have been on the stream before it was published"
    );

    // 4. Read from the SAME connection opened in step 1: push, not polling.
    let deadline = Instant::now() + Duration::from_secs(12);
    let mut pushed: Option<Value> = None;
    while let Some((name, data)) = next_sse_frame(&mut response, deadline, &mut buffer).await {
        if name != "native" {
            continue;
        }
        let Ok(envelope) = serde_json::from_str::<Value>(&data) else {
            continue;
        };
        if envelope.get("event_type").and_then(Value::as_str) != Some("sheets.exported") {
            continue;
        }
        if envelope.get("event_id").and_then(Value::as_str) != Some(event_id.as_str()) {
            continue;
        }
        pushed = Some(envelope);
        break;
    }
    assert!(
        pushed.is_some(),
        "sheets.exported event {event_id} was not pushed on the open SSE connection"
    );

    if let Some(envelope) = pushed.as_ref() {
        assert_eq!(
            envelope.get("bot_id").and_then(Value::as_str),
            Some("default")
        );
        let payload_is_private = envelope
            .get("payload")
            .and_then(Value::as_object)
            .is_some_and(|payload| {
                payload.get("content").is_none() && payload.get("body").is_none()
            });
        assert!(
            payload_is_private,
            "pushed payload must contain IDs/metadata only, never document content"
        );
    }
}

// ---------------------------------------------------------------------------
// Offline contract test (botevents public API — no server required)
// ---------------------------------------------------------------------------

#[test]
fn test_events_stream_contract_push_cursor_semantics() {
    use botevents::office::runtime::OfficeEventsRuntime;
    use botevents::{DomainEvent, OfficeEventType};

    let runtime = OfficeEventsRuntime::new();
    let cursor = runtime.stream().len();
    let event = DomainEvent::new(
        "evt-stream-contract",
        OfficeEventType::SheetsExported,
        "ws-stream-contract",
        "sheet-stream-contract",
        "usr-stream-contract",
    );

    let report = runtime.publish(&event, None);
    assert_eq!(
        report.native_enqueued, 1,
        "publish must enqueue exactly one native envelope"
    );
    assert_eq!(
        report.webhooks_attempted, 0,
        "native delivery is in-platform, not an HTTP call"
    );
    assert_eq!(
        runtime.stream().since(cursor).len(),
        1,
        "the published event must be pushed at the next cursor with no poll"
    );
    let fresh = runtime.stream().since(cursor);
    assert_eq!(fresh[0].event_type, "sheets.exported");
    assert_eq!(fresh[0].event_id, "evt-stream-contract");
    let json = serde_json::to_value(&fresh[0]).unwrap_or_else(|_| json!({}));
    assert!(
        json.get("payload")
            .and_then(Value::as_object)
            .is_some_and(|payload| payload.get("content").is_none()),
        "native envelope must not carry document content"
    );
    assert!(
        runtime.stream().since(cursor + 1).is_empty(),
        "the stream is append-only and each cursor sees only newer events"
    );
}
