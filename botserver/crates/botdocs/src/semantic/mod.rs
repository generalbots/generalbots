//! Semantic document layer for AI (#2451).
//!
//! A pure, dependency-light view over an HTML-ish document that gives agents a
//! surgical contract instead of whole-document rewrites:
//!
//! - [`parse`] turns HTML into typed, page-addressed blocks with stable ids;
//! - [`read_blocks`] returns a bounded structured view (page + token budget);
//! - [`patch`] applies typed operations by id with optimistic concurrency and
//!   returns an inverse patch for one-action revert;
//! - [`diff`] produces an ordered change set for preview;
//! - [`cite`] resolves a stable id to a bounded, highlightable snippet.
//!
//! Nothing in this module touches a server, a database, storage or the
//! network, so every behaviour is unit-testable offline.

pub mod cite;
pub mod diff;
pub mod ids;
pub mod model;
pub mod parse;
pub mod patch;
pub mod read;

pub use cite::cite;
pub use diff::diff;
pub use model::*;
pub use parse::{parse, render};
pub use patch::patch;
pub use read::read_blocks;

#[cfg(test)]
mod tests;
