//! Enforcement core: decides between a stored grant, a fresh prompt and a
//! denial for every `(user, app, action_class)` attempt, and manages pending
//! consent requests with a five-minute TTL plus the allow-once handoff used
//! by command executors.
//!
//! Split by responsibility (#1730): the in-memory service and its types
//! (`service`), the authorize/deny entry points (`authorize`) and the user
//! resolution/handoff path (`resolve`). The re-exports keep every public and
//! crate-visible item at its historical `enforce::` path.

mod authorize;
mod blocking;
mod resolve;
mod service;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_blocking;
#[cfg(test)]
mod tests_owner;

pub use authorize::{authorize, authorize_with_context};
pub use blocking::{
    audit_view, is_blocking, raise_blocking, turn_superseded, withdraw, withdraw_thread, AUDIT_KEY,
};
pub use resolve::{resolve, take_approved_match, take_pending_approved, Resolver, NOT_OWNER};
pub use service::{
    ConsentContext, ConsentDecision, ConsentService, PendingMap, PendingRequest, ResolvedOutcome,
    PENDING_TTL_SECS,
};

