use std::sync::Arc;

use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::Extension;
use botcore::shared::state::AppState;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use super::shared::{ensure_user, err, first_bot_id, provider_or_error, ApiError};
use crate::core::ai_accounts;
use crate::security::auth_api::types::AuthenticatedUser;

#[derive(Deserialize)]
pub struct UseForVibeBody {
    pub bot_id: Uuid,
    #[serde(default = "default_agent")]
    pub agent: String,
}

fn default_agent() -> String {
    "agentic".to_string()
}

const VIBE_AGENTS: [&str; 3] = ["reasoning", "agentic", "fast"];

/// POST /api/ai-accounts/:provider/use-for-vibe
///
/// Assigns a linked subscription account to one Vibe agent slot. Only the
/// provider slug and model are written; the account token is resolved per run,
/// so no credential ever lands in the shared bot configuration.
pub async fn use_for_vibe(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(provider): Path<String>,
    Json(body): Json<UseForVibeBody>,
) -> Result<Json<Value>, ApiError> {
    let provider = provider_or_error(&provider)?;
    ensure_user(&state, &user)?;
    if !VIBE_AGENTS.contains(&body.agent.as_str()) {
        return Err(err(
            StatusCode::BAD_REQUEST,
            "agent must be reasoning, agentic or fast",
        ));
    }
    let Some(linked) = ai_accounts::linked_account(&state.conn, user.user_id, provider) else {
        return Err(err(
            StatusCode::CONFLICT,
            "Connect the account before assigning it to Vibe.",
        ));
    };
    let model = ai_accounts::default_model(provider).to_string();
    let config = botcore::config::ConfigManager::new(state.conn.clone());
    let prefix = format!("vibe-llm-{}-", body.agent);
    for (key, value) in [
        (format!("{prefix}provider"), provider.to_string()),
        (format!("{prefix}model"), model.clone()),
    ] {
        config
            .set_config(&body.bot_id, &key, &value)
            .map_err(|error| {
                log::warn!("ai accounts: cannot persist {key}: {error}");
                err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Could not save the Vibe assignment",
                )
            })?;
    }
    // Remove any credential/url from a previous provider in this slot.
    for key in [format!("{prefix}url"), format!("{prefix}key")] {
        if let Err(error) = config.set_config(&body.bot_id, &key, "") {
            log::warn!("ai accounts: cannot clear {key}: {error}");
        }
    }
    Ok(Json(json!({
        "success": true,
        "provider": provider,
        "model": model,
        "agent": body.agent,
        "account_email": linked.account_email,
        "plan": linked.plan,
    })))
}

/// POST /api/ai-accounts/:provider/use-for-chat
///
/// Selects a linked subscription as the bot's chat provider. Only provider and
/// model are written; any previous HTTP endpoint/key is cleared so the request
/// cannot accidentally use a different account.
pub async fn use_for_chat(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(provider): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let provider = provider_or_error(&provider)?;
    ensure_user(&state, &user)?;
    let Some(_linked) = ai_accounts::linked_account(&state.conn, user.user_id, provider) else {
        return Err(err(
            StatusCode::CONFLICT,
            "Connect the account before selecting it for chat.",
        ));
    };
    let bot_id = first_bot_id(&state)?;
    let model = ai_accounts::default_model(provider).to_string();
    let config = botcore::config::ConfigManager::new(state.conn.clone());
    for (key, value) in [
        ("llm-provider".to_string(), provider.to_string()),
        ("llm-model".to_string(), model.clone()),
    ] {
        config.set_config(&bot_id, &key, &value).map_err(|error| {
            log::warn!("ai accounts: cannot persist {key}: {error}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Could not select the chat provider",
            )
        })?;
    }
    for key in [
        "llm-url",
        "llm-key",
        "llm-model-fast",
        "llm-model-balanced",
        "llm-model-quality",
    ] {
        if let Err(error) = config.set_config(&bot_id, key, "") {
            log::warn!("ai accounts: cannot clear {key}: {error}");
        }
    }
    Ok(Json(json!({
        "success": true,
        "provider": provider,
        "model": model,
        "bot_id": bot_id,
    })))
}
