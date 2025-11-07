use crate::basic::ScriptService;
use crate::config::ConfigManager;
use crate::shared::models::{Automation, TriggerKind};
use crate::shared::state::AppState;
use chrono::Utc;
use cron::Schedule;
use diesel::prelude::*;
use log::{error, info};
use std::str::FromStr;
use std::sync::Arc;
use tokio::time::{interval, Duration};
mod compact_prompt;

pub struct AutomationService {
    state: Arc<AppState>,
}

impl AutomationService {
    pub fn new(state: Arc<AppState>) -> Self {
        // Start the compact prompt scheduler
        crate::automation::compact_prompt::start_compact_prompt_scheduler(Arc::clone(&state));
        Self { state }
    }

    pub async fn spawn(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Automation service started");

        let mut ticker = interval(Duration::from_secs(60));

        loop {
            ticker.tick().await;
            if let Err(e) = self.check_scheduled_tasks().await {
                error!("Error checking scheduled tasks: {}", e);
            }
        }
    }

    async fn check_scheduled_tasks(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use crate::shared::models::system_automations::dsl::{
            id, is_active, kind, last_triggered as lt_column, system_automations,
        };

        let mut conn = self
            .state
            .conn
            .lock()
            .map_err(|e| format!("Failed to acquire lock: {}", e))?;

        let automations: Vec<Automation> = system_automations
            .filter(is_active.eq(true))
            .filter(kind.eq(TriggerKind::Scheduled as i32))
            .load::<Automation>(&mut *conn)?;

        for automation in automations {
            if let Some(schedule_str) = &automation.schedule {
                if let Ok(parsed_schedule) = Schedule::from_str(schedule_str) {
                    let now = Utc::now();
                    let next_run = parsed_schedule.upcoming(Utc).next();

                    if let Some(next_time) = next_run {
                        let time_until_next = next_time - now;
                        if time_until_next.num_minutes() < 1 {
                            if let Some(last_triggered) = automation.last_triggered {
                                if (now - last_triggered).num_minutes() < 1 {
                                    continue;
                                }
                            }

                            self.execute_automation(&automation).await?;

                            diesel::update(system_automations.filter(id.eq(automation.id)))
                                .set(lt_column.eq(Some(now)))
                                .execute(&mut *conn)?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn execute_automation(
        &self,
        automation: &Automation,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Executing automation: {}", automation.param);

        let bot_name: String = {
            use crate::shared::models::schema::bots::dsl::*;
            let mut conn = self
                .state
                .conn
                .lock()
                .map_err(|e| format!("Lock failed: {}", e))?;
            bots.filter(id.eq(automation.bot_id))
                .select(name)
                .first(&mut *conn)?
        };

        let script_path = format!(
            "./work/{}.gbai/{}.gbdialog/{}.ast",
            bot_name, bot_name, automation.param
        );

        let script_content = match tokio::fs::read_to_string(&script_path).await {
            Ok(content) => content,
            Err(e) => {
                error!("Failed to read script {}: {}", script_path, e);
                return Ok(());
            }
        };

        let session = {
            let mut sm = self.state.session_manager.lock().await;
            let admin_user = uuid::Uuid::nil();
            sm.get_or_create_user_session(admin_user, automation.bot_id, "Automation")?
                .ok_or("Failed to create session")?
        };

        let script_service = ScriptService::new(Arc::clone(&self.state), session);

        match script_service.compile(&script_content) {
            Ok(ast) => {
                if let Err(e) = script_service.run(&ast) {
                    error!("Script execution failed: {}", e);
                }
            }
            Err(e) => {
                error!("Script compilation failed: {}", e);
            }
        }

        Ok(())
    }

    async fn execute_compact_prompt(
        &self,
        automation: &Automation,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Executing prompt compaction for bot: {}", automation.bot_id);

        let config_manager = ConfigManager::new(Arc::clone(&self.state.conn));
        let compact_threshold = config_manager
            .get_config(&automation.bot_id, "prompt-compact", None)?
            .parse::<usize>()
            .unwrap_or(0);

        if compact_threshold == 0 {
            return Ok(());
        }

        let mut session_manager = self.state.session_manager.lock().await;
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
}

pub async fn execute_compact_prompt(state: Arc<crate::shared::state::AppState>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use crate::shared::models::system_automations::dsl::{is_active, system_automations};
    use diesel::prelude::*;
    use log::info;

    let state_clone = state.clone();
let service = AutomationService::new(state_clone);

    let mut conn = state
        .conn
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    let automations: Vec<crate::shared::models::Automation> = system_automations
        .filter(is_active.eq(true))
        .load::<crate::shared::models::Automation>(&mut *conn)?;

    for automation in automations {
        if let Err(e) = service.execute_compact_prompt(&automation).await {
            error!(
                "Failed to compact prompt for bot {}: {}",
                automation.bot_id, e
            );
        }
    }

    info!("Prompt compaction cycle completed");
    Ok(())
}
