//! Security-backed redaction and the process-wide event pipeline.

use std::sync::{Arc, OnceLock};

use botevents::{DefaultRedactor, EventPipeline, FrameRedactor};
use serde_json::Value;

/// Security-backed redactor: known credential values from the environment,
/// then the security crate's masking rules, applied to every string in the
/// payload.
struct SecurityRedactor {
    known: DefaultRedactor,
}

impl SecurityRedactor {
    fn new() -> Self {
        Self {
            known: DefaultRedactor::from_env(),
        }
    }
}

impl FrameRedactor for SecurityRedactor {
    fn redact(&self, value: &mut Value) {
        self.known.redact(value);
        scrub_security(value);
    }
}

fn scrub_security(value: &mut Value) {
    match value {
        Value::String(text) => {
            *text = botsecurity_crypto::secrets::redact_sensitive_data(text);
        }
        Value::Array(items) => {
            for item in items {
                scrub_security(item);
            }
        }
        Value::Object(map) => {
            for item in map.values_mut() {
                scrub_security(item);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
}

pub(super) fn pipeline() -> &'static Arc<EventPipeline> {
    static PIPELINE: OnceLock<Arc<EventPipeline>> = OnceLock::new();
    PIPELINE.get_or_init(|| {
        Arc::new(EventPipeline::new(Arc::new(SecurityRedactor::new())))
    })
}

/// Keeps the in-process sequencer ahead of a sequence number persisted out of
/// band (takeover events write directly to `run_events`). Without this the
/// next minted event reuses the same `(thread_id, seq)` and the unique
/// constraint drops it, silently losing timeline frames.
pub fn observe_persisted_seq(thread_id: &str, seq: u64) {
    pipeline().observe_seq(thread_id, seq);
}

/// Runtime flag. Compiling the contract is unconditional; emitting is opt-in
/// so an existing deployment is unchanged until `GB_RUN_EVENTS` is set.
#[must_use]
pub fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        // The desktop edition ships the conversation workspace: run cards,
        // structured tools and generative UI are product features there, not
        // an opt-in. Server deployments keep the opt-in default so an upgrade
        // does not change their traffic shape (#1662).
        match std::env::var("GB_RUN_EVENTS").ok().as_deref() {
            Some("1") | Some("true") | Some("yes") => true,
            Some("0") | Some("false") | Some("no") => false,
            _ => cfg!(feature = "desktop-local"),
        }
    })
}
