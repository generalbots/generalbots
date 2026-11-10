pub mod auth;
pub mod collaboration;
pub mod handlers;
pub mod layouts;
pub mod ooxml;
pub mod pptx;
pub mod routes;
pub mod storage;
pub mod types;
pub mod ui;
pub mod ui_fragments;
pub mod utils;

use std::sync::Arc;

use crate::storage::DriveOps;
use botlib::db_pool::DbPool;

pub type GetDefaultBotFn = Arc<dyn Fn(&mut diesel::PgConnection) -> (uuid::Uuid, String) + Send + Sync>;

pub struct SlidesState<D: DriveOps> {
    pub drive: Option<D>,
    pub conn: Arc<DbPool>,
    pub get_default_bot: GetDefaultBotFn,
}

#[cfg(test)]
#[path = "owner_isolation_tests.rs"]
mod owner_isolation_tests;

#[cfg(test)]
#[path = "create_tests.rs"]
mod create_tests;
