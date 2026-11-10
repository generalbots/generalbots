//! Chat turn preludes split out of `exec.rs` (#1641, #1644, #1645).
//!
//! Attachments become prompt content; explicit URL/command requests become
//! computer-use proposals. Keeping them here keeps the pipeline entry point
//! readable and every file under the repository limit.
//!
//! #1730 — the preludes outgrew the 450-line limit, so they are split by
//! responsibility into sibling modules; the `super::turn_hooks::*` paths used
//! by `exec.rs` stay on this module.

mod attachments;
mod computer_action;
mod informational;
mod media;

pub(super) use attachments::ingest_attachment;
pub(super) use computer_action::propose_computer_action;
pub use informational::is_informational_request;
