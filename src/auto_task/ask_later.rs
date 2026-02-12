use crate::core::shared::models::UserSession;
use crate::core::shared::state::AppState;

fn is_sensitive_config_key(key: &str) -> bool {
    let key_lower = key.to_lowercase();
    let sensitive_patterns = [
        "password", "secret", "token", "key", "credential", "auth",
        "api_key", "apikey", "pass", "pwd", "cert", "private",
    ];
    sensitive_patterns.iter().any(|p| key_lower.contains(p))
}
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::Text;
use log::{info, trace};
use rhai::{Dynamic, Engine};
use std::sync::Arc;
use uuid::Uuid;

pub fn ask_later_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = state.clone();
    let user_clone = user.clone();

    engine.register_fn(
        "ask_later",
        move |label: &str, config_key: &str, reason: &str| -> Dynamic {
            let state = state_clone.clone();
            let user = user_clone.clone();

            let result = save_pending_info(&state, &user, label, config_key, reason, None);

            match result {
                Ok(id) => {
                    info!(
                        "Pending info saved: {} -> {} (id: {})",
                        label, config_key, id
                    );
                    Dynamic::from(id.to_string())
                }
                Err(e) => {
                    log::error!("Failed to save pending info: {}", e);
                    Dynamic::UNIT
                }
            }
        },
    );

    let state_clone2 = state.clone();
    let user_clone2 = user.clone();

    engine.register_fn(
        "ask_later_with_type",
        move |label: &str, config_key: &str, reason: &str, field_type: &str| -> Dynamic {
            let state = state_clone2.clone();
            let user = user_clone2.clone();

            let result =
                save_pending_info(&state, &user, label, config_key, reason, Some(field_type));

            match result {
                Ok(id) => {
                    info!(
                        "Pending info saved with type {}: {} -> {} (id: {})",
                        field_type, label, config_key, id
                    );
                    Dynamic::from(id.to_string())
                }
                Err(e) => {
                    log::error!("Failed to save pending info: {}", e);
                    Dynamic::UNIT
                }
            }
        },
    );

    let state_clone3 = state.clone();
    let user_clone3 = user.clone();

    engine.register_fn(
        "fill_pending_info",
        move |config_key: &str, value: &str| -> bool {
            let state = state_clone3.clone();
            let user = user_clone3.clone();

            match fill_pending_info(&state, &user, config_key, value) {
                Ok(_) => {
                    let safe_value = if is_sensitive_config_key(config_key) {
                        "[REDACTED]"
                    } else {
                        value
                    };
                    info!("Pending info filled: {} = {}", config_key, safe_value);
                    true
                }
                Err(e) => {
                    log::error!("Failed to fill pending info: {}", e);
                    false
                }
            }
        },
    );

    let state_clone4 = state.clone();
    let user_clone4 = user.clone();

    engine.register_fn("get_pending_info_count", move || -> i64 {
        let state = state_clone4.clone();
        let user = user_clone4.clone();

        match get_pending_info_count(&state, &user) {
            Ok(count) => count,
            Err(e) => {
                log::error!("Failed to get pending info count: {}", e);
                0
            }
        }
    });

    engine.register_fn("list_pending_info", move || -> Dynamic {
        let state = state.clone();
        let user = user.clone();

        match list_pending_info(&state, &user) {
            Ok(items) => {
                let array: Vec<Dynamic> = items
                    .into_iter()
                    .map(|item| {
                        let mut map = rhai::Map::new();
                        map.insert("id".into(), Dynamic::from(item.id));
                        map.insert("label".into(), Dynamic::from(item.field_label));
                        map.insert("config_key".into(), Dynamic::from(item.config_key));
                        map.insert(
                            "reason".into(),
                            Dynamic::from(item.reason.unwrap_or_default()),
                        );
                        map.insert("field_type".into(), Dynamic::from(item.field_type));
                        Dynamic::from(map)
                    })
                    .collect();
                Dynamic::from(array)
            }
            Err(e) => {
                log::error!("Failed to list pending info: {}", e);
                Dynamic::from(Vec::<Dynamic>::new())
            }
        }
    });

    trace!("ASK LATER keyword registered");
}

fn save_pending_info(
    state: &AppState,
    user: &UserSession,
    label: &str,
    config_key: &str,
    reason: &str,
    field_type: Option<&str>,
) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
    let bot_id = user.bot_id;
    let field_type_str = field_type.unwrap_or("text");
    let id = Uuid::new_v4();

    let mut conn = state.conn.get()?;

    sql_query(
        "INSERT INTO pending_info (id, bot_id, field_name, field_label, field_type, reason, config_key)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .bind::<Text, _>(config_key)
    .bind::<Text, _>(label)
    .bind::<Text, _>(field_type_str)
    .bind::<Text, _>(reason)
    .bind::<Text, _>(config_key)
    .execute(&mut conn)?;

    Ok(id)
}

fn fill_pending_info(
    state: &AppState,
    user: &UserSession,
    config_key: &str,
    value: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let bot_id = user.bot_id;

    let mut conn = state.conn.get()?;

    sql_query(
        "UPDATE pending_info SET filled_at = NOW() WHERE bot_id = $1 AND config_key = $2 AND filled_at IS NULL",
    )
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .bind::<Text, _>(config_key)
    .execute(&mut conn)?;

    let config_manager = crate::core::config::ConfigManager::new(state.conn.clone());
    config_manager.set_config(&bot_id, config_key, value)?;

    Ok(())
}

fn get_pending_info_count(
    state: &AppState,
    user: &UserSession,
) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
    let bot_id = user.bot_id;

    let mut conn = state.conn.get()?;

    let result: CountResult = sql_query(
        "SELECT COUNT(*) as count FROM pending_info WHERE bot_id = $1 AND filled_at IS NULL",
    )
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .get_result(&mut conn)?;

    Ok(result.count)
}

#[derive(Debug, Clone)]
pub struct PendingInfoItem {
    pub id: String,
    pub field_label: String,
    pub config_key: String,
    pub reason: Option<String>,
    pub field_type: String,
}

fn list_pending_info(
    state: &AppState,
    user: &UserSession,
) -> Result<Vec<PendingInfoItem>, Box<dyn std::error::Error + Send + Sync>> {
    let bot_id = user.bot_id;

    let mut conn = state.conn.get()?;

    let results: Vec<PendingInfoRow> = sql_query(
        "SELECT id, field_label, config_key, reason, field_type
         FROM pending_info
         WHERE bot_id = $1 AND filled_at IS NULL
         ORDER BY created_at ASC",
    )
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .get_results(&mut conn)?;

    let items = results
        .into_iter()
        .map(|row| PendingInfoItem {
            id: row.id.to_string(),
            field_label: row.field_label,
            config_key: row.config_key,
            reason: row.reason,
            field_type: row.field_type,
        })
        .collect();

    Ok(items)
}

#[derive(QueryableByName)]
struct CountResult {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    count: i64,
}

#[derive(QueryableByName)]
struct PendingInfoRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
    #[diesel(sql_type = Text)]
    field_label: String,
    #[diesel(sql_type = Text)]
    config_key: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<Text>)]
    reason: Option<String>,
    #[diesel(sql_type = Text)]
    field_type: String,
}
