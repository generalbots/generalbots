use crate::config::ConfigManager;
use crate::llm_models;
use crate::shared::models::Automation;
use crate::shared::state::AppState;
use diesel::prelude::*;
use log::{error, info, trace};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use uuid::Uuid;

pub fn start_compact_prompt_scheduler(state: Arc<AppState>) {
    tokio::spawn(async move {
        // Initial 30 second delay before first run
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

async fn execute_compact_prompt(state: Arc<AppState>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use crate::shared::models::system_automations::dsl::{is_active, system_automations};

    let automations: Vec<Automation> = {
        let mut conn = state
            .conn
            .lock()
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
    // Skip if already compacting this bot
    use once_cell::sync::Lazy;
    use scopeguard::guard;
    static IN_PROGRESS: Lazy<tokio::sync::Mutex<HashSet<Uuid>>> = Lazy::new(|| {
        tokio::sync::Mutex::new(HashSet::new())
    });
    
    {
        let mut in_progress = IN_PROGRESS.lock().await;
        if in_progress.contains(&automation.bot_id) {
            trace!("Skipping compaction for bot {} - already in progress", automation.bot_id);
            return Ok(());
        }
        in_progress.insert(automation.bot_id);
    }

    // Ensure cleanup happens when function exits
    let bot_id = automation.bot_id;
    let _cleanup = guard((), |_| {
        tokio::spawn(async move {
            let mut in_progress = IN_PROGRESS.lock().await;
            in_progress.remove(&bot_id);
            trace!("Released compaction lock for bot {}", bot_id);
        });
    });

    info!("Executing prompt compaction for bot: {}", automation.bot_id);

    let config_manager = ConfigManager::new(Arc::clone(&state.conn));
    let compact_threshold = config_manager
        .get_config(&automation.bot_id, "prompt-compact", None)?
        .parse::<usize>()
        .unwrap_or(0);

    if compact_threshold == 0 {
        return Ok(());
    }

    // Get sessions without holding lock
    let sessions = {
        let mut session_manager = state.session_manager.lock().await;
        session_manager.get_user_sessions(Uuid::nil())?
    };

    for session in sessions {
        if session.bot_id != automation.bot_id {
            continue;
        }

        // Get history without holding lock
        let history = {
            let mut session_manager = state.session_manager.lock().await;
            session_manager.get_conversation_history(session.id, session.user_id)?
        };

            info!(
                "Compacting prompt for session {}: {} messages",
                session.id,
                history.len()
            );

            // Compact entire conversation history when threshold is reached
            let mut compacted = String::new();
            for (role, content) in &history {
                compacted.push_str(&format!("{}: {}\n", role, content));
            }

            // Clone needed references for async task
            let llm_provider = state.llm_provider.clone();
            let compacted_clone = compacted.clone();
            
            // Run LLM summarization with proper tracing and filtering
            trace!("Starting summarization for session {}", session.id);
            let summarized = match llm_provider.summarize(&compacted_clone).await {
                Ok(summary) => {
                    trace!("Successfully summarized session {} ({} chars)", 
                        session.id, summary.len());
                    // Use handler to filter <think> content
                    let handler = llm_models::get_handler(
                        &config_manager.get_config(
                            &automation.bot_id, 
                            "llm-model", 
                            None
                        ).unwrap_or_default()
                    );
                    let filtered = handler.process_content(&summary);
                    format!("SUMMARY: {}", filtered)
                },
                Err(e) => {
                    error!("Failed to summarize conversation for session {}: {}", session.id, e);
                    trace!("Using fallback summary for session {}", session.id);
                    format!("SUMMARY: {}", compacted) // Fallback
                }
            };
            info!(
                "Prompt compacted {}: {} messages",
                session.id,
                history.len()
            );

            // Remove all old messages and save only the summary
            {
                let mut session_manager = state.session_manager.lock().await;
                // First delete all existing messages for this session
                if let Err(e) = session_manager.clear_messages(session.id) {
                    error!("Failed to clear messages for session {}: {}", session.id, e);
                    return Err(e);
                }
                // Then save just the summary
                session_manager.save_message(session.id, session.user_id, 3, &summarized, 1)?;
            }
    }

    Ok(())
}
