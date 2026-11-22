//! Run event contract for the conversation workspace (#1663).
//!
//! One emission pipeline:
//!
//! ```text
//! source -> redactor -> validator -> sequencer -> append-only log -> transport
//! ```
//!
//! The types here are transport independent. `botserver` adapts them to the
//! WebSocket `ChannelSink` and to the `run_events` table; the crate itself has
//! no database and no socket dependency, which keeps the contract testable and
//! reusable by the eval harness.
//!
//! The [`office`] module adds the separate document-platform event contract
//! (#2503): a versioned event taxonomy, a privacy-aware payload builder,
//! HMAC-signed webhooks with retries, dead-lettering and idempotent replay,
//! an in-platform native subscription path and a schema registry with an
//! example for every event.

pub mod agui;
pub mod envelope;
pub mod events;
pub mod office;
pub mod pipeline;
pub mod redact;
pub mod schema;
pub mod sequencer;
pub mod state;
pub mod validate;

pub use envelope::{Envelope, EventSource, PendingEvent};
pub use events::{
    ActivityDeltaData, ActivitySnapshotData, CustomData, EventType, RunErrorData,
    RunFinishedData, RunStartedData, StateDeltaData, StateSnapshotData, StepFinishedData,
    StepStartedData, SubagentFinishedData, SubagentStartedData, TextMessageContentData,
    TextMessageEndData, TextMessageStartData, ToolCallArgsData, ToolCallEndData,
    ToolCallResultData, ToolCallStartData, Usage,
};
pub use pipeline::{EventPipeline, PipelineError};
pub use redact::{DefaultRedactor, FrameRedactor};
pub use schema::envelope_schema;
pub use sequencer::{SequencerRegistry, ThreadSequencer};
pub use state::RunState;
pub use validate::{ValidationIssue, ValidationIssues, Validator};

// Office document events and webhooks (#2503). Kept transport independent so
// the server can wire its own HTTP transport, ACL provider and durable store.
pub use office::{
    registry, registry_json, ContentAcl, DeadLetter, DeadLetterQueue, DeliveryAttempt,
    DeliveryLog, DeliveryOutcome, DeliveryPayloadBuilder, DeliveryReceipt, DeliveryRecord,
    DeliveryResult, DeliveryStatus, DeliveryTransport, DenyAllContent, DispatchReport,
    DomainEvent, EventContent, IdempotencyLedger, NativeDispatcher, NativeEnvelope,
    NativeSubscription, OfficeEventType, ReplayError, RetryPolicy, SchemaEntry, Signature,
    WebhookDispatcher, WebhookSecret, WebhookSubscription, DEFAULT_TOLERANCE_SECS,
    DELIVERY_HEADER, EVENT_HEADER, PAYLOAD_SCHEMA_VERSION, SIGNATURE_HEADER, verify,
};
