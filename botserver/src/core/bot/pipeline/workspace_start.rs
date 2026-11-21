//! Bounded, visible workspace start before a turn that needs it.
//!
//! Claude Code, Codex and Antigravity answer from inside the user's agent
//! workspace. When the workspace was stopped, its start (or the first
//! provisioning) ran silently for minutes before the answer came. The start
//! now shows as the "starting the virtual machine" step, and the turn waits
//! at most [`START_BUDGET`] before it tells the user to try again. The start
//! itself keeps running in the background, so the next turn finds it ready.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use botagent::models::AgentSessionRow;
use tokio::task::JoinHandle;
use uuid::Uuid;

use super::types::SessionIdentity;
use super::workspace_progress::{WorkspaceProgress, PHASE_INFRA};
use crate::core::bot::agent_vm_hook::WorkspaceError;

/// Longest a turn waits for the workspace to start.
pub(super) const START_BUDGET: Duration = Duration::from_secs(180);
/// A start that finishes within this shows no step at all.
const QUIET_START: Duration = Duration::from_secs(2);
/// A turn that timed out marks the user; a second wait inside the same turn
/// (or right after it) fails fast while that start is still running.
const RECENT_TIMEOUT: Duration = Duration::from_secs(60);
const STEP: &str = "container_start";

fn timeouts() -> &'static Mutex<HashMap<Uuid, Instant>> {
    static TIMEOUTS: OnceLock<Mutex<HashMap<Uuid, Instant>>> = OnceLock::new();
    TIMEOUTS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn timed_out_recently(user_id: Uuid) -> bool {
    let Ok(mut map) = timeouts().lock() else {
        return false;
    };
    map.retain(|_, at| at.elapsed() < RECENT_TIMEOUT);
    map.contains_key(&user_id)
}

fn note_timeout(user_id: Uuid, timed_out: bool) {
    if let Ok(mut map) = timeouts().lock() {
        if timed_out {
            map.insert(user_id, Instant::now());
        } else {
            map.remove(&user_id);
        }
    }
}

fn spawn_start(
    session_id: Uuid,
    user_id: Uuid,
    bot_uuid: Uuid,
) -> JoinHandle<Result<AgentSessionRow, WorkspaceError>> {
    tokio::spawn(crate::core::bot::agent_vm_hook::ensure_agent_vm_result(
        session_id, user_id, bot_uuid,
    ))
}

/// `None` when the start did not finish within `limit`; the task keeps
/// running.
async fn wait_for(
    task: &mut JoinHandle<Result<AgentSessionRow, WorkspaceError>>,
    limit: Duration,
) -> Option<Result<AgentSessionRow, WorkspaceError>> {
    match tokio::time::timeout(limit, task).await {
        Err(_) => None,
        Ok(Ok(result)) => Some(result),
        Ok(Err(join_error)) => Some(Err((500, format!("workspace start task failed: {join_error}")))),
    }
}

fn timeout_message(user_id: Uuid, pool: &botlib::db_pool::DbPool) -> String {
    let locale = botcore::i18n::locale_for_user_pool(pool, user_id);
    botcore::i18n::t(&locale, "error-workspace-start-timeout")
}

/// [`crate::core::bot::agent_vm_hook::ensure_agent_vm_result`] bounded by
/// [`START_BUDGET`]. The timeout error carries the user's message.
pub(super) async fn ensure_within_budget(
    identity: &SessionIdentity<'_>,
) -> Result<AgentSessionRow, WorkspaceError> {
    let user_id = identity.user_id;
    if botagent::vm::start_in_progress(&user_id) && timed_out_recently(user_id) {
        return Err((504, timeout_message(user_id, &identity.state.conn)));
    }
    let mut task = spawn_start(identity.session_id, user_id, identity.bot_uuid);
    let result = wait_for(&mut task, START_BUDGET).await;
    finish(identity, result)
}

fn finish(
    identity: &SessionIdentity<'_>,
    result: Option<Result<AgentSessionRow, WorkspaceError>>,
) -> Result<AgentSessionRow, WorkspaceError> {
    note_timeout(identity.user_id, result.is_none());
    match result {
        Some(outcome) => outcome,
        None => {
            botlog::safe_warn!(
                "workspace for user {} did not start within {}s; the start continues in the background",
                identity.user_id,
                START_BUDGET.as_secs()
            );
            Err((504, timeout_message(identity.user_id, &identity.state.conn)))
        }
    }
}

/// True when this turn is answered by a subscription CLI inside the
/// workspace (`sub:<provider>` in the chat, or the bot's own provider).
fn needs_workspace(identity: &SessionIdentity<'_>, model_preference: &str) -> bool {
    use crate::core::ai_accounts as acc;
    let preference = model_preference.trim();
    if preference.starts_with("acct:") {
        return false;
    }
    let provider = match acc::models::parse_choice(preference) {
        Some((agent, _)) => agent.to_string(),
        None => botcore::config::ConfigManager::new(identity.state.conn.clone())
            .get_config(&identity.bot_uuid, "llm-provider", Some(""))
            .unwrap_or_default(),
    };
    is_workspace_provider(&provider)
}

fn is_workspace_provider(provider: &str) -> bool {
    use crate::core::ai_accounts as acc;
    matches!(provider.trim(), acc::CLAUDE_CODE | acc::CODEX_CLI | acc::ANTIGRAVITY)
}

/// Before a subscription-CLI turn: make sure the workspace runs, showing the
/// start as a step when it takes more than a moment. `Err` is the message
/// for the user; the turn ends with it.
pub(super) async fn before_subscription_turn(
    identity: &SessionIdentity<'_>,
    model_preference: &str,
) -> Result<(), String> {
    if identity.user_id.is_nil()
        || !super::agent_turn::is_enabled(identity.session_id)
        || crate::core::bot::agent_vm_hook::service().is_none()
        || !needs_workspace(identity, model_preference)
    {
        return Ok(());
    }
    let user_id = identity.user_id;
    let locale = botcore::i18n::locale_for_user_pool(&identity.state.conn, user_id);
    if botagent::vm::start_in_progress(&user_id) && timed_out_recently(user_id) {
        return Err(botcore::i18n::t(&locale, "error-workspace-start-timeout"));
    }
    let mut task = spawn_start(identity.session_id, user_id, identity.bot_uuid);
    if let Some(result) = wait_for(&mut task, QUIET_START).await {
        return quiet_outcome(identity, result, &locale);
    }
    let progress = WorkspaceProgress::new(identity);
    if let Some(progress) = progress.as_ref() {
        progress
            .start(
                PHASE_INFRA,
                STEP,
                &botcore::i18n::t(&locale, "workspace-step-starting-vm"),
                &botcore::i18n::t(&locale, "workspace-step-starting-vm-detail"),
            )
            .await;
    }
    let result = wait_for(&mut task, START_BUDGET.saturating_sub(QUIET_START)).await;
    let outcome = finish(identity, result);
    let message = outcome.as_ref().err().map(|error| user_message(error, &locale));
    if let Some(progress) = progress.as_ref() {
        match &message {
            None => progress.finish(PHASE_INFRA, STEP, "ok", "").await,
            Some(text) => progress.finish(PHASE_INFRA, STEP, "error", text).await,
        }
    }
    match message {
        None => Ok(()),
        Some(text) => Err(text),
    }
}

fn quiet_outcome(
    identity: &SessionIdentity<'_>,
    result: Result<AgentSessionRow, WorkspaceError>,
    locale: &botcore::i18n::Locale,
) -> Result<(), String> {
    note_timeout(identity.user_id, false);
    result.map(|_| ()).map_err(|error| user_message(&error, locale))
}

/// The timeout already carries the user's message; any other failure keeps
/// its detail in the log and gives the user a plain one.
fn user_message(error: &WorkspaceError, locale: &botcore::i18n::Locale) -> String {
    let (code, detail) = error;
    if *code == 504 {
        return detail.clone();
    }
    botlog::safe_warn!("workspace start before a subscription turn failed ({code}): {detail}");
    botcore::i18n::t(locale, "error-workspace-start-failed")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_workspace_clis_need_the_workspace() {
        assert!(is_workspace_provider("claude-code"));
        assert!(is_workspace_provider("codex-cli"));
        assert!(is_workspace_provider("antigravity"));
        assert!(!is_workspace_provider("chatgpt-plan"));
        assert!(!is_workspace_provider("openai"));
        assert!(!is_workspace_provider(""));
    }

    #[test]
    fn a_timeout_is_remembered_and_cleared() {
        let user = Uuid::new_v4();
        assert!(!timed_out_recently(user));
        note_timeout(user, true);
        assert!(timed_out_recently(user));
        note_timeout(user, false);
        assert!(!timed_out_recently(user));
    }

    #[test]
    fn the_budget_is_three_minutes_and_the_quiet_window_short() {
        assert_eq!(START_BUDGET, Duration::from_secs(180));
        assert!(QUIET_START < Duration::from_secs(5));
    }

    #[tokio::test]
    async fn a_slow_start_is_left_running_after_the_limit() {
        let mut task: JoinHandle<Result<AgentSessionRow, WorkspaceError>> = tokio::spawn(async {
            tokio::time::sleep(Duration::from_secs(30)).await;
            Err((500, "late".to_string()))
        });
        assert!(wait_for(&mut task, Duration::from_millis(50)).await.is_none());
        assert!(!task.is_finished());
        task.abort();
    }
}
