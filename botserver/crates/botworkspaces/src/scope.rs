//! Tenant branch-scope resolution — a re-export of the single shared
//! implementation in `botsecurity_core::scope` (#1347).
//!
//! Nine crates used to carry their own copy of this module; scope is the
//! most security-relevant helper in the tree, so one implementation now
//! guards every tenant boundary and the per-crate modules only re-export
//! it to keep `crate::scope::` call sites working.

pub use botsecurity_core::scope::{
    branch_from_claim, branch_from_jwt, branch_from_jwt_pool, email_from_jwt,
    email_from_session, email_from_user_id,
};
