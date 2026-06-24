use std::sync::Arc;

use botlib::traits::DriveRepository;
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub struct DocState {
    pub pool: Arc<DbPool>,
    pub drive: Arc<dyn DriveRepository>,
    pub bucket_name: String,
}
