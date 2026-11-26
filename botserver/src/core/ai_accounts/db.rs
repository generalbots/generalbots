use super::*;
use botcore::shared::utils::DbPool;
use diesel::prelude::*;
use diesel::sql_types::{Bool, Nullable, Text, Uuid as DieselUuid};
use serde_json::{json, Value};
use uuid::Uuid;

fn status_for(row: Option<&Value>) -> String {
    row.and_then(|value| value.get("status"))
        .and_then(Value::as_str)
        .unwrap_or("unlinked")
        .to_string()
}

/// Ensures the suite identity has a `users` row before an AI-account FK is
/// written. The desktop login fallback mints a session from
/// `admin-credentials.json` without inserting the row; without this the first
/// connect fails with a foreign-key violation.
pub fn ensure_user(pool: &DbPool, user_id: Uuid, email: Option<&str>) -> Result<(), String> {
    if user_id.is_nil() {
        return Err("Anonymous sessions cannot own an AI account.".to_string());
    }
    let email = email
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("admin@localhost");
    let is_bootstrap_admin =
        crate::main_module::directory_setup::is_bootstrap_admin_identity(user_id, Some(email));
    let mut conn = pool.get().map_err(|error| error.to_string())?;
    diesel::sql_query(
        "INSERT INTO users (id, username, email, password_hash, is_admin) \
         VALUES ($1, $2, $2, '', $3) \
         ON CONFLICT DO NOTHING",
    )
    .bind::<DieselUuid, _>(user_id)
    .bind::<Text, _>(email)
    .bind::<Bool, _>(is_bootstrap_admin)
    .execute(&mut conn)
    .map_err(|error| error.to_string())?;

    // #1839 — the row may already exist from an earlier connect (inserted with
    // `is_admin = false`). Only ever promote; never demote an existing admin.
    if is_bootstrap_admin {
        let _ = diesel::sql_query(
            "UPDATE users SET is_admin = TRUE, is_active = TRUE, updated_at = NOW() \
             WHERE id = $1 OR lower(email) = lower($2)",
        )
        .bind::<DieselUuid, _>(user_id)
        .bind::<Text, _>(email)
        .execute(&mut conn);
    }
    Ok(())
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
    matches!(
        normalize_provider(provider),
        Some(CLAUDE_CODE) | Some(CHATGPT_PLAN) | Some(CODEX_CLI) | Some(ANTIGRAVITY)
    )
}

/// Model used when the bot has no explicit subscription model.
pub fn default_model(provider: &str) -> &'static str {
    match normalize_provider(provider) {
        Some(CLAUDE_CODE) => "claude-sonnet-4-5",
        Some(CHATGPT_PLAN) => botllm::openai_responses::DEFAULT_MODEL,
        Some(CODEX_CLI) => "codex-default",
        Some(ANTIGRAVITY) => "antigravity-default",
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

pub fn provider_label(provider: &str) -> &'static str {
    match provider {
        CLAUDE_CODE => "Claude Code",
        CHATGPT_PLAN => "ChatGPT plan",
        CODEX_CLI => "Codex",
        ANTIGRAVITY => "Antigravity",
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
        .map_or_else(|_| { log::warn!("defaulting after error at {}:{}", file!(), line!()); Default::default() }, |v| v);
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
        .map_or_else(|_| { log::warn!("defaulting after error at {}:{}", file!(), line!()); Default::default() }, |v| v)
        .into_iter()
        .next();
        return status_for(row.as_ref().map(|r| json!({ "status": r.status })).as_ref())
            == "linked";
    }
    false
}
