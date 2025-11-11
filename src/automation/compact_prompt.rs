use crate::config::ConfigManager;
use crate::llm_models;
use crate::shared::models::Automation;
use crate::shared::state::AppState;
use diesel::prelude::*;
use log::{error, trace};
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
            if let Err(e) = execute_compact_prompt(Arc::clone(&state)).await {
                error!("Prompt compaction failed: {}", e);
            }
        }
    });
}
async fn execute_compact_prompt(
    state: Arc<AppState>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use crate::shared::models::system_automations::dsl::{is_active, system_automations};
    let automations: Vec<Automation> = {
        let mut conn = state
            .conn
            .get()
            .map_err(|e| format!("Failed to acquire lock: {}", e))?;
        system_automations
            .filter(is_active.eq(true))
            .load::<Automation>(&mut *conn)?
    };
    for automation in automations {
        if let Err(e) = compact_prompt_for_bot(&state, &automation).await {
            error!(
                "Failed to compact prompt for bot {}: {}",
                automation.bot_id, e
            );
        }
    }
    Ok(())
}
async fn compact_prompt_for_bot(
    state: &Arc<AppState>,
    automation: &Automation,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use once_cell::sync::Lazy;
    use scopeguard::guard;
    static IN_PROGRESS: Lazy<tokio::sync::Mutex<HashSet<Uuid>>> =
        Lazy::new(|| tokio::sync::Mutex::new(HashSet::new()));
    {
        let mut in_progress = IN_PROGRESS.lock().await;
        if in_progress.contains(&automation.bot_id) {
            return Ok(());
        }
        in_progress.insert(automation.bot_id);
    }
    let bot_id = automation.bot_id;
    let _cleanup = guard((), |_| {
        tokio::spawn(async move {
            let mut in_progress = IN_PROGRESS.lock().await;
            in_progress.remove(&bot_id);
        });
    });
    let config_manager = ConfigManager::new(state.conn.clone());
    let compact_threshold = config_manager
        .get_config(&automation.bot_id, "prompt-compact", None)?
        .parse::<i32>()
        .unwrap_or(0);
    if compact_threshold == 0 {
        return Ok(());
    } else if compact_threshold < 0 {
        trace!(
            "Negative compact threshold detected for bot {}, skipping",
            automation.bot_id
        );
    }
    let sessions = {
        let mut session_manager = state.session_manager.lock().await;
        session_manager.get_user_sessions(Uuid::nil())?
    };
    for session in sessions {
        if session.bot_id != automation.bot_id {
            trace!("Skipping session {} - bot_id {} doesn't match automation bot_id {}", 
                session.id, session.bot_id, automation.bot_id);
            continue;
        }
        let history = {
            let mut session_manager = state.session_manager.lock().await;
            session_manager.get_conversation_history(session.id, session.user_id)?
        };

        let mut messages_since_summary = 0;
        let mut has_new_messages = false;
        let mut last_summary_index = history.iter().position(|(role, _)|
         role == "compact")
            .unwrap_or(0);
        
        for (i, (role, _)) in history.iter().enumerate().skip(last_summary_index + 1) {
            if role == "compact" {
                continue;
            }
            messages_since_summary += 1;
            has_new_messages = true;
        }

        if !has_new_messages {
            trace!("Skipping session {} - no new messages since last summary", session.id);
            continue;
        }
        if messages_since_summary < compact_threshold as usize {
            trace!("Skipping compaction for session {} - only {} new messages since last summary (threshold: {})", 
                session.id, messages_since_summary, compact_threshold);
            continue;
        }

        trace!(
            "Compacting prompt for session {}: {} messages since last summary",
            session.id,
            messages_since_summary
        );
        let mut compacted = String::new();
        let messages_to_include = history.iter()
            .skip(history.len().saturating_sub(messages_since_summary ))
            .take(messages_since_summary + 1);
        
        for (role, content) in messages_to_include {
            compacted.push_str(&format!("{}: {}\n", role, content));
        }
        let llm_provider = state.llm_provider.clone();
        let compacted_clone = compacted.clone();
        let summarized = match llm_provider.summarize(&compacted_clone).await {
            Ok(summary) => {
                trace!(
                    "Successfully summarized conversation for session {}, summary length: {}",
                    session.id,
                    summary.len()
                );
                let handler = llm_models::get_handler(
                    &config_manager
                        .get_config(&automation.bot_id, "llm-model", None)
                        .unwrap_or_default(),
                );
                let filtered = handler.process_content(&summary);
                format!("SUMMARY: {}", filtered)
            }
            Err(e) => {
                error!(
                    "Failed to summarize conversation for session {}: {}",
                    session.id, e
                );
                format!("SUMMARY: {}", compacted)
            }
        };
        trace!(
            "Prompt compacted {}: {} messages",
            session.id,
            history.len()
        );
        {
            let mut session_manager = state.session_manager.lock().await;
            session_manager.save_message(session.id, session.user_id, 9, &summarized, 1)?;
        }
    }
    Ok(())
}
