//! Frame conformance, golden transcripts and replay for the conversation
//! workspace (#1664).
//!
//! This module extends the existing eval harness (`dataset`, `runner`,
//! `scoring`, `ci_gate`) instead of building a second one. It supplies:
//!
//! - checked-in fixtures for every event family, valid and invalid;
//! - golden transcripts replayed through the same reducer the client uses;
//! - prompt sets that measure restraint (UI only when it beats prose);
//! - a frame gate wired into the same CI decision as the text gate.

pub mod fixtures;
pub mod gate;
pub mod metrics;
pub mod prompts;
pub mod replay;

pub use fixtures::{
    load_golden, load_frame_fixtures, load_redaction_cases, FrameFixture, FixtureReport,
    GoldenTranscript, RedactionCase,
};
pub use gate::{FrameGateConfig, FrameGateVerdict};
pub use metrics::{measure, FrameMetrics};
pub use prompts::{load_prompt_cases, PromptCase, RestraintReport};
pub use replay::{ReplayError, ReplayState};
