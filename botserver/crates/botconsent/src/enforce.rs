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
mod resolve;
mod service;

#[cfg(test)]
mod tests;

pub use authorize::{authorize, authorize_with_context};
pub use resolve::{resolve, take_pending_approved};
pub use service::{
    ConsentContext, ConsentDecision, ConsentService, PendingMap, PendingRequest, ResolvedOutcome,
    PENDING_TTL_SECS,
};

pub(crate) use resolve::resolve_in_memory;
pub(crate) use service::sweep_maps;
