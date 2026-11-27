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
use crate::llm::subscription::workspace_ready;

/// Longest a turn waits for the workspace to start.
pub(super) const START_BUDGET: Duration = Duration::from_secs(180);
/// A start that finishes within this shows no step at all.
const QUIET_START: Duration = Duration::from_secs(2);
/// The quiet window when the records show the workspace running: checking a
/// running container takes 2 to 4 s through WSL (more while another check of
/// the same person holds the lock), a real start much longer.
const QUIET_CHECK: Duration = Duration::from_secs(6);
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
    tokio::spawn(async move {
        let result =
            crate::core::bot::agent_vm_hook::ensure_agent_vm_result(session_id, user_id, bot_uuid).await;
        workspace_ready::note_check(user_id, result.is_ok());
        result
    })
}

/// True when the records show the person's workspace running.
fn recorded_running(user_id: Uuid) -> bool {
    crate::core::bot::agent_vm_hook::service().is_some_and(|service| {
        matches!(botagent::vm::get_user_vm(service, &user_id), Ok(Some(row)) if row.status == botagent::vm::STATUS_RUNNING)
    })
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
/// [`START_BUDGET`]. The timeout error carries the user's message. A
/// workspace that answers within [`QUIET_START`] shows no step: the start-up
/// step appears (and then finishes) only when the person really waits.
pub(super) async fn ensure_within_budget(
    identity: &SessionIdentity<'_>,
) -> Result<AgentSessionRow, WorkspaceError> {
    let user_id = identity.user_id;
    if botagent::vm::start_in_progress(&user_id) && timed_out_recently(user_id) {
        return Err((504, timeout_message(user_id, &identity.state.conn)));
    }
    let mut task = spawn_start(identity.session_id, user_id, identity.bot_uuid);
    if let Some(result) = wait_for(&mut task, QUIET_START).await {
        note_timeout(user_id, false);
        return result;
    }
    let locale = botcore::i18n::locale_for_user_pool(&identity.state.conn, user_id);
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
    if let Some(progress) = progress.as_ref() {
        match &outcome {
            Ok(_) => progress.finish(PHASE_INFRA, STEP, "ok", "").await,
            Err(error) => {
                progress
                    .finish(PHASE_INFRA, STEP, "error", &user_message(error, &locale))
                    .await;
            }
        }
    }
    outcome
}

/// Starts the same check without waiting for it, for a workspace the records
/// already show running; a failure is logged.
pub(super) fn check_in_background(identity: &SessionIdentity<'_>) {
    let user_id = identity.user_id;
    let task = spawn_start(identity.session_id, user_id, identity.bot_uuid);
    tokio::spawn(async move {
        match task.await {
            Ok(Ok(_)) => {}
            Ok(Err((_, error))) => {
                botlog::safe_warn!("background workspace check for user {user_id}: {error}");
            }
            Err(join_error) => {
                botlog::safe_warn!("background workspace check for user {user_id} failed: {join_error}");
            }
        }
    });
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
            .unwrap_or_else(|e| { log::debug!("using default for llm-provider config: {e}"); Default::default() }),
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
) -> Result<(), super::llm::refusal::RefusalError> {
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
        return Err(workspace_refusal(identity, model_preference,
            botcore::i18n::t(&locale, "error-workspace-start-timeout")));
    }
    // Verified a moment ago and still recorded running: the check that runs
    // beside the turn (`agent_vm_context`) keeps it honest, so this turn
    // neither waits nor shows the start-up step.
    let running = recorded_running(user_id);
    if running && workspace_ready::verified_recently(user_id) {
        return Ok(());
    }
    let quiet = if running { QUIET_CHECK } else { QUIET_START };
    let mut task = spawn_start(identity.session_id, user_id, identity.bot_uuid);
    if let Some(result) = wait_for(&mut task, quiet).await {
        return quiet_outcome(identity, model_preference, result, &locale);
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
    let result = wait_for(&mut task, START_BUDGET.saturating_sub(quiet)).await;
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
        Some(text) => Err(workspace_refusal(identity, model_preference, text)),
    }
}

fn quiet_outcome(
    identity: &SessionIdentity<'_>,
    model_preference: &str,
    result: Result<AgentSessionRow, WorkspaceError>,
    locale: &botcore::i18n::Locale,
) -> Result<(), super::llm::refusal::RefusalError> {
    note_timeout(identity.user_id, false);
    result.map(|_| ()).map_err(|error| {
        workspace_refusal(identity, model_preference, user_message(&error, locale))
    })
}

/// Builds an `agent_workspace_failed` refusal from the model preference.
fn workspace_refusal(
    identity: &SessionIdentity<'_>,
    model_preference: &str,
    message: String,
) -> super::llm::refusal::RefusalError {
    use super::llm::refusal::{self, RefusalError};
    use crate::core::ai_accounts as acc;
    let agent = acc::models::parse_choice(model_preference.trim())
        .map(|(a, _)| a.to_string())
        .unwrap_or_else(|| {
            botcore::config::ConfigManager::new(identity.state.conn.clone())
                .get_config(&identity.bot_uuid, "llm-provider", Some(""))
                .unwrap_or_else(|e| { log::debug!("using default for llm-provider config: {e}"); Default::default() })
        });
    let label = acc::provider_label(&agent);
    botlog::safe_warn!(
        "Subscription turn refused: code=agent_workspace_failed agent=[{agent}] user={}, session={}",
        identity.user_id, identity.session_id
    );
    RefusalError::new(refusal::AGENT_WORKSPACE_FAILED, message)
        .param("agent", agent)
        .param("label", label)
}

/// The message key for a failure of the virtual machine service itself
/// (restarting, or not answering), which the person can act on.
fn service_message_key(error: &WorkspaceError) -> Option<&'static str> {
    let status = axum::http::StatusCode::from_u16(error.0).ok()?;
    if status != axum::http::StatusCode::SERVICE_UNAVAILABLE {
        return None;
    }
    let code = [
        botagent::vm::SERVICE_RESTARTING_CODE,
        botagent::vm::SERVICE_FAILED_CODE,
    ]
    .into_iter()
    .find(|code| error.1.starts_with(code))?;
    let envelope = botsecurity_core::error_sanitizer::SafeErrorResponse::new("workspace_error", error.1.clone())
        .with_code(code);
    match botagent::vm::service_code(&envelope)? {
        botagent::vm::SERVICE_RESTARTING_CODE => Some("error-workspace-service-restarting"),
        botagent::vm::SERVICE_FAILED_CODE => Some("error-workspace-service-failed"),
        _ => None,
    }
}

/// The timeout already carries the user's message; any other failure keeps
/// its detail in the log and gives the user a plain one.
fn user_message(error: &WorkspaceError, locale: &botcore::i18n::Locale) -> String {
    let (code, detail) = error;
    if *code == 504 {
        return detail.clone();
    }
    botlog::safe_warn!("workspace start before a subscription turn failed ({code}): {detail}");
    let key = service_message_key(error).unwrap_or("error-workspace-start-failed");
    botcore::i18n::t(locale, key)
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
        assert!(QUIET_START < QUIET_CHECK && QUIET_CHECK < Duration::from_secs(10));
    }

    #[test]
    fn a_service_failure_gets_its_own_message() {
        let restarting = (503, format!("{}: restarting", botagent::vm::SERVICE_RESTARTING_CODE));
        let failed = (503, format!("{}: no answer", botagent::vm::SERVICE_FAILED_CODE));
        assert_eq!(service_message_key(&restarting), Some("error-workspace-service-restarting"));
        assert_eq!(service_message_key(&failed), Some("error-workspace-service-failed"));
        assert_eq!(service_message_key(&(500, "incus_service_failed".to_string())), None);
        assert_eq!(service_message_key(&(503, "other".to_string())), None);
        botcore::i18n::ensure_initialized();
        let en = botcore::i18n::Locale::new("en").unwrap_or_default();
        let pt = botcore::i18n::Locale::new("pt-BR").unwrap_or_default();
        assert_eq!(
            user_message(&restarting, &en),
            "Restarting the Virtual Machine service…"
        );
        assert_eq!(
            user_message(&failed, &pt),
            "O serviço da Máquina Virtual não respondeu. Reinicie o computador ou o WSL."
        );
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
