//! Per-user AI accounts (#1654).
//!
//! Only status and display metadata live in Postgres. Claude credentials stay
//! inside the user's VM; ChatGPT tokens live in Vault at `vault_path`.

use diesel::prelude::*;
use diesel::sql_types::{Bool, Nullable, Text, Uuid as DieselUuid};
use serde_json::{json, Value};
use uuid::Uuid;

use botcore::shared::utils::DbPool;

pub const CLAUDE_CODE: &str = "claude-code";
pub const CHATGPT_PLAN: &str = "chatgpt-plan";
pub const PROVIDERS: [&str; 2] = [CLAUDE_CODE, CHATGPT_PLAN];

/// Where a resolved LLM configuration came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedLlm {
    pub provider: String,
    pub model: String,
    pub source: &'static str,
}

/// User account → bot/branch config → global default (#1654).
///
/// The first non-empty candidate wins; a subscription account must never fall
/// through to another user's bot token by accident, so callers pass the user's
/// row only when it is `linked`.
pub fn choose_llm(
    user: Option<(&str, &str)>,
    bot: Option<(&str, &str)>,
    global: Option<(&str, &str)>,
) -> Option<ResolvedLlm> {
    for (candidate, source) in [(user, "user"), (bot, "bot"), (global, "global")] {
        if let Some((provider, model)) = candidate {
            let provider = provider.trim();
            if !provider.is_empty() {
                return Some(ResolvedLlm {
                    provider: provider.to_string(),
                    model: model.trim().to_string(),
                    source,
                });
            }
        }
    }
    None
}

/// Returns the canonical provider slug, or `None` for an unknown provider.
pub fn normalize_provider(value: &str) -> Option<&'static str> {
    match value.trim().to_ascii_lowercase().as_str() {
        "claude" | "claude-code" => Some(CLAUDE_CODE),
        "chatgpt" | "chatgpt-plan" | "openai-plan" => Some(CHATGPT_PLAN),
        _ => None,
    }
}

mod db;
pub use db::{
    choose_for_request, ensure_user, is_linked, linked_account, list_accounts, mark_linked,
    update_status, LinkedAccount,
};

#[cfg(test)]
mod tests;
