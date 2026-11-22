//! Typed approvals and e-signature workflow (issue #2501).
//!
//! This slice adds routing (parallel + sequential), per-step SLA, delegation,
//! version-bound signed decisions, an e-signature step, and an immutable
//! publish-to-records artefact. The API is mounted together with the consent
//! router so no additional server wiring is required.

pub mod api;
pub mod engine;
pub mod signing;
pub mod store;
pub mod types;

pub use engine::content_hash;
pub use store::ApprovalStore;
pub use types::{
    ApprovalRecord, ApprovalStatus, ApprovalStep, DecisionKind, Delegation, PublishedArtifact,
    SignedDecision, SlaEvent, SlaEventKind, StepKind, StepMode,
};
