use std::sync::Arc;

use botcore::shared::state::AppState;
use uuid::Uuid;

use super::sink::ChannelSink;
use super::types::{PipelineResult, SessionIdentity};

/// Enriches the LLM prompt with KB context, memory recall and integration
/// mention blocks, saves the user turn, and streams the LLM response.

/// The persisted conversation shows the user's own words; the agent-only
/// `[User attached a file stored at inbox/…]` block must stay in the prompt
/// copy only (replay used to show it inside the user's bubble).
fn display_user_text(text: &str) -> String {
    const MARKER: &str = "[User attached a file stored at inbox/";
    let Some(position) = text.find(MARKER) else {
        return text.to_string();
    };
    let before = text[..position].trim();
    if !before.is_empty() {
        return before.to_string();
    }
    let file_name = text[position + MARKER.len()..]
        .split(';')
        .next()
        .unwrap_or("")
        .trim();
    if file_name.is_empty() {
        "📎 attachment".to_string()
    } else {
        format!("📎 {file_name}")
    }
}

pub(super) async fn build_and_stream(
    identity: &SessionIdentity<'_>,
    rx: &mut tokio::sync::mpsc::Receiver<botlib::models::BotResponse>,
    messages: &[serde_json::Value],
    user_text: &str,
    #[cfg(feature = "integrations")] mentions: &[botlib::models::MentionRef],
    is_switcher_replay: bool,
    model_preference: &str,
) -> PipelineResult<()> {
    let SessionIdentity {
        state,
        bot_uuid,
        session_id,
        user_id,
        bot_name,
        ..
    } = *identity;
    let user_query = user_text.to_string();
    let mut messages_val = serde_json::Value::Array(messages.to_vec());
    if tokio::time::timeout(
        std::time::Duration::from_secs(30),
        super::kb::inject_kb(
            state,
            bot_uuid,
            session_id,
            user_id,
            bot_name,
            &user_query,
            &mut messages_val,
        ),
    )
    .await
    .is_err()
    {
        botlog::safe_warn!("ws_handler: inject_kb_context TIMEOUT after 30s for session {session_id}");
    }

    #[cfg(feature = "memory-os")]
    crate::core::bot::memory_hook::inject_recall(
        state,
        user_id,
        &user_query,
        &mut messages_val,
    );

    // Mention system blocks land right before the user turn so the LLM sees
    // the advertised integration surface adjacent to the request it enables.
    #[cfg(feature = "integrations")]
    if !mentions.is_empty() {
        super::mentions::append_integration_mention_blocks(
            state,
            bot_uuid,
            user_id,
            mentions,
            &mut messages_val,
        )
        .await;
    }

    if !is_switcher_replay {
        let display_text = display_user_text(user_text);
        let mut sm = state.session_manager.lock().await;
        let _ = sm.save_message(session_id, user_id, 1, &display_text, 1);
        // Name the conversation as soon as it starts, from the user's own
        // words. Only runs while the title is still a placeholder, so a later
        // rename (manual or automatic) is preserved.
        let _ = sm.ensure_title_from_message(session_id, user_id, &display_text);
    }

    if let Some(arr) = messages_val.as_array_mut() {
        arr.push(serde_json::json!({
            "role": "user",
            "content": user_text
        }));
    }

    let run = super::event_sink::RunContext::new(&session_id.to_string(), "chat")
        .with_state(state.clone())
        .with_scope(bot_uuid, user_id, None);
    let started = std::time::Instant::now();
    let title: String = user_text.chars().take(80).collect();
    super::event_sink::run_started(identity.sink, &run, Some(title.as_str())).await;
    super::event_sink::step_started(identity.sink, &run, "llm", "Answering", Some(0)).await;

    let timed_out = tokio::time::timeout(
        std::time::Duration::from_secs(300),
        super::llm::stream_llm_response(
            identity,
            rx,
            &messages_val,
            &user_query,
            model_preference,
        ),
    )
    .await
    .is_err();
    if timed_out {
        botlog::safe_warn!("stream_llm_response TIMEOUT after 300s for session {session_id}");
        super::event_sink::step_finished(
            identity.sink,
            &run,
            "llm",
            "error",
            Some(started.elapsed().as_millis() as u64),
            None,
        )
        .await;
        super::event_sink::run_error(
            identity.sink,
            &run,
            "The model did not answer before the turn timeout.",
            Some("llm_timeout"),
        )
        .await;
        super::event_sink::run_finished(
            identity.sink,
            &run,
            botevents::RunState::Failed,
            started.elapsed().as_millis() as u64,
            None,
        )
        .await;
    } else {
        let cancelled = crate::core::bot::runs::bus::bus().is_cancelled(&run.thread_id);
        let waiting = crate::core::bot::runs::bus::bus()
            .thread(&run.thread_id)
            .state()
            .is_waiting();
        let final_state = if cancelled {
            botevents::RunState::Cancelled
        } else {
            botevents::RunState::Succeeded
        };
        super::event_sink::step_finished(
            identity.sink,
            &run,
            "llm",
            if cancelled { "cancelled" } else { "ok" },
            Some(started.elapsed().as_millis() as u64),
            None,
        )
        .await;
        // #1670 — a turn that asked a question or proposed a plan is not
        // finished; the answer starts the next run in the same thread.
        if !waiting {
            super::event_sink::run_finished(
                identity.sink,
                &run,
                final_state,
                started.elapsed().as_millis() as u64,
                None,
            )
            .await;
        }
    }

    if let Ok(run_uuid) = uuid::Uuid::parse_str(&run.run_id) {
        match state.conn.get() {
            Ok(mut conn) => {
                if let Err(error) = crate::core::bot::runs::store::attach_message_run(
                    &mut conn,
                    session_id,
                    run_uuid,
                    &run.thread_id,
                ) {
                    log::warn!("message run pointer failed: {error}");
                }
            }
            Err(error) => log::warn!("message run pointer skipped: pool error: {error}"),
        }
    }

    Ok(())
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
    let mut llm_url = cfg.get_config(&bot_uuid, "llm-url", Some("")).unwrap_or_default();
    let mut llm_key = cfg.get_config(&bot_uuid, "llm-key", Some("")).unwrap_or_default();
    let mut llm_model = cfg.get_config(&bot_uuid, "llm-model", Some("")).unwrap_or_default();
    let llm_provider = cfg.get_config(&bot_uuid, "llm-provider", Some("")).unwrap_or_default();

    // #1652/#1653/#1654 — subscription accounts are resolved per user before
    // any HTTP provider is built. `choose_for_request` refuses to fall back
    // when the requested provider is a subscription and no account is linked.
    if crate::core::ai_accounts::is_subscription_provider(&llm_provider) {
        let linked = crate::core::ai_accounts::linked_account(
            &state.conn, user_id, &llm_provider,
        );
        let resolved = crate::core::ai_accounts::choose_for_request(
            Some(&llm_provider),
            linked.as_ref(),
            Some((llm_provider.as_str(), llm_model.as_str())),
            None,
        )?;
        let Some(resolved) = resolved else {
            return Ok(None);
        };
        let provider = crate::llm::subscription::build(
            state, user_id, session_id, bot_uuid, &resolved,
        )
        .await?;
        botlog::safe_info!(
            "BOT {bot_uuid} subscription LLM: provider=[{}] model=[{}] source=[{}]",
            resolved.provider, resolved.model, resolved.source
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
        let tier_model = cfg.get_config(&bot_uuid, &tier_key, Some("")).unwrap_or_default();
        if !tier_model.is_empty() {
            botlog::safe_info!(
                "BOT {bot_uuid} model tier '{tier}' -> model=[{tier_model}]"
            );
            llm_model = tier_model;
        } else {
            botlog::safe_info!(
                "BOT {bot_uuid} model tier '{tier}' is not configured; using the default model [{llm_model}]"
            );
        }
    }
    if let Ok(val) = std::env::var("LLM_URL") { if !val.is_empty() { llm_url = val; } }
    if let Ok(val) = std::env::var("LLM_KEY") { if !val.is_empty() { llm_key = val; } }
    if let Ok(val) = std::env::var("LLM_MODEL") { if !val.is_empty() { llm_model = val; } }
    let llm_host = url::Url::parse(&llm_url).ok()
        .and_then(|u| u.host_str().map(|h| h.to_string()))
        .unwrap_or_else(|| "<unparseable>".to_string());
    botlog::safe_info!("BOT {bot_uuid} LLM CONFIG: host=[{llm_host}] key_len=[{}] model=[{llm_model}] provider=[{llm_provider}]", llm_key.len());
    if !llm_url.is_empty() {
        let explicit = if llm_provider.is_empty() { None } else { crate::llm::provider_factory::llm_provider_type_from_name(&llm_provider) };
        let provider = crate::llm::provider_factory::create_llm_provider_from_url(
            &llm_url,
            if llm_model.is_empty() { None } else { Some(llm_model.clone()) },
            None, explicit,
        );
        Ok(Some((
            Arc::new(crate::llm::BotlibLLMProviderWrapper::new(
                provider, llm_model.clone(), llm_key.clone(),
            )) as Arc<dyn botlib::traits::LLMProvider>,
            llm_key, llm_model,
        )))
    } else {
        log::warn!("CRISTO BOT: llm-url is EMPTY, falling back to global provider");
        Ok(None)
    }
}

/// When the answer mode is Data/Chart, generates and sends the specialized
/// response; returns true when the pipeline has already replied.
pub(super) async fn respond_in_answer_mode(
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
        crate::core::bot::answer_mode::AnswerMode::Data => crate::core::bot::answer_mode::generate_data_response(
            state, user_text, bot_uuid, bot_name, session_id, user_id,
        ).await,
        crate::core::bot::answer_mode::AnswerMode::Chart => crate::core::bot::answer_mode::generate_chart_response(
            state, user_text, bot_uuid, bot_name, session_id, user_id,
        ).await,
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
                bot_uuid.to_string(), session_id.to_string(),
                user_id.to_string(),
                format!("<p>Error: {e}</p>"), "web",
            );
            let _ = sink.send_bot_response(&err_resp).await;
        }
    }
    Ok(true)
}
