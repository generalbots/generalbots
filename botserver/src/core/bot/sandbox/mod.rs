//! Sandbox for model- and MCP-authored UI resources (#1674).
//!
//! Model HTML never runs on the application origin. A resource is stored
//! content-addressed, served from a separate origin with a short-lived signed
//! URL scoped to the conversation and the resource hash, and delivered with a
//! CSP derived from the resource's declared `_meta.ui.csp`.

pub mod csp;
pub mod routes;
pub mod sign;
pub mod store;

pub use csp::UiCsp;
pub use sign::{sign_resource, verify_resource};
pub use store::{publish_resource, read_resource};
