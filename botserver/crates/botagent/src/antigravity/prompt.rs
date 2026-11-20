//! Run one non-interactive Antigravity turn (`agy --print`) inside the VM.

use axum::http::StatusCode;

use crate::cli_turn::{cli_model, failure_reason, turn_script, TURN_TIMEOUT_SECS};
use crate::vm;
use crate::AgentService;

use super::paths::{ANTIGRAVITY_HOME, ANTIGRAVITY_USER, PROMPT_TXT};
use super::{b64, internal};

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
        "agy --output-format text --print-timeout {limit}s{model_arg} --print \"$(cat \"$GB_PROMPT\")\"",
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
    let script = turn_script(
        ANTIGRAVITY_USER,
        ANTIGRAVITY_HOME,
        PROMPT_TXT,
        &b64(prompt),
        &invocation(model),
    );
    let (ok, stdout, stderr) =
        vm::exec_in_vm(state, vm_name, &["sh".into(), "-c".into(), script]).await?;
    if !ok {
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
        assert!(cmd.starts_with("agy --output-format text --print-timeout 280s --print "));
        assert!(!cmd.contains("--model"));
        assert!(!cmd.contains('\''));
    }

    #[test]
    fn a_real_model_is_passed_through() {
        assert!(invocation("gemini-3-pro").contains(" --model gemini-3-pro "));
    }
}
