use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tauri::{Emitter, Window};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};

use super::fs_scope;

// Declared here (with an explicit path) so the shared pure helper lives at
// desktop/path_scope.rs without changing the unowned desktop/mod.rs. If the
// module is later promoted to a top-level declaration, drop the `#[path]`.
#[path = "path_scope.rs"]
pub mod path_scope;

// #2692 — grants are minted and consumed only by the trusted local window.
// The implementation lives in its own file to keep this module under the
// 450-line limit; `confirmation::trusted_label` is the single choke point.
#[path = "drive_confirmation.rs"]
pub mod confirmation;

/// Resolve a webview path against the configured scopes, failing closed.
///
/// The helper logs the detailed reason and returns one fixed wire string, so
/// an injected script cannot map the host filesystem through error messages.
fn scoped(raw: &str) -> Result<PathBuf, String> {
    path_scope::resolve_in_scopes(raw, &fs_scope::scopes())
}

/// Log the detail locally and return a generic IPC error.
fn wire_error(context: &str, detail: impl std::fmt::Display) -> String {
    log::warn!("drive IPC: {context}: {detail}");
    "The requested file operation could not be completed".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileItem {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: Option<u64>,
}

/// Orders directory entries the way the Drive app lists them: folders first,
/// then case-insensitive by name.
pub fn sort_files(files: &mut [FileItem]) {
    files.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });
}

/// Whether a canonical path is one of the scope roots themselves.
fn is_scope_root(path: &Path) -> bool {
    fs_scope::scopes().iter().any(|root| root == path)
}

/// Mints a single-use confirmation after a native, user-visible approval.
///
/// The UI calls this command first, then calls `delete_path` / `create_folder`.
/// Register it in both `generate_handler!` lists in `botapp/src/main.rs`.
#[tauri::command]
pub fn request_destructive_confirmation(
    app: tauri::AppHandle,
    window: Window,
    path: &str,
    operation: &str,
) -> Result<String, String> {
    // #2692 — a grant is only minted for the trusted local window. A script on
    // the sidecar origin is refused here even if its capability allows invoke.
    let label = confirmation::trusted_label(&window)?;
    let operation = confirmation::Operation::parse(operation)?;
    let resolved = scoped(path)?;
    if operation == confirmation::Operation::Delete && is_scope_root(&resolved) {
        return Err("Refusing to confirm deletion of a root folder".to_string());
    }
    let target = resolved.to_string_lossy().to_string();
    // Native strings live outside the webview catalogue; move this prompt into
    // botapp/i18n/*.json when those catalogues are next edited (#2497 follow-up).
    let approved = app
        .dialog()
        .message(format!("Allow {} on {}?", operation.as_str(), target))
        .title("Confirm file change")
        .buttons(MessageDialogButtons::YesNo)
        .blocking_show();
    if !approved {
        return Err("Confirmation was cancelled".to_string());
    }
    let token = confirmation::mint(operation, &target, &label)?;
    log::info!(
        "drive IPC: confirmation granted operation={} target={} window={} token={token}",
        operation.as_str(),
        target,
        label
    );
    Ok(token)
}

/// The destination of an upload: the source file name inside the destination
/// directory.
///
/// Refuses to continue when both resolve to the same file: creating the
/// destination would truncate the source before a single byte is copied.
pub fn upload_destination(src: &Path, dest_dir: &Path) -> Result<PathBuf, String> {
    let name = src
        .file_name()
        .ok_or_else(|| "Invalid source file".to_string())?;
    let dest = dest_dir.join(name);
    if dest == src {
        return Err("The source and destination are the same file".to_string());
    }
    Ok(dest)
}

/// Upload progress as a percentage. An empty file (or an unknown size) reports
/// 100 so the UI never shows a stuck progress bar, and the result is clamped so
/// a file that grew mid-copy cannot report 120%.
pub fn upload_percent(done: u64, total: u64) -> u64 {
    if total == 0 {
        return 100;
    }
    done.saturating_mul(100)
        .checked_div(total)
        .unwrap_or(100)
        .min(100)
}

/// List files in a directory.
///
/// # Errors
/// Returns an error if the path does not exist or cannot be read.
#[tauri::command]
pub fn list_files(path: &str) -> Result<Vec<FileItem>, String> {
    // #2497 — the webview may only enumerate inside a configured scope, and
    // never the secret store.
    let target = scoped(path)?;
    let base_path = target.as_path();
    let mut files = Vec::new();

    if !base_path.exists() {
        return Err("Path does not exist".into());
    }

    for entry in fs::read_dir(base_path).map_err(|e| wire_error("read directory", e))? {
        let entry = entry.map_err(|e| wire_error("read directory entry", e))?;
        let path = entry.path();
        let metadata = entry.metadata().ok();

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let size = metadata.as_ref().map(std::fs::Metadata::len);
        let is_dir = metadata.is_some_and(|m| m.is_dir());

        files.push(FileItem {
            name,
            path: path.to_str().unwrap_or("").to_string(),
            is_dir,
            size,
        });
    }

    sort_files(&mut files);

    Ok(files)
}

/// Upload a file to the specified destination.
///
/// # Errors
/// Returns an error if the source file is invalid or the copy operation fails.
#[tauri::command]
pub fn upload_file(window: Window, src_path: &str, dest_path: &str) -> Result<(), String> {
    // #2692 — a write (and the overwrite of an existing destination) is only
    // permitted from the trusted local window, never from the sidecar origin.
    confirmation::trusted_label(&window)?;
    // #2497 — both ends are scoped: a script cannot copy a secret out of the
    // stack, nor write into a system directory.
    let src = scoped(src_path)?;
    let dest_dir = scoped(dest_path)?;
    let dest = upload_destination(&src, &dest_dir)?;

    if !dest_dir.exists() {
        fs::create_dir_all(&dest_dir).map_err(|e| wire_error("create upload directory", e))?;
    }

    let mut source_file = File::open(&src).map_err(|e| wire_error("open upload source", e))?;
    let mut dest_file =
        File::create(&dest).map_err(|e| wire_error("create upload destination", e))?;
    let file_size = source_file
        .metadata()
        .map_err(|e| wire_error("read upload metadata", e))?
        .len();

    let mut buffer = [0; 8192];
    let mut total_read: u64 = 0;

    loop {
        let bytes_read = source_file
            .read(&mut buffer)
            .map_err(|e| wire_error("read upload source", e))?;
        if bytes_read == 0 {
            break;
        }
        dest_file
            .write_all(&buffer[..bytes_read])
            .map_err(|e| wire_error("write upload destination", e))?;

        total_read += bytes_read as u64;

        let progress = upload_percent(total_read, file_size);

        window
            .emit("upload_progress", progress)
            .map_err(|e| wire_error("emit upload progress", e))?;
    }

    Ok(())
}

/// Create a new folder at the specified path.
///
/// Direct callers (offline regression tests) keep the original signature; the
/// IPC entry point is [`create_folder_from_window`].
///
/// # Errors
/// Returns an error if the folder already exists or cannot be created.
pub fn create_folder(path: &str, name: &str) -> Result<(), String> {
    create_folder_for(path, name, None)
}

/// #2692 — IPC entry point. The renamed command keeps the `create_folder`
/// wire name (and its ACL permission) while binding the grant to the caller.
#[tauri::command(rename = "create_folder")]
pub fn create_folder_from_window(window: Window, path: &str, name: &str) -> Result<(), String> {
    // Re-check on every invocation: a window navigated away after the grant
    // was minted is no longer trusted.
    let label = confirmation::trusted_label(&window)?;
    create_folder_for(path, name, Some(&label))
}

fn create_folder_for(path: &str, name: &str, label: Option<&str>) -> Result<(), String> {
    // #2497 — `name` is a single segment, never `../` and never a reserved
    // secret file name; the parent must be inside a scope.
    let parent = scoped(path)?;
    let safe_name = fs_scope::validate_name(name)?;
    let full_path = parent.join(safe_name);
    // #2497 — creating a folder is a mutation: require the native, single-use
    // confirmation minted for exactly this target and window.
    let target = full_path.to_string_lossy().to_string();
    match label {
        Some(label) => {
            confirmation::consume_for(confirmation::Operation::CreateFolder, &target, label)
        }
        None => confirmation::consume(confirmation::Operation::CreateFolder, &target),
    }?;

    if full_path.exists() {
        return Err("Folder already exists".into());
    }

    fs::create_dir(&full_path).map_err(|e| wire_error("create folder", e))?;
    log::info!("drive IPC: created folder {}", full_path.display());
    Ok(())
}

/// Delete a file or folder at the specified path.
///
/// Direct callers (offline regression tests) keep the original signature; the
/// IPC entry point is [`delete_path_from_window`].
///
/// # Errors
/// Returns an error if the path does not exist or the item cannot be deleted.
pub fn delete_path(path: &str) -> Result<(), String> {
    delete_path_for(path, None)
}

/// #2692 — IPC entry point. The renamed command keeps the `delete_path` wire
/// name (and its ACL permission) while binding the grant to the caller.
#[tauri::command(rename = "delete_path")]
pub fn delete_path_from_window(window: Window, path: &str) -> Result<(), String> {
    // The caller must still be the trusted local window at consume time; a
    // grant held by another page cannot be replayed here.
    let label = confirmation::trusted_label(&window)?;
    delete_path_for(path, Some(&label))
}

fn delete_path_for(path: &str, label: Option<&str>) -> Result<(), String> {
    // #2497 — deleting is the dangerous half: only a scoped path, never a
    // secret file, and never a scope root itself (that would wipe the stack
    // in one call).
    let resolved = scoped(path)?;
    if is_scope_root(&resolved) {
        return Err("Refusing to delete a root folder".to_string());
    }
    let target = resolved.as_path();
    // #2497 — the proof-of-concept `invoke('delete_path', {path})` now stops
    // here unless the user approved this exact target in this window.
    let canonical = target.to_string_lossy().to_string();
    match label {
        Some(label) => {
            confirmation::consume_for(confirmation::Operation::Delete, &canonical, label)
        }
        None => confirmation::consume(confirmation::Operation::Delete, &canonical),
    }?;

    if !target.exists() {
        return Err("Path does not exist".into());
    }

    if target.is_dir() {
        fs::remove_dir_all(target).map_err(|e| wire_error("delete directory", e))?;
    } else {
        fs::remove_file(target).map_err(|e| wire_error("delete file", e))?;
    }
    log::info!("drive IPC: deleted {}", target.display());

    Ok(())
}

/// Get the user's home directory path.
///
/// # Errors
/// Returns an error if the home directory cannot be determined.
#[tauri::command]
pub fn get_home_dir() -> Result<String, String> {
    dirs::home_dir()
        .and_then(|p| p.to_str().map(String::from))
        .ok_or_else(|| "Could not determine home directory".into())
}
