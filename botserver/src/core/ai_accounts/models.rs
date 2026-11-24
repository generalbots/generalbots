//! Which model a subscription agent answers with, and which of its models
//! the person switched on for the chat menu.
//!
//! The enabled list is stored per user and provider in
//! `user_llm_accounts.metadata.enabled_models`; with nothing stored only the
//! CLI default is enabled.

use botagent::cli_models::{self, Cli, CliModel, DEFAULT_ID};
use diesel::prelude::*;
use diesel::sql_types::{Nullable, Text, Uuid as DieselUuid};
use serde_json::{json, Value};
use uuid::Uuid;

use botcore::shared::utils::DbPool;

use super::{normalize_provider, provider_label, ANTIGRAVITY, CLAUDE_CODE, CODEX_CLI};

/// The CLI behind a subscription provider; `None` for a provider that does
/// not run a CLI in the VM.
pub fn cli_for(provider: &str) -> Option<Cli> {
    match normalize_provider(provider)? {
        CLAUDE_CODE => Some(Cli::ClaudeCode),
        CODEX_CLI => Some(Cli::Codex),
        ANTIGRAVITY => Some(Cli::Antigravity),
        _ => None,
    }
}

/// The agents whose model can be chosen, in menu order.
pub const CLI_PROVIDERS: [&str; 3] = [CLAUDE_CODE, CODEX_CLI, ANTIGRAVITY];

/// Splits a chat menu value `sub:<provider>` or `sub:<provider>/<model>`.
/// An empty model after the slash means the provider alone.
pub fn parse_choice(preference: &str) -> Option<(&str, Option<&str>)> {
    let rest = preference.trim().strip_prefix("sub:")?;
    let (provider, model) = match rest.split_once('/') {
        Some((provider, model)) => (provider.trim(), Some(model.trim()).filter(|m| !m.is_empty())),
        None => (rest.trim(), None),
    };
    (!provider.is_empty()).then_some((provider, model))
}

/// The model to run when `model` is in the provider's list, else an error
/// the person can act on.
pub fn check_model(provider: &str, model: &str, list: &[CliModel]) -> Result<String, String> {
    list.iter()
        .find(|m| m.id == model)
        .map(|m| m.id.clone())
        .ok_or_else(|| {
            let label = normalize_provider(provider).map_or("AI", provider_label);
            format!(
                "The model \u{201c}{model}\u{201d} is not available for {label}. Choose another model in the chat menu."
            )
        })
}

/// The ids to store for a PUT: every id must be in the list; the result
/// follows the list's order without duplicates.
pub fn validate_enabled(requested: &[String], list: &[CliModel]) -> Result<Vec<String>, String> {
    if let Some(unknown) = requested.iter().find(|id| !list.iter().any(|m| &m.id == *id)) {
        return Err(format!("unknown model: {unknown}"));
    }
    Ok(list
        .iter()
        .filter(|m| requested.contains(&m.id))
        .map(|m| m.id.clone())
        .collect())
}

/// The enabled ids: what was stored, else only the CLI default.
pub fn effective_enabled(stored: Option<Vec<String>>) -> Vec<String> {
    stored.unwrap_or_else(|| vec![DEFAULT_ID.to_string()])
}

/// The GET body's `models` entries.
pub fn entries(list: &[CliModel], enabled: &[String]) -> Vec<Value> {
    list.iter()
        .map(|m| {
            json!({
                "id": m.id,
                "label": m.label,
                "enabled": enabled.contains(&m.id),
                "default": m.id == DEFAULT_ID,
            })
        })
        .collect()
}

/// The chat menu's models: the enabled ids with a label from `known`; an id
/// the list does not know (a live model while the VM sleeps) keeps its id.
pub fn menu_models(known: &[CliModel], enabled: &[String]) -> Vec<Value> {
    let mut ordered: Vec<&String> = known
        .iter()
        .filter(|m| enabled.contains(&m.id))
        .map(|m| &m.id)
        .collect();
    ordered.extend(enabled.iter().filter(|id| !known.iter().any(|m| &m.id == *id)));
    ordered
        .into_iter()
        .map(|id| {
            let label = known.iter().find(|m| &m.id == id).map_or(id.as_str(), |m| m.label.as_str());
            json!({ "id": id, "label": label })
        })
        .collect()
}

/// The stored enabled list, `None` when the person never saved one.
pub fn load_enabled(pool: &DbPool, user_id: Uuid, provider: &str) -> Option<Vec<String>> {
    #[derive(QueryableByName)]
    struct Row {
        #[diesel(sql_type = Nullable<Text>)]
        enabled: Option<String>,
    }
    let mut conn = pool.get().ok()?;
    let row = diesel::sql_query(
        "SELECT (metadata->'enabled_models')::text AS enabled FROM user_llm_accounts \
         WHERE user_id = $1 AND provider = $2 LIMIT 1",
    )
    .bind::<DieselUuid, _>(user_id)
    .bind::<Text, _>(provider)
    .load::<Row>(&mut conn)
    .ok()?
    .into_iter()
    .next()?;
    serde_json::from_str::<Vec<String>>(&row.enabled?).ok()
}

/// Stores the enabled list without touching the account's status.
pub fn store_enabled(pool: &DbPool, user_id: Uuid, provider: &str, enabled: &[String]) -> Result<(), String> {
    let value = serde_json::to_string(enabled).map_err(|error| error.to_string())?;
    let mut conn = pool.get().map_err(|error| error.to_string())?;
    diesel::sql_query(
        "INSERT INTO user_llm_accounts (user_id, provider, metadata, updated_at) \
         VALUES ($1, $2, jsonb_build_object('enabled_models', $3::jsonb), NOW()) \
         ON CONFLICT (user_id, provider) DO UPDATE SET \
           metadata = user_llm_accounts.metadata || jsonb_build_object('enabled_models', $3::jsonb), \
           updated_at = NOW()",
    )
    .bind::<DieselUuid, _>(user_id)
    .bind::<Text, _>(provider)
    .bind::<Text, _>(value)
    .execute(&mut conn)
    .map_err(|error| error.to_string())?;
    Ok(())
}

/// The user's VM name, when one exists (never provisions one).
pub fn vm_name(user_id: Uuid) -> Option<String> {
    let service = crate::core::bot::agent_vm_hook::service()?;
    botagent::vm::get_user_vm(service, &user_id).ok().flatten().map(|row| row.vm_name)
}

/// The provider's list, read live from the CLI when it can print one.
pub async fn resolve_list(user_id: Uuid, cli: Cli) -> Vec<CliModel> {
    let vm = vm_name(user_id);
    match crate::core::bot::agent_vm_hook::service() {
        Some(service) => cli_models::resolve(service, vm.as_deref(), cli).await,
        None => cli_models::curated(cli),
    }
}

/// The chat menu's `subscriptions`: the linked agents with their enabled
/// models. Never runs anything in the VM: labels come from the last live
/// list, else the curated one.
pub fn chat_subscriptions(pool: &DbPool, user_id: Uuid) -> Vec<Value> {
    if user_id.is_nil() {
        return Vec::new();
    }
    let vm = vm_name(user_id);
    CLI_PROVIDERS
        .iter()
        .filter_map(|provider| {
            super::linked_account(pool, user_id, provider)?;
            let cli = cli_for(provider)?;
            let known = cli_models::known(vm.as_deref(), cli);
            let enabled = effective_enabled(load_enabled(pool, user_id, provider));
            Some(json!({
                "provider": provider,
                "label": provider_label(provider),
                "linked": true,
                "models": menu_models(&known, &enabled),
            }))
        })
        .collect()
}

/// The model a `sub:<provider>/<model>` turn runs with, checked against the
/// provider's list.
pub async fn choose_model(user_id: Uuid, provider: &str, model: &str) -> Result<String, String> {
    let cli = cli_for(provider).ok_or_else(|| {
        let label = normalize_provider(provider).map_or("AI", provider_label);
        format!("{label} does not offer a choice of model.")
    })?;
    // A model the last list (or the curated one) already offers needs no
    // round trip to the VM (about 2 s through WSL once the list is stale):
    // the turn goes on and a stale list is read again beside it.
    let vm = vm_name(user_id);
    if let Ok(chosen) = check_model(provider, model, &cli_models::known(vm.as_deref(), cli)) {
        refresh_if_stale(vm, cli);
        return Ok(chosen);
    }
    let list = resolve_list(user_id, cli).await;
    check_model(provider, model, &list)
}

/// Reads the live list in the background when the cached one has expired.
fn refresh_if_stale(vm: Option<String>, cli: Cli) {
    let (Some(vm), Some(service)) = (vm, crate::core::bot::agent_vm_hook::service()) else {
        return;
    };
    if cli_models::cached(&vm, cli).is_some() {
        return;
    }
    tokio::spawn(async move {
        cli_models::live(service, &vm, cli).await;
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list() -> Vec<CliModel> {
        cli_models::curated(Cli::ClaudeCode)
    }

    #[tokio::test]
    async fn a_listed_model_is_chosen_without_reading_the_list_again() {
        assert_eq!(choose_model(Uuid::new_v4(), CODEX_CLI, DEFAULT_ID).await, Ok(DEFAULT_ID.to_string()));
        assert_eq!(choose_model(Uuid::new_v4(), CLAUDE_CODE, "opus").await, Ok("opus".to_string()));
        assert!(choose_model(Uuid::new_v4(), CODEX_CLI, "not-a-model").await.is_err());
    }

    #[test]
    fn a_menu_value_names_the_provider_and_maybe_a_model() {
        assert_eq!(parse_choice("sub:codex-cli"), Some(("codex-cli", None)));
        assert_eq!(parse_choice(" sub:claude-code/opus "), Some(("claude-code", Some("opus"))));
        assert_eq!(parse_choice("sub:codex-cli/gpt-6.2-sol"), Some(("codex-cli", Some("gpt-6.2-sol"))));
        assert_eq!(parse_choice("sub:antigravity/"), Some(("antigravity", None)));
        assert_eq!(parse_choice("sub:"), None);
        assert_eq!(parse_choice("acct:x/y"), None);
    }

    #[test]
    fn a_model_outside_the_list_is_refused_clearly() {
        assert_eq!(check_model("claude-code", "opus", &list()), Ok("opus".to_string()));
        let error = check_model("claude-code", "gpt-4o", &list()).unwrap_err();
        assert!(error.contains("gpt-4o") && error.contains("Claude Code"));
    }

    #[test]
    fn enabling_keeps_list_order_and_refuses_unknown_ids() {
        let requested = vec!["haiku".to_string(), "default".to_string(), "haiku".to_string()];
        assert_eq!(validate_enabled(&requested, &list()), Ok(vec!["default".to_string(), "haiku".to_string()]));
        assert!(validate_enabled(&["nope".to_string()], &list()).is_err());
        assert_eq!(validate_enabled(&[], &list()), Ok(Vec::new()));
    }

    #[test]
    fn only_the_cli_default_is_enabled_until_the_person_saves() {
        assert_eq!(effective_enabled(None), vec![DEFAULT_ID.to_string()]);
        assert_eq!(effective_enabled(Some(Vec::new())), Vec::<String>::new());
        let rows = entries(&list(), &effective_enabled(None));
        assert_eq!(rows[0]["id"], "default");
        assert_eq!(rows[0]["enabled"], true);
        assert_eq!(rows[0]["default"], true);
        assert!(rows[1..].iter().all(|r| r["enabled"] == false && r["default"] == false));
    }

    #[test]
    fn the_menu_lists_enabled_models_and_keeps_unknown_ids() {
        let enabled = vec!["live-only".to_string(), "sonnet".to_string()];
        let menu = menu_models(&list(), &enabled);
        assert_eq!(menu[0], json!({ "id": "sonnet", "label": "Sonnet (latest)" }));
        assert_eq!(menu[1], json!({ "id": "live-only", "label": "live-only" }));
    }

    #[test]
    fn only_cli_agents_have_a_model_list() {
        assert_eq!(cli_for("claude"), Some(Cli::ClaudeCode));
        assert_eq!(cli_for("codex"), Some(Cli::Codex));
        assert_eq!(cli_for("antigravity"), Some(Cli::Antigravity));
        assert_eq!(cli_for("chatgpt-plan"), None);
    }
}
