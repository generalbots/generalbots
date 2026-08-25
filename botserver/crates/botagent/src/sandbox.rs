//! Sandboxed code execution service (#1172 backend): ephemeral Incus
//! containers with hard resource limits, base64-embedded sources and a
//! strict wall-clock timeout. Every run is recorded in `sandbox_runs`.

use axum::http::StatusCode;
use botlib::security::command_guard::SafeCommand;
use chrono::Utc;
use diesel::prelude::*;
use std::sync::Arc;
use std::time::{Duration, Instant};
use uuid::Uuid;

use crate::models::{ExecBody, SandboxRunRow};
use crate::schema::sandbox_runs;
use crate::vm;
use crate::AgentService;

const ALLOWED_LANGUAGES: &[&str] = &["python", "node", "shell"];
const MAX_CODE_CHARS: usize = 10_000;
const MAX_FILE_COUNT: usize = 10;
const MAX_FILE_CHARS: usize = 20_000;
const MAX_OUTPUT_CHARS: usize = 20_000;
const DEFAULT_TIMEOUT_MS: u64 = 30_000;
const MAX_TIMEOUT_MS: u64 = 120_000;

fn internal(msg: String) -> (StatusCode, String) {
    tracing::error!("{msg}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Sandbox execution failed".to_string(),
    )
}

fn bad_request(msg: &str) -> (StatusCode, String) {
    (StatusCode::BAD_REQUEST, msg.to_string())
}

/// Minimal denylist for shell payloads (defense in depth alongside the
/// container isolation).
pub fn is_dangerous(code: &str) -> bool {
    let lowered = code.to_lowercase();
    ["rm -rf /", "mkfs", "dd if=", ":(){ :|:& };:"]
        .iter()
        .any(|pattern| lowered.contains(pattern))
}

fn interpreter_and_file(language: &str) -> Option<(&'static str, &'static str)> {
    match language {
        "python" => Some(("python3", "/tmp/run.py")),
        "node" => Some(("node", "/tmp/run.js")),
        "shell" => Some(("sh", "/tmp/run.sh")),
        _ => None,
    }
}

fn sanitize_rel_name(name: &str) -> Option<String> {
    let clean: String = name
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
        .collect();
    if clean.is_empty() || clean.starts_with('.') {
        None
    } else {
        Some(clean)
    }
}

async fn run_incus(
    state: &AgentService,
    args: Vec<String>,
) -> Result<std::process::Output, (StatusCode, String)> {
    let mut cmd = SafeCommand::new(state.incus_bin()).map_err(|e| internal(format!("incus guard: {e}")))?;
    for arg in &args {
        cmd = cmd.arg(arg).map_err(|e| internal(format!("incus arg: {e}")))?;
    }
    tokio::task::spawn_blocking(move || cmd.execute())
        .await
        .map_err(|e| internal(format!("incus join: {e}")))?
        .map_err(|e| internal(format!("incus execution: {e}")))
}

/// Execute a shell one-liner inside the sandbox container. The script travels
/// through `trusted_shell_script_arg` (SafeCommand's escape hatch for sh -c);
/// its content never touches a host shell because it runs inside the container.
async fn exec_script(
    state: &AgentService,
    name: &str,
    script: &str,
) -> Result<std::process::Output, (StatusCode, String)> {
    let cmd = SafeCommand::new(state.incus_bin())
        .map_err(|e| internal(format!("incus guard: {e}")))?
        .arg("exec")
        .map_err(|e| internal(format!("incus arg: {e}")))?
        .arg(name)
        .map_err(|e| internal(format!("incus arg: {e}")))?
        .arg("--")
        .map_err(|e| internal(format!("incus arg: {e}")))?
        .arg("sh")
        .map_err(|e| internal(format!("incus arg: {e}")))?
        .arg("-c")
        .map_err(|e| internal(format!("incus arg: {e}")))?
        .trusted_shell_script_arg(script)
        .map_err(|e| internal(format!("sandbox script rejected: {e}")))?;
    tokio::task::spawn_blocking(move || cmd.execute())
        .await
        .map_err(|e| internal(format!("incus join: {e}")))?
        .map_err(|e| internal(format!("sandbox execution: {e}")))
}

/// Best-effort teardown; failures are logged and swallowed.
fn spawn_cleanup(state: &AgentService, name: &str) {
    let bin = state.incus_bin().to_string();
    let name = name.to_string();
    tokio::spawn(async move {
        let cmd = match SafeCommand::new(&bin)
            .map_err(|e| format!("guard: {e}"))
            .and_then(|c| c.arg("delete").map_err(|e| format!("arg: {e}")))
            .and_then(|c| c.arg("--force").map_err(|e| format!("arg: {e}")))
            .and_then(|c| c.arg(&name).map_err(|e| format!("arg: {e}")))
        {
            Ok(cmd) => cmd,
            Err(e) => {
                tracing::warn!("sandbox cleanup build failed for {name}: {e}");
                return;
            }
        };
        if let Ok(Ok(output)) = tokio::task::spawn_blocking(move || cmd.execute()).await {
            if !output.status.success() {
                tracing::warn!(
                    "sandbox cleanup for {name} exited non-zero: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
    });
}

fn clip(text: &str) -> String {
    text.chars().take(MAX_OUTPUT_CHARS).collect()
}

/// Run untrusted code in a throwaway container and persist the result.
pub async fn run_sandbox(
    state: &Arc<AgentService>,
    org_id: Option<Uuid>,
    user_id: Option<Uuid>,
    body: &ExecBody,
) -> Result<serde_json::Value, (StatusCode, String)> {
    let language = body.language.trim().to_lowercase();
    if !ALLOWED_LANGUAGES.contains(&language.as_str()) {
        return Err(bad_request("Unsupported language"));
    }
    if body.code.is_empty() || body.code.chars().count() > MAX_CODE_CHARS {
        return Err(bad_request("Code size out of bounds"));
    }
    if language == "shell" && is_dangerous(&body.code) {
        return Err(bad_request("Code rejected by safety policy"));
    }
    let (interpreter, target) =
        interpreter_and_file(&language).ok_or_else(|| bad_request("Unsupported language"))?;

    let timeout_ms = body.timeout_ms.unwrap_or(DEFAULT_TIMEOUT_MS).min(MAX_TIMEOUT_MS);

    let name = format!("sbx-{}", Uuid::new_v4().simple().to_string().chars().take(8).collect::<String>());

    let started = Instant::now();

    run_incus(
        state,
        vec![
            "launch".to_string(),
            "images:ubuntu/24.04".to_string(),
            name.clone(),
            "--config".to_string(),
            "limits.memory=256MiB".to_string(),
            "--config".to_string(),
            "limits.cpu.priority=10".to_string(),
            "--config".to_string(),
            "security.privileged=false".to_string(),
        ],
    )
    .await?;
    if let Err(e) = vm::wait_for_running(state.as_ref(), &name).await {
        spawn_cleanup(state, &name);
        return Err(e);
    }

    // Stage auxiliary files first so user code can read them at runtime.
    if let Some(files) = &body.files {
        if files.len() > MAX_FILE_COUNT {
            spawn_cleanup(state, &name);
            return Err(bad_request("Too many files"));
        }
        for (raw_name, content) in files {
            let staged = match sanitize_rel_name(raw_name) {
                Some(n) if content.chars().count() <= MAX_FILE_CHARS => n,
                _ => {
                    spawn_cleanup(state, &name);
                    return Err(bad_request("Invalid file entry"));
                }
            };
            let dest = format!("/tmp/f/{staged}");
            let script = format!(
                "mkdir -p /tmp/f && echo {} | base64 -d > {dest}",
                b64_encode(content.as_bytes())
            );
            if let Err(e) = exec_script(state, &name, &script).await {
                tracing::error!("sandbox file staging failed: {}", e.1);
            }
        }
    }

    let script = format!(
        "echo {} | base64 -d > {target} && {interpreter} {target}",
        b64_encode(body.code.as_bytes())
    );

    // The wall-clock timeout bounds the async wait; a timed-out container is
    // force-deleted by cleanup below, which also terminates any running exec.
    let outcome = tokio::time::timeout(
        Duration::from_millis(timeout_ms),
        exec_script(state, &name, &script),
    )
    .await;

    spawn_cleanup(state, &name);

    let duration_ms = started.elapsed().as_millis().min(i32::MAX as u128) as i32;

    let (status, exit_code, stdout_ref, stderr_ref) = match outcome {
        Err(_) => ("timeout".to_string(), None, String::new(), String::new()),
        Ok(Err(handler_err)) => {
            tracing::error!("sandbox exec failed: {}", handler_err.1);
            ("error".to_string(), None, String::new(), handler_err.1)
        }
        Ok(Ok(output)) => (
            if output.status.success() { "completed" } else { "failed" }.to_string(),
            output.status.code(),
            clip(&String::from_utf8_lossy(&output.stdout)),
            clip(&String::from_utf8_lossy(&output.stderr)),
        ),
    };

    let row = SandboxRunRow {
        id: Uuid::new_v4(),
        org_id,
        user_id,
        language: language.clone(),
        status: status.clone(),
        exit_code,
        stdout_ref: Some(stdout_ref.clone()),
        stderr_ref: Some(stderr_ref.clone()),
        duration_ms: Some(duration_ms),
        created_at: Utc::now(),
    };

    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
    diesel::insert_into(sandbox_runs::table)
        .values(&row)
        .execute(&mut conn)
        .map_err(|e| internal(format!("sandbox_runs insert: {e}")))?;

    Ok(serde_json::json!({
        "id": row.id,
        "status": status,
        "exit_code": exit_code,
        "stdout": stdout_ref,
        "stderr": stderr_ref,
        "duration_ms": duration_ms,
    }))
}

/// Dependency-free standard-alphabet Base64 encoder used to embed sources
/// into container shell scripts without newline or metacharacter issues.
pub fn b64_encode(data: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[(triple >> 18) as usize & 0x3F] as char);
        out.push(TABLE[(triple >> 12) as usize & 0x3F] as char);
        if chunk.len() > 1 {
            out.push(TABLE[(triple >> 6) as usize & 0x3F] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[triple as usize & 0x3F] as char);
        } else {
            out.push('=');
        }
    }
    out
}
