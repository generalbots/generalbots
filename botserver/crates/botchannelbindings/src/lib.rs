mod api;
pub mod models;
pub mod schema;

use diesel::PgConnection;
use r2d2::Pool;

pub type DbPool = Pool<diesel::r2d2::ConnectionManager<PgConnection>>;

#[derive(Clone)]
pub struct ChannelBindingsService {
    pub pool: DbPool,
}

impl ChannelBindingsService {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

pub use api::configure_routes;
