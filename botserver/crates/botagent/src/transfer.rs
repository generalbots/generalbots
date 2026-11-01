//! Conversation-folder file transfer (#1584).
//!
//! The Drive bridge moves bytes between an object store and the user's agent
//! container. Container IO runs through `vm::exec_in_vm`; this module only owns
//! path validation, chunked writes and base64 reads.

use axum::http::StatusCode;
use base64::Engine;
use std::sync::Arc;
use uuid::Uuid;

use crate::vm;
use crate::AgentService;

/// Per-file transfer cap (100 MB, per #1584) and per-turn cap (200 MB).
pub const MAX_FILE_BYTES: usize = 100 * 1024 * 1024;
pub const MAX_TURN_BYTES: usize = 200 * 1024 * 1024;

/// Base64 characters per exec: big enough to keep the round-trip count low,
/// small enough to stay well under the kernel argument limit.
const CHUNK: usize = 60_000;

#[derive(Debug, Clone)]
pub struct FolderFile {
    pub name: String,
    pub size: u64,
}

fn bad_request(msg: &str) -> (StatusCode, String) {
    (StatusCode::BAD_REQUEST, msg.to_string())
}

/// Rejects traversal, absolute paths, symlink components and unknown bytes.
pub fn safe_rel_path(raw: &str) -> Result<String, (StatusCode, String)> {
    let trimmed = raw.trim().trim_start_matches('/');
    if trimmed.is_empty() || trimmed.len() > 512 {
        return Err(bad_request("Invalid file path"));
    }
    let mut parts = Vec::new();
    for segment in trimmed.split('/') {
        if segment.is_empty() || segment == "." || segment == ".." {
            return Err(bad_request("Invalid file path"));
        }
        if !segment
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | ' '))
        {
            return Err(bad_request("Invalid file path"));
        }
        parts.push(segment);
    }
    Ok(parts.join("/"))
}

fn shell_quote(path: &str) -> String {
    format!("'{path}'")
}

async fn run(
    state: &Arc<AgentService>,
    vm_name: &str,
    script: String,
    timeout_ms: u64,
) -> Result<(bool, String, String), (StatusCode, String)> {
    let result = tokio::time::timeout(
        std::time::Duration::from_millis(timeout_ms),
        vm::exec_in_vm(state, vm_name, &["sh".into(), "-c".into(), script]),
    )
    .await;
    match result {
        Err(_) => Ok((false, String::new(), "Execution timed out".to_string())),
        Ok(result) => result,
    }
}

fn folder_for(conversation_id: &str) -> Result<String, (StatusCode, String)> {
    let clean = vm::sanitize_id(conversation_id)?;
    Ok(format!("/opt/vibe/conversations/{clean}"))
}

/// Reads one file of the conversation folder.
pub async fn read_file(
    state: &Arc<AgentService>,
    vm_name: &str,
    conversation_id: &str,
    rel_path: &str,
) -> Result<Vec<u8>, (StatusCode, String)> {
    let rel = safe_rel_path(rel_path)?;
    let folder = folder_for(conversation_id)?;
    let target = format!("{folder}/{rel}");
    let script = format!("base64 -w0 {}", shell_quote(&target));
    let (ok, stdout, stderr) = run(state, vm_name, script, 120_000).await?;
    if !ok {
        return Err((
            StatusCode::NOT_FOUND,
            format!("File not found: {}", stderr.trim()),
        ));
    }
    let encoded: String = stdout.chars().filter(|c| !c.is_whitespace()).collect();
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded.as_bytes())
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("decode failed: {e}"),
            )
        })?;
    if bytes.len() > MAX_FILE_BYTES {
        return Err(bad_request("File exceeds the 100 MB transfer limit"));
    }
    Ok(bytes)
}

/// Writes bytes into the conversation folder, creating parent folders.
pub async fn write_file(
    state: &Arc<AgentService>,
    vm_name: &str,
    conversation_id: &str,
    rel_path: &str,
    bytes: &[u8],
) -> Result<usize, (StatusCode, String)> {
    if bytes.len() > MAX_FILE_BYTES {
        return Err(bad_request("File exceeds the 100 MB transfer limit"));
    }
    let rel = safe_rel_path(rel_path)?;
    let folder = folder_for(conversation_id)?;
    let target = format!("{folder}/{rel}");
    let parent = target
        .rsplit_once('/')
        .map(|(dir, _)| dir.to_string())
        .unwrap_or(folder);
    let tmp = format!("{target}.b64tmp");
    let prep = format!(
        "mkdir -p {parent} && : > {tmp}",
        parent = shell_quote(&parent),
        tmp = shell_quote(&tmp)
    );
    let (ok, _, stderr) = run(state, vm_name, prep, 30_000).await?;
    if !ok {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to prepare target: {}", stderr.trim()),
        ));
    }

    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    let raw = encoded.as_bytes();
    let mut offset = 0;
    while offset < raw.len() {
        let end = (offset + CHUNK).min(raw.len());
        let chunk = &encoded[offset..end];
        let cmd = format!("printf '%s' '{chunk}' >> {tmp}", tmp = shell_quote(&tmp));
        let (ok, _, stderr) = run(state, vm_name, cmd, 30_000).await?;
        if !ok {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to write chunk: {}", stderr.trim()),
            ));
        }
        offset = end;
    }
    let finish = format!(
        "base64 -d {tmp} > {target} && rm -f {tmp}",
        tmp = shell_quote(&tmp),
        target = shell_quote(&target)
    );
    let (ok, _, stderr) = run(state, vm_name, finish, 60_000).await?;
    if !ok {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to decode file: {}", stderr.trim()),
        ));
    }
    Ok(bytes.len())
}

/// Lists regular files directly under `subdir` (`output`, `data`, `program`).
pub async fn list_folder(
    state: &Arc<AgentService>,
    vm_name: &str,
    conversation_id: &str,
    subdir: &str,
) -> Result<Vec<FolderFile>, (StatusCode, String)> {
    let safe_sub = safe_rel_path(subdir)?;
    if safe_sub.contains('/') {
        return Err(bad_request("Only top-level folders can be listed"));
    }
    let folder = folder_for(conversation_id)?;
    let script = format!(
        "find {path} -maxdepth 1 -type f -printf '%f\\t%s\\n' 2>/dev/null | sort | head -200",
        path = shell_quote(&format!("{folder}/{safe_sub}"))
    );
    let (ok, stdout, _) = run(state, vm_name, script, 30_000).await?;
    if !ok {
        return Ok(Vec::new());
    }
    Ok(stdout
        .lines()
        .filter_map(|line| {
            let (name, size) = line.split_once('\t')?;
            let name = name.trim();
            if name.is_empty() {
                return None;
            }
            Some(FolderFile {
                name: name.to_string(),
                size: size.trim().parse().unwrap_or(0),
            })
        })
        .collect())
}

/// Convenience: the active user's VM name, or None when it is not running.
pub async fn active_vm_name(
    state: &Arc<AgentService>,
    user_id: &Uuid,
) -> Option<String> {
    let row = vm::get_user_vm(state, user_id).ok()??;
    if row.status != vm::STATUS_RUNNING {
        return None;
    }
    Some(row.vm_name)
}
