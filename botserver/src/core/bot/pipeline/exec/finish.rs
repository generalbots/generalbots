use super::super::types::{PipelineError, PipelineResult, SessionIdentity};

pub(super) async fn stream_and_finalize(
    identity: &SessionIdentity<'_>,
    rx: &mut tokio::sync::mpsc::Receiver<botlib::models::BotResponse>,
    messages: &[serde_json::Value],
    user_text: &str,
    #[cfg(feature = "integrations")] mentions: &[botlib::models::MentionRef],
    is_switcher_replay: bool,
    model_preference: &str,
    agent_enabled: bool,
) -> PipelineResult<()> {
    let SessionIdentity {
        sink,
        state,
        bot_uuid,
        session_id,
        ..
    } = *identity;

    // An agent-off request for a file or a command: the answer carries a card
    // to turn the Agent on (kept with the answer; sent once it is complete).
    let agent_request = (cfg!(feature = "agent-vm")
        && !agent_enabled
        && super::super::turn_events::needs_vm(user_text))
    .then(|| {
        super::super::turn_events::agent_enable_frame(
            super::super::turn_events::AGENT_REASON,
            user_text,
        )
    });
    if let Some(frame) = &agent_request {
        super::super::turn_events::record(
            session_id,
            super::super::turn_events::AGENT_ENABLE_REQUEST,
            frame.clone(),
        );
    }

    let turn_timeout_secs = super::super::enriched_llm::turn_timeout_secs();
    if tokio::time::timeout(
        std::time::Duration::from_secs(turn_timeout_secs),
        super::super::enriched_llm::build_and_stream(
            identity,
            rx,
            messages,
            user_text,
            #[cfg(feature = "integrations")]
            mentions,
            is_switcher_replay,
            model_preference,
        ),
    )
    .await
    .is_err()
    {
        botlog::safe_warn!(
            "stream_llm_response TIMEOUT after {turn_timeout_secs}s for session {session_id}"
        );
        return Err(PipelineError::LLM(format!(
            "Turn timed out after {turn_timeout_secs}s"
        )));
    }

    if let Some(frame) = agent_request {
        if let Err(e) = sink.send_raw_json(&frame).await {
            log::warn!("exec: agent-enable frame not delivered: {e}");
        }
    }

    // #1635 — the pipeline streams typed BotResponse frames, which have no
    // citations field. Send the sources as one explicit frame after the answer
    // so the renderer can attach the chips to the message that just completed.
    let citations = {
        let mut pending = state.pending_citations.lock().await;
        pending.remove(&session_id.to_string()).map_or_else(Default::default, |v| v)
    };
    if !citations.is_empty() {
        let frame = serde_json::json!({
            "type": "citations",
            "message_type": "citations",
            "bot_id": bot_uuid.to_string(),
            "session_id": session_id.to_string(),
            "citations": citations,
        });
        if let Err(e) = sink.send_raw_json(&frame).await {
            log::warn!("exec: citations frame not delivered: {e}");
        }
    }

    // #2143 gap D — publish this turn's artifacts; a Drive outage never fails it.
    // #2239 — skip workspace artifact publish when Agent/VM is off for this turn.
    #[cfg(all(feature = "agent-vm", feature = "drive"))]
    if agent_enabled {
        super::super::output_publish::publish_and_attach(identity).await;
    }

    #[cfg(not(all(feature = "agent-vm", feature = "drive")))]
    std::hint::black_box(agent_enabled);

    Ok(())
}
