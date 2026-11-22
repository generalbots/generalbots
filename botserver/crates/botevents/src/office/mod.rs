//! Office document events and webhook delivery (#2503).
//!
//! This module is the transport-independent contract for the document
//! platform's event stream. It is deliberately split into small pieces so the
//! server can wire each one to its own adapter (HTTP transport, ACL provider,
//! durable store) while tests exercise the whole flow in memory:
//!
//! ```text
//! DomainEvent -> filters -> payload builder (privacy) -> HMAC sign
//!     -> transport -> retry -> delivery log -> DLQ / idempotency ledger
//! DomainEvent -> native subscriptions -> in-platform queue (no HTTP)
//! ```
//!
//! No document body is ever placed in a payload unless the subscription opts
//! in **and** a `ContentAcl` grants the acting user access to that document.

pub mod delivery;
pub mod dispatcher;
pub mod event;
pub mod native;
pub mod registry;
pub mod retry;
pub mod runtime;
pub mod signing;
pub mod subscription;
pub mod taxonomy;

pub use delivery::{
    DeadLetter, DeadLetterQueue, DeliveryAttempt, DeliveryLog, DeliveryOutcome, DeliveryReceipt,
    DeliveryRecord, DeliveryResult, DeliveryStatus, DeliveryTransport, IdempotencyLedger,
};
pub use dispatcher::{
    DispatchReport, NativeDispatcher, NativeEnvelope, ReplayError, WebhookDispatcher,
};
pub use event::{ContentAcl, DeliveryPayloadBuilder, DenyAllContent, DomainEvent, EventContent};
pub use registry::{registry, registry_json, SchemaEntry};
pub use retry::RetryPolicy;
pub use runtime::{
    runtime, DeadLetterView, DemoTransport, OfficeEventStream, OfficeEventsRuntime, ReplayResult,
};
pub use signing::{
    verify, Signature, DEFAULT_TOLERANCE_SECS, DELIVERY_HEADER, EVENT_HEADER, SIGNATURE_HEADER,
};
pub use subscription::{NativeSubscription, WebhookSecret, WebhookSubscription};
pub use taxonomy::{OfficeEventType, PAYLOAD_SCHEMA_VERSION};
