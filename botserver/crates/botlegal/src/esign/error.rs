//! Refusal reasons for approval operations.

use std::fmt;

use super::model::StepState;

/// Why an approval operation was refused. Callers can match on the variant
/// instead of string-matching a message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApprovalError {
    StepNotActive { state: StepState },
    StepNotFound { id: String },
    StepOutOfOrder { step_id: String },
    EmptyActor,
    DigestRequired,
    DigestMismatch { expected: String, got: String },
    ReasonRequired,
    DelegateRequired,
    SlaNotElapsed { deadline: i64, now: i64 },
    ChainHalted,
    NotApproved,
    NotPublished,
    SerializationFailed,
}

impl fmt::Display for ApprovalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StepNotActive { state } => write!(f, "step is not active (state {state})"),
            Self::StepNotFound { id } => write!(f, "no such step: {id}"),
            Self::StepOutOfOrder { step_id } => {
                write!(f, "step {step_id} is not the active step in this sequential route")
            }
            Self::EmptyActor => f.write_str("an actor is required"),
            Self::DigestRequired => f.write_str("a document digest is required"),
            Self::DigestMismatch { expected, got } => {
                write!(f, "digest mismatch: expected {expected}, got {got}")
            }
            Self::ReasonRequired => f.write_str("a reason is required for this decision"),
            Self::DelegateRequired => f.write_str("a delegate target is required"),
            Self::SlaNotElapsed { deadline, now } => {
                write!(f, "SLA has not elapsed (deadline {deadline}, now {now})")
            }
            Self::ChainHalted => {
                f.write_str("the approval chain was halted by a rejection or expiry")
            }
            Self::NotApproved => f.write_str("all steps must be approved before signing"),
            Self::NotPublished => f.write_str("the workflow must be signed before publish"),
            Self::SerializationFailed => f.write_str("record payload could not be serialized"),
        }
    }
}

impl std::error::Error for ApprovalError {}
