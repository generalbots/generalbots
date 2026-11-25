//! Loopback-only configuration for the services of the local desktop stack.
//!
//! Each `harden_*` function takes the text of a generated config file and
//! returns the hardened text, or `None` when it is already hardened, so every
//! start can rewrite existing installs idempotently.

use std::path::Path;

pub const LOOPBACK: &str = "127.0.0.1";
const SCRAM: &str = "scram-sha-256";

fn rejoin(lines: Vec<String>) -> String {
    let mut text = lines.join("\n");
    text.push('\n');
    text
}

fn indent_of(line: &str) -> &str {
    &line[..line.len() - line.trim_start().len()]
}

fn hcl_key(trimmed: &str) -> &str {
    trimmed.split('=').next().unwrap_or("").trim()
}

fn hcl_port(trimmed: &str) -> Option<u16> {
    let value = trimmed.split_once('=')?.1.trim().trim_matches('"');
    value.rsplit(':').next()?.trim().parse().ok()
}

/// Vault: the TCP listener and its cluster listener bind to loopback only.
pub fn harden_vault_hcl(text: &str) -> Option<String> {
    let mut out = Vec::new();
    let mut changed = false;
    let mut in_listener = false;
    let mut has_cluster = false;
    let mut listener_port = 8200;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("listener") {
            in_listener = true;
            has_cluster = false;
        } else if in_listener && trimmed.starts_with('}') {
            if !has_cluster {
                out.push(format!("  cluster_address = \"{LOOPBACK}:{}\"", listener_port + 1));
                changed = true;
            }
            in_listener = false;
        } else if in_listener && matches!(hcl_key(trimmed), "address" | "cluster_address") {
            let key = hcl_key(trimmed);
            let default_port = if key == "address" { 8200 } else { listener_port + 1 };
            let port = hcl_port(trimmed).unwrap_or(default_port);
            if key == "address" {
                listener_port = port;
            } else {
                has_cluster = true;
            }
            let wanted = format!("{}{key} = \"{LOOPBACK}:{port}\"", indent_of(line));
            changed |= wanted.trim() != trimmed;
            out.push(wanted);
            continue;
        }
        out.push(line.trim_end().to_string());
    }
    changed.then(|| rejoin(out))
}

/// Qdrant: `service.host` is loopback (its default is every interface).
pub fn harden_qdrant_yaml(text: &str) -> Option<String> {
    let lines: Vec<&str> = text.lines().collect();
    let wanted_value = format!("host: {LOOPBACK}");
    let mut out: Vec<String> = lines.iter().map(|l| l.trim_end().to_string()).collect();
    let Some(service) = lines.iter().position(|l| l.trim() == "service:" && indent_of(l).is_empty()) else {
        out.push("service:".to_string());
        out.push(format!("  {wanted_value}"));
        return Some(rejoin(out));
    };
    let block_end = lines[service + 1..]
        .iter()
        .position(|l| !l.trim().is_empty() && indent_of(l).is_empty())
        .map_or(lines.len(), |offset| service + 1 + offset);
    let child_indent = lines[service + 1..block_end]
        .iter()
        .find(|l| !l.trim().is_empty())
        .map_or("  ", |l| indent_of(l));
    for index in service + 1..block_end {
        if lines[index].trim_start().starts_with("host:") {
            if lines[index].trim() == wanted_value {
                return None;
            }
            out[index] = format!("{}{wanted_value}", indent_of(lines[index]));
            return Some(rejoin(out));
        }
    }
    out.insert(service + 1, format!("{child_indent}{wanted_value}"));
    Some(rejoin(out))
}

/// PostgreSQL: one active `listen_addresses` line, set to loopback.
pub fn harden_postgresql_conf(text: &str) -> Option<String> {
    let wanted = format!("listen_addresses = '{LOOPBACK}'");
    let mut out = Vec::new();
    let mut found = false;
    let mut changed = false;
    for line in text.lines() {
        if line.trim_start().starts_with("listen_addresses") {
            let active = line.split('#').next().unwrap_or("").trim();
            changed |= found || active != wanted;
            if !found {
                out.push(wanted.clone());
            }
            found = true;
            continue;
        }
        out.push(line.trim_end().to_string());
    }
    if !found {
        out.push(wanted);
        changed = true;
    }
    changed.then(|| rejoin(out))
}

fn is_host_rule(first_field: &str) -> bool {
    matches!(first_field, "host" | "hostssl" | "hostnossl" | "hostgssenc" | "hostnogssenc")
}

/// pg_hba: TCP rules ask for a SCRAM password instead of trusting any client.
pub fn harden_pg_hba(text: &str) -> Option<String> {
    let mut out = Vec::new();
    let mut changed = false;
    for line in text.lines() {
        let rule = line.split('#').next().unwrap_or("");
        let fields: Vec<&str> = rule.split_whitespace().collect();
        if fields.first().is_some_and(|f| is_host_rule(f)) && fields.len() >= 5 {
            let rewritten: Vec<&str> = fields
                .iter()
                .enumerate()
                .map(|(i, f)| if i >= 4 && *f == "trust" { SCRAM } else { f })
                .collect();
            if rewritten != fields {
                out.push(rewritten.join("\t"));
                changed = true;
                continue;
            }
        }
        out.push(line.trim_end().to_string());
    }
    changed.then(|| rejoin(out))
}

pub fn pg_hba_is_hardened(text: &str) -> bool {
    harden_pg_hba(text).is_none()
}

/// Rewrites one file when `harden` changes it. Returns whether it was written.
pub fn apply(path: &Path, harden: fn(&str) -> Option<String>) -> std::io::Result<bool> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e),
    };
    match harden(&text) {
        Some(hardened) => std::fs::write(path, hardened).map(|()| true),
        None => Ok(false),
    }
}

/// Loopback binding for every generated config of a local stack. Files that do
/// not exist yet are skipped; their generators write the loopback form.
pub fn harden_stack_configs(stack: &Path) {
    // Services start in parallel; one rewrite at a time keeps a reader from
    // hardening a half-written file.
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let guard = LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let files: [(&str, fn(&str) -> Option<String>); 4] = [
        ("conf/vault/config.hcl", harden_vault_hcl),
        ("conf/vector_db/config.yaml", harden_qdrant_yaml),
        ("data/tables/pgdata/postgresql.conf", harden_postgresql_conf),
        ("conf/postgresql.conf", harden_postgresql_conf),
    ];
    for (relative, harden) in files {
        match apply(&stack.join(relative), harden) {
            Ok(true) => log::info!("stack hardening: {relative} now binds to loopback only"),
            Ok(false) => {}
            Err(e) => log::warn!("stack hardening: could not update {relative}: {e}"),
        }
    }
    match apply(&stack.join("conf/pg_hba.conf"), harden_pg_hba) {
        Ok(true) => log::info!("stack hardening: conf/pg_hba.conf asks for passwords"),
        Ok(false) => {}
        Err(e) => log::warn!("stack hardening: could not update conf/pg_hba.conf: {e}"),
    }
    seal_plain_secret_files(stack);
    drop(guard);
}

/// Seals Vault secret files an older install left in plain text. Reading one
/// through `protected_file` seals it; a missing file is skipped.
pub fn seal_plain_secret_files(stack: &Path) {
    for relative in ["vault-unseal-keys", "conf/vault/init.json"] {
        let path = stack.join(relative);
        let Ok(bytes) = std::fs::read(&path) else { continue };
        if !crate::protected_file::is_sealed(&bytes) {
            if let Err(e) = crate::protected_file::read_secret(&path) {
                log::warn!("stack hardening: could not protect {relative}: {e}");
            }
        }
    }
}

#[cfg(test)]
#[path = "desktop_hardening_tests.rs"]
mod tests;
