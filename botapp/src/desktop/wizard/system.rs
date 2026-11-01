//! Local checks used by the wizard: workspace writability, model reachability,
//! stack reset and OS folder/URL opening. Nothing here contacts a service
//! unless the user explicitly asked to test a model provider.

use super::{log_path, stack_path, workspace_path, ModelRequest};
use log::warn;
use serde::Serialize;
use std::path::Path;

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

pub fn check_workspace(path: &str) -> WorkspaceCheck {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return WorkspaceCheck {
            writable: false,
            free_bytes: 0,
            reason: "Choose a folder for the workspace.".to_string(),
        };
    }
    let dir = Path::new(trimmed);
    if let Err(e) = std::fs::create_dir_all(dir) {
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
            free_bytes: free_bytes(dir),
            reason: format!("The folder is not writable: {e}"),
        };
    }
    let _ = std::fs::remove_file(&probe);
    WorkspaceCheck {
        writable: true,
        free_bytes: free_bytes(dir),
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

#[cfg(not(unix))]
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

/// Empties exactly the stack folder. The caller must type the folder name.
pub fn reset_stack(confirmation: &str) -> Result<(), String> {
    let stack = stack_path();
    let name = stack
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_default();
    if name.is_empty() || stack.parent().is_none() {
        return Err("Refusing to reset an unsafe stack path.".to_string());
    }
    if confirmation.trim() != name {
        return Err(format!(
            "Type the folder name ({name}) to confirm the reset."
        ));
    }
    let entries = std::fs::read_dir(&stack).map_err(|e| format!("read stack folder: {e}"))?;
    for entry in entries.flatten() {
        let path = entry.path();
        let result = if path.is_dir() {
            std::fs::remove_dir_all(&path)
        } else {
            std::fs::remove_file(&path)
        };
        if let Err(e) = result {
            warn!("reset stack: could not remove {}: {e}", path.display());
        }
    }
    Ok(())
}

pub fn open_folder(app: &tauri::AppHandle, kind: &str) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    let path = match kind {
        "logs" => log_path(),
        "vault" => stack_path().join("conf").join("vault"),
        "workspace" => workspace_path(),
        _ => stack_path(),
    };
    std::fs::create_dir_all(&path).ok();
    app.opener()
        .open_path(path.to_string_lossy().to_string(), None::<&str>)
        .map_err(|e| format!("could not open the folder: {e}"))
}

const ALLOWED_URLS: &[&str] = &[
    "https://console.cloud.google.com/apis/credentials",
    "https://portal.azure.com/#view/Microsoft_AAD_RegisteredApps/ApplicationsListBlade",
    "https://www.dropbox.com/developers/apps",
];

pub fn open_url(app: &tauri::AppHandle, url: &str) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    if !ALLOWED_URLS.contains(&url) {
        return Err("That address is not part of the guided setup.".to_string());
    }
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| format!("could not open the browser: {e}"))
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
