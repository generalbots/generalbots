//! Run one non-interactive Antigravity turn (`agy --print`) inside the VM.

use axum::http::StatusCode;

use crate::cli_turn::{cli_model, failure_reason, plan_limit, run_turn, TURN_TIMEOUT_SECS};
use crate::AgentService;

use super::paths::{ANTIGRAVITY_HOME, ANTIGRAVITY_USER, PROMPT_TXT};
use super::internal;

/// The `agy` print-mode invocation run as the `antigravity` user. Plain text
/// output is the answer; the CLI's own time limit ends the turn a little
/// before the outer `timeout`, so an unfinished turn reports instead of
/// being killed silently.
fn invocation(model: &str) -> String {
    let model = cli_model(model);
    let model_arg = if model.is_empty() {
        String::new()
    } else {
        format!(" --model {model}")
    };
    format!(
        "xargs -0 -a \"$GB_PROMPT\" agy --output-format text --print-timeout {limit}s{model_arg} --print",
        limit = TURN_TIMEOUT_SECS.saturating_sub(20),
    )
}

pub async fn run_prompt(
    state: &AgentService,
    vm_name: &str,
    prompt: &str,
    model: &str,
) -> Result<String, (StatusCode, String)> {
    if prompt.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Empty prompt".to_string()));
    }
    let (ok, stdout, stderr) = run_turn(
        state,
        vm_name,
        ANTIGRAVITY_USER,
        ANTIGRAVITY_HOME,
        PROMPT_TXT,
        prompt,
        &invocation(model),
    )
    .await?;
    if !ok {
        if let Some(limit) = plan_limit(&stdout, &stderr) {
            return Err(limit);
        }
        return Err(internal(format!(
            "antigravity print failed: {}",
            failure_reason(&stdout, &stderr)
        )));
    }
    let answer = stdout.trim().to_string();
    if answer.is_empty() {
        return Err(internal("Antigravity returned an empty response".to_string()));
    }
    Ok(answer)
}

#[cfg(test)]
mod tests {
    use super::invocation;

    #[test]
    fn agy_runs_in_print_mode_with_a_time_limit() {
        let cmd = invocation("antigravity-default");
        assert!(cmd.starts_with("xargs -0 -a \"$GB_PROMPT\" agy --output-format text --print-timeout 280s"));
        assert!(cmd.ends_with(" --print"), "xargs appends the prompt as the value");
        assert!(!cmd.contains("$("), "the command guard refuses $(");
        assert!(!cmd.contains("--model"));
        assert!(!cmd.contains('\''));
    }

    #[test]
    fn a_real_model_is_passed_through() {
        assert!(invocation("gemini-3-pro").contains(" --model gemini-3-pro "));
    }
}
