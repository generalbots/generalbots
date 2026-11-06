//! Database pool access — shared implementation lives in `botcore`.
//!
//! Kept as a re-export so `crate::db::pool()` call sites stay unchanged.

pub use botcore::shared::utils::{map_diesel_err, pool};
