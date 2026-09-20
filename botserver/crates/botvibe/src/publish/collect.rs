//! `publish::collect` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

pub(crate) const PUBLISH_DEFAULT_ENV: &str = "production";

/// Env var overriding the maximum total bytes the publish path will read
/// into memory for a single project archive (#934).
pub(crate) const PUBLISH_MAX_BYTES_ENV: &str = "VIBE_PUBLISH_MAX_BYTES";

pub(crate) const PUBLISH_DEFAULT_MAX_BYTES: u64 = 20 * 1024 * 1024;

pub(crate) fn publish_max_bytes() -> u64 {
    std::env::var(PUBLISH_MAX_BYTES_ENV)
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(PUBLISH_DEFAULT_MAX_BYTES)
}

/// Budget shared with the #1505 git deploy-source: the same env-var knob
/// governs both payload collectors so neither can bypass the size cap.
pub(crate) fn publish_max_bytes_budget() -> u64 {
    publish_max_bytes()
}

pub fn publish_project_tool() -> ToolHandler {
    Arc::new(|args: Value, state: &dyn VibeState| {
        let pool = state.db_pool().clone();
        let args = args.clone();
        Box::pin(publish_project(args, pool))
    })
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn api_base() -> String {
    let (_, vibe) = botcoresecrets::app_runtime();
    if vibe.is_empty() {
        std::env::var("VIBE_API_URL").unwrap_or_else(|_| "http://localhost:8080".to_string())
    } else {
        vibe
    }
}

pub(crate) fn walk_workspace(
    dir: &Path,
    root: &Path,
    out: &mut Vec<Value>,
    total_bytes: &mut u64,
) -> Result<(), String> {
    let entries = std::fs::read_dir(dir).map_err(|e| format!("read_dir {dir:?}: {e}"))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("read_dir entry: {e}"))?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        // Only source ships — skip VCS and heavy build/artifact directories.
        if matches!(
            name.as_str(),
            ".git" | ".forgejo" | "node_modules" | "target" | "dist" | ".next" | "build"
        ) {
            continue;
        }
        if path.is_dir() {
            walk_workspace(&path, root, out, total_bytes)?;
            continue;
        }
        let rel = path
            .strip_prefix(root)
            .map_err(|e| format!("strip prefix: {e}"))?;
        let rel = rel.to_string_lossy().replace('\\', "/");
        match std::fs::read(&path) {
            Ok(bytes) => {
                *total_bytes += bytes.len() as u64;
                if *total_bytes > publish_max_bytes() {
                    return Err(format!(
                        "workspace exceeds publish size budget ({} bytes)",
                        publish_max_bytes()
                    ));
                }
                out.push(serde_json::json!({ "path": rel, "content": bytes }));
            }
            Err(e) => log::warn!("Vibe publish: skip unreadable file {rel}: {e}"),
        }
    }
    Ok(())
}

pub(crate) async fn publish_project(args: Value, pool: crate::types::DbPool) -> VibeToolResult {
    let started = std::time::Instant::now();
    match do_publish(args, pool).await {
        Ok(data) => VibeToolResult {
            success: true,
            data,
            error: None,
            latency_ms: started.elapsed().as_millis() as u64,
        },
        Err(e) => VibeToolResult {
            success: false,
            data: Value::Null,
            error: Some(e),
            latency_ms: started.elapsed().as_millis() as u64,
        },
    }
}
