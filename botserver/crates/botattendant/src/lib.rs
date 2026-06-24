pub mod cmdb;
pub mod handlers;
pub mod incident;
pub mod knowledge;
pub mod models;
pub mod routes;
pub mod routes_itsm;
pub mod schema;
pub mod ui;

use std::sync::Arc;

pub type DbPool = r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>;

pub type GetDefaultBotFn = fn(&mut diesel::PgConnection) -> (uuid::Uuid, String);

pub struct AttendantConfig {
    pub pool: Arc<DbPool>,
    pub get_default_bot: GetDefaultBotFn,
}

pub use routes::configure_attendant_routes;
pub use models::*;
pub use ui::configure_attendant_ui_routes;
pub use routes_itsm::configure_itsm_routes;
pub use routes_itsm::ItsmState;
