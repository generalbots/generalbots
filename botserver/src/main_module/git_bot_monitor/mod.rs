//! Reform #1502/#1504 — git-pull bot monitor module root.
//!
//! Submodules:
//! - [`core`] — git plumbing and checkout management.
//! - [`materialize`] — work-layout materialization (symlink-safe, #2863).
//! - [`path_safety`] — symlink-escape guard used by every materialization loop.
//! - [`loop_ops`] — periodic sync loop, TEST/PROD hook dispatch, DB lists.
//! - [`import`] — one-shot Drive → git source import (#1501).
//! - [`source_ops`] — AutoTask source writes into the bot's repository (#1505).
//! - [`bot_config`] — `.gbot` channel prompts inside the repository (#1501).
//! - [`archive`] — #1501 step 3: archive legacy Drive source prefixes.

pub(crate) mod archive;
pub(crate) mod bot_config;
pub(crate) mod core;
pub(crate) mod import;
pub(crate) mod loop_ops;
pub(crate) mod materialize;
pub(crate) mod path_safety;
pub(crate) mod source_ops;

pub use archive::run_archive_pass;
pub use import::run_import_pass;
pub use loop_ops::start;
pub(crate) use source_ops::GitBotSourceOps;
