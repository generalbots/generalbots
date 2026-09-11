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

/// Looks a bot name up by id or by branch id. The channel routers hand the
/// adapter the workspace's default branch as the bot handle, so both
/// identifiers must reach the same row.
#[cfg(feature = "telegram")]
fn resolve_channel_bot_name(
    pool: &botcore::shared::utils::DbPool,
    bot_id: &Uuid,
) -> Option<String> {
    use diesel::prelude::*;

    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct ChannelBotNameRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
    }

    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            tracing::warn!("channel bot lookup could not acquire a connection: {e}");
            return None;
        }
    };

    diesel::sql_query("SELECT name FROM bots WHERE id = $1 OR branch_id = $1 LIMIT 1")
        .bind::<diesel::sql_types::Uuid, _>(bot_id)
        .get_result::<ChannelBotNameRow>(&mut conn)
        .optional()
        .ok()
        .flatten()
        .map(|row| row.name)
}

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

/// Stores inbound media in `{bot}.gbai/{bot}.gbdrive/` and returns the path
/// relative to the bot's `gbdrive` directory.
#[cfg(feature = "telegram")]
pub fn make_put_media_fn(app_state: &Arc<AppState>) -> bottelegram::state::PutMediaFn {
    let drive = app_state.drive.clone();
    let pool = app_state.conn.clone();

    Arc::new(
        move |bot_id: Uuid, rel_path: String, data: Vec<u8>, content_type: Option<String>| {
            let drive = drive.clone();
            let pool = pool.clone();

            Box::pin(async move {
                let Some(repository) = drive.as_ref() else {
                    return Err("Drive service not available".to_string());
                };

                let bot_name = resolve_channel_bot_name(&pool, &bot_id)
                    .ok_or_else(|| format!("no bot registered for handle {bot_id}"))?;

                let bucket = format!("{bot_name}.gbai");
                let key = format!("{bot_name}.gbdrive/{rel_path}");

                repository
                    .put_object(&bucket, &key, data, content_type.as_deref())
                    .await
                    .map_err(|e| format!("drive put failed for {key}: {e}"))?;

                Ok(rel_path)
            }) as std::pin::Pin<
                Box<dyn std::future::Future<Output = Result<String, String>> + Send>,
            >
        },
    ) as bottelegram::state::PutMediaFn
}
