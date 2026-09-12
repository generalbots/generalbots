use axum::http::StatusCode;
use diesel::RunQueryDsl;
use std::path::PathBuf;
use crate::db;

/// On-disk root of the shipped bot templates. Mirrors the resolution used by
/// `botcore::bootstrap::BootstrapManager::templates_source_dir` so the deploy
/// endpoint stages exactly what the sync listed (#1354).
pub fn templates_source_dir() -> PathBuf {
    let work = PathBuf::from("work/templates");
    if work.is_dir() {
        work
    } else {
        PathBuf::from("bottemplates/bots")
    }
}

/// Locate a shipped template's `.gbai` directory under `templates_source_dir()`.
///
/// Templates are laid out as `bots/<name>/<name>.gbai/` (two levels deep), so
/// joining `name` once never matches; the tree is walked instead and the `.gbai`
/// directory whose stem equals `name` is returned (#1354).
pub fn find_template_dir(name: &str) -> Option<PathBuf> {
    find_gbai_dir(&templates_source_dir(), name, 3)
}

fn find_gbai_dir(dir: &std::path::Path, name: &str, depth: usize) -> Option<PathBuf> {
    if depth == 0 {
        return None;
    }
    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        if path.file_name().and_then(|n| n.to_str()) == Some(format!("{name}.gbai").as_str()) {
            return Some(path);
        }
        if let Some(found) = find_gbai_dir(&path, name, depth - 1) {
            return Some(found);
        }
    }
    None
}

pub fn ensure_schema_sync() -> Result<(), (StatusCode, String)> {
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS app_templates (
            id UUID PRIMARY KEY, name TEXT NOT NULL, description TEXT NOT NULL DEFAULT '',
            kind VARCHAR(50) NOT NULL DEFAULT 'app', version TEXT NOT NULL DEFAULT '1.0',
            author TEXT NOT NULL DEFAULT '', created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000')",
    ).execute(&mut conn).map_err(db::map_diesel_err)?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS app_template_deploys (
            id UUID PRIMARY KEY, template_id UUID NOT NULL, status VARCHAR(30) NOT NULL DEFAULT 'deployed',
            target TEXT NOT NULL DEFAULT 'production', deployed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000')",
    ).execute(&mut conn).map_err(db::map_diesel_err)?;
    Ok(())
}