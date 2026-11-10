//! Docs app wiring: the docs-scoped identity middleware (#2135) and the
//! per-install owner used by the single-user desktop profile.
pub mod install_owner;
pub mod user_middleware;

pub use botdocs::*;
