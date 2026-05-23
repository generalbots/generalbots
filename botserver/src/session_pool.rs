use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::PgConnection;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;
pub type DbConn = PooledConnection<ConnectionManager<PgConnection>>;

pub struct SessionPool {
    pub pool: DbPool,
}

impl SessionPool {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}
