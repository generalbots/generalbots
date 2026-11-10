//! `pg_dump` / `pg_restore` round-trip for the restore drill (#2117).
//!
//! The drill proves that a dump taken from a production-like database restores
//! into a scratch database with the same rows per tenant. Every helper reports
//! the tool it needs, so the suite can skip with a precise reason when the
//! client binaries are absent instead of silently passing.

use std::path::Path;
use std::process::Command;

/// `true` when the tool is on `PATH`.
pub fn available(tool: &str) -> bool {
    which::which(tool).is_ok()
}

fn run(program: &str, args: &[&str]) -> Result<(), String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| format!("{program}: {e}"))?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(format!(
        "{program} exited with {}: {}",
        output.status,
        stderr.trim()
    ))
}

/// Runs one SQL file through `psql` against `url`.
pub fn psql_file(url: &str, path: &Path) -> Result<(), String> {
    let sql = path.to_string_lossy().to_string();
    run("psql", &[url, "-v", "ON_ERROR_STOP=1", "-q", "-f", sql.as_str()])
}

/// Runs one SQL statement through `psql` against `url`.
pub fn psql_command(url: &str, statement: &str) -> Result<(), String> {
    run("psql", &[url, "-v", "ON_ERROR_STOP=1", "-q", "-c", statement])
}

/// Swaps the database name in a connection URL, keeping credentials and params.
pub fn scratch_url(base: &str, name: &str) -> Result<String, String> {
    let (without_query, query) = match base.split_once('?') {
        Some((head, tail)) => (head, Some(tail)),
        None => (base, None),
    };
    let cut = without_query
        .rfind('/')
        .ok_or_else(|| format!("cannot find the database name in {base}"))?;
    let mut url = format!("{}/{name}", without_query.get(..cut).unwrap_or(without_query));
    if let Some(query) = query {
        url.push('?');
        url.push_str(query);
    }
    Ok(url)
}

/// A database name derived from the pid, so parallel runs never collide.
pub fn scratch_name(prefix: &str) -> String {
    format!("{prefix}_{}", std::process::id())
}

/// `pg_dump -Fc` of the whole database.
pub fn dump(url: &str, path: &Path) -> Result<(), String> {
    let target = path.to_string_lossy().to_string();
    run("pg_dump", &["-Fc", "-d", url, "-f", target.as_str()])
}

/// Creates the scratch database; the identifier is quoted because it is derived
/// from the process id rather than from user input.
pub fn create_database(url: &str, name: &str) -> Result<(), String> {
    psql_command(url, &format!("CREATE DATABASE \"{name}\""))
}

/// Drops the scratch database, terminating any leftover connection.
pub fn drop_database(url: &str, name: &str) -> Result<(), String> {
    psql_command(url, &format!("DROP DATABASE IF EXISTS \"{name}\" WITH (FORCE)"))
}

/// `pg_restore` of a custom-format dump into an already created database.
pub fn restore(url: &str, path: &Path) -> Result<(), String> {
    let source = path.to_string_lossy().to_string();
    run(
        "pg_restore",
        &[
            "--no-owner",
            "--no-privileges",
            "--exit-on-error",
            "-d",
            url,
            source.as_str(),
        ],
    )
}
