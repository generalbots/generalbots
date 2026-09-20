//! `proxy_sites::sites` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

pub(crate) const SECTION_BEGIN: &str = "# BEGIN GB VIBE SITES (auto-managed by botserver — do not edit)";

pub(crate) const SECTION_END: &str = "# END GB VIBE SITES";

pub(crate) const MAX_FILES: usize = 2_000;

pub(crate) const MAX_SINGLE_FILE_BYTES: usize = 10 * 1024 * 1024; // 10 MiB per file

/// Site directory slug: same sanitizer as the ALM repo name, plus the
/// enterprise hardening (reserved names, charset, length).
pub fn site_slug(project_name: &str) -> String {
    crate::vm_lifecycle::VmLifecycle::alm_repo(project_name)
}

/// systemd unit name for a site env: `{slug}` for production, `{slug}-test`
/// for the test twin.
pub(crate) fn site_unit_name(slug: &str, env: SiteEnv) -> String {
    match env {
        SiteEnv::Production => slug.to_string(),
        SiteEnv::Test => format!("{slug}-test"),
    }
}

/// Decode a workspace file entry's content. The canonical producer
/// (`walk_workspace`) serializes raw bytes (a JSON array of numbers), but
/// accept a plain string too so callers passing text payloads work.
pub(crate) fn payload_bytes(entry: &serde_json::Value) -> Result<Vec<u8>, String> {
    let content = &entry["content"];
    if let Ok(bytes) = serde_json::from_value::<Vec<u8>>(content.clone()) {
        return Ok(bytes);
    }
    if let Some(text) = content.as_str() {
        return Ok(text.as_bytes().to_vec());
    }
    Err(format!(
        "payload decode: unsupported content for '{}'",
        entry["path"].as_str().unwrap_or_default()
    ))
}

/// Slice out the current managed section (everything between markers, or
/// empty when markers are absent — fresh setups append at the tail).
pub(crate) fn extract_section(original: &str) -> String {
    match (original.find(SECTION_BEGIN), original.find(SECTION_END)) {
        (Some(b), Some(e)) if e >= b => original[b + SECTION_BEGIN.len()..e].to_string(),
        _ => String::new(),
    }
}
