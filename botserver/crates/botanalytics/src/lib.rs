pub mod analytics_types;
pub mod handlers;
pub mod handlers_activity;
pub mod handlers_charts;
pub mod handlers_dashboard;
pub mod insights;
pub mod insights_types;
pub mod reports;
pub mod routes;
pub mod schema;

#[cfg(feature = "goals")]
pub mod goals;

#[cfg(feature = "goals")]
pub mod goals_ui;

#[cfg(feature = "goals")]
pub mod goals_types;

use std::sync::Arc;
use uuid::Uuid;

pub use botlib::db_pool::DbPool;

pub type GetDefaultBotFn = Arc<dyn Fn(&mut diesel::PgConnection) -> Uuid + Send + Sync>;

pub type GetBotContextFn = Arc<dyn Fn() -> (Uuid, Uuid) + Send + Sync>;

/// Reuse the platform extractor: the auth middleware verifies the bearer
/// token (opaque suite session or signed JWT) and inserts this value into the
/// request extensions. The previous local extractor base64-decoded any JWT
/// payload without checking its signature, so a forged token was accepted.
pub use botsecurity_auth::auth_api::auth_types_authenticated_user::AuthenticatedUser;
