//! Feature routers that are only compiled when their domain feature is on.
//!
//! The whole module set is gated on the union of the features its helpers
//! serve. Compiling the helpers in a build where none of those features exist
//! produced dead-code and unused-import warnings for code that only the full
//! product uses.

#[cfg(any(feature = "paper", feature = "designer", feature = "dashboards", feature = "canvas", feature = "social", feature = "billing", feature = "saas", feature = "marketing", feature = "telegram", feature = "instagram", feature = "msteams", feature = "attendant"))]
pub(crate) use axum::Router;
#[cfg(any(feature = "paper", feature = "designer", feature = "dashboards", feature = "canvas", feature = "social", feature = "billing", feature = "saas", feature = "marketing", feature = "telegram", feature = "instagram", feature = "msteams", feature = "attendant"))]
pub(crate) use std::sync::Arc;
#[cfg(any(feature = "paper", feature = "designer", feature = "dashboards", feature = "canvas", feature = "social", feature = "billing", feature = "saas", feature = "marketing", feature = "telegram", feature = "instagram", feature = "msteams", feature = "attendant"))]
pub(crate) use botcore::shared::state::AppState;

#[cfg(any(feature = "paper", feature = "designer", feature = "dashboards", feature = "canvas", feature = "social", feature = "billing", feature = "saas", feature = "marketing", feature = "telegram", feature = "instagram", feature = "msteams", feature = "attendant"))]
#[derive(diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct DefaultBranchRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    branch_id: uuid::Uuid,
}

#[cfg(feature = "saas")]
#[derive(diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct DefaultBotRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: uuid::Uuid,
}

#[cfg(any(feature = "paper", feature = "designer", feature = "dashboards", feature = "canvas", feature = "social", feature = "billing", feature = "saas", feature = "marketing", feature = "telegram", feature = "instagram", feature = "msteams", feature = "attendant"))]
mod resolve_default_branch;
#[cfg(any(feature = "paper", feature = "designer", feature = "dashboards", feature = "canvas", feature = "social", feature = "billing", feature = "saas", feature = "marketing", feature = "telegram", feature = "instagram", feature = "msteams", feature = "attendant"))]
pub(crate) use resolve_default_branch::*;

#[cfg(feature = "saas")]
mod resolve_default_bot_id;
#[cfg(feature = "saas")]
pub(crate) use resolve_default_bot_id::*;

#[cfg(any(feature = "paper", feature = "designer", feature = "dashboards", feature = "canvas", feature = "social", feature = "billing", feature = "saas", feature = "marketing", feature = "telegram", feature = "instagram", feature = "msteams", feature = "attendant"))]
mod make_saas_router;
#[cfg(any(feature = "paper", feature = "designer", feature = "dashboards", feature = "canvas", feature = "social", feature = "billing", feature = "saas", feature = "marketing", feature = "telegram", feature = "instagram", feature = "msteams", feature = "attendant"))]
pub(crate) use make_saas_router::*;
