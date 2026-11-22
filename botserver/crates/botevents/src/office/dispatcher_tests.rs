use std::sync::{Arc, Mutex};

use chrono::Utc;
use serde_json::json;

use super::{
    DispatchReport, NativeSubscription, ReplayError, WebhookDispatcher,
};
use crate::office::delivery::DeliveryTransport;
use crate::office::event::{AllowAllContent, DenyAllContent, DomainEvent, EventContent};
use crate::office::retry::RetryPolicy;
use crate::office::signing::{self, DEFAULT_TOLERANCE_SECS, SIGNATURE_HEADER};
use crate::office::subscription::WebhookSubscription;
use crate::office::taxonomy::OfficeEventType;

struct RecordedCall {
    url: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

#[derive(Clone, Copy)]
enum ResponseMode {
    AlwaysOk,
    AlwaysFail(u16),
    FailThenOk(u32),
}

impl Default for ResponseMode {
    fn default() -> Self {
        Self::AlwaysOk
    }
}

#[derive(Default)]
struct RecordingTransport {
    calls: Mutex<Vec<RecordedCall>>,
    mode: Mutex<ResponseMode>,
}

impl RecordingTransport {
    fn set_mode(&self, mode: ResponseMode) {
        *self.mode.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = mode;
    }

    fn call_count(&self) -> usize {
        self.calls
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len()
    }

    fn last_call_body(&self) -> Vec<u8> {
        self.calls
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .last()
            .map(|call| call.body.clone())
            .unwrap_or_default()
    }
}

impl DeliveryTransport for RecordingTransport {
    fn deliver(&self, url: &str, headers: &[(String, String)], body: &[u8]) -> crate::office::delivery::DeliveryOutcome {
        use crate::office::delivery::DeliveryOutcome;
        let mut calls = self.calls.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let index = calls.len() as u32;
        calls.push(RecordedCall {
            url: url.to_string(),
            headers: headers.to_vec(),
            body: body.to_vec(),
        });
        let mode = *self.mode.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        match mode {
            ResponseMode::AlwaysOk => DeliveryOutcome {
                status_code: 200,
                latency_ms: 5,
                error: None,
            },
            ResponseMode::AlwaysFail(status) => DeliveryOutcome {
                status_code: status,
                latency_ms: 7,
                error: Some("upstream error".to_string()),
            },
            ResponseMode::FailThenOk(failures) => {
                if index < failures {
                    DeliveryOutcome {
                        status_code: 503,
                        latency_ms: 7,
                        error: Some("temporarily unavailable".to_string()),
                    }
                } else {
                    DeliveryOutcome {
                        status_code: 200,
                        latency_ms: 5,
                        error: None,
                    }
                }
            }
        }
    }
}

fn subscription() -> WebhookSubscription {
    WebhookSubscription::new(
        "sub-1",
        "https://hooks.example.test/gb",
        "sec-1",
        "signing-secret-value",
    )
}

fn comment_event() -> DomainEvent {
    DomainEvent::new(
        "evt-1",
        OfficeEventType::DocCommentAdded,
        "ws-1",
        "doc-1",
        "usr-1",
    )
    .with_tag("finance")
    .with_metadata("comment_id", json!("cmt-1"))
}

#[test]
fn doc_comment_added_delivers_a_signed_payload_with_the_right_ids() {
    let transport = Arc::new(RecordingTransport::default());
    let dispatcher = WebhookDispatcher::new(transport.clone(), Arc::new(DenyAllContent));
    let subscription = subscription();
    dispatcher.add_subscription(subscription.clone());

    let report = dispatcher.publish(&comment_event());
    assert_eq!(
        report,
        DispatchReport {
            webhooks_attempted: 1,
            delivered: 1,
            dead_lettered: 0,
            native_enqueued: 0,
        }
    );

    let calls = transport.calls.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].url, "https://hooks.example.test/gb");
    let header = calls[0]
        .headers
        .iter()
        .find(|(name, _)| name == SIGNATURE_HEADER)
        .map(|(_, value)| value.clone())
        .unwrap_or_default();
    assert!(signing::verify(
        &subscription,
        &header,
        &calls[0].body,
        Utc::now().timestamp(),
        DEFAULT_TOLERANCE_SECS
    ));
    let body = String::from_utf8(calls[0].body.clone()).unwrap_or_default();
    assert!(body.contains("evt-1"));
    assert!(body.contains("doc-1"));
    assert!(body.contains("usr-1"));
    assert!(body.contains("doc.comment.added"));
}

#[test]
fn a_5xx_endpoint_is_retried_then_dead_lettered() {
    let transport = Arc::new(RecordingTransport::default());
    transport.set_mode(ResponseMode::AlwaysFail(503));
    let dispatcher = WebhookDispatcher::new(transport.clone(), Arc::new(DenyAllContent))
        .with_retry(RetryPolicy::new(3, 100, 1_000, 2));
    dispatcher.add_subscription(subscription());

    let report = dispatcher.publish(&comment_event());
    assert_eq!(report.delivered, 0);
    assert_eq!(report.dead_lettered, 1);
    assert_eq!(transport.call_count(), 3);

    let records = dispatcher.log().records();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].attempt_count(), 3);
    assert_eq!(records[0].attempts[0].delay_before_ms, 0);
    assert_eq!(records[0].attempts[1].delay_before_ms, 100);
    assert_eq!(records[0].attempts[2].delay_before_ms, 200);
    assert_eq!(dispatcher.dlq().len(), 1);
    assert!(dispatcher.log().should_alert(1));
}

#[test]
fn replay_succeeds_exactly_once_per_idempotency_key() {
    let transport = Arc::new(RecordingTransport::default());
    transport.set_mode(ResponseMode::FailThenOk(3));
    let dispatcher = WebhookDispatcher::new(transport.clone(), Arc::new(DenyAllContent))
        .with_retry(RetryPolicy::new(3, 100, 1_000, 2));
    dispatcher.add_subscription(subscription());

    let report = dispatcher.publish(&comment_event());
    assert_eq!(report.dead_lettered, 1);
    let key = "evt-1:sub-1";
    assert!(dispatcher.dlq().get(key).is_some());

    transport.set_mode(ResponseMode::AlwaysOk);
    let first = dispatcher.replay(key);
    assert!(matches!(first, Ok(_)));
    let calls_after_replay = transport.call_count();
    assert_eq!(calls_after_replay, 4);

    let second = dispatcher.replay(key);
    assert!(matches!(second, Ok(_)));
    assert_eq!(transport.call_count(), calls_after_replay);
    assert_eq!(dispatcher.dlq().len(), 0);
    assert_eq!(
        dispatcher.replay("missing:key"),
        Err(ReplayError::NotFound)
    );
}

#[test]
fn payloads_hide_content_unless_enabled_and_authorised() {
    let content = EventContent::new("Confidential", "secret body text");
    let event = comment_event().with_content(content);

    let denied_transport = Arc::new(RecordingTransport::default());
    let denied = WebhookDispatcher::new(denied_transport.clone(), Arc::new(DenyAllContent));
    denied.add_subscription(subscription().with_content(true));
    let report = denied.publish(&event);
    assert_eq!(report.delivered, 1);
    let body = String::from_utf8(denied_transport.last_call_body()).unwrap_or_default();
    assert!(!body.contains("secret body text"));
    assert!(!body.contains("content"));

    let allowed_transport = Arc::new(RecordingTransport::default());
    let allowed = WebhookDispatcher::new(allowed_transport.clone(), Arc::new(AllowAllContent));
    allowed.add_subscription(subscription().with_content(true));
    let report = allowed.publish(&event);
    assert_eq!(report.delivered, 1);
    let body = String::from_utf8(allowed_transport.last_call_body()).unwrap_or_default();
    assert!(body.contains("secret body text"));
}

#[test]
fn a_native_automation_reacts_to_sheet_export_without_http() {
    let transport = Arc::new(RecordingTransport::default());
    let dispatcher = WebhookDispatcher::new(transport.clone(), Arc::new(DenyAllContent));
    dispatcher.native().subscribe(
        NativeSubscription::new("native-1", "bot-1")
            .with_event_types(vec![OfficeEventType::SheetsExported]),
    );

    let event = DomainEvent::new(
        "evt-2",
        OfficeEventType::SheetsExported,
        "ws-1",
        "sheet-1",
        "usr-1",
    );
    let report = dispatcher.publish(&event);
    assert_eq!(report.native_enqueued, 1);
    assert_eq!(transport.call_count(), 0);

    let queue = dispatcher.native().drain("bot-1");
    assert_eq!(queue.len(), 1);
    assert_eq!(queue[0].event_type, "sheets.exported");
    assert_eq!(queue[0].event_id, "evt-2");
    assert_eq!(dispatcher.native().pending(), 0);
}

#[test]
fn subscriptions_only_receive_events_matching_their_filters() {
    let transport = Arc::new(RecordingTransport::default());
    let dispatcher = WebhookDispatcher::new(transport.clone(), Arc::new(DenyAllContent));
    dispatcher.add_subscription(subscription().with_apps(vec!["sheets"]));

    let report = dispatcher.publish(&comment_event());
    assert_eq!(report.webhooks_attempted, 0);
    assert_eq!(transport.call_count(), 0);
}
