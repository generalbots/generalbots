//! Run a Codex prompt inside the VM.

use axum::http::StatusCode;

use crate::AgentService;
use crate::vm;

use super::internal;
use super::paths::CODEX_HOME;

pub async fn run_prompt(
    state: &AgentService,
    vm_name: &str,
    prompt: &str,
    model: &str,
) -> Result<String, (StatusCode, String)> {
    let b64_prompt = base64::engine::general_purpose::STANDARD.encode(prompt.as_bytes());
    let model_arg = if model.trim().is_empty() {
        String::new()
    } else {
        format!("--model {} ", model.trim())
    };
    let script = format!(
        "export HOME={home}; export CODEX_HOME={home}/.codex; \
         printf '%s' '{prompt_b64}' | base64 -d > /tmp/gb-codex-prompt.txt; \
         codex exec {model_arg} \"$(cat /tmp/gb-codex-prompt.txt)\" 2>/tmp/gb-codex-err.txt; \
         echo \"---STDERR---\"; cat /tmp/gb-codex-err.txt 2>/dev/null || true",
        home = CODEX_HOME,
        prompt_b64 = b64_prompt,
        model_arg = model_arg
    );
    let (ok, stdout, stderr) = vm::exec_in_vm(
        state,
        vm_name,
        &["sh".into(), "-c".into(), script],
    )
    .await?;
    if !ok {
        return Err(internal(format!(
            "codex exec failed: {}",
            if stderr.trim().is_empty() {
                stdout.trim()
            } else {
                stderr.trim()
            }
        )));
    }
    // Strip stderr section from output
    let output = stdout.split("---STDERR---").next().unwrap_or(&stdout).trim().to_string();
    if output.is_empty() {
        Err(internal("Codex returned an empty response".to_string()))
    } else {
        Ok(output)
    }
}
