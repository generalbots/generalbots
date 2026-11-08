//! Read-only host prerequisite check for the first-run wizard (#1742).
//!
//! Before starting any bundled service the wizard runs
//! `PROD-DEPENDENCIES.sh --check` (Linux) or `PROD-DEPENDENCIES.ps1 -Check`
//! (Windows). Those scripts install nothing; [`check`] only executes them and
//! [`parse`] classifies every reported gap as desktop shell / agent VM / media.

mod parse;

use botlib::security::command_guard::SafeCommand;
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tauri::Manager;

const CHECK_TIMEOUT_SECS: u64 = 30;
const OUTPUT_TAIL_CHARS: usize = 6000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HostCheckItem {
    /// `desktop`, `agent` or `media`.
    pub feature: String,
    /// Script vocabulary: command, library, package, group, membership, service, storage, runtime.
    pub kind: String,
    pub name: String,
    /// `missing`, `warn` or `ok`.
    pub status: String,
    pub detail: String,
    pub remediation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HostCheckResult {
    pub platform: String,
    /// False when the script could not be located; the wizard then continues.
    pub available: bool,
    pub script: String,
    pub exit_code: i32,
    /// `ready`, `limited`, `desktop-blocked`, `unsupported`, `unavailable` or `unknown`.
    pub status: String,
    pub desktop_ready: bool,
    pub agent_ready: bool,
    pub media_ready: bool,
    pub items: Vec<HostCheckItem>,
    pub remediation: String,
    pub output: String,
    pub message: String,
}

pub(super) fn base(
    platform: &str,
    script: &Path,
    exit_code: i32,
    remediation: String,
    output: &str,
) -> HostCheckResult {
    HostCheckResult {
        platform: platform.to_string(),
        available: true,
        script: script.to_string_lossy().to_string(),
        exit_code,
        status: "unknown".to_string(),
        desktop_ready: true,
        agent_ready: true,
        media_ready: true,
        items: Vec::new(),
        remediation,
        output: output.to_string(),
        message: String::new(),
    }
}

fn unavailable(platform: &str, message: &str) -> HostCheckResult {
    HostCheckResult {
        platform: platform.to_string(),
        available: false,
        script: String::new(),
        exit_code: 0,
        status: "unavailable".to_string(),
        desktop_ready: true,
        agent_ready: true,
        media_ready: true,
        items: Vec::new(),
        remediation: String::new(),
        output: String::new(),
        message: message.to_string(),
    }
}

pub(super) fn default_remediation(platform: &str, script: &Path) -> String {
    if platform == "windows" {
        let name = script
            .file_name()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_else(|| "PROD-DEPENDENCIES.ps1".to_string());
        format!(".\\{name} -Install")
    } else {
        let display = script.display();
        format!("sudo {display} --install")
    }
}

pub(super) fn summarize(result: &mut HostCheckResult) {
    if result.exit_code == 3 {
        result.status = "unsupported".to_string();
        result.message = "The host check reported an unsupported platform.".to_string();
        return;
    }
    if !result.desktop_ready {
        let count = result
            .items
            .iter()
            .filter(|item| item.feature == "desktop" && item.status == "missing")
            .count();
        result.status = "desktop-blocked".to_string();
        result.message =
            format!("The desktop shell cannot run: {count} required host prerequisite(s) missing.");
        return;
    }
    if !result.agent_ready || !result.media_ready {
        result.status = "limited".to_string();
        result.message =
            "Chat and Drive can run; optional features need host prerequisites.".to_string();
        return;
    }
    result.status = "ready".to_string();
    result.message = "All runtime prerequisites are satisfied.".to_string();
}

pub(super) fn classify(result: &mut HostCheckResult) {
    let missing = |feature: &str, items: &[HostCheckItem]| {
        items
            .iter()
            .any(|item| item.feature == feature && item.status == "missing")
    };
    result.desktop_ready = !missing("desktop", &result.items);
    result.agent_ready = !missing("agent", &result.items);
    result.media_ready = !missing("media", &result.items);
}

fn tail(text: &str, limit: usize) -> String {
    let total = text.chars().count();
    if total <= limit {
        return text.to_string();
    }
    text.chars().skip(total - limit).collect()
}

fn resolve_script(app: &tauri::AppHandle, platform: &str) -> Option<PathBuf> {
    let name = if platform == "windows" {
        "PROD-DEPENDENCIES.ps1"
    } else {
        "PROD-DEPENDENCIES.sh"
    };
    if let Ok(dir) = std::env::var("GB_DEPENDENCIES_DIR") {
        let candidate = PathBuf::from(dir).join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    let mut roots: Vec<PathBuf> = Vec::new();
    // Portable builds extract the checker into their per-user payload stack.
    roots.push(crate::desktop::sidecar::Sidecar::stack_path());
    if let Ok(dir) = app.path().resource_dir() {
        roots.push(dir);
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            roots.push(dir.to_path_buf());
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        roots.push(cwd);
    }
    #[cfg(debug_assertions)]
    {
        if let Some(repo) = PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent() {
            roots.push(repo.to_path_buf());
        }
    }
    for root in roots {
        let mut current = Some(root.as_path());
        for _ in 0..4 {
            let Some(dir) = current else { break };
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
            current = dir.parent();
        }
    }
    None
}

fn safe_command(platform: &str, script: &Path) -> Result<SafeCommand, String> {
    let script_arg = script.to_string_lossy();
    let script_ref: &str = script_arg.as_ref();
    let (mut command, args): (SafeCommand, Vec<&str>) = if platform == "windows" {
        (
            SafeCommand::new("powershell").map_err(|e| e.to_string())?,
            vec![
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
                script_ref,
                "-Check",
            ],
        )
    } else {
        (
            SafeCommand::new("bash").map_err(|e| e.to_string())?,
            vec![script_ref, "--check"],
        )
    };
    for value in &args {
        command = command.trusted_arg(value).map_err(|e| e.to_string())?;
    }
    Ok(command)
}

async fn read_pipe<R: Read + Send + 'static>(pipe: Option<R>) -> String {
    match pipe {
        Some(mut pipe) => tokio::task::spawn_blocking(move || {
            let mut text = String::new();
            let _ = pipe.read_to_string(&mut text);
            text
        })
        .await
        .unwrap_or_default(),
        None => String::new(),
    }
}

async fn run_script(platform: &str, script: &Path) -> Result<(i32, String), String> {
    let mut child = safe_command(platform, script)?
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("could not start {display}: {e}", display = script.display()))?;
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(CHECK_TIMEOUT_SECS);
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {}
            Err(e) => return Err(format!("could not read the host check result: {e}")),
        }
        if tokio::time::Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err("the host check did not finish in time".to_string());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    };
    let mut text = read_pipe(stdout).await;
    let stderr = read_pipe(stderr).await;
    if !stderr.trim().is_empty() {
        if !text.is_empty() {
            text.push('\n');
        }
        text.push_str(&stderr);
    }
    Ok((status.code().unwrap_or(-1), text))
}

/// Runs the platform script read-only and returns the structured report.
pub async fn check(app: &tauri::AppHandle) -> HostCheckResult {
    let platform = std::env::consts::OS;
    if platform != "linux" && platform != "windows" {
        return unavailable(
            platform,
            "Host prerequisites are not checked on this platform; setup continues without it.",
        );
    }
    let Some(script) = resolve_script(app, platform) else {
        return unavailable(
            platform,
            "The host check script was not found next to the application; setup continues without it.",
        );
    };
    match run_script(platform, &script).await {
        Ok((exit_code, output)) => {
            let mut result = if platform == "windows" {
                parse::windows(&output, exit_code, &script)
            } else {
                parse::linux(&output, exit_code, &script)
            };
            result.output = tail(&result.output, OUTPUT_TAIL_CHARS);
            result
        }
        Err(reason) => {
            let mut result = base(
                platform,
                &script,
                -1,
                default_remediation(platform, &script),
                "",
            );
            result.status = "unknown".to_string();
            result.message =
                format!("The host check could not run: {reason}. Setup continues without it.");
            result
        }
    }
}
