use super::*;

pub fn set_session_pool(pool: DbPool) {
    if let Err(e) = SESSION_POOL.set(pool) {
        log::warn!("Session pool already initialized: {e:?}");
    }
}
