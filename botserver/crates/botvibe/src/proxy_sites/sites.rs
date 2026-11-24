//! `proxy_sites::sites` — split per #1443 (AGENTS.md 450-line rule).

use super::*;
use uuid::Uuid;

pub(crate) const SECTION_BEGIN: &str = "# BEGIN GB VIBE SITES (auto-managed by botserver — do not edit)";

pub(crate) const SECTION_END: &str = "# END GB VIBE SITES";

pub(crate) const MAX_FILES: usize = 2_000;

pub(crate) const MAX_SINGLE_FILE_BYTES: usize = 10 * 1024 * 1024; // 10 MiB per file

/// Tenant-agnostic base slug: same sanitizer as the ALM repo name. Kept
/// separate so reserved-name validation still sees the bare project name —
/// appending a tenant tag must never turn a reserved name like `admin` into
/// an accepted one.
pub(crate) fn site_slug_base(project_name: &str) -> String {
    crate::vm_lifecycle::VmLifecycle::alm_repo(project_name)
}

/// #2648 — site directory/host slug, scoped by the owning tenant. Two
/// branches may legally own a project with the same name, so the slug carries
/// the same 8-char branch tag the VM container name uses. Everything downstream
/// (`SiteTarget` dir + host, systemd unit, python port, Caddy block) derives
/// from this slug and is therefore tenant-isolated as well.
///
/// Length: `alm_repo` caps the base at 32 chars, so the tagged slug is at
/// most 41 chars (46 for the `-test` twin) — inside the 63-char DNS label
/// limit enforced by [`validate_slug`].
pub fn site_slug(project_name: &str, branch_id: Uuid) -> String {
    format!(
        "{}-{}",
        site_slug_base(project_name),
        crate::vm_lifecycle::VmLifecycle::tenant_tag(branch_id)
    )
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
