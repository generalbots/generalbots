//! Per-user AI accounts (#1654).
//!
//! Only status and display metadata live in Postgres. Claude credentials stay
//! inside the user's VM; ChatGPT tokens live in Vault at `vault_path`.

use diesel::prelude::*;
use diesel::sql_types::{Nullable, Text, Uuid as DieselUuid};
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

fn status_for(row: Option<&Value>) -> String {
    row.and_then(|value| value.get("status"))
        .and_then(Value::as_str)
        .unwrap_or("unlinked")
        .to_string()
}

/// Records a successful provider login without storing any credential.
pub fn mark_linked(
    pool: &DbPool,
    user_id: Uuid,
    provider: &str,
    account_email: Option<&str>,
    plan: Option<&str>,
    vm_name: Option<&str>,
    vault_path: Option<&str>,
) -> Result<(), String> {
    let mut conn = pool.get().map_err(|error| error.to_string())?;
    diesel::sql_query(
        "INSERT INTO user_llm_accounts \
           (user_id, provider, status, account_email, plan, vm_name, vault_path, linked_at, last_verified_at, updated_at) \
         VALUES ($1, $2, 'linked', $3, $4, $5, $6, NOW(), NOW(), NOW()) \
         ON CONFLICT (user_id, provider) DO UPDATE SET \
           status = 'linked', \
           account_email = COALESCE(EXCLUDED.account_email, user_llm_accounts.account_email), \
           plan = COALESCE(EXCLUDED.plan, user_llm_accounts.plan), \
           vm_name = COALESCE(EXCLUDED.vm_name, user_llm_accounts.vm_name), \
           vault_path = COALESCE(EXCLUDED.vault_path, user_llm_accounts.vault_path), \
           linked_at = NOW(), last_verified_at = NOW(), updated_at = NOW()",
    )
    .bind::<DieselUuid, _>(user_id)
    .bind::<Text, _>(provider)
    .bind::<Nullable<Text>, _>(account_email)
    .bind::<Nullable<Text>, _>(plan)
    .bind::<Nullable<Text>, _>(vm_name)
    .bind::<Nullable<Text>, _>(vault_path)
    .execute(&mut conn)
    .map_err(|error| error.to_string())?;
    Ok(())
}

/// Whether the provider needs a per-user subscription account (#1652/#1653).
pub fn is_subscription_provider(provider: &str) -> bool {
    matches!(normalize_provider(provider), Some(CLAUDE_CODE) | Some(CHATGPT_PLAN))
}

/// Model used when the bot has no explicit subscription model.
pub fn default_model(provider: &str) -> &'static str {
    match normalize_provider(provider) {
        Some(CLAUDE_CODE) => "claude-sonnet-4-5",
        Some(CHATGPT_PLAN) => botllm::openai_responses::DEFAULT_MODEL,
        _ => "",
    }
}

/// Display metadata for a linked subscription account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkedAccount {
    pub provider: String,
    pub account_email: Option<String>,
    pub plan: Option<String>,
    pub vm_name: Option<String>,
    pub vault_path: Option<String>,
}

/// Reads the user's `linked` row for one provider. A row in any other state is
/// deliberately ignored: an expired or revoked account must never be used.
pub fn linked_account(pool: &DbPool, user_id: Uuid, provider: &str) -> Option<LinkedAccount> {
    let provider = normalize_provider(provider)?;
    let mut conn = pool.get().ok()?;
    #[derive(QueryableByName)]
    struct Row {
        #[diesel(sql_type = Text)]
        provider: String,
        #[diesel(sql_type = Nullable<Text>)]
        account_email: Option<String>,
        #[diesel(sql_type = Nullable<Text>)]
        plan: Option<String>,
        #[diesel(sql_type = Nullable<Text>)]
        vm_name: Option<String>,
        #[diesel(sql_type = Nullable<Text>)]
        vault_path: Option<String>,
    }
    let row = diesel::sql_query(
        "SELECT provider, account_email, plan, vm_name, vault_path \
         FROM user_llm_accounts \
         WHERE user_id = $1 AND provider = $2 AND status = 'linked' LIMIT 1",
    )
    .bind::<DieselUuid, _>(user_id)
    .bind::<Text, _>(provider)
    .load::<Row>(&mut conn)
    .ok()?
    .into_iter()
    .next()?;
    Some(LinkedAccount {
        provider: row.provider,
        account_email: row.account_email,
        plan: row.plan,
        vm_name: row.vm_name,
        vault_path: row.vault_path,
    })
}

/// Pure decision used by both the chat pipeline and the Vibe loop.
///
/// A requested subscription provider never falls back to the bot or global
/// token: without a linked account the caller gets an actionable error.
pub fn choose_for_request(
    requested: Option<&str>,
    linked: Option<&LinkedAccount>,
    bot: Option<(&str, &str)>,
    global: Option<(&str, &str)>,
) -> Result<Option<ResolvedLlm>, String> {
    if let Some(provider) = requested.filter(|value| is_subscription_provider(value)) {
        let canonical = normalize_provider(provider).unwrap_or(provider);
        let account = linked
            .filter(|row| row.provider == canonical)
            .ok_or_else(|| {
                format!(
                    "Connect your {} account in Settings \u{2192} AI accounts before using this model.",
                    provider_label(canonical)
                )
            })?;
        debug_assert_eq!(account.provider, canonical);
        let model = bot
            .map(|(_, model)| model.trim())
            .filter(|model| !model.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| default_model(canonical).to_string());
        return Ok(Some(ResolvedLlm {
            provider: canonical.to_string(),
            model,
            source: "user",
        }));
    }
    Ok(choose_llm(None, bot, global))
}

fn provider_label(provider: &str) -> &'static str {
    match provider {
        CLAUDE_CODE => "Claude Code",
        CHATGPT_PLAN => "ChatGPT plan",
        _ => "AI",
    }
}


pub fn list_accounts(pool: &DbPool, user_id: Uuid) -> Vec<Value> {
    let mut rows: Vec<Value> = Vec::new();
    if let Ok(mut conn) = pool.get() {
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type = Text)]
            provider: String,
            #[diesel(sql_type = Text)]
            status: String,
            #[diesel(sql_type = Nullable<Text>)]
            account_email: Option<String>,
            #[diesel(sql_type = Nullable<Text>)]
            plan: Option<String>,
            #[diesel(sql_type = Nullable<Text>)]
            vm_name: Option<String>,
            #[diesel(sql_type = Nullable<Text>)]
            vault_path: Option<String>,
            #[diesel(sql_type = Nullable<Text>)]
            last_verified_at: Option<String>,
        }
        let loaded = diesel::sql_query(
            "SELECT provider, status, account_email, plan, vm_name, vault_path, \
                    last_verified_at::text AS last_verified_at \
             FROM user_llm_accounts WHERE user_id = $1",
        )
        .bind::<DieselUuid, _>(user_id)
        .load::<Row>(&mut conn)
        .unwrap_or_default();
        for row in loaded {
            rows.push(json!({
                "provider": row.provider,
                "status": row.status,
                "account_email": row.account_email,
                "plan": row.plan,
                "vm_name": row.vm_name,
                "vault_path": row.vault_path,
                "last_verified_at": row.last_verified_at,
            }));
        }
    }
    PROVIDERS
        .iter()
        .filter(|provider| {
            !rows
                .iter()
                .any(|row| row.get("provider").and_then(Value::as_str) == Some(**provider))
        })
        .map(|provider| {
            json!({
                "provider": provider,
                "status": "unlinked",
                "account_email": null,
                "plan": null,
                "vm_name": null,
                "vault_path": null,
                "last_verified_at": null,
            })
        })
        .collect::<Vec<_>>()
        .into_iter()
        .chain(rows)
        .collect()
}

pub fn update_status(
    pool: &DbPool,
    user_id: Uuid,
    provider: &str,
    status: &str,
    account_email: Option<&str>,
    plan: Option<&str>,
    vm_name: Option<&str>,
) -> Result<(), String> {
    let mut conn = pool.get().map_err(|error| error.to_string())?;
    diesel::sql_query(
        "INSERT INTO user_llm_accounts (user_id, provider, status, account_email, plan, vm_name, updated_at) \
         VALUES ($1, $2, $3, $4, $5, $6, NOW()) \
         ON CONFLICT (user_id, provider) DO UPDATE SET \
           status = EXCLUDED.status, \
           account_email = COALESCE(EXCLUDED.account_email, user_llm_accounts.account_email), \
           plan = COALESCE(EXCLUDED.plan, user_llm_accounts.plan), \
           vm_name = COALESCE(EXCLUDED.vm_name, user_llm_accounts.vm_name), \
           linked_at = CASE WHEN EXCLUDED.status = 'linked' THEN NOW() ELSE user_llm_accounts.linked_at END, \
           updated_at = NOW()",
    )
    .bind::<DieselUuid, _>(user_id)
    .bind::<Text, _>(provider)
    .bind::<Text, _>(status)
    .bind::<Nullable<Text>, _>(account_email)
    .bind::<Nullable<Text>, _>(plan)
    .bind::<Nullable<Text>, _>(vm_name)
    .execute(&mut conn)
    .map_err(|error| error.to_string())?;
    Ok(())
}

/// Whether the user has a usable linked account for this provider.
pub fn is_linked(pool: &DbPool, user_id: Uuid, provider: &str) -> bool {
    if let Ok(mut conn) = pool.get() {
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type = Text)]
            status: String,
        }
        let row = diesel::sql_query(
            "SELECT status FROM user_llm_accounts WHERE user_id = $1 AND provider = $2 LIMIT 1",
        )
        .bind::<DieselUuid, _>(user_id)
        .bind::<Text, _>(provider)
        .load::<Row>(&mut conn)
        .unwrap_or_default()
        .into_iter()
        .next();
        return status_for(row.as_ref().map(|r| json!({ "status": r.status })).as_ref()) == "linked";
    }
    false
}

#[cfg(test)]
mod tests {
    use super::{choose_llm, normalize_provider, ResolvedLlm};

    #[test]
    fn user_account_wins_over_bot_and_global() {
        let resolved = choose_llm(
            Some(("claude-code", "sonnet")),
            Some(("openai", "gpt-4o")),
            Some(("kilo", "dots")),
        )
        .expect("resolved");
        assert_eq!(
            resolved,
            ResolvedLlm {
                provider: "claude-code".into(),
                model: "sonnet".into(),
                source: "user"
            }
        );
    }

    #[test]
    fn bot_config_wins_when_the_user_has_no_account() {
        let resolved = choose_llm(None, Some(("openai", "gpt-4o")), Some(("kilo", "dots")))
            .expect("resolved");
        assert_eq!(resolved.source, "bot");
        assert_eq!(resolved.provider, "openai");
    }

    #[test]
    fn global_is_the_last_resort_and_empty_values_are_skipped() {
        let resolved = choose_llm(None, Some(("", "")), Some(("kilo", "dots"))).expect("resolved");
        assert_eq!(resolved.source, "global");
        assert!(choose_llm(None, None, None).is_none());
    }

    #[test]
    fn provider_aliases_are_canonicalized() {
        assert_eq!(normalize_provider("Claude"), Some("claude-code"));
        assert_eq!(normalize_provider("chatgpt"), Some("chatgpt-plan"));
        assert_eq!(normalize_provider("openai-plan"), Some("chatgpt-plan"));
        assert_eq!(normalize_provider("kilo"), None);
    }

    #[test]
    fn a_subscription_without_a_linked_account_is_an_error() {
        let error = super::choose_for_request(
            Some("claude-code"),
            None,
            Some(("openai", "gpt-4o")),
            Some(("kilo", "dots")),
        )
        .expect_err("must not fall back");
        assert!(error.contains("Connect your Claude Code account"));
    }

    #[test]
    fn a_linked_account_wins_and_keeps_the_bot_model() {
        let account = super::LinkedAccount {
            provider: "chatgpt-plan".to_string(),
            account_email: Some("user@example.com".to_string()),
            plan: Some("plus".to_string()),
            vm_name: None,
            vault_path: Some("gbo/users/u/chatgpt-plan".to_string()),
        };
        let resolved = super::choose_for_request(
            Some("chatgpt-plan"),
            Some(&account),
            Some(("chatgpt-plan", "gpt-5-codex")),
            Some(("kilo", "dots")),
        )
        .expect("resolved")
        .expect("some");
        assert_eq!(resolved.provider, "chatgpt-plan");
        assert_eq!(resolved.model, "gpt-5-codex");
        assert_eq!(resolved.source, "user");
    }

    #[test]
    fn default_models_are_provider_specific() {
        assert_eq!(super::default_model("claude"), "claude-sonnet-4-5");
        assert_eq!(super::default_model("chatgpt"), "gpt-5-codex");
        assert!(super::is_subscription_provider("claude-code"));
        assert!(!super::is_subscription_provider("openai"));
    }
}
