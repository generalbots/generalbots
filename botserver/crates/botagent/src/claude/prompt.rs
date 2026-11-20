//! One non-interactive subscription turn and model-name sanitising.

use axum::http::StatusCode;

use crate::cli_turn::{cli_model, failure_reason, run_turn};
use crate::AgentService;

use super::internal;
use super::paths::{CLAUDE_HOME, CLAUDE_USER, PROMPT_TXT};

/// #1939 — delivery policy for VM agent turns.
const DELIVERY_DIRECTIVE: &str = "Deliver the exact artifact the user asked for, in the requested \
format. Prefer a maintained open-source library or command-line tool that already produces it, and \
search the web first when unsure which one is current. Install what the task needs, but stay inside \
the sandbox limits. Never substitute another format silently: if the exact format cannot be produced \
reliably, say so and offer the closest alternative you can verify.";

/// Runs one non-interactive turn (`claude -p`) as the `claude` user. The
/// prompt is base64 encoded on the host and read from stdin, so quotes,
/// newlines and shell metacharacters are data and its length is not bound
/// by the argument limit.
pub async fn run_prompt(
    state: &AgentService,
    vm_name: &str,
    prompt: &str,
    model: &str,
) -> Result<String, (StatusCode, String)> {
    if prompt.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Empty prompt".to_string()));
    }
    // #1939 — the subscription agent gets the same “ask, done” policy as the
    // chat prompt: exact requested format, open-source tooling first.
    let prompt = format!("{DELIVERY_DIRECTIVE}\n\n{prompt}");
    let model = cli_model(model);
    let model_arg = if model.is_empty() {
        String::new()
    } else {
        format!(" --model {model}")
    };
    let invocation = format!("claude -p --output-format text{model_arg} < \"$GB_PROMPT\"");
    let (ok, stdout, stderr) =
        run_turn(state, vm_name, CLAUDE_USER, CLAUDE_HOME, PROMPT_TXT, &prompt, &invocation).await?;
    if !ok {
        return Err(internal(format!(
            "Claude Code failed: {}",
            failure_reason(&stdout, &stderr)
        )));
    }
    let answer = stdout.trim().to_string();
    if answer.is_empty() {
        return Err(internal("Claude Code returned an empty response".to_string()));
    }
    Ok(answer)
}
