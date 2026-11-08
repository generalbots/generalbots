//! One non-interactive subscription turn and model-name sanitising.

use axum::http::StatusCode;

use crate::AgentService;

use super::b64;
use super::internal;
use super::paths::{CLAUDE_HOME, CLAUDE_USER, PROMPT_B64, PROMPT_TXT};
use super::status::run_script;

/// #1939 — delivery policy for VM agent turns.
const DELIVERY_DIRECTIVE: &str = "Deliver the exact artifact the user asked for, in the requested \
format. Prefer a maintained open-source library or command-line tool that already produces it, and \
search the web first when unsure which one is current. Install what the task needs, but stay inside \
the sandbox limits. Never substitute another format silently: if the exact format cannot be produced \
reliably, say so and offer the closest alternative you can verify.";

/// Runs one non-interactive turn with the subscription. The prompt is base64
/// encoded on the host, so quotes, newlines and shell metacharacters are data.
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
    let model = sanitize_model(model);
    let model_arg = if model.is_empty() {
        String::new()
    } else {
        format!(" --model {model}")
    };
    let script = format!(
        "printf '%s' '{payload}' | base64 -d > {b64_path}; \
         chown {user}:{user} {b64_path}; chmod 600 {b64_path}; \
         base64 -d {b64_path} > {txt_path}; chown {user}:{user} {txt_path}; chmod 600 {txt_path}; \
         timeout 300 su {user} -s /bin/sh -c 'cd {home} && claude -p \"$(cat {txt_path})\" --output-format text{model_arg}'",
        payload = b64(prompt.as_str()),
        b64_path = PROMPT_B64,
        txt_path = PROMPT_TXT,
        user = CLAUDE_USER,
        home = CLAUDE_HOME,
        model_arg = model_arg
    );
    let (ok, stdout, stderr) = run_script(state, vm_name, &script).await?;
    if !ok {
        let reason = if stderr.trim().is_empty() {
            stdout.trim()
        } else {
            stderr.trim()
        };
        return Err(internal(format!("Claude Code failed: {reason}")));
    }
    Ok(stdout.trim().to_string())
}

fn sanitize_model(model: &str) -> String {
    model
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '.' | '_'))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::sanitize_model;

    #[test]
    fn model_is_restricted_to_a_safe_alphabet() {
        assert_eq!(
            sanitize_model("claude-sonnet-4.5; rm -rf /"),
            "claude-sonnet-4.5rm-rf"
        );
        assert_eq!(sanitize_model(""), "");
    }
}
