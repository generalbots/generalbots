//! Directory/Zitadel bootstrap helpers (extracted from `bootstrap.rs`).
//!
//! #1730 — the module outgrew the 450-line limit, so it is split by
//! responsibility: the first-run administrator reconciliation and the Zitadel
//! configuration/bootstrap live in sibling modules. Every item is re-exported
//! here, so the existing `main_module::directory_setup::*` call sites are
//! unchanged.
//!
//! #2140 — the SaaS JWT secret resolver is **not** here: the single
//! implementation is `botsecurity_core::verified_jwt::resolve_saas_jwt_secret`
//! (see `botsecurity_core::jwt_secret`). A local copy with a public fallback
//! used to live in this module and is gone on purpose.

mod bootstrap_admin;
#[cfg(feature = "directory")]
mod zitadel;

pub(crate) use bootstrap_admin::{ensure_bootstrap_admin_user, is_bootstrap_admin_identity};
#[cfg(feature = "directory")]
pub(crate) use zitadel::{
    bootstrap_directory_admin, default_zitadel_config, init_directory_service,
};
