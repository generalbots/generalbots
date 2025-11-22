use crate::basic::ScriptService;
use crate::shared::models::{Automation, TriggerKind};
use crate::shared::state::AppState;
use chrono::Utc;
use cron::Schedule;
use diesel::prelude::*;
use log::error;
use std::str::FromStr;
use std::sync::Arc;
use tokio::time::{interval, Duration};
mod compact_prompt;
#[cfg(feature = "vectordb")]
pub mod vectordb_indexer;

#[cfg(feature = "vectordb")]
pub use vectordb_indexer::{IndexingStats, IndexingStatus, VectorDBIndexer};

pub struct AutomationService {
    state: Arc<AppState>,
}
impl AutomationService {
    pub fn new(state: Arc<AppState>) -> Self {
        crate::automation::compact_prompt::start_compact_prompt_scheduler(Arc::clone(&state));
        Self { state }
    }
    pub async fn spawn(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut ticker = interval(Duration::from_secs(5));
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
            .get()
            .map_err(|e| format!("Failed to acquire database connection: {}", e))?;
        let automations: Vec<Automation> = system_automations
            .filter(is_active.eq(true))
            .filter(kind.eq(TriggerKind::Scheduled as i32))
            .load::<Automation>(&mut conn)?;
        for automation in automations {
            if let Some(schedule_str) = &automation.schedule {
                match Schedule::from_str(schedule_str.trim()) {
                    Ok(parsed_schedule) => {
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
                                if let Err(e) = self.execute_automation(&automation).await {
                                    error!("Error executing automation {}: {}", automation.id, e);
                                }
                                if let Err(e) =
                                    diesel::update(system_automations.filter(id.eq(automation.id)))
                                        .set(lt_column.eq(Some(now)))
                                        .execute(&mut conn)
                                {
                                    error!(
                                        "Error updating last_triggered for automation {}: {}",
                                        automation.id, e
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!(
                            "Error parsing schedule for automation {} ({}): {}",
                            automation.id, schedule_str, e
                        );
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
        let bot_name: String = {
            use crate::shared::models::schema::bots::dsl::*;
            let mut conn = self
                .state
                .conn
                .get()
                .map_err(|e| format!("Failed to acquire database connection: {}", e))?;
            bots.filter(id.eq(automation.bot_id))
                .select(name)
                .first(&mut conn)?
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
            let admin_user = automation.bot_id;
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
}
