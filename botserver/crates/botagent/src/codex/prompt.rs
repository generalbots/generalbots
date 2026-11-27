//! Run one non-interactive Codex turn (`codex exec`) inside the VM.

use botsecurity_core::error_sanitizer::SafeErrorResponse;

use crate::cli_turn::{cli_model, failure_reason, plan_limit, run_turn};
use crate::AgentService;

use super::paths::{CODEX_HOME, CODEX_USER, LAST_MESSAGE, PROMPT_TXT};

fn failed(detail: String) -> SafeErrorResponse {
    tracing::error!("codex print turn failed: {detail}");
    SafeErrorResponse::new("internal_error", "Codex account operation failed")
}

/// The `codex exec` invocation run as the `codex` user. The prompt comes from
/// stdin (`-`), the workspace is not a git checkout, and only the final agent
/// message (`--output-last-message`) is the answer; progress goes to stderr.
fn invocation(model: &str) -> String {
    let model = cli_model(model);
    let model_arg = if model.is_empty() {
        String::new()
    } else {
        format!(" --model {model}")
    };
    format!(
        "rm -f {last}; codex exec --skip-git-repo-check --color never{model_arg} \
         --output-last-message {last} - < \"$GB_PROMPT\" >/dev/null && cat {last}",
        last = LAST_MESSAGE,
    )
}

pub async fn run_prompt(
    state: &AgentService,
    vm_name: &str,
    prompt: &str,
    model: &str,
) -> Result<String, SafeErrorResponse> {
    if prompt.trim().is_empty() {
        return Err(SafeErrorResponse::bad_request("Empty prompt"));
    }
    let (ok, stdout, stderr) =
        run_turn(state, vm_name, CODEX_USER, CODEX_HOME, PROMPT_TXT, prompt, &invocation(model)).await?;
    if !ok {
        if let Some(limit) = plan_limit(&stdout, &stderr) {
            return Err(limit);
        }
        return Err(failed(format!(
            "codex exec failed: {}",
            failure_reason(&stdout, &stderr)
        )));
    }
    let answer = stdout.trim().to_string();
    if answer.is_empty() {
        return Err(failed("Codex returned an empty response".to_string()));
    }
    Ok(answer)
}

#[cfg(test)]
mod tests {
    use super::invocation;

    #[test]
    fn codex_runs_headless_outside_a_git_checkout() {
        let cmd = invocation("codex-default");
        assert!(cmd.contains("codex exec --skip-git-repo-check --color never --output-last-message"));
        assert!(!cmd.contains("--model"));
        assert!(cmd.contains(" - < \"$GB_PROMPT\""));
        assert!(!cmd.contains('\''));
    }

    #[test]
    fn a_real_model_is_passed_through() {
        assert!(invocation("gpt-5-codex").contains(" --model gpt-5-codex "));
    }
}
