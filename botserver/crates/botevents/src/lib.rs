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

pub mod agui;
pub mod envelope;
pub mod events;
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
