//! Reform #1502/#1504 — git-pull bot monitor module root.
//!
//! Submodules:
//! - [`core`] — git plumbing, checkout management, work-layout materialization.
//! - [`loop_ops`] — periodic sync loop, TEST/PROD hook dispatch, DB lists.
//! - [`import`] — one-shot Drive → git source import (#1501).

pub(crate) mod core;
pub(crate) mod import;
pub(crate) mod loop_ops;

pub use loop_ops::start;
pub use import::run_import_pass;
