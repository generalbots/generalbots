use axum::extract::Json;
use botsecurity_auth::auth_api::types::AuthenticatedUser;
use diesel::prelude::*;
use serde_json::Value;
use uuid::Uuid;

use crate::error::IntegrationError;

#[derive(diesel::QueryableByName)]
struct DefaultBotRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
}

/// Resolves the shared pool, mapping the tuple error of `db::pool()` onto the
/// control-plane error type without leaking the raw message.
fn shared_pool() -> Result<&'static botcore::shared::utils::DbPool, IntegrationError> {
    crate::db::pool().map_err(|(status, message)| {
        log::error!("integration context: database pool unavailable ({status}): {message}");
        IntegrationError::Storage("database pool unavailable".to_string())
    })
}

/// GET /api/apps/integrations/context
///
/// Resolves the default active bot so suite apps can build tenant-scoped
/// connection URLs without hardcoding bot ids. Same resolution approach as
/// the WhatsApp channel bootstrap: first active bot by creation order.
/// Vault-independent by design - this handler reads only the database and
/// never touches the secrets manager.
///
/// #2100 — an installation with no active bot (a fresh desktop profile)
/// answers `200` with the documented no-workspace payload
/// `{ "bot_id": null, "reason": "no_active_bot" }`. It used to answer `404`,
/// which the Integrations app reported as "Status unavailable" instead of the
/// honest "no bot/workspace yet".
pub async fn context(user: AuthenticatedUser) -> Result<Json<Value>, IntegrationError> {
    let pool = shared_pool()?;
    let mut conn = pool.get()?;
    let row = diesel::sql_query(
        "SELECT id, name FROM bots WHERE is_active = TRUE ORDER BY created_at ASC LIMIT 1",
    )
    .get_result::<DefaultBotRow>(&mut conn)
    .optional()?;

    let Some(bot) = row else {
        log::debug!(
            "integration context: no active bot for caller {}",
            user.user_id
        );
        return Ok(Json(serde_json::json!({
            "bot_id": Value::Null,
            "bot_name": "",
            "connections_url": Value::Null,
            "reason": "no_active_bot",
        })));
    };

    log::debug!("integration context resolved for caller {}", user.user_id);
    Ok(Json(serde_json::json!({
        "bot_id": bot.id,
        "bot_name": bot.name,
        "connections_url": format!("/api/bots/{}/integration-connections", bot.id),
        "reason": Value::Null,
    })))
}
