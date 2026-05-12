pub mod analytics_types;
pub mod handlers;
pub mod handlers_activity;
pub mod handlers_charts;
pub mod insights;
pub mod insights_types;
pub mod routes;
pub mod schema;

#[cfg(feature = "goals")]
pub mod goals;

#[cfg(feature = "goals")]
pub mod goals_ui;

use std::sync::Arc;
use uuid::Uuid;

pub type DbPool = r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>;

pub type GetDefaultBotFn = Arc<dyn Fn(&mut diesel::PgConnection) -> (Uuid, String) + Send + Sync>;

pub type GetBotContextFn = Arc<dyn Fn() -> (Uuid, Uuid) + Send + Sync>;

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub username: String,
}
