//! Shared support code for the messaging channel routers.
//!
//! Channel adapters need two things from the server: the bot configuration of
//! the tenant (Vault-first) and a way to persist inbound media into the bot's
//! Drive. Both used to be wired inline in `feature_routers.rs`, which this
//! module keeps from growing further.

use botcore::shared::state::AppState;
use std::sync::Arc;
use uuid::Uuid;

/// Config reader used by adapters that address a bot by UUID (`telegram`,
/// `msteams`).
pub type UuidConfigFn = dyn Fn(&Uuid, &str, Option<&str>) -> Result<String, String> + Send + Sync;

/// Config reader used by adapters that address a bot by string handle
/// (`instagram`).
pub type HandleConfigFn = dyn Fn(&str, &str, Option<&str>) -> Result<String, String> + Send + Sync;

#[cfg(feature = "instagram")]
fn resolve_channel_bot_id(
    pool: &botcore::shared::utils::DbPool,
    name: &str,
) -> Option<Uuid> {
    use diesel::prelude::*;

    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct ChannelBotIdRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
    }

    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            tracing::warn!("channel bot lookup could not acquire a connection: {e}");
            return None;
        }
    };

    diesel::sql_query("SELECT id FROM bots WHERE name = $1 LIMIT 1")
        .bind::<diesel::sql_types::Text, _>(name)
        .get_result::<ChannelBotIdRow>(&mut conn)
        .optional()
        .ok()
        .flatten()
        .map(|row| row.id)
}

/// Vault-first bot configuration reader for the messaging channel routers.
///
/// These routers previously passed a constant `"stub"` closure, so the channel
/// adapters read credentials such as `telegram-bot-token` as the literal string
/// "stub" and every outbound call authenticated with an invalid token.
#[cfg(any(feature = "telegram", feature = "msteams"))]
pub fn make_channel_config_reader(app_state: &Arc<AppState>) -> Arc<UuidConfigFn> {
    let manager = botcore::config::ConfigManager::new(app_state.conn.clone());

    Arc::new(move |bot_id: &Uuid, key: &str, default: Option<&str>| {
        manager.get_config(bot_id, key, default).map_err(|e| {
            tracing::warn!("bot config read failed for '{key}': {e}");
            e.to_string()
        })
    })
}

/// [`make_channel_config_reader`] for adapters that address the bot by string
/// handle: a UUID handle is used directly, anything else is resolved by name.
#[cfg(feature = "instagram")]
pub fn make_channel_config_reader_by_handle(app_state: &Arc<AppState>) -> Arc<HandleConfigFn> {
    let pool = app_state.conn.clone();
    let manager = botcore::config::ConfigManager::new(app_state.conn.clone());

    Arc::new(move |handle: &str, key: &str, default: Option<&str>| {
        let bot_id = match Uuid::parse_str(handle) {
            Ok(bot_id) => Some(bot_id),
            Err(_) => resolve_channel_bot_id(&pool, handle),
        };

        let Some(bot_id) = bot_id else {
            tracing::warn!("bot config read for '{key}' skipped: unknown bot handle '{handle}'");
            return Ok(default.unwrap_or("").to_string());
        };

        manager.get_config(&bot_id, key, default).map_err(|e| {
            tracing::warn!("bot config read failed for '{key}': {e}");
            e.to_string()
        })
    })
}

/// Stores inbound media in the bot's Drive and returns the path relative to
/// its `gbdrive` directory.
///
/// The bucket is resolved per bot through
/// [`bot_drive_location_for`](botbasic_core::utils::bot_drive_location_for), not
/// derived from the bot name: an org-hosted bot lives in `{slug}.gborg` with
/// the bot directory as a key prefix, so `format!("{bot_name}.gbai")` staged
/// every inbound file into a bucket nothing else reads.
#[cfg(any(feature = "telegram", feature = "whatsapp"))]
fn put_channel_media(
    pool: &botcore::shared::utils::DbPool,
    drive: &Arc<dyn botlib::traits::DriveRepository>,
    bot_id: Uuid,
    rel_path: String,
    data: Vec<u8>,
    content_type: Option<String>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, String>> + Send>> {
    let pool = pool.clone();
    let drive = drive.clone();

    Box::pin(async move {
        let mut conn = pool
            .get()
            .map_err(|e| format!("DB error resolving drive location: {e}"))?;
        let loc = botbasic_core::utils::bot_drive_location_for(&mut conn, bot_id);
        drop(conn);

        let bucket = loc.bucket.clone();
        let key = loc.key_for(&rel_path);

        drive
            .put_object(&bucket, &key, data, content_type.as_deref())
            .await
            .map_err(|e| format!("drive put failed for {bucket}/{key}: {e}"))?;

        Ok(rel_path)
    })
}

/// Stores inbound Telegram media in the bot's Drive and returns the path
/// relative to the bot's `gbdrive` directory.
#[cfg(feature = "telegram")]
pub fn make_put_media_fn(app_state: &Arc<AppState>) -> bottelegram::state::PutMediaFn {
    let drive = app_state.drive.clone();
    let pool = app_state.conn.clone();

    Arc::new(
        move |bot_id: Uuid, rel_path: String, data: Vec<u8>, content_type: Option<String>| {
            let Some(repository) = drive.clone() else {
                return Box::pin(std::future::ready(Err(
                    "Drive service not available".to_string()
                ))) as std::pin::Pin<
                    Box<dyn std::future::Future<Output = Result<String, String>> + Send>,
                >;
            };

            put_channel_media(
                &pool,
                &repository,
                bot_id,
                rel_path,
                data,
                content_type,
            )
        },
    ) as bottelegram::state::PutMediaFn
}

/// Stores inbound WhatsApp media in the bot's Drive and returns the path
/// relative to the bot's `gbdrive` directory.
#[cfg(feature = "whatsapp")]
pub fn make_wa_put_media_fn(app_state: &Arc<AppState>) -> botwhatsapp::state::PutMediaFn {
    let drive = app_state.drive.clone();
    let pool = app_state.conn.clone();

    Arc::new(
        move |bot_id: Uuid, rel_path: String, data: Vec<u8>, content_type: Option<String>| {
            let Some(repository) = drive.clone() else {
                return Box::pin(std::future::ready(Err(
                    "Drive service not available".to_string()
                ))) as std::pin::Pin<
                    Box<dyn std::future::Future<Output = Result<String, String>> + Send>,
                >;
            };

            put_channel_media(
                &pool,
                &repository,
                bot_id,
                rel_path,
                data,
                content_type,
            )
        },
    ) as botwhatsapp::state::PutMediaFn
}
