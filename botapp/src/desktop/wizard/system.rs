//! Local checks used by the wizard: workspace writability, model reachability,
//! stack reset and OS folder/URL opening. Nothing here contacts a service
//! unless the user explicitly asked to test a model provider.

use super::{log_path, stack_path, workspace_path, ModelRequest};
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceCheck {
    pub writable: bool,
    pub free_bytes: u64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelTestResult {
    pub ok: bool,
    pub message: String,
    pub latency_ms: u64,
}

/// #2693 — resolve a webview-supplied workspace directory before any disk
/// operation. The string arrives from the sidecar origin, so it is hostile:
/// `fs_scope::resolve` canonicalises it (defeating `..` and symlinks) and
/// confines it to the desktop's configured roots. The subdirectory rule keeps
/// filesystem roots such as `/` or `C:\` from becoming the workspace itself.
/// Both the workspace probe and the provisioner call this before touching disk.
pub(crate) fn resolve_workspace(raw: &str) -> Result<PathBuf, String> {
    if raw.trim().is_empty() {
        return Err("Choose a folder for the workspace.".to_string());
    }
    if raw.contains('\0') {
        return Err("A workspace path cannot contain a NUL byte.".to_string());
    }
    let resolved = crate::desktop::fs_scope::resolve(raw)?;
    if resolved.parent().and_then(|parent| parent.file_name()).is_none()
        || resolved.file_name().is_none()
    {
        return Err("Choose a workspace folder below a named directory.".to_string());
    }
    // `fs_scope::resolve` checks protected names on the *input*; canonicalising
    // may have exposed one through a symlink, so check the result as well.
    crate::desktop::fs_scope::check_not_protected(&resolved)?;
    Ok(resolved)
}

pub fn check_workspace(path: &str) -> WorkspaceCheck {
    let dir = match resolve_workspace(path) {
        Ok(dir) => dir,
        Err(reason) => {
            return WorkspaceCheck {
                writable: false,
                free_bytes: 0,
                reason,
            };
        }
    };
    if let Err(e) = std::fs::create_dir_all(&dir) {
        return WorkspaceCheck {
            writable: false,
            free_bytes: 0,
            reason: format!("The folder cannot be created: {e}"),
        };
    }
    let probe = dir.join(".gb-write-test");
    if let Err(e) = std::fs::write(&probe, b"ok") {
        return WorkspaceCheck {
            writable: false,
            free_bytes: free_bytes(&dir),
            reason: format!("The folder is not writable: {e}"),
        };
    }
    let _ = std::fs::remove_file(&probe);
    WorkspaceCheck {
        writable: true,
        free_bytes: free_bytes(&dir),
        reason: String::new(),
    }
}

#[cfg(unix)]
pub fn free_bytes(path: &Path) -> u64 {
    use std::ffi::CString;
    let Ok(raw) = CString::new(path.to_string_lossy().as_bytes()) else {
        return 0;
    };
    // SAFETY: `statvfs` only writes into the provided struct, which is fully
    // initialised first; the path pointer comes from a valid CString.
    unsafe {
        let mut stats: libc::statvfs = std::mem::zeroed();
        if libc::statvfs(raw.as_ptr(), &mut stats) != 0 {
            return 0;
        }
        (stats.f_bavail as u64).saturating_mul(stats.f_frsize as u64)
    }
}

#[cfg(windows)]
pub fn free_bytes(path: &Path) -> u64 {
    use std::os::windows::ffi::OsStrExt;
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let mut free_bytes_available: u64 = 0;
    let mut total_bytes: u64 = 0;
    let mut total_free_bytes: u64 = 0;
    extern "system" {
        fn GetDiskFreeSpaceExW(
            lpDirectoryName: *const u16,
            lpFreeBytesAvailableToCaller: *mut u64,
            lpTotalNumberOfBytes: *mut u64,
            lpTotalNumberOfFreeBytes: *mut u64,
        ) -> i32;
    }
    unsafe {
        if GetDiskFreeSpaceExW(
            wide.as_ptr(),
            &mut free_bytes_available,
            &mut total_bytes,
            &mut total_free_bytes,
        ) != 0 {
            free_bytes_available
        } else {
            0
        }
    }
}

#[cfg(not(any(unix, windows)))]
pub fn free_bytes(_path: &Path) -> u64 {
    0
}

pub async fn test_model(request: ModelRequest) -> ModelTestResult {
    if request.mode == "local" {
        let stack_bin = stack_path().join("bin").join("llm");
        let bundled = installers_have_llm();
        if stack_bin.exists() || bundled {
            return ModelTestResult {
                ok: true,
                message: "Bundled model is available on this machine.".to_string(),
                latency_ms: 0,
            };
        }
        return ModelTestResult {
            ok: false,
            message: "The bundled model runtime is missing from the installer bundle.".to_string(),
            latency_ms: 0,
        };
    }

    if request.endpoint.trim().is_empty() {
        return ModelTestResult {
            ok: false,
            message: "Enter the provider endpoint.".to_string(),
            latency_ms: 0,
        };
    }
    let needs_key = request.mode != "free";
    if needs_key && request.api_key.trim().is_empty() {
        return ModelTestResult {
            ok: false,
            message: "Enter the key for this provider.".to_string(),
            latency_ms: 0,
        };
    }

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(12))
        .build()
    {
        Ok(client) => client,
        Err(e) => {
            return ModelTestResult {
                ok: false,
                message: format!("Could not create the HTTP client: {e}"),
                latency_ms: 0,
            };
        }
    };

    let started = std::time::Instant::now();
    let mut builder = client.post(request.endpoint.trim());
    if !request.api_key.trim().is_empty() {
        if request.provider == "anthropic" {
            builder = builder
                .header("x-api-key", request.api_key.trim())
                .header("anthropic-version", "2023-06-01");
        } else {
            builder = builder.bearer_auth(request.api_key.trim());
        }
    }
    let body = serde_json::json!({
        "model": request.model,
        "max_tokens": 1,
        "messages": [{ "role": "user", "content": "ping" }]
    });

    let latency_ms = started.elapsed().as_millis() as u64;
    match builder.json(&body).send().await {
        Ok(response) => {
            let status = response.status();
            let elapsed = started.elapsed().as_millis() as u64;
            if status.is_success() {
                return ModelTestResult {
                    ok: true,
                    message: "Provider answered and accepted the key.".to_string(),
                    latency_ms: elapsed,
                };
            }
            let message = match status.as_u16() {
                401 | 403 => {
                    if request.api_key.trim().is_empty() {
                        "This provider requires an API key. Create a free account and paste the key."
                    } else {
                        "The provider rejected this key. Check the value and try again."
                    }
                }
                404 => "The endpoint was not found. Check the address.",
                400 | 422 => "The provider answered; check the model identifier if requests fail.",
                429 => "The provider is rate limiting this key. Try again in a moment.",
                _ => "The provider did not accept the test request.",
            };
            ModelTestResult {
                ok: false,
                message: message.to_string(),
                latency_ms: elapsed,
            }
        }
        Err(e) => {
            let message = if e.is_timeout() {
                "The provider did not answer in time.".to_string()
            } else if e.is_connect() {
                "No connection to that address.".to_string()
            } else {
                format!("The test request failed: {e}")
            };
            ModelTestResult {
                ok: false,
                message,
                latency_ms,
            }
        }
    }
}

fn installers_have_llm() -> bool {
    let dir = std::env::var("BOTSERVER_INSTALLERS_PATH")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| stack_path().join("installers"));
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };
    entries.flatten().any(|entry| {
        entry
            .file_name()
            .to_string_lossy()
            .to_ascii_lowercase()
            .starts_with("llama")
    })
}

/// #2689 — pure target guard. A reset may only act on the exact directory
/// configured for this install. Both paths are canonicalized, so a symlink
/// alias or a `..` override cannot authorize a different directory; anything
/// that cannot be resolved is refused (fail closed).
pub(crate) fn reset_target_is_exact(configured: &Path, target: &Path) -> bool {
    let Ok(configured) = std::fs::canonicalize(configured) else {
        return false;
    };
    let Ok(target) = std::fs::canonicalize(target) else {
        return false;
    };
    // `starts_with` in both directions is equality on canonical paths: a
    // child, a parent or a different symlink alias never authorizes a reset.
    target.starts_with(&configured) && configured.starts_with(&target)
}

/// #2689 — pure per-entry guard. After symlinks are resolved, every path a
/// reset removes must stay inside the canonical stack directory.
pub(crate) fn entry_is_inside_stack(stack: &Path, entry: &Path) -> bool {
    let Ok(stack) = std::fs::canonicalize(stack) else {
        return false;
    };
    let Ok(entry) = std::fs::canonicalize(entry) else {
        return false;
    };
    entry.starts_with(stack)
}

/// Empties exactly the stack folder configured for this install. The caller
/// must have consumed a native, single-use confirmation grant first (#2689);
/// the old guessable folder-name check is gone with the vulnerable command.
pub fn reset_stack() -> Result<(), String> {
    let stack = stack_path();
    if stack.file_name().is_none() || stack.parent().is_none() {
        return Err("Refusing to reset an unsafe stack path.".to_string());
    }
    if !reset_target_is_exact(&stack, &stack) {
        return Err("Refusing to reset an unverified stack path.".to_string());
    }
    #[cfg(windows)]
    stop_windows_stack(&stack)?;
    empty_stack_dir(&stack)?;
    crate::desktop::payload::extract_if_present()
        .map_err(|e| format!("could not restore the bundled programs after reset: {e}"))?;
    Ok(())
}

/// Removes only entries proven to be inside `stack`. Kept separate so the
/// deletion itself can be exercised against a temporary stack in tests.
fn empty_stack_dir(stack: &Path) -> Result<(), String> {
    let canonical = std::fs::canonicalize(stack).map_err(|e| format!("read stack folder: {e}"))?;
    let entries = std::fs::read_dir(stack).map_err(|e| format!("read stack folder: {e}"))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("read stack entry: {e}"))?;
        let path = entry.path();
        if !entry_is_inside_stack(&canonical, &path) {
            return Err(format!(
                "Refusing to remove an entry outside the stack: {}",
                path.display()
            ));
        }
        // `file_type` does not follow symlinks: a link is always removed as a
        // file, never as a directory tree, even when it points inside the stack.
        let file_type = entry
            .file_type()
            .map_err(|e| format!("read stack entry: {e}"))?;
        let result = if file_type.is_dir() {
            std::fs::remove_dir_all(&path)
        } else {
            std::fs::remove_file(&path)
        };
        if let Err(e) = result {
            return Err(format!("Reset could not remove {}: {e}", path.display()));
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "system/reset_tests.rs"]
mod reset_tests;

#[cfg(windows)]
fn stop_windows_stack(stack: &Path) -> Result<(), String> {
    use botlib::security::command_guard::SafeCommand;
    let pg_ctl = stack.join("bin/tables/bin/pg_ctl.exe");
    let pgdata = stack.join("data/tables/pgdata");
    if pg_ctl.is_file() && pgdata.join("postmaster.pid").is_file() {
        let status = SafeCommand::new(&pg_ctl.to_string_lossy())
            .and_then(|c| c.args(&["-D", &pgdata.to_string_lossy(), "status"]))
            .and_then(|c| c.execute()).map_err(|e| format!("check local database: {e}"))?;
        if status.status.success() {
        let output = SafeCommand::new(&pg_ctl.to_string_lossy())
            .and_then(|c| c.args(&["-D", &pgdata.to_string_lossy(), "stop", "-m", "fast", "-w", "-t", "30"]))
            .and_then(|c| c.execute()).map_err(|e| format!("stop local database: {e}"))?;
        if !output.status.success() {
            return Err("The local database could not be stopped; reset has not deleted any files.".to_string());
        }
        } else if status.status.code() != Some(3) {
            return Err("The local database state could not be checked; reset has not deleted any files.".to_string());
        }
    }
    let script = "$ErrorActionPreference='Stop'; $root=[IO.Path]::GetFullPath($env:GB_RESET_STACK).TrimEnd('\\')+'\\'; Get-CimInstance Win32_Process | Where-Object { $_.ExecutablePath -and $_.ExecutablePath.StartsWith($root,[StringComparison]::OrdinalIgnoreCase) -and $_.ProcessId -ne [int]$env:GB_RESET_APP_PID } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction Stop }";
    let output = SafeCommand::new("powershell")
        .and_then(|c| c.args(&["-NoProfile", "-NonInteractive", "-Command"]))
        .and_then(|c| c.trusted_arg(script))
        .and_then(|c| c.env("GB_RESET_STACK", &stack.to_string_lossy()))
        .and_then(|c| c.env("GB_RESET_APP_PID", &std::process::id().to_string()))
        .and_then(|c| c.execute()).map_err(|e| format!("stop local services: {e}"))?;
    if !output.status.success() {
        return Err("The local services could not be stopped; reset has not deleted any files.".to_string());
    }
    Ok(())
}

pub fn open_folder(_app: &tauri::AppHandle, kind: &str) -> Result<(), String> {
    let path = match kind {
        "logs" => log_path(),
        "vault" => stack_path().join("conf").join("vault"),
        "workspace" => workspace_path(),
        _ => stack_path(),
    };
    std::fs::create_dir_all(&path).ok();
    opener::open(path)
        .map_err(|e| crate::desktop::i18n::t_args("folder-open-failed", &[("error", &e.to_string())]))
}

const ALLOWED_URLS: &[&str] = &[
    "https://console.cloud.google.com/apis/credentials",
    "https://portal.azure.com/#view/Microsoft_AAD_RegisteredApps/ApplicationsListBlade",
    "https://www.dropbox.com/developers/apps",
];

pub fn open_url(_app: &tauri::AppHandle, url: &str) -> Result<(), String> {
    if !ALLOWED_URLS.contains(&url) {
        return Err(crate::desktop::i18n::t("setup-url-not-allowed"));
    }
    opener::open(url)
        .map_err(|e| crate::desktop::i18n::t_args("browser-open-failed", &[("error", &e.to_string())]))
}

pub fn pick_folder(app: &tauri::AppHandle) -> Option<String> {
    use tauri_plugin_dialog::DialogExt;
    match app.dialog().file().blocking_pick_folder() {
        Some(file) => file
            .into_path()
            .ok()
            .map(|path| path.to_string_lossy().to_string()),
        None => None,
    }
}

#[cfg(test)]
#[path = "system/tests.rs"]
mod tests;
