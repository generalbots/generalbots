//! `GET|PUT /api/ai-accounts/:provider/models` — the models a subscription
//! agent's CLI accepts, and which of them the chat menu offers.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use botcore::shared::state::AppState;
use serde::Deserialize;
use serde_json::{json, Value};

use super::shared::{ensure_user, err, err_db, provider_or_error, ApiError};
use crate::core::ai_accounts::models as m;
use crate::security::auth_api::types::AuthenticatedUser;

#[derive(Debug, Deserialize)]
pub struct EnabledModelsBody {
    pub enabled: Vec<String>,
}

fn cli_or_error(
    locale: &botcore::i18n::Locale,
    provider: &str,
) -> Result<(&'static str, botagent::cli_models::Cli), ApiError> {
    let provider = provider_or_error(locale, provider)?;
    let cli = m::cli_for(provider)
        .ok_or_else(|| err(locale, StatusCode::BAD_REQUEST, "error-ai-account-unknown-provider"))?;
    Ok((provider, cli))
}

/// GET /api/ai-accounts/:provider/models
pub async fn list_models(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    request_locale: botcore::i18n::RequestLocale,
    Path(provider): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let locale = request_locale.locale();
    let (provider, cli) = cli_or_error(locale, &provider)?;
    let list = m::resolve_list(user.user_id, cli).await;
    let enabled = m::effective_enabled(m::load_enabled(&state.conn, user.user_id, provider));
    Ok(Json(json!({
        "provider": provider,
        "models": m::entries(&list, &enabled),
    })))
}

/// PUT /api/ai-accounts/:provider/models
pub async fn save_models(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    request_locale: botcore::i18n::RequestLocale,
    Path(provider): Path<String>,
    Json(body): Json<EnabledModelsBody>,
) -> Result<Json<Value>, ApiError> {
    let locale = request_locale.locale();
    let (provider, cli) = cli_or_error(locale, &provider)?;
    ensure_user(locale, &state, &user)?;
    let list = m::resolve_list(user.user_id, cli).await;
    let enabled = m::validate_enabled(&body.enabled, &list)
        .map_err(|message| (StatusCode::BAD_REQUEST, Json(json!({ "error": message }))))?;
    m::store_enabled(&state.conn, user.user_id, provider, &enabled)
        .map_err(|error| err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, error))?;
    Ok(Json(json!({
        "provider": provider,
        "models": m::entries(&list, &enabled),
    })))
}
