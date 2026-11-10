//! Directory/Zitadel bootstrap helpers (extracted from `bootstrap.rs`).
//!
//! #1730 — the module outgrew the 450-line limit, so it is split by
//! responsibility: the SaaS JWT secret resolution, the first-run administrator
//! reconciliation and the Zitadel configuration/bootstrap live in sibling
//! modules. Every item is re-exported here, so the existing
//! `main_module::directory_setup::*` call sites are unchanged.

mod bootstrap_admin;
mod jwt_secret;
#[cfg(feature = "directory")]
mod zitadel;

pub(crate) use bootstrap_admin::{
    bootstrap_admin_identity, ensure_bootstrap_admin_user, is_bootstrap_admin_identity,
    BootstrapAdminIdentity,
};
pub(crate) use jwt_secret::resolve_saas_jwt_secret;
#[cfg(feature = "directory")]
pub(crate) use zitadel::{
    bootstrap_directory_admin, default_zitadel_config, init_directory_service,
};
