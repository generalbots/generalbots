//! Process-wide runtime for the Office event API (#2503).
//!
//! The rest of [`crate::office`] is transport-independent and store-agnostic.
//! This module is the small adapter the HTTP/WebSocket surface mounts: it owns
//! one `WebhookDispatcher` (retries, dead letters, idempotency ledger and
//! native subscriptions), an append-only stream of native envelopes for
//! browsers, and a deterministic in-process transport. Nothing here performs
//! network I/O — the transport models a 5xx endpoint so the replay flow is
//! reproducible without an external service.
//!
//! A single runtime is shared by every request through [`runtime()`]. Its
//! dead-letter queues and idempotency ledgers are held in per-tenant
//! dispatchers keyed by the caller's verified org/branch, so no caller can
//! list or replay another tenant's signed delivery. The event/streaming
//! handlers in `botserver` only read from it.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::{json, Value};

use super::delivery::{DeadLetter, DeliveryOutcome, DeliveryReceipt, DeliveryTransport};
use super::dispatcher::{DispatchReport, NativeEnvelope, ReplayError, WebhookDispatcher};
use super::event::{DenyAllContent, DomainEvent};
use super::retry::RetryPolicy;
use super::subscription::{NativeSubscription, WebhookSubscription};
use super::taxonomy::OfficeEventType;

/// Number of failed deliveries before the demo transport succeeds. With the
/// runtime's two-attempt retry policy the first publish dead-letters and the
/// replay (a later delivery for the same key) succeeds.
const DEMO_FAILURES_BEFORE_SUCCESS: u32 = 2;
const DEMO_WEBHOOK_URL: &str = "https://events.example.test/hook";

const NIL_SCOPE_ID: &str = "00000000-0000-0000-0000-000000000000";

/// The deterministic dead letter used by the admin replay walkthrough.
fn demo_event() -> DomainEvent {
    DomainEvent::new(
        "evt-demo-001",
        OfficeEventType::SheetsExported,
        "ws_123",
        "sheet_456",
        "usr_789",
    )
    .with_tag("finance")
    .with_metadata("title", json!("Quarterly report"))
}

/// Owner of an event pipeline: both ids come from the caller's verified JWT
/// (or a verified session binding) in `botserver`. Strings keep this crate
/// free of a UUID dependency; the server passes canonical UUID text.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EventTenantScope {
    org_id: String,
    branch_id: String,
}

impl EventTenantScope {
    /// Builds a scope only when both identifiers are present and non-nil.
    /// Missing scope must never degrade into a global queue (fail closed).
    #[must_use]
    pub fn new(org_id: impl Into<String>, branch_id: impl Into<String>) -> Option<Self> {
        let scope = Self {
            org_id: org_id.into(),
            branch_id: branch_id.into(),
        };
        scope.is_valid().then_some(scope)
    }

    #[must_use]
    pub fn is_valid(&self) -> bool {
        is_scope_id(&self.org_id) && is_scope_id(&self.branch_id)
    }
}

fn is_scope_id(id: &str) -> bool {
    !id.trim().is_empty() && id == id.trim() && id != NIL_SCOPE_ID
}

/// Pure tenant-ownership predicate (issue #2825). A caller may touch a
/// record only when both identities are valid and exactly equal. No I/O, so
/// the two-tenant rule is unit-testable without a server.
fn scope_owns(owner: &EventTenantScope, caller: &EventTenantScope) -> bool {
    owner.is_valid() && caller.is_valid() && owner == caller
}

/// Compatibility scope for the unscoped offline API used by contract tests.
/// No HTTP handler may use it; production callers must supply `new(...)`.
fn legacy_scope() -> EventTenantScope {
    EventTenantScope {
        org_id: "legacy-org".to_string(),
        branch_id: "legacy-branch".to_string(),
    }
}

/// One tenant's isolated dispatcher plus the scope that owns it.
struct TenantRuntime {
    scope: EventTenantScope,
    dispatcher: WebhookDispatcher,
}

/// Deterministic transport for the admin/event surface. Failures are counted
/// per idempotency key (the `X-GB-Delivery` header), so a publish exhausts its
/// retry budget into the DLQ and a subsequent replay of the same key succeeds
/// without any external service or mutable global toggle.
pub struct DemoTransport {
    failures_before_success: u32,
    deliveries: Mutex<HashMap<String, u32>>,
}

impl DemoTransport {
    #[must_use]
    pub fn new(failures_before_success: u32) -> Self {
        Self {
            failures_before_success,
            deliveries: Mutex::new(HashMap::new()),
        }
    }
}

impl DeliveryTransport for DemoTransport {
    fn deliver(&self, _url: &str, headers: &[(String, String)], _body: &[u8]) -> DeliveryOutcome {
        let key = headers
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(super::signing::DELIVERY_HEADER))
            .map_or_else(|| "unknown".to_string(), |(_, value)| value.clone());
        let mut deliveries = self
            .deliveries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let count = deliveries.entry(key).or_insert(0);
        *count += 1;
        if *count <= self.failures_before_success {
            DeliveryOutcome {
                status_code: 503,
                latency_ms: 1,
                error: Some("demo endpoint returned 5xx".to_string()),
            }
        } else {
            DeliveryOutcome {
                status_code: 200,
                latency_ms: 1,
                error: None,
            }
        }
    }
}

/// Append-only log of native envelopes. Browsers subscribe with a cursor
/// (`?cursor=`); the SSE/WebSocket handlers only ever read a slice, so several
/// subscribers can consume the same events independently.
#[derive(Debug, Default)]
pub struct OfficeEventStream {
    envelopes: Mutex<Vec<NativeEnvelope>>,
}

impl OfficeEventStream {
    #[must_use]
    pub fn new() -> Self {
        Self {
            envelopes: Mutex::new(Vec::new()),
        }
    }

    pub fn push(&self, envelope: NativeEnvelope) {
        self.envelopes
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(envelope);
    }

    #[must_use]
    pub fn since(&self, cursor: usize) -> Vec<NativeEnvelope> {
        let envelopes = self
            .envelopes
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if cursor >= envelopes.len() {
            Vec::new()
        } else {
            envelopes[cursor..].to_vec()
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.envelopes
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// A dead letter as exposed to the admin API. Deliberately excludes `body`
/// and `payload` from the internal `DeadLetter`: only IDs and metadata ever
/// leave the process through this surface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DeadLetterView {
    pub idempotency_key: String,
    pub event_id: String,
    pub event_type: String,
    pub subscription_id: String,
    pub url: String,
    pub attempts: u32,
    pub last_error: Option<String>,
    pub dead_lettered_at: DateTime<Utc>,
}

impl From<&DeadLetter> for DeadLetterView {
    fn from(letter: &DeadLetter) -> Self {
        Self {
            idempotency_key: letter.idempotency_key.clone(),
            event_id: letter.event_id.clone(),
            event_type: letter
                .payload
                .get("type")
                .and_then(Value::as_str)
                .map_or_else(String::new, str::to_string),
            subscription_id: letter.subscription_id.clone(),
            url: letter.url.clone(),
            attempts: letter.attempts,
            last_error: letter.last_error.clone(),
            dead_lettered_at: letter.dead_lettered_at,
        }
    }
}

/// Result of one admin replay. `deduped` is true when the idempotency ledger
/// already held a receipt, i.e. no second delivery was made.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReplayResult {
    pub receipt: DeliveryReceipt,
    pub deduped: bool,
}

/// The process-wide event runtime shared by every HTTP/WS handler.
///
/// Each tenant's dead-letter queue and idempotency ledger live in their own
/// dispatcher. The map is keyed by the verified org/branch scope, so a caller
/// cannot even address another tenant's queue by guessing a key.
pub struct OfficeEventsRuntime {
    tenants: Mutex<HashMap<EventTenantScope, Arc<TenantRuntime>>>,
    stream: OfficeEventStream,
    sequence: AtomicU64,
}

impl Default for OfficeEventsRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl OfficeEventsRuntime {
    #[must_use]
    pub fn new() -> Self {
        let runtime = Self {
            tenants: Mutex::new(HashMap::new()),
            stream: OfficeEventStream::new(),
            sequence: AtomicU64::new(1),
        };
        // Unscoped compatibility surface (offline contract tests only).
        runtime.register_tenant(Arc::new(new_tenant_runtime(&legacy_scope())));
        runtime.publish(&demo_event(), Some(DEMO_WEBHOOK_URL));
        runtime
    }

    fn tenants(&self) -> MutexGuard<'_, HashMap<EventTenantScope, Arc<TenantRuntime>>> {
        self.tenants
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn tenant(&self, scope: &EventTenantScope) -> Option<Arc<TenantRuntime>> {
        if !scope.is_valid() {
            return None;
        }
        self.tenants().get(scope).cloned()
    }

    fn register_tenant(&self, tenant: Arc<TenantRuntime>) {
        self.tenants().insert(tenant.scope.clone(), tenant);
    }

    fn tenant_or_create(&self, scope: &EventTenantScope) -> Arc<TenantRuntime> {
        if let Some(tenant) = self.tenant(scope) {
            return tenant;
        }
        let tenant = Arc::new(new_tenant_runtime(scope));
        self.register_tenant(Arc::clone(&tenant));
        // Each tenant sees the same seeded demo letter; the already-registered
        // scope makes this a normal publish, not a recursive tenant creation.
        self.publish_scoped(scope, &demo_event(), Some(DEMO_WEBHOOK_URL));
        tenant
    }

    #[must_use]
    pub fn stream(&self) -> &OfficeEventStream {
        &self.stream
    }

    /// Generates a unique, server-controlled event id. Clients never supply
    /// their own id (or content) through the publish endpoint.
    #[must_use]
    pub fn next_event_id(&self) -> String {
        let sequence = self.sequence.fetch_add(1, Ordering::SeqCst);
        format!("evt-live-{}-{}", Utc::now().timestamp_millis(), sequence)
    }

    /// Publishes one event into the caller's tenant dispatcher (webhooks when
    /// a URL is supplied, plus the native queue mirrored into the stream).
    /// An invalid scope performs no delivery at all.
    pub fn publish_scoped(
        &self,
        scope: &EventTenantScope,
        event: &DomainEvent,
        webhook_url: Option<&str>,
    ) -> DispatchReport {
        if !scope.is_valid() {
            return DispatchReport::default();
        }
        let tenant = self.tenant_or_create(scope);
        if let Some(url) = webhook_url {
            let id = format!("sub-{}", event.event_id);
            if !tenant.dispatcher.subscriptions().iter().any(|s| s.id == id) {
                // Per-publish secret, derived at runtime — no credential is
                // embedded in source or shared across deliveries.
                let secret = format!("demo-signing-{}", Utc::now().timestamp_millis());
                tenant.dispatcher.add_subscription(
                    WebhookSubscription::new(id, url, "runtime-key", secret)
                        .with_event_types(vec![event.event_type]),
                );
            }
        }
        let mut report = tenant.dispatcher.publish_webhooks(event);
        let envelopes = tenant.dispatcher.native().publish_and_collect(event);
        for envelope in &envelopes {
            self.stream.push(envelope.clone());
        }
        report.native_enqueued = envelopes.len();
        report
    }

    /// Unscoped compatibility surface for offline contract tests. HTTP
    /// handlers must use `publish_scoped` with a verified tenant scope.
    pub fn publish(&self, event: &DomainEvent, webhook_url: Option<&str>) -> DispatchReport {
        self.publish_scoped(&legacy_scope(), event, webhook_url)
    }

    /// Lists only the dead letters owned by the verified tenant. An unknown or
    /// unresolvable scope sees nothing (fail closed).
    #[must_use]
    pub fn dead_letters_scoped(&self, scope: &EventTenantScope) -> Vec<DeadLetterView> {
        let Some(tenant) = self.tenant(scope) else {
            return Vec::new();
        };
        tenant
            .dispatcher
            .dlq()
            .list()
            .iter()
            .map(DeadLetterView::from)
            .collect()
    }

    #[must_use]
    pub fn dead_letters(&self) -> Vec<DeadLetterView> {
        self.dead_letters_scoped(&legacy_scope())
    }

    /// Replays a dead letter owned by the verified tenant. A key from a
    /// foreign tenant, an unknown key or a missing scope is indistinguishable
    /// from "not found" — no other tenant's subscription URL is ever touched.
    pub fn replay_scoped(
        &self,
        scope: &EventTenantScope,
        idempotency_key: &str,
    ) -> Result<ReplayResult, ReplayError> {
        let Some(tenant) = self.tenant(scope) else {
            return Err(ReplayError::NotFound);
        };
        if !scope_owns(&tenant.scope, scope) {
            return Err(ReplayError::NotFound);
        }
        let deduped = tenant.dispatcher.ledger().contains(idempotency_key);
        let receipt = tenant.dispatcher.replay(idempotency_key)?;
        Ok(ReplayResult { receipt, deduped })
    }

    /// Unscoped compatibility surface for offline contract tests.
    pub fn replay(&self, idempotency_key: &str) -> Result<ReplayResult, ReplayError> {
        self.replay_scoped(&legacy_scope(), idempotency_key)
    }
}

/// Builds a dispatcher with its own retry policy and native subscription.
/// Per-tenant state never crosses this boundary.
fn new_tenant_runtime(scope: &EventTenantScope) -> TenantRuntime {
    let transport = Arc::new(DemoTransport::new(DEMO_FAILURES_BEFORE_SUCCESS));
    let dispatcher = WebhookDispatcher::new(transport, Arc::new(DenyAllContent))
        .with_retry(RetryPolicy::new(2, 1, 1, 1));
    // The suite automation consumes this in-platform queue; no HTTP hook.
    dispatcher
        .native()
        .subscribe(NativeSubscription::new("sub-suite-automation", "default"));
    TenantRuntime {
        scope: scope.clone(),
        dispatcher,
    }
}

static RUNTIME: OnceLock<OfficeEventsRuntime> = OnceLock::new();

/// The single runtime mounted by `botserver`.
#[must_use]
pub fn runtime() -> &'static OfficeEventsRuntime {
    RUNTIME.get_or_init(OfficeEventsRuntime::new)
}

#[cfg(test)]
#[path = "runtime_tests.rs"]
mod runtime_tests;

#[cfg(test)]
#[path = "runtime_tenant_tests.rs"]
mod runtime_tenant_tests;
