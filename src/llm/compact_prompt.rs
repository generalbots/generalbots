use crate::core::config::ConfigManager;
use crate::llm::llm_models;
use crate::shared::state::AppState;
use log::{error, info, trace};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use uuid::Uuid;
pub fn start_compact_prompt_scheduler(state: Arc<AppState>) {
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(30)).await;
        let mut interval = interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            if let Err(e) = compact_prompt_for_bots(&Arc::clone(&state)).await {
                error!("Prompt compaction failed: {}", e);
            }
        }
    });
}
async fn compact_prompt_for_bots(
    state: &Arc<AppState>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use once_cell::sync::Lazy;
    use scopeguard::guard;
    static SESSION_IN_PROGRESS: Lazy<tokio::sync::Mutex<HashSet<Uuid>>> =
        Lazy::new(|| tokio::sync::Mutex::new(HashSet::new()));

    let sessions = {
        let mut session_manager = state.session_manager.lock().await;
        session_manager.get_user_sessions(Uuid::nil())?
    };
    for session in sessions {
        let config_manager = ConfigManager::new(state.conn.clone());
        let compact_threshold = config_manager
            .get_config(&session.bot_id, "prompt-compact", None)?
            .parse::<i32>()
            .unwrap_or(0);

        if compact_threshold == 0 {
            return Ok(());
        } else if compact_threshold < 0 {
            trace!(
                "Negative compact threshold detected for bot {}, skipping",
                session.bot_id
            );
        }
        let session_id = session.id;
        let history = {
            let mut session_manager = state.session_manager.lock().await;
            session_manager.get_conversation_history(session.id, session.user_id)?
        };

        let mut messages_since_summary = 0;
        let mut has_new_messages = false;
        let last_summary_index = history
            .iter()
            .rev()
            .position(|(role, _)| role == "compact")
            .map(|pos| history.len() - pos - 1);

        // Calculate start index: if there's a summary, start after it; otherwise start from 0
        let start_index = last_summary_index.map(|idx| idx + 1).unwrap_or(0);

        for (_i, (role, _)) in history.iter().enumerate().skip(start_index) {
            if role == "compact" {
                continue;
            }
            messages_since_summary += 1;
            has_new_messages = true;
        }

        if !has_new_messages && last_summary_index.is_some() {
            continue;
        }
        if messages_since_summary < compact_threshold as usize {
            continue;
        }

        {
            let mut session_in_progress = SESSION_IN_PROGRESS.lock().await;
            if session_in_progress.contains(&session.id) {
                trace!(
                    "Skipping session {} - compaction already in progress",
                    session.id
                );
                continue;
            }
            session_in_progress.insert(session.id);
        }

        trace!(
            "Compacting prompt for session {}: {} messages since last summary",
            session.id,
            messages_since_summary
        );

        let mut conversation = String::new();
        conversation
            .push_str("Please summarize this conversation between user and bot: \n\n [[[***** \n");

        for (role, content) in history.iter().skip(start_index) {
            if role == "compact" {
                continue;
            }
            conversation.push_str(&format!(
                "{}: {}\n",
                if role == "user" { "user" } else { "assistant" },
                content
            ));
        }
        conversation.push_str("\n *****]]] \n Give me full points only, no explanations.");

        let messages = vec![serde_json::json!({
            "role": "user",
            "content": conversation
        })];

        let llm_provider = state.llm_provider.clone();
        let mut filtered = String::new();
        let config_manager = crate::config::ConfigManager::new(state.conn.clone());
        let model = config_manager
            .get_config(&Uuid::nil(), "llm-model", None)
            .unwrap_or_default();
        let key = config_manager
            .get_config(&Uuid::nil(), "llm-key", None)
            .unwrap_or_default();

        let summarized = match llm_provider
            .generate("", &serde_json::Value::Array(messages), &model, &key)
            .await
        {
            Ok(summary) => {
                trace!(
                    "Successfully summarized session {} ({} chars)",
                    session.id,
                    summary.len()
                );
                // Use handler to filter <think> content
                let handler = llm_models::get_handler(
                    config_manager
                        .get_config(&session.bot_id, "llm-model", None)
                        .unwrap()
                        .as_str(),
                );

                filtered = handler.process_content(&summary);
                format!("SUMMARY: {}", filtered)
            }
            Err(e) => {
                error!(
                    "Failed to summarize conversation for session {}: {}",
                    session.id, e
                );
                trace!("Using fallback summary for session {}", session.id);
                format!("SUMMARY: {}", filtered) // Fallback
            }
        };
        info!(
            "Prompt compacted {}: {} messages",
            session.id,
            history.len()
        );
        {
            let mut session_manager = state.session_manager.lock().await;
            session_manager.save_message(session.id, session.user_id, 9, &summarized, 1)?;
        }

        let _session_cleanup = guard((), |_| {
            tokio::spawn(async move {
                let mut in_progress = SESSION_IN_PROGRESS.lock().await;
                in_progress.remove(&session_id);
            });
        });
    }
    Ok(())
}
