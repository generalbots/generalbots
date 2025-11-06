use crate::config::ConfigManager;
use crate::shared::models::Automation;
use crate::shared::state::AppState;
use diesel::prelude::*;
use log::{error, info};
use std::sync::Arc;
use tokio::time::{interval, Duration};

pub fn start_compact_prompt_scheduler(state: Arc<AppState>) {
    tokio::spawn(async move {
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
    info!("Executing prompt compaction for bot: {}", automation.bot_id);

    let config_manager = ConfigManager::new(Arc::clone(&state.conn));
    let compact_threshold = config_manager
        .get_config(&automation.bot_id, "prompt-compact", None)?
        .parse::<usize>()
        .unwrap_or(0);

    if compact_threshold == 0 {
        return Ok(());
    }

    let mut session_manager = state.session_manager.lock().await;
    let sessions = session_manager.get_user_sessions(uuid::Uuid::nil())?;

    for session in sessions {
        if session.bot_id != automation.bot_id {
            continue;
        }

        let history = session_manager.get_conversation_history(session.id, session.user_id)?;

        if history.len() > compact_threshold {
            info!(
                "Compacting prompt for session {}: {} messages",
                session.id,
                history.len()
            );

            let mut compacted = String::new();
            for (role, content) in &history[..history.len() - compact_threshold] {
                compacted.push_str(&format!("{}: {}\n", role, content));
            }

            let summarized = format!("SUMMARY: {}", compacted);
            session_manager.save_message(session.id, session.user_id, 3, &summarized, 1)?;
        }
    }

    Ok(())
}
