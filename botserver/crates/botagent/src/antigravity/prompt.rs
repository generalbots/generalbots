//! Run one non-interactive Antigravity turn (`agy --print`) inside the VM.

use axum::http::StatusCode;

use crate::cli_turn::{cli_model, failure_reason, plan_limit, run_turn, stage_file, TURN_TIMEOUT_SECS};
use crate::vm;
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

/// Largest prompt passed to `agy` as its argument. Linux refuses a single
/// argument over 128 KiB, so a longer conversation or paste would fail.
const ARG_LIMIT: usize = 96 * 1024;

/// Where a prompt over [`ARG_LIMIT`] is kept whole for the turn (0600, owned
/// by the CLI user, removed after the turn).
const FULL_PROMPT: &str = "/home/antigravity/.gb-turn-prompt.txt";

/// What `agy` receives as its argument: the prompt itself, or, when it is too
/// long for an argument, an instruction to read the whole of it from
/// [`FULL_PROMPT`]. Nothing is cut either way.
fn argument_prompt(prompt: &str) -> Option<String> {
    (prompt.len() > ARG_LIMIT).then(|| {
        format!(
            "This turn's whole conversation, ending with the person's latest message, is in the file              {FULL_PROMPT} ({} characters); it is too long to pass here. Read the entire file before              answering, then answer the latest message in it, in the person's language.",
            prompt.chars().count()
        )
    })
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
    let pointer = argument_prompt(prompt);
    if pointer.is_some() {
        stage_file(state, vm_name, ANTIGRAVITY_USER, FULL_PROMPT, prompt.as_bytes().to_vec()).await?;
    }
    let turn = run_turn(
        state,
        vm_name,
        ANTIGRAVITY_USER,
        ANTIGRAVITY_HOME,
        PROMPT_TXT,
        pointer.as_deref().unwrap_or(prompt),
        &invocation(model),
    )
    .await;
    if pointer.is_some() {
        let clean = ["sh".to_string(), "-c".to_string(), format!("rm -f {FULL_PROMPT}")];
        if vm::exec_in_vm(state, vm_name, &clean).await.is_err() {
            tracing::debug!("antigravity: the long prompt file stays until the next turn");
        }
    }
    let (ok, stdout, stderr) = turn?;
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
    use super::{argument_prompt, invocation, ARG_LIMIT, FULL_PROMPT};

    #[test]
    fn a_long_prompt_is_read_from_a_file_and_never_cut() {
        assert_eq!(argument_prompt("olá"), None);
        let long = "é".repeat(ARG_LIMIT);
        let pointer = argument_prompt(&long).expect("over the argument limit");
        assert!(pointer.contains(FULL_PROMPT));
        assert!(pointer.contains(&format!("({} characters)", ARG_LIMIT)));
        assert!(pointer.len() < 1024);
    }

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
