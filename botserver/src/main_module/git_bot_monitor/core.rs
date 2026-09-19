//! Reform #1502/#1504 — git-pull bot monitor (core sync primitives).
//!
//! Bot sources live in Forgejo repositories (one repo per project inside the
//! branch org, #1503) instead of Drive buckets. This module keeps a git
//! checkout of every bot-kind git-mode Vibe project fresh and materializes
//! the pulled sources into the standard work layout the runtime already reads
//! (`work/{branch}.gborg/{branch}.gbai/{bot}.gbdialog`) for BOTH environments:
//! `{bot}` (PROD) and `{bot}-test` (TEST twin, #1504).

use std::path::{Path, PathBuf};

use botlib::security::SafeCommand;

pub(crate) const MAX_MATERIALIZED_FILES: usize = 400;

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
    let out = cmd.execute().map_err(|e| e.to_string())?;
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    if !out.status.success() {
        return Err(if stderr.trim().is_empty() { stdout } else { stderr });
    }
    Ok(stdout)
}

pub(crate) fn alm_repo(name: &str) -> String {
    botvibe::vm_lifecycle::VmLifecycle::alm_repo(name)
}

/// Authenticated clone URL against the INTERNAL ALM base (same source
/// `git_mode.rs` uses so clones work inside the container).
pub(crate) fn authenticated_clone_url(org: &str, repo: &str) -> Option<String> {
    let (alm_base, alm_token, _) = botcoresecrets::alm_config();
    if alm_base.is_empty() || alm_token.is_empty() {
        return None;
    }
    let base = alm_base.trim_end_matches('/');
    Some(format!("{base}/{org}/{repo}.git").replace("://", &format!("://{}@", alm_token)))
}

pub(crate) fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let to = dst.join(entry.file_name());
        if entry.path().is_dir() {
            copy_dir_recursive(&entry.path(), &to)?;
        } else {
            std::fs::copy(entry.path(), to)?;
        }
    }
    Ok(())
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
    let url = match authenticated_clone_url(org, &project.repo_slug) {
        Some(u) => u,
        None => {
            return Err("ALM config missing (secret/gbo/alm) — git monitor idle".to_string())
        }
    };
    let tmp = root.join(format!(".clone-{}", project.repo_slug));
    let _ = std::fs::remove_dir_all(&tmp);
    match run_git(&root, &["clone", "--quiet", &url, tmp.to_string_lossy().as_ref()]) {
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

/// Resolve the `.gbdialog` source directory of a checkout. Repos store it at
/// the root (`.gbdialog/`) or bot-named (`{bot}.gbdialog/`) — accept both.
pub(crate) fn dialog_source_dir(checkout: &Path, bot_name: &str) -> Option<PathBuf> {
    [
        checkout.join(".gbdialog"),
        checkout.join(format!("{bot_name}.gbdialog")),
    ]
    .into_iter()
    .find(|p| p.is_dir())
}

/// Copy every source file of `source_dir` into `work_dir`, returning the
/// (drive-layout) `.bas` paths that were materialized.
pub(crate) fn materialize_dialog_dir(
    source_dir: &Path,
    work_dir: &Path,
    branch_slug: &str,
    bot_name: &str,
) -> Result<Vec<String>, String> {
    std::fs::create_dir_all(work_dir).map_err(|e| format!("mkdir {}: {e}", work_dir.display()))?;
    let mut materialized = Vec::new();
    let mut total = 0usize;
    for entry in std::fs::read_dir(source_dir)
        .map_err(|e| format!("read {}: {e}", source_dir.display()))?
    {
        let entry = entry.map_err(|e| format!("entry: {e}"))?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let file_name = entry.file_name().to_string_lossy().to_string();
        if !file_name.ends_with(".bas")
            && !file_name.ends_with(".ast")
            && !file_name.ends_with(".json")
        {
            continue;
        }
        total += 1;
        if total > MAX_MATERIALIZED_FILES {
            return Err(format!(
                "materialize {bot_name}: more than {MAX_MATERIALIZED_FILES} source files — trim the repo"
            ));
        }
        let to = work_dir.join(&file_name);
        std::fs::copy(&path, &to).map_err(|e| format!("copy {file_name}: {e}"))?;
        if file_name.ends_with(".bas") {
            let tool = file_name.trim_end_matches(".bas").to_string();
            materialized.push(format!("{branch_slug}.gbai/{bot_name}.gbdialog/{tool}.bas"));
        }
    }
    Ok(materialized)
}

/// Materialize one bot environment (PROD or TEST) from a checkout and return
/// the changed drive-layout paths.
pub(crate) fn materialize_checkout(
    checkout: &Path,
    work_root: &Path,
    branch_slug: &str,
    bot_name: &str,
) -> Result<Vec<String>, String> {
    let work_dir = work_root
        .join(format!("{branch_slug}.gborg/{branch_slug}.gbai"))
        .join(format!("{bot_name}.gbdialog"));
    let source_dir = match dialog_source_dir(checkout, bot_name) {
        Some(d) => d,
        None => return Ok(Vec::new()),
    };
    let materialized = materialize_dialog_dir(&source_dir, &work_dir, branch_slug, bot_name)?;

    // `.gbot` folder (bot config files) — materialized for completeness.
    let gbot_sources = [
        checkout.join(".gbot"),
        checkout.join(format!("{bot_name}.gbot")),
    ];
    if let Some(gbot_dir) = gbot_sources.into_iter().find(|p| p.is_dir()) {
        let gbot_target = work_root
            .join(format!("{branch_slug}.gborg/{branch_slug}.gbai"))
            .join(format!("{bot_name}.gbot"));
        let _ = copy_dir_recursive(&gbot_dir, &gbot_target);
    }
    Ok(materialized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authenticated_url_embeds_token_once() {
        // Pure string test — no Vault call: the helper is exercised through
        // integration; here we verify the format contract of the replace.
        let base = "http://alm:4747";
        let url = format!("{base}/org/repo.git").replace("://", "://tok@");
        assert_eq!(url, "http://tok@alm:4747/org/repo.git");
    }

    #[test]
    fn dialog_source_dir_prefers_root_layout() {
        let tmp = std::env::temp_dir().join(format!("gbmon-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        let root_dialog = tmp.join(".gbdialog");
        std::fs::create_dir_all(&root_dialog).unwrap();
        assert_eq!(
            dialog_source_dir(&tmp, "mybot"),
            Some(root_dialog.clone())
        );
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn materialize_copies_bas_and_reports_paths() {
        let tmp = std::env::temp_dir().join(format!("gbmon-mat-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        let src = tmp.join("checkout/.gbdialog");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("start.bas"), "TALK \"hi\"").unwrap();
        std::fs::write(src.join("tables.json"), "{}").unwrap();
        let work_root = tmp.join("work");
        let out = materialize_checkout(&tmp.join("checkout"), &work_root, "br", "mybot").unwrap();
        assert_eq!(out, vec!["br.gbai/mybot.gbdialog/start.bas".to_string()]);
        assert!(work_root
            .join("br.gborg/br.gbai/mybot.gbdialog/start.bas")
            .exists());
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
