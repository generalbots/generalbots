use std::sync::Arc;

use axum::extract::{Json, Query, State};
use botsecurity_auth::auth_api::types::AuthenticatedUser;
use diesel::prelude::*;
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

use crate::error::IntegrationError;
use crate::state::IntegrationState;

#[derive(Deserialize)]
pub struct MentionsQuery {
    pub q: Option<String>,
}

#[derive(diesel::QueryableByName)]
struct DefaultBotRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
}

#[derive(diesel::QueryableByName)]
struct MentionConnectionRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    provider_slug: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    display_name: String,
}

/// Strips LIKE wildcards so a crafted `q` cannot widen the match beyond the
/// intended provider-slug prefix; results stay owner-scoped either way.
fn mention_prefix_pattern(q: Option<String>) -> String {
    let prefix = q.unwrap_or_default().trim().to_lowercase();
    let escaped: String = prefix.chars().filter(|c| *c != '%' && *c != '_').collect();
    format!("{escaped}%")
}

/// GET /api/apps/integrations/mentions?q=
///
/// Autocomplete source for @integration mentions in chat (#939 phase D).
///
/// Scope mirrors the context handler: no organization claim is required on
/// the caller, but every returned row is strictly owner-scoped - it must
/// belong to the authenticated user, reference the context default bot and
/// be active. Provider slugs are matched by case-insensitive prefix with a
/// hard limit of 10 rows. The response carries connection ids plus
/// non-sensitive labels only; credentials and Vault paths are never read.
pub async fn mentions(
    State(state): State<Arc<IntegrationState>>,
    user: AuthenticatedUser,
    Query(query): Query<MentionsQuery>,
) -> Result<Json<Value>, IntegrationError> {
    let pattern = mention_prefix_pattern(query.q);
    let mut conn = state.pool.get()?;
    let default_bot = diesel::sql_query(
        "SELECT id FROM bots WHERE is_active = TRUE ORDER BY created_at ASC LIMIT 1",
    )
    .get_result::<DefaultBotRow>(&mut conn)
    .optional()?
    .ok_or(IntegrationError::NotFound)?;
    let rows = diesel::sql_query(
        "SELECT id, provider_slug, display_name FROM integration_connections \
         WHERE bot_id = $1 AND owner_user_id = $2 AND status = 'active' \
         AND provider_slug ILIKE $3 \
         ORDER BY created_at DESC LIMIT 10",
    )
    .bind::<diesel::sql_types::Uuid, _>(default_bot.id)
    .bind::<diesel::sql_types::Uuid, _>(user.user_id)
    .bind::<diesel::sql_types::Text, _>(pattern)
    .load::<MentionConnectionRow>(&mut conn)?;

    let items: Vec<Value> = rows
        .into_iter()
        .map(|row| {
            let llm_available =
                !crate::providers::implemented_action_names(&row.provider_slug).is_empty();
            serde_json::json!({
                "id": row.id,
                "label": row.display_name,
                "provider": row.provider_slug,
                "llm_available": llm_available,
            })
        })
        .collect();
    log::debug!(
        "integration mention search for caller {} returned {} item(s)",
        user.user_id,
        items.len()
    );
    Ok(Json(serde_json::json!({ "items": items })))
}
