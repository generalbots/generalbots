//! Cloud ↔ Suite single-sign-on endpoints (`/api/auth/cloud-sso`,
//! `/api/auth/suite-sso`, `/api/auth/unified-login`).
//!
//! #1730 — the handler file outgrew the 450-line limit, so it is split by
//! responsibility: the handlers and their payload types stay in `handlers`,
//! the JWT primitives in `jwt`, the HTML handoff pages in `pages` and the
//! Suite session store in `session`. The public paths remain on this module so
//! call sites (`api_setup.rs`) do not change.

mod handlers;
mod jwt;
mod pages;
mod session;

pub use handlers::{
    handle_cloud_sso, handle_suite_sso, handle_unified_login, SsoRequest, SsoResponse, SuiteSsoQuery,
};
pub(crate) use session::create_suite_session;
