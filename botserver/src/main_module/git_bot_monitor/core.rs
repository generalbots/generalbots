//! Reform #1502/#1504 — git-pull bot monitor (core sync primitives).
//!
//! Bot sources live in Forgejo repositories (one repo per project inside the
//! branch org, #1503) instead of Drive buckets. This module keeps a git
//! checkout of every bot-kind git-mode Vibe project fresh and materializes
//! the pulled sources into the standard work layout the runtime already reads
//! (`work/{branch}.gborg/{branch}.gbai/{bot}.gbdialog`) for BOTH environments:
//! `{bot}` (PROD) and `{bot}-test` (TEST twin, #1504).

use std::path::{Path, PathBuf};

use botcore::shared::utils::{get_work_path, DbPool};
use botlib::security::SafeCommand;
use diesel::prelude::*;

// #2863: materialization helpers (now symlink-safe) live in their own module;
// re-export keeps the existing `super::core::…` call sites unchanged.
pub(crate) use super::materialize::{
    copy_dir_recursive, dialog_source_dir, materialize_checkout,
};

/// A bot-kind git-mode project to monitor.
#[derive(Debug, Clone)]
pub(crate) struct MonitoredBot {
    pub project_id: uuid::Uuid,
    pub name: String,
    pub repo_slug: String,
    pub branch_slug: Option<String>,
}

pub(crate) fn run_git(cwd: &Path, args: &[&str]) -> Result<String, String> {
    let mut cmd = SafeCommand::new("git").map_err(|e| e.to_string())?;
    cmd = cmd.arg("-C").map_err(|e| e.to_string())?;
    cmd = cmd.trusted_arg(cwd.to_string_lossy().as_ref())?;
    for a in args {
        cmd = cmd.trusted_arg(a)?;
    }
    // #2861: authenticate every git operation through a short-lived
    // `http.extraHeader` in the child's environment. The token is never an
    // argv element and never written into the checkout's `.git/config`, so a
    // local `ps` or any later process cannot recover it. `run_git` is the
    // single entry point for clone/fetch/push in this monitor, so applying it
    // here covers every remote operation.
    let (_, alm_token, _) = botcoresecrets::alm_config();
    if !alm_token.is_empty() {
        cmd = botlib::security::git_auth::apply_git_auth(cmd, &alm_token)
            .map_err(|e| e.to_string())?;
    }
    let out = cmd.execute().map_err(|e| e.to_string())?;
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    if !out.status.success() {
        return Err(if stderr.trim().is_empty() {
            stdout
        } else {
            stderr
        });
    }
    Ok(stdout)
}

pub(crate) fn alm_repo(name: &str) -> String {
    botvibe::vm_lifecycle::VmLifecycle::alm_repo(name)
}

/// Token-free clone URL against the INTERNAL ALM base (same source
/// `git_mode.rs` uses so clones work inside the container). #2861: the ALM
/// token is supplied to `git` via `run_git`'s ephemeral auth environment,
/// never embedded in the URL that becomes an argv element / `.git/config`
/// remote.
pub(crate) fn clone_url(org: &str, repo: &str) -> Option<String> {
    let (alm_base, alm_token, _) = botcoresecrets::alm_config();
    if alm_base.is_empty() || alm_token.is_empty() {
        return None;
    }
    Some(botlib::security::git_auth::plain_clone_url(
        &alm_base, org, repo,
    ))
}

/// Ensure a checkout exists at `vibe-workspaces/{repo}` — the SAME directory
/// the Vibe agent edits, so a git pull keeps the agent workspace and the
/// deployed sources one and the same.
pub(crate) fn ensure_checkout(project: &MonitoredBot, org: &str) -> Result<PathBuf, String> {
    let root = botvibe::harness::workspace_root();
    let cwd = root.join(&project.repo_slug);
    std::fs::create_dir_all(&cwd).map_err(|e| format!("mkdir {}: {e}", cwd.display()))?;
    if cwd.join(".git").exists() {
        return Ok(cwd);
    }
    let url = match clone_url(org, &project.repo_slug) {
        Some(u) => u,
        None => return Err("ALM config missing (secret/gbo/alm) — git monitor idle".to_string()),
    };
    let tmp = root.join(format!(".clone-{}", project.repo_slug));
    let _ = std::fs::remove_dir_all(&tmp);
    match run_git(
        &root,
        &["clone", "--quiet", &url, tmp.to_string_lossy().as_ref()],
    ) {
        Ok(_) => {
            for entry in std::fs::read_dir(&tmp).map_err(|e| format!("read clone: {e}"))? {
                let entry = entry.map_err(|e| format!("clone entry: {e}"))?;
                let to = cwd.join(entry.file_name());
                if entry.path().is_dir() {
                    std::fs::rename(entry.path(), &to)
                        .or_else(|_| copy_dir_recursive(&entry.path(), &to))
                        .map_err(|e: std::io::Error| format!("move {e}"))?;
                } else {
                    std::fs::rename(entry.path(), &to).map_err(|e| format!("move {e}"))?;
                }
            }
            let _ = std::fs::remove_dir_all(&tmp);
            log::info!(
                "[git_monitor] cloned {org}/{} → {}",
                project.repo_slug,
                cwd.display()
            );
            Ok(cwd)
        }
        Err(e) => {
            let _ = std::fs::remove_dir_all(&tmp);
            Err(format!("clone {org}/{}: {e}", project.repo_slug))
        }
    }
}

/// Self-heal for pre-reform projects (#1503 backfill): when the clone fails
/// because the Forgejo repo does not exist yet ("Repository not found"),
/// provision it from the currently-deployed PROD `.gbdialog` sources and
/// retry the clone once. Provisioning = seed workspace →
/// `git_mode::ensure_git_repo` (creates the Forgejo repo, commits, pushes
/// `main`). Retried on the next tick until it succeeds; never fatal.
pub(crate) fn ensure_checkout_with_heal(
    pool: &DbPool,
    project: &MonitoredBot,
    org: &str,
) -> Result<PathBuf, String> {
    match ensure_checkout(project, org) {
        Ok(cwd) => Ok(cwd),
        Err(clone_err) => {
            let not_found = clone_err.contains("not found")
                || clone_err.contains("does not appear to be a git repository");
            if !not_found {
                return Err(clone_err);
            }
            log::info!(
                "[git_monitor] {0}/{1} has no repo yet — provisioning from deployed sources",
                org,
                project.repo_slug
            );
            provision_repo(pool, project, org)?;
            ensure_checkout(project, org)
        }
    }
}

/// Builds the project workspace from the deployed PROD `.gbdialog` (falling
/// back to a minimal README for fresh bots) and wires the Forgejo repo,
/// origin remote and initial `main` push via `git_mode::ensure_git_repo`.
fn provision_repo(pool: &DbPool, project: &MonitoredBot, org: &str) -> Result<(), String> {
    let safe_name = botvibe::harness::sanitize_project_id(&project.name)?;
    let cwd = botvibe::harness::ensure_workspace(&safe_name)?;
    let registry = botvibe::projects::ProjectRegistry::new(pool.clone());
    let mut p = registry
        .get(project.project_id)
        .map_err(|e| format!("project load: {e}"))?
        .ok_or_else(|| format!("project {} vanished mid-heal", project.project_id))?;
    // Pre-reform rows may still carry source_control='native', which makes
    // ensure_git_repo a no-op — the reform owns bot sources, so flip the
    // project to git mode (persisted) BEFORE any early return: a partial
    // heal from a previous boot (workspace seeded, repo never created) must
    // not wedge the loop.
    if p.source_control != "git" {
        let mut conn = pool.get().map_err(|e| format!("pool: {e}"))?;
        diesel::sql_query(
            "UPDATE vibe_projects SET source_control = 'git', updated_at = now() WHERE id = $1",
        )
        .bind::<diesel::sql_types::Uuid, _>(project.project_id)
        .execute(&mut conn)
        .map_err(|e| format!("source_control flip: {e}"))?;
        drop(conn);
        log::info!(
            "[git_monitor] provision {0}/{1}: source_control → git (was '{2}')",
            org,
            project.repo_slug,
            p.source_control
        );
        p.source_control = "git".to_string();
    }
    // Seed from the deployed PROD sources when they exist (pre-reform bots
    // keep their content through the git move); fresh bots get a minimal
    // tree so the initial commit is never empty.
    let work_root = PathBuf::from(get_work_path());
    let branch_slug = project.branch_slug.clone().unwrap_or(Default::default());
    let org_dialog = work_root.join(format!(
        "{0}.gborg/{0}.gbai/{1}.gbdialog",
        branch_slug, project.name
    ));
    let single_dialog = work_root.join(format!("{0}.gbai/{1}.gbdialog", branch_slug, project.name));
    let deployed = if org_dialog.is_dir() {
        Some(org_dialog)
    } else if single_dialog.is_dir() {
        Some(single_dialog)
    } else {
        None
    };
    match deployed {
        Some(src) => {
            copy_dir_recursive(&src, &cwd.join(".gbdialog"))
                .map_err(|e| format!("seed .gbdialog: {e}"))?;
            log::info!(
                "[git_monitor] provision {0}/{1}: seeded .gbdialog from deployed sources",
                org,
                project.repo_slug
            );
        }
        None if !cwd.join(".git").exists() => std::fs::write(
            cwd.join("README.md"),
            format!(
                "# {0}\n\nVibe-managed bot sources (reform #1503).\n",
                project.name
            ),
        )
        .map_err(|e| format!("seed README: {e}"))?,
        None => {}
    }
    // Always finish the wiring: with an existing partial-heal workspace
    // (.git, no origin or unpushed content) ensure_git_repo adds the origin,
    // commits and pushes main; a fully wired workspace exits on the origin
    // check. Idempotent from any state.
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("runtime: {e}"))?;
    rt.block_on(botvibe::git_mode::ensure_git_repo(&p))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// #2861: the clone URL that reaches git argv must never carry the ALM
    /// token, and the auth environment values must not carry it verbatim
    /// either (only the encoded `x-access-token` credential may).
    #[test]
    fn clone_argv_never_contains_the_token() {
        const TOKEN: &str = "alm-token-do-not-leak-0123456789";
        let url = botlib::security::git_auth::plain_clone_url("http://alm:4747", "org", "repo");
        // The argv `run_git` builds for the clone, including the `-C` prefix.
        let argv = [
            "git",
            "-C",
            "/work",
            "clone",
            "--quiet",
            &url,
            "/work/.clone-repo",
        ];
        for arg in argv {
            assert!(!arg.contains(TOKEN), "token leaked in argv element {arg}");
        }
        assert!(botlib::security::git_auth::git_auth_env(TOKEN)
            .iter()
            .all(|(_, value)| !value.contains(TOKEN)));
    }

    #[test]
    fn clone_url_has_no_embedded_credentials() {
        let url = botlib::security::git_auth::plain_clone_url("http://alm:4747", "org", "repo");
        assert_eq!(url, "http://alm:4747/org/repo.git");
        assert!(!url.contains('@'), "URL embeds credentials: {url}");
    }
}
