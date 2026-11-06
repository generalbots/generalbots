//! Run event emission for chat turns (#1663).
//!
//! This is the botserver adapter around `botevents`: it owns the process-wide
//! pipeline (one sequencer per thread), applies the shared security redactor
//! and writes validated frames through the existing `ChannelSink`.
//!
//! The transport stays `ChannelSink`/`raw_json`; the event contract, ordering,
//! validation and redaction live in `botevents`. Features that build runs
//! (#1666, #1667, #1668) call the helpers below instead of hand-building
//! frames.
//!
//! Split by responsibility: `redactor` (pipeline + runtime flag), `context`
//! (turn identity), `emit` (frame transport) and the `lifecycle`/`steps`
//! helpers that build each event.

mod context;
mod emit;
mod lifecycle;
mod redactor;
mod steps;

pub use context::RunContext;
pub use emit::{can_emit, emit, emit_custom};
pub use lifecycle::{run_error, run_finished, run_started};
pub use redactor::{enabled, observe_persisted_seq};
pub use steps::{step_finished, step_started, text_delta, text_end, tool_result, tool_started};
