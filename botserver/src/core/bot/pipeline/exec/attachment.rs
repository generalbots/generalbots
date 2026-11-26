//! The turn's attachment: stored, turned into prompt content, and reported
//! to the person as an `attachment_read` activity (#1641, #2562).

use super::super::types::SessionIdentity;

/// What the rest of the turn needs from the attachment.
pub(super) struct TurnAttachment {
    pub user_text: String,
    pub note: Option<String>,
    pub media: Vec<serde_json::Value>,
}

/// Reads the message's attachment, if any, and emits its activity.
pub(super) async fn read(
    identity: &SessionIdentity<'_>,
    parsed: &serde_json::Value,
    user_text: String,
    model_preference: &str,
) -> TurnAttachment {
    let SessionIdentity {
        sink,
        state,
        bot_uuid,
        session_id,
        user_id,
        bot_name,
        ..
    } = *identity;
    // Early run context for activity events that fire before the LLM call.
    let early_run = super::super::event_sink::RunContext::new(&session_id.to_string(), "chat")
        .with_scope(bot_uuid, user_id, None)
        .with_state(state.clone());

    // #1641 — store the attachment and turn it into prompt content.
    // #2562 — emit attachment_read activities around the ingestion.
    let has_file = parsed.get("file").is_some()
        || parsed
            .get("files")
            .and_then(|v| v.as_array())
            .is_some_and(|a| !a.is_empty());
    let file_name = parsed
        .get("file")
        .and_then(|f| f.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("")
        .to_string();
    let file_size = parsed
        .get("file")
        .and_then(|f| f.get("content_base64"))
        .and_then(|b| b.as_str())
        .map(|b| b.len() * 3 / 4)
        .unwrap_or(0);
    let attach_activity = if has_file {
        let summary = super::super::activity::attachment_summary(
            &file_name, "", file_size, 0, false, "",
        );
        let act = super::super::activity::Activity::started(
            super::super::activity::ActivityKind::AttachmentRead,
            summary,
        )
        .with_run_id(&early_run.run_id);
        super::super::activity::emit_activity(sink, &early_run, &act).await;
        Some(act)
    } else {
        None
    };

    let vision = has_file && {
        let url = super::super::enriched_llm::turn_llm_url(state, bot_uuid, model_preference).await;
        botllm::media_parts::sees_images(&url)
    };
    let attachment = super::super::turn_hooks::ingest_attachment(
        state, bot_uuid, bot_name, parsed, user_text, vision,
    )
    .await;
    super::super::turn_watch::stage("attachment read");
    let user_text = attachment.user_text;
    let attachment_note = attachment.note;
    let attachment_media = attachment.media;
    let media_fail_reason = attachment.media_fail_reason;
    let store_fail_reason = attachment.store_fail_reason;

    // Emit the completed/failed attachment activity.
    if let Some(act) = attach_activity {
        let drive_path = format!("inbox/{file_name}");
        // Count only the actual extracted text (delimited by the BEGIN/END
        // ATTACHMENT markers), not the wrapper instructions sent to the model.
        // Without this, unsupported formats report chars_read = wrapper length
        // and the step card does not show the "format not readable" warning.
        let excerpt_chars = attachment_note
            .as_ref()
            .and_then(|n| {
                let marker = format!("--- BEGIN ATTACHMENT {file_name} ---\n");
                n.find(&marker).map(|start| {
                    let content_start = start + marker.len();
                    let end_marker = format!("\n--- END ATTACHMENT {file_name} ---");
                    let end = n[content_start..].find(&end_marker)
                        .map(|e| content_start + e)
                        .unwrap_or(n.len());
                    n[content_start..end].chars().count()
                })
            })
            .unwrap_or(0);
        let truncated = attachment_note
            .as_ref()
            .is_some_and(|n| n.contains("excerpt truncated"));
        let mime = super::super::activity::mime_from_name(&file_name);
        if let Some(reason) = store_fail_reason {
            // The file was not kept (a full disk answers HTTP 507): the card
            // says so instead of reporting a stored attachment.
            let locale = botcore::i18n::locale_for_user_pool(&state.conn, user_id);
            let key = if reason == "storage_full" {
                "error-attachment-storage-full"
            } else {
                "error-attachment-not-stored"
            };
            let fail = act.failed(&botcore::i18n::t(&locale, key));
            super::super::activity::emit_activity(sink, &early_run, &fail).await;
        } else if let Some(reason) = media_fail_reason {
            // #2561 — the file was stored but its media content could not be
            // read (e.g. no image reader installed). Emit as failed with the
            // reason code so the interface can show it.
            let fail = act.failed(reason);
            super::super::activity::emit_activity(sink, &early_run, &fail).await;
        } else if attachment_note.is_some() || user_text.contains(&format!("inbox/{file_name}")) {
            let summary = super::super::activity::attachment_summary(
                &file_name,
                &mime,
                file_size,
                excerpt_chars,
                truncated,
                &drive_path,
            );
            // The model received the file itself (image/PDF to a vision
            // model): the card must not call it unreadable.
            let mut summary = summary;
            if !attachment_media.is_empty() {
                summary["seen_by_model"] = serde_json::Value::Bool(true);
            }
            let done = act.completed(summary);
            super::super::activity::emit_activity(sink, &early_run, &done).await;
        } else {
            let key = if file_size > 50 * 1024 * 1024 {
                "error-attachment-too-large"
            } else {
                "error-attachment-not-read"
            };
            let locale = botcore::i18n::locale_for_user_pool(&state.conn, user_id);
            let fail = act.failed(&botcore::i18n::t(&locale, key));
            super::super::activity::emit_activity(sink, &early_run, &fail).await;
        }
    }

    TurnAttachment {
        user_text,
        note: attachment_note,
        media: attachment_media,
    }
}
