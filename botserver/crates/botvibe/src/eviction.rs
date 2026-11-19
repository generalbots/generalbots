//! Project cap policy — a branch keeps at most `VIBE_MAX_PROJECTS_PER_KIND`
//! (default 2) projects of each kind (bot / website / custom).
//!
//! `#2570` — reaching the cap REFUSES the new project. It used to satisfy the
//! cap by deleting the oldest project of the same kind on that branch, wiping
//! its Incus VMs, published site, domains, databases and workspace without the
//! owner being asked; a project that a user left alone for a while could vanish
//! between visits. A cap is a limit on growth, not a licence to destroy data, so
//! the user now gets an explicit error and deletes something themselves.
//!
//! Asset cleanup still exists for the paths that genuinely delete (explicit
//! project delete, branch delete); it is best-effort and logged.

use std::sync::Arc;

use uuid::Uuid;

use crate::projects::{Project, ProjectRegistry};
use crate::vm_lifecycle::VmLifecycle;

/// Maximum projects of one kind per branch. Overridable for tests/dev via
/// `VIBE_MAX_PROJECTS_PER_KIND`; the cap never goes below 1.
pub fn max_projects_per_kind() -> usize {
    std::env::var("VIBE_MAX_PROJECTS_PER_KIND")
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|v| *v >= 1)
        .unwrap_or(2)
}

/// Delete every asset owned by a project (best-effort, errors returned as
/// strings for the caller to log). Shared by explicit DELETE and eviction.
/// Steps: Incus VMs (rows + containers) → published proxy site (purge) →
/// bound project_domains (Caddy routes + rows) → on-disk workspace directory.
pub async fn delete_project_assets(p: &Project, lifecycle: &VmLifecycle) -> Vec<String> {
    let mut errors = Vec::new();

    // 1. Cascade-delete the project's VMs (rows + Incus containers).
    if let Err(e) = lifecycle.delete_all_for_project(p.id) {
        errors.push(format!("vm cleanup: {e}"));
    }

    // 2. Unpublish + purge the published proxy site, if any (removes payload,
    //    systemd unit for python sites, Caddy route and release history).
    let slug = VmLifecycle::alm_repo(&p.name);
    if !slug.is_empty() {
        if let Err(e) = crate::proxy_sites::unpublish_site(&slug, true).await {
            errors.push(format!("site unpublish: {e}"));
        }
    }

    // 2b. Unbind every domain binding for the project (Caddy route removal
    //     via the admin API + project_domains row delete). VM-deployed sites
    //     have no proxy payload dir, so the site unpublish above cannot
    //     remove their routes — without this step the deleted project's
    //     domain keeps answering (502) and the row lingers forever.
    let domains = crate::domains::ProjectDomains::new(lifecycle.pool().clone());
    match domains.list(p.id) {
        Ok(binds) => {
            for bind in binds {
                if let Err(e) = domains.unbind(bind.id).await {
                    errors.push(format!("domain unbind {}: {e}", bind.domain));
                }
            }
        }
        Err(e) => errors.push(format!("domain list: {e}")),
    }

    // 2c. #1386 — drop the project's per-environment databases (public +
    //     `_dev` twin). Best-effort: failures are reported, not fatal.
    for e in crate::project_db::drop_project_databases(lifecycle.pool(), p.branch_id, &p.name) {
        errors.push(format!("database drop: {e}"));
    }

    // 2d. Remove the project's persisted runs (and their telemetry) so the
    //     Knowledge Graph — which aggregates `vibe_runs` when no project is
    //     selected — never renders ghost nodes for a deleted project.
    if let Err(e) = crate::run_store::delete_runs_for_project(lifecycle.pool(), p.id) {
        errors.push(format!("run history cleanup: {e}"));
    }

    // 3. Remove the workspace directory (the disk leak — it can hold node_modules,
    //    venvs and build output for the whole project lifetime).
    let key = VmLifecycle::alm_repo(&p.name);
    if !key.is_empty() && !key.contains("..") && !key.contains('/') {
        let dir = crate::harness::workspace_root().join(&key);
        match tokio::fs::remove_dir_all(&dir).await {
            Ok(_) => log::info!("Vibe: removed workspace dir {}", dir.display()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => errors.push(format!("workspace removal {}: {e}", dir.display())),
        }
    }

    errors
}

/// Refuse a new `project_type` project on `branch_id` once the branch already
/// holds `max_projects_per_kind()` of them (`#2570`).
///
/// Runs before the insert, so the branch is allowed to hold exactly `cap`
/// projects: a newcomer is the `cap + 1`-th and gets a message naming the cap
/// and the kind, which the projects pane can show as-is.
pub fn enforce_project_cap(
    registry: &ProjectRegistry,
    branch_id: Uuid,
    project_type: &str,
) -> Result<(), String> {
    let cap = max_projects_per_kind();
    let query = crate::projects::ListProjectsQuery {
        branch_id: Some(branch_id),
        project_type: Some(project_type.to_string()),
        status: None,
        limit: Some(500),
        offset: None,
        all_branches: false,
    };
    let existing = registry.list(&query)?;
    if existing.len() >= cap {
        return Err(format!(
            "this workspace already has {count} {project_type} project(s), the limit is {cap}; delete one to create another",
            count = existing.len()
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cap_env_override() {
        // Safe restore around the process-global env.
        let prev = std::env::var("VIBE_MAX_PROJECTS_PER_KIND").ok();
        std::env::set_var("VIBE_MAX_PROJECTS_PER_KIND", "1");
        assert_eq!(max_projects_per_kind(), 1);
        std::env::set_var("VIBE_MAX_PROJECTS_PER_KIND", "0");
        assert_eq!(max_projects_per_kind(), 2, "cap never below 1");
        match prev {
            Some(v) => std::env::set_var("VIBE_MAX_PROJECTS_PER_KIND", v),
            None => std::env::remove_var("VIBE_MAX_PROJECTS_PER_KIND"),
        }
    }
}
