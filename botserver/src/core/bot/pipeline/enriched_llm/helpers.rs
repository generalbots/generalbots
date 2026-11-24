use super::*;
use botcore::shared::state::AppState;
use std::sync::Arc;
use uuid::Uuid;

/// The address the turn's model will be called at: the account picked in the
/// chat's model menu, else the bot's `llm-url` (Vault), else the `LLM_URL`
/// override. Used to decide whether attached files can be sent as images.
pub(crate) async fn turn_llm_url(
    state: &Arc<AppState>,
    bot_uuid: Uuid,
    model_preference: &str,
) -> String {
    if let Some(choice) = botsettings::model_config::accounts::resolve_choice(model_preference).await {
        return choice.url;
    }
    let from_vault = botcore::config::ConfigManager::new(state.conn.clone())
        .get_config(&bot_uuid, "llm-url", Some(""))
        .unwrap_or_else(|e| {
            log::warn!("BOT {bot_uuid} llm-url config unavailable; using env override: {e}");
            String::new()
        });
    if !from_vault.trim().is_empty() {
        return from_vault;
    }
    std::env::var("LLM_URL").unwrap_or_else(|_| {
        log::debug!("LLM_URL is not set; no global model override");
        String::new()
    })
}

/// Resolves the per-bot LLM provider (Vault config → env override), logging
/// the host/model pair. Returns None when llm-url is empty (global fallback).
///
/// A subscription provider (`claude-code` / `chatgpt-plan`) never falls back
/// to the bot or global API key: it resolves the requesting user's linked
/// account and returns an actionable error when the account is missing.
pub(crate) async fn resolve_bot_llm_provider(
    state: &Arc<AppState>,
    bot_uuid: Uuid,
    user_id: Uuid,
    session_id: Uuid,
    model_preference: &str,
) -> Result<Option<(Arc<dyn botlib::traits::LLMProvider>, String, String)>, String> {
    use botcore::config::ConfigManager;
    let cfg = ConfigManager::new(state.conn.clone());
    let mut llm_url = cfg
        .get_config(&bot_uuid, "llm-url", Some(""))
        .map_or_else(|_| { log::warn!("defaulting after error at {}:{}", file!(), line!()); Default::default() }, |v| v);
    let mut llm_key = cfg
        .get_config(&bot_uuid, "llm-key", Some(""))
        .map_or_else(|_| { log::warn!("defaulting after error at {}:{}", file!(), line!()); Default::default() }, |v| v);
    let configured_model = cfg
        .get_config(&bot_uuid, "llm-model", Some(""))
        .map_or_else(|_| { log::warn!("defaulting after error at {}:{}", file!(), line!()); Default::default() }, |v| v);
    let configured_provider = cfg
        .get_config(&bot_uuid, "llm-provider", Some(""))
        .map_or_else(|_| { log::warn!("defaulting after error at {}:{}", file!(), line!()); Default::default() }, |v| v);

    // A connected subscription agent picked in the chat's model menu answers
    // this turn (`sub:<provider>`); the bot's own default is left untouched.
    #[cfg(feature = "agent-vm")]
    let agent_choice = model_preference
        .trim()
        .strip_prefix("sub:")
        .filter(|agent| crate::core::ai_accounts::is_subscription_provider(*agent))
        .map(|agent| {
            (
                agent.to_string(),
                crate::core::ai_accounts::default_model(agent).to_string(),
            )
        });
    #[cfg(not(feature = "agent-vm"))]
    let agent_choice: Option<(String, String)> = None;

    let (llm_provider, mut llm_model) = match agent_choice {
        Some((provider, model)) => (provider, model),
        None => (configured_provider, configured_model),
    };

    // A model picked in the chat's menu from a provider account (Settings >
    // Modelo): that account's address, key and model answer this turn.
    if let Some(choice) = botsettings::model_config::accounts::resolve_choice(model_preference).await {
        let host = url::Url::parse(&choice.url)
            .ok()
            .and_then(|u| u.host_str().map(str::to_string))
            .unwrap_or_else(|| "<unparseable>".to_string());
        botlog::safe_info!("BOT {bot_uuid} account model: host=[{host}] model=[{}]", choice.model);
        let provider = crate::llm::provider_factory::create_llm_provider_from_url(
            &choice.url,
            Some(choice.model.clone()),
            None,
            None,
        );
        return Ok(Some((
            Arc::new(crate::llm::BotlibLLMProviderWrapper::new(
                provider,
                choice.model.clone(),
                choice.key.clone(),
            )) as Arc<dyn botlib::traits::LLMProvider>,
            choice.key,
            choice.model,
        )));
    }

    // #1652/#1653/#1654 — subscription accounts are resolved per user before
    // any HTTP provider is built. `choose_for_request` refuses to fall back
    // when the requested provider is a subscription and no account is linked.
    #[cfg(not(feature = "agent-vm"))]
    if matches!(llm_provider.trim(), "claude-code" | "chatgpt-plan" | "codex-cli" | "antigravity") {
        botlog::safe_info!(
            "BOT {bot_uuid} user {user_id} session {session_id}: provider [{llm_provider}] needs the agent workspace, which this build does not include"
        );
        return Err(
            "Subscription accounts are not available in this edition. Choose another model in Settings."
                .to_string(),
        );
    }
    #[cfg(feature = "agent-vm")]
    if crate::core::ai_accounts::is_subscription_provider(&llm_provider) {
        let linked = crate::core::ai_accounts::linked_account(&state.conn, user_id, &llm_provider);
        let resolved = crate::core::ai_accounts::choose_for_request(
            Some(&llm_provider),
            linked.as_ref(),
            Some((llm_provider.as_str(), llm_model.as_str())),
            None,
        )?;
        let Some(resolved) = resolved else {
            return Ok(None);
        };
        // #2239 — subscription providers run inside the user's agent VM; refuse
        // to wake or provision them when Agent/VM is disabled for this conversation.
        if (resolved.provider == crate::core::ai_accounts::CLAUDE_CODE
            || resolved.provider == crate::core::ai_accounts::CODEX_CLI
            || resolved.provider == crate::core::ai_accounts::ANTIGRAVITY)
            && !super::super::agent_turn::is_enabled(session_id)
        {
            return Err(super::super::agent_turn::DISABLED_MESSAGE.to_string());
        }
        let provider =
            crate::llm::subscription::build(state, user_id, session_id, bot_uuid, &resolved)
                .await?;
        botlog::safe_info!(
            "BOT {bot_uuid} subscription LLM: provider=[{}] model=[{}] source=[{}]",
            resolved.provider,
            resolved.model,
            resolved.source
        );
        return Ok(Some((provider, String::new(), resolved.model)));
    }

    // #1637 — a tier is a real model, not a tone. The picker's value
    // selects `llm-model-<tier>`; when the bot has no model configured for
    // that tier the request keeps the default model, and `auto` always
    // means the default (unchanged behaviour).
    let tier = model_preference.trim().to_lowercase();
    if matches!(tier.as_str(), "fast" | "balanced" | "quality") {
        let tier_key = format!("llm-model-{tier}");
        let tier_model = cfg
            .get_config(&bot_uuid, &tier_key, Some(""))
            .map_or_else(|_| { log::warn!("defaulting after error at {}:{}", file!(), line!()); Default::default() }, |v| v);
        if !tier_model.is_empty() {
            botlog::safe_info!("BOT {bot_uuid} model tier '{tier}' -> model=[{tier_model}]");
            llm_model = tier_model;
        } else {
            botlog::safe_info!(
                "BOT {bot_uuid} model tier '{tier}' is not configured; using the default model [{llm_model}]"
            );
        }
    }
    if let Ok(val) = std::env::var("LLM_URL") {
        if !val.is_empty() {
            llm_url = val;
        }
    }
    if let Ok(val) = std::env::var("LLM_KEY") {
        if !val.is_empty() {
            llm_key = val;
        }
    }
    if let Ok(val) = std::env::var("LLM_MODEL") {
        if !val.is_empty() {
            llm_model = val;
        }
    }
    let llm_host = url::Url::parse(&llm_url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_string()))
        .unwrap_or_else(|| "<unparseable>".to_string());
    botlog::safe_info!("BOT {bot_uuid} LLM CONFIG: host=[{llm_host}] key_len=[{}] model=[{llm_model}] provider=[{llm_provider}]", llm_key.len());
    if !llm_url.is_empty() {
        let explicit = if llm_provider.is_empty() {
            None
        } else {
            crate::llm::provider_factory::llm_provider_type_from_name(&llm_provider)
        };
        let provider = crate::llm::provider_factory::create_llm_provider_from_url(
            &llm_url,
            if llm_model.is_empty() {
                None
            } else {
                Some(llm_model.clone())
            },
            None,
            explicit,
        );
        Ok(Some((
            Arc::new(crate::llm::BotlibLLMProviderWrapper::new(
                provider,
                llm_model.clone(),
                llm_key.clone(),
            )) as Arc<dyn botlib::traits::LLMProvider>,
            llm_key,
            llm_model,
        )))
    } else {
        log::warn!("CRISTO BOT: llm-url is EMPTY, falling back to global provider");
        Ok(None)
    }
}

/// When the answer mode is Data/Chart, generates and sends the specialized
/// response; returns true when the pipeline has already replied.
pub(crate) async fn respond_in_answer_mode(
    sink: &dyn ChannelSink,
    state: &Arc<AppState>,
    user_text: &str,
    bot_uuid: Uuid,
    bot_name: &str,
    session_id: Uuid,
    user_id: Uuid,
) -> PipelineResult<bool> {
    #[cfg(feature = "chat")]
    let answer_mode = crate::core::bot::answer_mode::get_answer_mode(state, &session_id).await;
    #[cfg(not(feature = "chat"))]
    let answer_mode = crate::core::bot::answer_mode::AnswerMode::Default;
    if answer_mode == crate::core::bot::answer_mode::AnswerMode::Default {
        return Ok(false);
    }
    let mode_response = match answer_mode {
        crate::core::bot::answer_mode::AnswerMode::Data => {
            crate::core::bot::answer_mode::generate_data_response(
                state, user_text, bot_uuid, bot_name, session_id, user_id,
            )
            .await
        }
        crate::core::bot::answer_mode::AnswerMode::Chart => {
            crate::core::bot::answer_mode::generate_chart_response(
                state, user_text, bot_uuid, bot_name, session_id, user_id,
            )
            .await
        }
        _ => return Ok(true),
    };
    match mode_response {
        Ok(resp) => {
            let _ = sink.send_bot_response(&resp).await;
            let mut sm = state.session_manager.lock().await;
            let _ = sm.save_message(session_id, user_id, 2, &resp.content, 2);
        }
        Err(e) => {
            let err_resp = botlib::models::BotResponse::new(
                bot_uuid.to_string(),
                session_id.to_string(),
                user_id.to_string(),
                format!("<p>Error: {e}</p>"),
                "web",
            );
            let _ = sink.send_bot_response(&err_resp).await;
        }
    }
    Ok(true)
}

/// Extracts KB source info from the injected system messages for the activity
/// summary. The KB module inserts a block starting with
/// `--- Informacoes de Contexto ---` containing score and document lines.
pub(super) fn extract_kb_sources_from_messages(
    messages: &serde_json::Value,
) -> Vec<super::super::activity::KbSource> {
    let Some(arr) = messages.as_array() else {
        return Vec::new();
    };
    let mut sources = Vec::new();
    for msg in arr {
        let content = msg.get("content").and_then(|c| c.as_str()).unwrap_or("");
        if !content.contains("Contexto") && !content.contains("Context") {
            continue;
        }
        // Parse lines like: "[Score: 0.87] Document: manual.md\nContent excerpt..."
        for line in content.lines() {
            if let Some(rest) = line.strip_prefix("[Score:") {
                let score = rest
                    .split(']')
                    .next()
                    .and_then(|s| s.trim().parse::<f32>().ok())
                    .unwrap_or(0.0);
                let doc = rest
                    .split("Document:")
                    .nth(1)
                    .unwrap_or("")
                    .trim()
                    .to_string();
                let excerpt = content
                    .lines()
                    .skip_while(|l| !l.starts_with("[Score:"))
                    .nth(1)
                    .unwrap_or("")
                    .to_string();
                sources.push(super::super::activity::KbSource {
                    document: doc,
                    excerpt,
                    score,
                });
            }
        }
    }
    sources
}
