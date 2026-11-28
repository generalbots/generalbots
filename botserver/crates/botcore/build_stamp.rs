//! Build stamp shared by the botcore and botapp build scripts.
//!
//! `build_info` reads `BOTSERVER_COMMIT` and `BOTSERVER_BUILD_DATE` with
//! `option_env!`, which only sees variables set for the crate being compiled.
//! The release workflow exports both; a local build has neither, so the
//! commit falls back to `git rev-parse --short HEAD` and the date to today
//! (UTC, `YYYY-MM-DD`, the workflow's format).

use std::process::Command;

pub fn emit() {
    println!("cargo:rerun-if-env-changed=BOTSERVER_COMMIT");
    println!("cargo:rerun-if-env-changed=BOTSERVER_BUILD_DATE");
    if let Some(commit) = env_value("BOTSERVER_COMMIT").or_else(git_commit) {
        println!("cargo:rustc-env=BOTSERVER_COMMIT={commit}");
    }
    let date = env_value("BOTSERVER_BUILD_DATE").unwrap_or_else(utc_today);
    println!("cargo:rustc-env=BOTSERVER_BUILD_DATE={date}");
    for path in git_refs_to_watch() {
        println!("cargo:rerun-if-changed={path}");
    }
}

fn env_value(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn git(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?.trim().to_string();
    (!text.is_empty()).then_some(text)
}

fn git_commit() -> Option<String> {
    git(&["rev-parse", "--short", "HEAD"])
}

/// The files that move when a commit lands or the branch changes, so the
/// stamp is refreshed without re-running the script on every build.
fn git_refs_to_watch() -> Vec<String> {
    let mut refs = vec!["HEAD".to_string(), "packed-refs".to_string()];
    if let Some(branch) = git(&["symbolic-ref", "-q", "HEAD"]) {
        refs.push(branch);
    }
    refs.iter()
        .filter_map(|name| git(&["rev-parse", "--git-path", name]))
        .filter(|path| std::path::Path::new(path).is_file())
        .collect()
}

fn utc_today() -> String {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0);
    let (year, month, day) = civil_from_days((seconds / 86_400) as i64);
    format!("{year:04}-{month:02}-{day:02}")
}

/// Days since 1970-01-01 to a proleptic Gregorian date (H. Hinnant's
/// algorithm).
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    (year, month, day)
}
