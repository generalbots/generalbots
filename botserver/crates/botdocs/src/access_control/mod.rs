//! Enterprise object access control for Docs objects (#2452).
//!
//! This is a deliberately **offline** slice of the enterprise-sharing epic: a
//! pure, testable policy core with no I/O, no HTTP and no clock reads. Every
//! timestamp is supplied by the caller, so a decision is a deterministic
//! function of `(principal, action, acl, now)` and can be verified without a
//! running server, a database or a browser.
//!
//! The three pieces mirror the issue's requirements:
//!
//! * [`Acl`] — owner, per-user and per-group grants, a link policy (scope,
//!   expiry, password, watermark, download/print/copy flags) and a guest flag.
//! * [`decide`] — returns an explicit [`Decision`] carrying a [`Reason`], so a
//!   caller can tell a viewer denial from an expired link or a guest restriction
//!   instead of guessing from a single boolean.
//! * [`AuditLog`] — append-only record of view/edit/comment/export/share/
//!   permission-change events, with secrets stripped from free-form details.
//!
//! The HTTP layer is expected to map [`Action`] onto its endpoints and to call
//! [`AuditLog::record`] after [`decide`]; this module itself performs no
//! enforcement at a route, which keeps it independently unit-testable.

mod audit;
mod decision;
mod model;
#[cfg(test)]
mod tests;

pub use audit::{AuditEvent, AuditInput, AuditLog, REDACTED, redact};
pub use decision::decide;
pub use model::{
    Action, Acl, Decision, LinkCredential, LinkPolicy, LinkScope, Principal, Reason, Role,
};
