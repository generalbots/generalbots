//! #1955 — the rsync half of the sync manager: dry runs, conflict detection
//! and the transfers themselves.

use std::path::Path as FsPath;
use std::time::Instant;

use botlib::security::command_guard::SafeCommand;

use super::{clean_dir, now, SyncConflict, SyncJob};

/// One rsync run may not take longer than this.
const RUN_TIMEOUT_SECONDS: u64 = 600;

struct RunOutcome {
    ok: bool,
    stdout: String,
    stderr: String,
}

fn execute_rsync(arguments: &[String]) -> Result<RunOutcome, String> {
    let mut command = SafeCommand::new("rsync")
        .map_err(|e| format!("rsync is not available on this host: {e}"))?;
    for argument in arguments {
        command = command
            .arg(argument)
            .map_err(|e| format!("rsync refused the argument: {e}"))?;
    }
    let output = command
        .execute()
        .map_err(|e| format!("rsync could not start: {e}"))?;
    Ok(RunOutcome {
        ok: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

fn transfer(from: &FsPath, to: &FsPath, delete: bool) -> Result<RunOutcome, String> {
    let mut arguments: Vec<String> = vec![
        "-a".to_string(),
        format!("--timeout={RUN_TIMEOUT_SECONDS}"),
        "--itemize-changes".to_string(),
        "--out-format=%i>%n".to_string(),
    ];
    if delete {
        arguments.push("--delete".to_string());
    }
    arguments.push(format!("{}/", from.display()));
    arguments.push(format!("{}/", to.display()));
    execute_rsync(&arguments)
}

/// Dry-run in one direction, returning the paths rsync would change.
fn pending_paths(from: &FsPath, to: &FsPath) -> Result<Vec<String>, String> {
    let arguments: Vec<String> = vec![
        "-a".to_string(),
        "-n".to_string(),
        "--itemize-changes".to_string(),
        "--out-format=%i>%n".to_string(),
        format!("{}/", from.display()),
        format!("{}/", to.display()),
    ];
    let outcome = execute_rsync(&arguments)?;
    if !outcome.ok {
        return Err(format!("rsync dry run failed: {}", outcome.stderr.trim()));
    }
    Ok(outcome
        .stdout
        .lines()
        .filter_map(|line| line.split_once('>'))
        .map(|(_, path)| path.trim().to_string())
        .filter(|path| !path.is_empty() && path != "./")
        .collect())
}

/// A path that both directions would change is a conflict: syncing either way
/// silently would drop the other side's work.
pub(super) fn scan_conflicts(
    local: &FsPath,
    drive: &FsPath,
) -> Result<Vec<SyncConflict>, String> {
    let pushed = pending_paths(local, drive)?;
    let pulled = pending_paths(drive, local)?;
    let mut conflicts: Vec<SyncConflict> = Vec::new();
    for path in pushed {
        if pulled.iter().any(|other| *other == path) {
            conflicts.push(SyncConflict {
                detail: "changed on both sides since the last sync".to_string(),
                path,
            });
        }
    }
    Ok(conflicts)
}

fn summarize(outcome: &RunOutcome) -> String {
    let changed = outcome
        .stdout
        .lines()
        .filter(|line| line.contains('>'))
        .count();
    if changed == 0 {
        "already in sync".to_string()
    } else {
        format!("{changed} path(s) updated")
    }
}

pub(super) fn perform(locale: &botcore::i18n::Locale, job: &mut SyncJob) -> Result<(), String> {
    let local = clean_dir(
        locale,
        &job.local_path,
        "error-drive-sync-field-local-folder",
    )?;
    let drive = clean_dir(
        locale,
        &job.drive_path,
        "error-drive-sync-field-drive-folder",
    )?;
    let started = Instant::now();
    job.last_status = "syncing".to_string();
    job.last_error = None;

    job.conflicts = scan_conflicts(&local, &drive)?;
    if !job.conflicts.is_empty() {
        job.last_status = "conflict".to_string();
        job.last_run = Some(now());
        job.last_duration_ms = Some(started.elapsed().as_millis() as u64);
        job.last_summary = Some(format!(
            "{} conflict(s) need a decision",
            job.conflicts.len()
        ));
        return Ok(());
    }

    let mut summaries: Vec<String> = Vec::new();
    match job.direction.as_str() {
        "pull" => summaries.push(summarize(&transfer(&drive, &local, false)?)),
        "two-way" => {
            summaries.push(format!(
                "upload: {}",
                summarize(&transfer(&local, &drive, false)?)
            ));
            summaries.push(format!(
                "download: {}",
                summarize(&transfer(&drive, &local, false)?)
            ));
        }
        _ => summaries.push(summarize(&transfer(&local, &drive, false)?)),
    }
    job.last_status = "ok".to_string();
    job.last_summary = Some(summaries.join("; "));
    job.last_run = Some(now());
    job.last_duration_ms = Some(started.elapsed().as_millis() as u64);
    Ok(())
}
