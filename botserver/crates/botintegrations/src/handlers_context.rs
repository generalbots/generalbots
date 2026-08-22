use std::sync::Arc;

use axum::extract::{Json, State};
use botsecurity_auth::auth_api::types::AuthenticatedUser;
use diesel::prelude::*;
use serde_json::Value;
use uuid::Uuid;

use crate::error::IntegrationError;
use crate::state::IntegrationState;

#[derive(diesel::QueryableByName)]
struct DefaultBotRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
}

/// GET /api/apps/integrations/context
///
/// Resolves the default active bot so suite apps can build tenant-scoped
/// connection URLs without hardcoding bot ids. Same resolution approach as
/// the WhatsApp channel bootstrap: first active bot by creation order.
/// Vault-independent by design - this handler reads only the database and
/// never touches the secrets manager.
pub async fn context(
    State(state): State<Arc<IntegrationState>>,
    user: AuthenticatedUser,
) -> Result<Json<Value>, IntegrationError> {
    let mut conn = state.pool.get()?;
    let row = diesel::sql_query(
        "SELECT id, name FROM bots WHERE is_active = TRUE ORDER BY created_at ASC LIMIT 1",
    )
    .get_result::<DefaultBotRow>(&mut conn)
    .optional()?
    .ok_or(IntegrationError::NotFound)?;
    log::debug!("integration context resolved for caller {}", user.user_id);
    Ok(Json(serde_json::json!({
        "bot_id": row.id,
        "bot_name": row.name,
        "connections_url": format!("/api/bots/{}/integration-connections", row.id),
    })))
}
