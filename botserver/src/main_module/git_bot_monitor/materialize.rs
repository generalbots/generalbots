//! Materialize git checkout sources (`.gbdialog` / `.gbot`) into the work
//! layout the runtime reads. Split out of [`super::core`] (#2863) so the
//! symlink-escape guard and the materialization loops stay reviewable.

use std::path::{Path, PathBuf};

use super::path_safety::{classify_entry, destination_is_safe, EntryKind};

pub(crate) const MAX_MATERIALIZED_FILES: usize = 400;

/// Recursively copy `src` into `dst`, refusing symlinks that escape the tree
/// and refusing to write through a symlink already planted in the work dir.
pub(crate) fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    copy_tree(src, dst, src, dst)
}

fn copy_tree(
    src: &Path,
    dst: &Path,
    src_root: &Path,
    dst_root: &Path,
) -> std::io::Result<()> {
    // A link occupying the destination directory would redirect every write
    // made below it. Fail closed before create_dir_all follows it.
    if let Ok(meta) = std::fs::symlink_metadata(dst) {
        if meta.file_type().is_symlink() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                format!("refusing symlinked destination {}", dst.display()),
            ));
        }
    }
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let to = dst.join(entry.file_name());
        match classify_entry(src_root, &path) {
            EntryKind::Directory => copy_tree(&path, &to, src_root, dst_root)?,
            EntryKind::File => {
                if !destination_is_safe(dst_root, &to) {
                    log::warn!(
                        "[git_monitor] refused unsafe destination {} (symlink escape?)",
                        to.display()
                    );
                    continue;
                }
                std::fs::copy(&path, &to)?;
            }
            EntryKind::Refuse => {
                log::warn!(
                    "[git_monitor] refused symlink/special entry {} (symlink escape?)",
                    path.display()
                );
            }
        }
    }
    Ok(())
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
    for entry in
        std::fs::read_dir(source_dir).map_err(|e| format!("read {}: {e}", source_dir.display()))?
    {
        let entry = entry.map_err(|e| format!("entry: {e}"))?;
        let path = entry.path();
        // #2863: classify with symlink_metadata so a committed link to an
        // external file is skipped before fs::copy can read or write through
        // it. Only real files and safe in-tree file symlinks reach the copy.
        match classify_entry(source_dir, &path) {
            EntryKind::File => {}
            EntryKind::Refuse => {
                log::warn!(
                    "[git_monitor] {bot_name}: refused symlink/special entry {} (symlink escape?)",
                    path.display()
                );
                continue;
            }
            EntryKind::Directory => continue,
        }
        let file_name = entry.file_name().to_string_lossy().to_string();
        // `.ast` is a build artifact, never a source: copying one from the
        // repository would install a script compiled by another machine (or
        // another revision of this one) over the freshly materialized `.bas`.
        if file_name.ends_with(".ast") {
            continue;
        }
        if !file_name.ends_with(".bas") && !file_name.ends_with(".json") {
            continue;
        }
        total += 1;
        if total > MAX_MATERIALIZED_FILES {
            return Err(format!(
                "materialize {bot_name}: more than {MAX_MATERIALIZED_FILES} source files — trim the repo"
            ));
        }
        let to = work_dir.join(&file_name);
        if !destination_is_safe(work_dir, &to) {
            log::warn!(
                "[git_monitor] {bot_name}: refused unsafe destination {} (symlink escape?)",
                to.display()
            );
            continue;
        }
        std::fs::copy(&path, &to).map_err(|e| format!("copy {file_name}: {e}"))?;
        if file_name.ends_with(".bas") {
            // The compiled artifact of the previous revision is now stale.
            // Removing it is what makes the compiler pick the new source up:
            // `DriveCompiler` treats a missing `.ast` as a forced recompile,
            // and the git ETag it records is not a value the Drive monitor can
            // reproduce (one is a commit sha, the other an S3 ETag), so the
            // ETag alone must not be the trigger. Materialization only runs
            // when the checkout actually moved, so this is not churn.
            let tool = file_name.trim_end_matches(".bas").to_string();
            let stale_ast = work_dir.join(format!("{tool}.ast"));
            if stale_ast.exists() {
                if let Err(e) = std::fs::remove_file(&stale_ast) {
                    log::warn!(
                        "[git_monitor] {bot_name}: could not drop stale {tool}.ast ({e}) — the tool keeps running the previous revision"
                    );
                } else {
                    log::info!("[git_monitor] {bot_name}: {tool}.bas updated, stale .ast invalidated for recompile");
                }
            }
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

    // `.gbot` folder (bot config files: channel prompts, styles) — the runtime
    // reads `PROMPT-{CHANNEL}.md` from here and silently falls back to a
    // generic prompt when the copy is missing, so a failure is reported.
    let gbot_sources = [
        checkout.join(".gbot"),
        checkout.join(format!("{bot_name}.gbot")),
    ];
    if let Some(gbot_dir) = gbot_sources.into_iter().find(|p| p.is_dir()) {
        let gbot_target = work_root
            .join(format!("{branch_slug}.gborg/{branch_slug}.gbai"))
            .join(format!("{bot_name}.gbot"));
        if let Err(e) = copy_dir_recursive(&gbot_dir, &gbot_target) {
            log::warn!(
                "[git_monitor] {bot_name}: .gbot materialization failed: {e} — channel prompts stay on the fallback"
            );
        }
    }
    Ok(materialized)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("gbmat-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn dialog_source_dir_prefers_root_layout() {
        let tmp = scratch("dialog");
        let root_dialog = tmp.join(".gbdialog");
        std::fs::create_dir_all(&root_dialog).unwrap();
        assert_eq!(dialog_source_dir(&tmp, "mybot"), Some(root_dialog.clone()));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn materialize_copies_bas_and_reports_paths() {
        let tmp = scratch("copy");
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

    /// A pushed `.bas` must invalidate the `.ast` compiled from the previous
    /// revision, and an `.ast` committed by mistake must never be installed —
    /// otherwise the bot keeps executing the old script after the push.
    #[test]
    fn materialize_invalidates_stale_ast() {
        let tmp = scratch("ast");
        let src = tmp.join("checkout/.gbdialog");
        std::fs::create_dir_all(&src).unwrap();
        let work_root = tmp.join("work");
        let work_dir = work_root.join("br.gborg/br.gbai/mybot.gbdialog");
        std::fs::create_dir_all(&work_dir).unwrap();
        // Previous revision: compiled artifact of the old source.
        std::fs::write(work_dir.join("start.ast"), "OLD AST").unwrap();

        std::fs::write(src.join("start.bas"), "TALK \\\"new\\\"").unwrap();
        // A committed artifact must be ignored, not copied.
        std::fs::write(src.join("start.ast"), "COMMITTED AST").unwrap();

        materialize_checkout(&tmp.join("checkout"), &work_root, "br", "mybot").unwrap();

        assert!(!work_dir.join("start.ast").exists());
        assert_eq!(
            std::fs::read_to_string(work_dir.join("start.bas")).unwrap(),
            "TALK \\\"new\\\""
        );
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    #[cfg(unix)]
    fn escaping_dialog_symlink_is_not_materialized() {
        let tmp = scratch("escape-dialog");
        let src = tmp.join("checkout/.gbdialog");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("start.bas"), "TALK \"hi\"").unwrap();
        let outside = tmp.join("outside.bas");
        std::fs::write(&outside, "SECRET HOST CONTENT").unwrap();
        // Absolute link into a host file — used to be copied verbatim.
        std::os::unix::fs::symlink(&outside, src.join("leak.bas")).unwrap();
        let work_root = tmp.join("work");
        let work_dir = work_root.join("br.gborg/br.gbai/mybot.gbdialog");

        materialize_checkout(&tmp.join("checkout"), &work_root, "br", "mybot").unwrap();

        assert!(!work_dir.join("leak.bas").exists(), "escaping symlink was materialized");
        assert_eq!(
            std::fs::read_to_string(work_dir.join("start.bas")).unwrap(),
            "TALK \"hi\""
        );
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    #[cfg(unix)]
    fn in_tree_dialog_symlink_is_materialized() {
        let tmp = scratch("safe-dialog");
        let src = tmp.join("checkout/.gbdialog");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("start.bas"), "TALK \"hi\"").unwrap();
        std::os::unix::fs::symlink("start.bas", src.join("alias.bas")).unwrap();
        let work_root = tmp.join("work");
        let work_dir = work_root.join("br.gborg/br.gbai/mybot.gbdialog");

        materialize_checkout(&tmp.join("checkout"), &work_root, "br", "mybot").unwrap();

        assert_eq!(
            std::fs::read_to_string(work_dir.join("alias.bas")).unwrap(),
            "TALK \"hi\""
        );
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    #[cfg(unix)]
    fn escaping_gbot_symlink_is_not_copied() {
        let tmp = scratch("escape-gbot");
        let src = tmp.join("checkout/.gbot");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("PROMPT.md"), "prompt").unwrap();
        let outside = tmp.join("outside.md");
        std::fs::write(&outside, "SECRET HOST CONTENT").unwrap();
        std::os::unix::fs::symlink(&outside, src.join("leak.md")).unwrap();
        let dst = tmp.join("work/br.gborg/br.gbai/mybot.gbot");

        copy_dir_recursive(&src, &dst).unwrap();

        assert!(!dst.join("leak.md").exists(), "escaping symlink was copied");
        assert_eq!(std::fs::read_to_string(dst.join("PROMPT.md")).unwrap(), "prompt");
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    #[cfg(unix)]
    fn planted_destination_symlink_is_not_written_through() {
        let tmp = scratch("dest-symlink");
        let src = tmp.join("checkout/.gbdialog");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("leak.bas"), "FROM REPO").unwrap();
        let work_root = tmp.join("work");
        let work_dir = work_root.join("br.gborg/br.gbai/mybot.gbdialog");
        std::fs::create_dir_all(&work_dir).unwrap();
        let outside = tmp.join("outside.bas");
        std::fs::write(&outside, "ORIGINAL").unwrap();
        // A symlink already occupying the destination must not be followed.
        std::os::unix::fs::symlink(&outside, work_dir.join("leak.bas")).unwrap();

        materialize_checkout(&tmp.join("checkout"), &work_root, "br", "mybot").unwrap();

        assert_eq!(std::fs::read_to_string(&outside).unwrap(), "ORIGINAL");
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
