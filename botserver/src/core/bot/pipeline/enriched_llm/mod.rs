use std::sync::Arc;

use botcore::shared::state::AppState;
use uuid::Uuid;

use super::sink::ChannelSink;
use super::types::{PipelineResult, SessionIdentity};

/// Enriches the LLM prompt with KB context, memory recall and integration
/// mention blocks, saves the user turn, and streams the LLM response.

/// The persisted conversation shows the user's own words; the agent-only
/// `[User attached a file stored at inbox/…]` block must stay in the prompt
/// Turn budget for one LLM call.
///
/// A CPU-only provider must prefill the whole prompt before the first token,
/// so the ceiling is configurable: `LLM_TURN_TIMEOUT_SECS` (default 300).
pub(super) fn turn_timeout_secs() -> u64 {
    std::env::var("LLM_TURN_TIMEOUT_SECS")
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .filter(|secs| *secs > 0)
        .unwrap_or(300)
}

/// copy only (replay used to show it inside the user's bubble).
/// Returns (display_text, attachment_meta) so the save point can encode
/// the metadata alongside the visible content (#2562).
fn display_user_text_with_meta(
    text: &str,
) -> (String, Vec<super::llm::message_meta::AttachmentInfo>) {
    const MARKER: &str = "[User attached a file stored at inbox/";
    let Some(position) = text.find(MARKER) else {
        return (text.to_string(), Vec::new());
    };
    let before = text[..position].trim();
    let file_name = text[position + MARKER.len()..]
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_string();
    let display = if !before.is_empty() {
        before.to_string()
    } else if file_name.is_empty() {
        "\u{1f4ce} attachment".to_string()
    } else {
        format!("\u{1f4ce} {file_name}")
    };
    let attachments = if file_name.is_empty() {
        Vec::new()
    } else {
        let drive_path = format!("inbox/{file_name}");
        let mime = super::activity::mime_from_name(&file_name);
        // Parse "size=NNN" from the marker when present (added in #2562).
        let rest = &text[position + MARKER.len()..];
        let size: usize = rest
            .find("size=")
            .and_then(|s| {
                rest[s + 5..]
                    .split(|c: char| !c.is_ascii_digit())
                    .next()
                    .and_then(|d| d.parse().ok())
            })
            .unwrap_or(0);
        vec![super::llm::message_meta::AttachmentInfo {
            name: file_name,
            mime_type: mime,
            size,
            drive_path,
        }]
    };
    (display, attachments)
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

    // #2562 — KB activity: emit only when knowledge bases are active.
    let kb_active = {
        let active_kbs = crate::core::bot::kb_context::get_active_kbs(&state.conn, session_id);
        let active_web = crate::core::bot::kb_context::get_active_websites(&state.conn, session_id);
        !active_kbs.is_empty() || !active_web.is_empty()
    };
    let pre_run = super::event_sink::RunContext::new(&session_id.to_string(), "chat")
        .with_state(state.clone())
        .with_scope(bot_uuid, user_id, None);
    let kb_act = if kb_active {
        let summary = super::activity::kb_search_summary(&user_query, &[]);
        let act =
            super::activity::Activity::started(super::activity::ActivityKind::KbSearch, summary)
                .with_run_id(&pre_run.run_id);
        super::activity::emit_activity(identity.sink, &pre_run, &act).await;
        Some(act)
    } else {
        None
    };

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
        if let Some(act) = &kb_act {
            let fail = act.failed("KB search timed out");
            super::activity::emit_activity(identity.sink, &pre_run, &fail).await;
        }
        botlog::safe_warn!(
            "ws_handler: inject_kb_context TIMEOUT after 30s for session {session_id}"
        );
    } else if let Some(act) = &kb_act {
        // Extract results from the injected context for the activity summary.
        let sources = helpers::extract_kb_sources_from_messages(&messages_val);
        let summary = super::activity::kb_search_summary(&user_query, &sources);
        let done = act.completed(summary);
        super::activity::emit_activity(identity.sink, &pre_run, &done).await;
    }

    #[cfg(feature = "memory-os")]
    crate::core::bot::memory_hook::inject_recall(state, user_id, &user_query, &mut messages_val);

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

    // #2059 — name the connected connectors and the ones that match this
    // request, so "post this on Instagram" does not need an @-mention.
    #[cfg(feature = "integrations")]
    super::connector_hints::inject(state, bot_uuid, user_id, &user_query, &mut messages_val);

    if !is_switcher_replay {
        let (display_text, attach_meta) = display_user_text_with_meta(user_text);
        // #2562 — encode attachment metadata so a reload shows file chips.
        let stored_text = if attach_meta.is_empty() {
            display_text.clone()
        } else {
            let meta = super::llm::message_meta::MessageMeta {
                attachments: attach_meta,
                ..Default::default()
            };
            super::llm::message_meta::encode(&display_text, &meta)
        };
        let mut sm = state.session_manager.lock().await;
        let _ = sm.save_message(session_id, user_id, 1, &stored_text, 1);
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
    // #1807 — name the generation stage so the timeline shows work before the
    // first token arrives. The extra fields keep the required STEP_STARTED
    // shape while telling the UI this is the agent phase, not a VM tool.
    super::event_sink::step_started_with(
        identity.sink,
        &run,
        "llm",
        "Answering",
        Some(0),
        serde_json::json!({
            "phase": "agent",
            "step": "generate",
            "detail": "Generating the program",
        }),
    )
    .await;

    // A CPU-only model must prefill the whole system prompt before the first
    // token, which can take many minutes for a ~9k-token prompt. The budget is
    // therefore configurable: `LLM_TURN_TIMEOUT_SECS` (default 300) lets an
    // operator give a slow local model a realistic window without loosening the
    // default for hosted providers.
    let turn_timeout_secs = turn_timeout_secs();
    let timed_out = tokio::time::timeout(
        std::time::Duration::from_secs(turn_timeout_secs),
        super::llm::stream_llm_response(identity, rx, &messages_val, &user_query, model_preference),
    )
    .await
    .is_err();
    if timed_out {
        botlog::safe_warn!(
            "stream_llm_response TIMEOUT after {turn_timeout_secs}s for session {session_id}"
        );
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

mod helpers;
pub(crate) use helpers::resolve_bot_llm_provider;
pub(super) use helpers::respond_in_answer_mode;
