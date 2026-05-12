use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use std::sync::Arc;
use uuid::Uuid;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub type AppState = Arc<DbPool>;

pub type GetDefaultBotFn = Arc<dyn Fn(&mut PgConnection) -> (Uuid, String) + Send + Sync>;

pub type TriggerContactChangeFn = Arc<dyn Fn(&mut PgConnection, Uuid, &str, Uuid) + Send + Sync>;

pub type TriggerDealStageChangeFn = Arc<dyn Fn(&mut PgConnection, Uuid, &str, &str, Uuid) + Send + Sync>;

#[derive(Clone)]
pub struct CrateState {
    pub db_pool: DbPool,
    pub get_default_bot: GetDefaultBotFn,
    pub trigger_contact_change: TriggerContactChangeFn,
    pub trigger_deal_stage_change: TriggerDealStageChangeFn,
}

impl CrateState {
    pub fn new(
        db_pool: DbPool,
        get_default_bot: GetDefaultBotFn,
        trigger_contact_change: TriggerContactChangeFn,
        trigger_deal_stage_change: TriggerDealStageChangeFn,
    ) -> Self {
        Self {
            db_pool,
            get_default_bot,
            trigger_contact_change,
            trigger_deal_stage_change,
        }
    }

    pub fn get_bot_context(&self) -> (Uuid, Uuid) {
        use diesel::prelude::*;
        use crate::schema::bots;

        let Ok(mut conn) = self.db_pool.get() else {
            return (Uuid::nil(), Uuid::nil());
        };
        let (bot_id, _bot_name) = (self.get_default_bot)(&mut conn);

        let org_id = bots::table
            .filter(bots::id.eq(bot_id))
            .select(bots::org_id)
            .first::<Option<Uuid>>(&mut conn)
            .unwrap_or(None)
            .unwrap_or(Uuid::nil());

        (org_id, bot_id)
    }
}

pub mod schema;
pub mod models;
pub mod requests;
pub mod error;
pub mod migration;
pub mod contacts_api;
pub(crate) mod contacts_api_helpers;
pub mod handlers;
pub mod ui;
pub mod routes;

#[cfg(feature = "calendar")]
pub mod calendar_types;
#[cfg(feature = "calendar")]
pub mod calendar_service;
#[cfg(feature = "calendar")]
pub(crate) mod calendar_service_helpers;
#[cfg(feature = "calendar")]
pub mod calendar_routes;

#[cfg(feature = "tasks")]
pub mod tasks_types;
#[cfg(feature = "tasks")]
pub mod tasks_service;
#[cfg(feature = "tasks")]
pub(crate) mod tasks_service_helpers;
#[cfg(feature = "tasks")]
pub mod tasks_routes;

#[cfg(feature = "external_sync")]
pub mod sync_types;
#[cfg(feature = "external_sync")]
pub mod sync_service;
#[cfg(feature = "external_sync")]
pub mod google_client;
#[cfg(feature = "external_sync")]
pub mod microsoft_client;
#[cfg(feature = "external_sync")]
pub mod sync_routes;

pub use error::ContactsError;
pub use migration::create_contacts_tables_migration;
