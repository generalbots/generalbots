//! Approval & e-signature workflow core for office documents (issue #2501).
//!
//! Pure, dependency-light domain logic for routing approvals, SLA expiry,
//! delegation, signed decisions and a deterministic publish-to-records export.
//! Nothing here touches HTTP, the database, the UI or the system clock:
//! callers supply timestamps and document digests, and drive the state machine
//! through explicit commands. This makes the whole slice testable offline.
//!
//! The outer loop is [`ApprovalWorkflow`]; each [`ApprovalStep`] owns the
//! per-step state machine (`pending -> approved | rejected | delegated |
//! expired`). [`RecordsExport`] is the immutable, hash-bound payload suitable
//! for filing in records once the workflow is signed.

pub mod error;
pub mod model;
pub mod record;
pub mod workflow;

#[cfg(test)]
mod tests;

pub use model::*;
pub use record::RecordsExport;
pub use workflow::ApprovalWorkflow;

/// Deterministic, dependency-free 64-bit FNV-1a digest rendered as 16 lowercase
/// hex digits. Used only for stable record/signature binding — it is a
/// reproducibility aid, not a cryptographic signature.
pub(crate) fn hash_hex(input: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in input.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    format!("{hash:016x}")
}
