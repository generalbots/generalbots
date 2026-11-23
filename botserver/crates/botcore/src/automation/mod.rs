// ScriptService stays in botserver layer
use crate::shared::models::{Automation, TriggerKind};
use log::{error, trace};
use tokio::time::{interval, Duration};
use uuid::Uuid;

/// Normalizes a cron schedule by converting 6-field (with seconds) to 5-field format.
/// If the schedule already has 5 fields or is in natural language format, it's returned as-is.
fn normalize_cron_schedule(schedule: &str) -> String {
    let trimmed = schedule.trim();
    let parts: Vec<&str> = trimmed.split_whitespace().collect();

    let result = match parts.len() {
        // 6 fields: assume seconds format, remove seconds
        6 => parts[1..].join(" "),
        // 4 fields: missing day-of-week, add "*"
        4 => format!("{} *", parts.join(" ")),
        // 5 fields: standard format
        5 => parts.join(" "),
        // Invalid: return as-is and let cron parser handle the error
        _ => trimmed.to_string(),
    };

    result.trim().to_string()
}

/// Verified tenant scope of a scheduled automation (#2826).
///
/// A classic `SET SCHEDULE` row only carries `branch_id`, never `org_id`; the
/// org must be resolved from the owning bot before any work path is built.
/// Both ids must be non-nil: the nil UUID is the legacy "no tenant" sentinel
/// and using it here made every org share `00000000-….gborg/`, so a schedule
/// from one tenant could run another tenant's compiled `.ast` when bot names
/// collide (#2685). The service never falls back to that global path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AutomationTenantScope {
    pub org_id: Uuid,
    pub branch_id: Uuid,
}

/// Why a scheduled automation could not be scoped to a tenant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutomationScopeError {
    MissingOrg,
    MissingBranch,
    BranchMismatch,
    MissingBotName,
    MissingParam,
    UnsafePathComponent,
}

impl std::fmt::Display for AutomationScopeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::MissingOrg => "automation has no org_id (nil UUID)",
            Self::MissingBranch => "automation has no branch_id (nil UUID)",
            Self::BranchMismatch => "automation branch_id does not match the owning bot",
            Self::MissingBotName => "automation bot has no name",
            Self::MissingParam => "automation has no script param",
            Self::UnsafePathComponent => {
                "automation path component contains a separator or traversal"
            }
        };
        f.write_str(message)
    }
}

impl std::error::Error for AutomationScopeError {}

impl AutomationTenantScope {
    /// Build a scope from database values, failing closed when anything is
    /// missing or inconsistent. `automation_branch_id` is the branch stored on
    /// the trigger row; the owning bot's `branch_id`/`org_id` must agree with
    /// it, otherwise the row is refused instead of guessing a tenant.
    pub fn resolve(
        org_id: Uuid,
        bot_branch_id: Uuid,
        automation_branch_id: Uuid,
    ) -> Result<Self, AutomationScopeError> {
        if org_id.is_nil() {
            return Err(AutomationScopeError::MissingOrg);
        }
        if bot_branch_id.is_nil() || automation_branch_id.is_nil() {
            return Err(AutomationScopeError::MissingBranch);
        }
        if bot_branch_id != automation_branch_id {
            return Err(AutomationScopeError::BranchMismatch);
        }
        Ok(Self {
            org_id,
            branch_id: bot_branch_id,
        })
    }
}

/// Build the compiled-script path for a scheduled automation, scoped to the
/// resolved tenant. Pure (no I/O) so the two-org isolation guarantee can be
/// asserted in an inline test without a database.
pub fn automation_ast_path(
    work_root: &str,
    scope: &AutomationTenantScope,
    bot_name: &str,
    param: &str,
) -> Result<String, AutomationScopeError> {
    if bot_name.is_empty() {
        return Err(AutomationScopeError::MissingBotName);
    }
    if param.is_empty() {
        return Err(AutomationScopeError::MissingParam);
    }
    if [bot_name, param]
        .iter()
        .any(|part| part.contains('/') || part.contains('\\') || part.contains(".."))
    {
        return Err(AutomationScopeError::UnsafePathComponent);
    }
    Ok(format!(
        "{work_root}/{}.gborg/{bot_name}.gbai/{bot_name}.gbdialog/{param}.ast",
        scope.org_id
    ))
}

#[derive(Debug)]
pub struct AutomationService {
    state: Arc<AppState>,
}
impl AutomationService {
    #[must_use]
    pub fn new(state: Arc<AppState>) -> Self {
        // episodic_memory scheduler started from botserver layer (botllm depends on botcore)
        Self { state }
    }
    pub async fn spawn(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut ticker = interval(Duration::from_secs(60));
        loop {
            ticker.tick().await;
            if let Err(e) = self.check_scheduled_tasks().await {
                log::error!("Error checking scheduled tasks: {}", e);
            }
        }
    }
    pub async fn check_scheduled_tasks(
        &self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use crate::shared::schema::system_automations::dsl::{
            branch_id as sa_branch_id, id, is_active, kind, last_triggered as lt_column,
            system_automations,
        };
        let mut conn = self
            .state
            .conn
            .get()
            .map_err(|e| format!("Failed to acquire database connection: {}", e))?;
        let automations: Vec<Automation> = system_automations
            .filter(is_active.eq(true))
            .filter(kind.eq(TriggerKind::Scheduled as i32))
            .filter(sa_branch_id.ne(Uuid::nil()))
            .load::<Automation>(&mut conn)?;
        for automation in automations {
            if let Some(schedule_str) = &automation.schedule {
                let normalized_schedule = normalize_cron_schedule(schedule_str);
                trace!(
                    "Parsing schedule: original='{}', normalized='{}'",
                    schedule_str,
                    normalized_schedule
                );
                match Schedule::from_str(&normalized_schedule) {
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
                                    log::error!(
                                        "Error executing automation {}: {}",
                                        automation.id,
                                        e
                                    );
                                    // Fail closed: an automation that could not be
                                    // scoped to a tenant must not be marked as run.
                                    continue;
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
                        trace!(
                            "Skipping automation {} with invalid schedule (original: {}, normalized: {}): {}",
                            automation.id, schedule_str, normalized_schedule, e
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
        let bot_id = automation.bot_id;
        // Resolve the tenant from the trigger row's branch and the owning bot
        // instead of the global nil-org sentinel. Filtering by the row's
        // branch_id means a stale/forged scope simply yields no bot and fails
        // closed rather than reading another org's compiled script.
        let (org_id, bot_branch_id, bot_name): (Uuid, Uuid, String) = {
            use crate::shared::schema::bots::dsl::{
                bots, branch_id as bot_branch_column, id as bot_id_column, name as bot_name_column,
                org_id as bot_org_column,
            };
            let mut conn = self
                .state
                .conn
                .get()
                .map_err(|e| format!("Failed to acquire database connection: {}", e))?;
            bots.filter(bot_id_column.eq(bot_id))
                .filter(bot_branch_column.eq(automation.branch_id))
                .select((bot_org_column, bot_branch_column, bot_name_column))
                .first::<(Uuid, Uuid, String)>(&mut conn)
                .map_err(|e| format!("Failed to resolve tenant scope for bot {bot_id}: {e}"))?
        };
        let scope = AutomationTenantScope::resolve(org_id, bot_branch_id, automation.branch_id)?;
        let script_path = automation_ast_path("./work", &scope, &bot_name, &automation.param)?;
        let script_content = match tokio::fs::read_to_string(&script_path).await {
            Ok(content) => content,
            Err(e) => {
                log::error!("Failed to read script {}: {}", script_path, e);
                return Ok(());
            }
        };
        let session = {
            let mut sm = self.state.session_manager.lock().await;
            sm.get_or_create_user_session(bot_id, bot_id, "Automation")?
                .ok_or("Failed to create session")?
        };
        // ScriptService stays in botserver — use ScriptRunner trait
        if let Some(runner) = &self.state.script_runner {
            match runner
                .run_script(&script_content, session.id, &bot_id.to_string())
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    log::error!("Script execution failed: {}", e);
                }
            }
        } else {
            log::error!(
                "No script runner available for automation {}",
                automation.id
            );
        }
        Ok(())
    }
}

use crate::shared::state::AppState;
use chrono::Utc;
use cron::Schedule;
use diesel::prelude::*;
use std::str::FromStr;
use std::sync::Arc; // incremental test

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_orgs_resolve_distinct_automation_work_dirs() {
        let (org_a, org_b) = (Uuid::new_v4(), Uuid::new_v4());
        let (branch_a, branch_b) = (Uuid::new_v4(), Uuid::new_v4());
        let scope_a = AutomationTenantScope::resolve(org_a, branch_a, branch_a)
            .expect("org A schedule must resolve");
        let scope_b = AutomationTenantScope::resolve(org_b, branch_b, branch_b)
            .expect("org B schedule must resolve");

        // Same bot name and script in both orgs (#2685 name collision): each
        // schedule must still resolve to its own org's work directory.
        let path_a =
            automation_ast_path("./work", &scope_a, "sales", "close_shift").expect("path A");
        let path_b =
            automation_ast_path("./work", &scope_b, "sales", "close_shift").expect("path B");

        assert_ne!(path_a, path_b);
        assert!(path_a.starts_with(&format!("./work/{org_a}.gborg/")));
        assert!(path_b.starts_with(&format!("./work/{org_b}.gborg/")));
        assert!(!path_a.contains(&org_b.to_string()));
        assert!(!path_b.contains(&org_a.to_string()));
        assert_eq!(scope_a.branch_id, branch_a);
        assert_eq!(scope_b.branch_id, branch_b);
    }

    #[test]
    fn nil_org_is_rejected() {
        let branch = Uuid::new_v4();
        assert_eq!(
            AutomationTenantScope::resolve(Uuid::nil(), branch, branch),
            Err(AutomationScopeError::MissingOrg)
        );
    }

    #[test]
    fn nil_branch_is_rejected() {
        let org = Uuid::new_v4();
        assert_eq!(
            AutomationTenantScope::resolve(org, Uuid::nil(), Uuid::nil()),
            Err(AutomationScopeError::MissingBranch)
        );
        assert_eq!(
            AutomationTenantScope::resolve(org, Uuid::nil(), Uuid::new_v4()),
            Err(AutomationScopeError::MissingBranch)
        );
    }

    #[test]
    fn branch_mismatch_is_rejected() {
        assert_eq!(
            AutomationTenantScope::resolve(Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()),
            Err(AutomationScopeError::BranchMismatch)
        );
    }

    #[test]
    fn automation_ast_path_rejects_empty_and_unsafe_components() {
        let branch = Uuid::new_v4();
        let scope = AutomationTenantScope::resolve(Uuid::new_v4(), branch, branch).expect("scope");
        assert_eq!(
            automation_ast_path("./work", &scope, "", "tool"),
            Err(AutomationScopeError::MissingBotName)
        );
        assert_eq!(
            automation_ast_path("./work", &scope, "sales", ""),
            Err(AutomationScopeError::MissingParam)
        );
        assert_eq!(
            automation_ast_path("./work", &scope, "sales", "../evil"),
            Err(AutomationScopeError::UnsafePathComponent)
        );
    }
}
