//! Run an Antigravity prompt inside the VM (headless mode).

use axum::http::StatusCode;

use crate::AgentService;
use crate::vm;

use super::internal;
use super::paths::ANTIGRAVITY_HOME;

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
        "export HOME={home}; \
         printf '%s' '{prompt_b64}' | base64 -d > /tmp/gb-antigravity-prompt.txt; \
         agy -p \"$(cat /tmp/gb-antigravity-prompt.txt)\" {model_arg} --output-format json 2>/tmp/gb-antigravity-err.txt; \
         echo \"---STDERR---\"; cat /tmp/gb-antigravity-err.txt 2>/dev/null || true",
        home = ANTIGRAVITY_HOME,
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
            "antigravity exec failed: {}",
            if stderr.trim().is_empty() {
                stdout.trim()
            } else {
                stderr.trim()
            }
        )));
    }
    let output = stdout.split("---STDERR---").next().unwrap_or(&stdout).trim().to_string();
    if output.is_empty() {
        Err(internal("Antigravity returned an empty response".to_string()))
    } else {
        // Try to parse JSON output and extract response text
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&output) {
            if let Some(text) = json.get("response").and_then(|v| v.as_str()) {
                return Ok(text.to_string());
            }
            if let Some(text) = json.get("text").and_then(|v| v.as_str()) {
                return Ok(text.to_string());
            }
            if let Some(text) = json.get("output").and_then(|v| v.as_str()) {
                return Ok(text.to_string());
            }
        }
        Ok(output)
    }
}
EOF