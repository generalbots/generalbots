//! Incus-only argv construction for the interactive terminal (#2126).
//!
//! AGENTS.md is explicit: containers run on **Incus**; `lxc`/`lxd` are never
//! the container layer. Every argv built here starts with [`INCUS`] so callers
//! hand it to [`botsecurity::command_guard::SafeCommand`] (which allow-lists
//! the binary and validates each argument) instead of re-deriving a command
//! line from a raw `Command::new("lxc")`.
//!
//! The module is not feature gated: the container-name rules and the argv
//! builders are plain data, so their unit tests run in the default build.

use std::fmt;

/// Container runtime. The only supported one (AGENTS.md, "Containers").
pub const INCUS: &str = "incus";

/// Every terminal container is session scoped and carries this prefix, so the
/// terminal can never attach to (or delete) a platform/Vibe container.
pub const CONTAINER_PREFIX: &str = "term-";

/// Ephemeral base image for a fresh terminal container.
pub const TEMPLATE: &str = "images:ubuntu/24.04";

/// Longest container name Incus accepts comfortably (DNS-label shape).
const MAX_NAME_LEN: usize = 63;

/// Longest session slug kept in the container name.
const MAX_SLUG_LEN: usize = 12;

/// Containers that host platform services. A terminal must never touch them,
/// mirroring `botvibe::vm_lifecycle::Lifecycle::is_protected_container_name`.
const PROTECTED: &[&str] = &[
    "bot",
    "system",
    "tables",
    "cache",
    "drive",
    "vault",
    "directory",
    "proxy",
    "dns",
    "email",
    "webmail",
    "meet",
    "vectordb",
    "llm",
    "alm",
    "alm-ci",
    "table-editor",
    "vibe",
    "vibe-runner",
    "host",
    "incus",
];

/// Reason an Incus argv could not be built.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IncusArgvError {
    /// The name breaks the project naming rule or is a protected container.
    InvalidContainerName(String),
    /// The session id has no usable characters left after slugging.
    EmptySessionId,
    /// The argv does not start with the supported container runtime.
    UnexpectedRuntime(String),
}

impl fmt::Display for IncusArgvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidContainerName(name) => {
                write!(f, "Invalid Incus container name: {name}")
            }
            Self::EmptySessionId => write!(f, "Session id has no usable characters"),
            Self::UnexpectedRuntime(runtime) => {
                write!(f, "Unexpected container runtime (Incus only): {runtime}")
            }
        }
    }
}

impl std::error::Error for IncusArgvError {}

impl From<IncusArgvError> for String {
    fn from(value: IncusArgvError) -> Self {
        value.to_string()
    }
}

/// True when `name` is a platform container the terminal must not touch.
pub fn is_protected_container_name(name: &str) -> bool {
    PROTECTED.contains(&name.to_ascii_lowercase().as_str())
}

/// Enforces the project naming rule for Incus containers: lowercase ASCII
/// alphanumerics and `-`, no leading/trailing `-`, no empty label, bounded
/// length, and never a protected platform container.
pub fn validate_container_name(name: &str) -> Result<(), IncusArgvError> {
    let invalid = || IncusArgvError::InvalidContainerName(name.to_string());

    if name.is_empty() || name.len() > MAX_NAME_LEN {
        return Err(invalid());
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(invalid());
    }
    if name.starts_with('-') || name.ends_with('-') || name.contains("--") {
        return Err(invalid());
    }
    if is_protected_container_name(name) {
        return Err(invalid());
    }
    Ok(())
}

/// Enforces the terminal naming rule: a valid container name **and** the
/// `term-` prefix, so `stop`/`delete` can only ever target a terminal
/// container.
pub fn validate_terminal_container(name: &str) -> Result<(), IncusArgvError> {
    validate_container_name(name)?;
    let slug = name
        .strip_prefix(CONTAINER_PREFIX)
        .ok_or_else(|| IncusArgvError::InvalidContainerName(name.to_string()))?;
    if slug.is_empty() {
        return Err(IncusArgvError::InvalidContainerName(name.to_string()));
    }
    Ok(())
}

/// Slugs a raw session id (lowercase, `[a-z0-9]`, `-` separators) into the
/// container suffix, truncated so names stay well inside the length limit.
pub fn slugify_session_id(session_id: &str) -> Result<String, IncusArgvError> {
    let mut slug = String::with_capacity(MAX_SLUG_LEN);
    for ch in session_id.chars() {
        let lower = ch.to_ascii_lowercase();
        if lower.is_ascii_alphanumeric() {
            if slug.len() < MAX_SLUG_LEN {
                slug.push(lower);
            }
            continue;
        }
        if !slug.is_empty() && !slug.ends_with('-') && slug.len() < MAX_SLUG_LEN {
            slug.push('-');
        }
    }
    let trimmed = slug.trim_matches('-').to_string();
    if trimmed.is_empty() {
        return Err(IncusArgvError::EmptySessionId);
    }
    Ok(trimmed)
}

/// Container name for a terminal session: `term-{slug}`.
pub fn session_container_name(session_id: &str) -> Result<String, IncusArgvError> {
    let slug = slugify_session_id(session_id)?;
    let name = format!("{CONTAINER_PREFIX}{slug}");
    validate_terminal_container(&name)?;
    Ok(name)
}

fn argv(verb: &str, name: &str, extra: &[&str]) -> Result<Vec<String>, IncusArgvError> {
    validate_terminal_container(name)?;
    let mut args = Vec::with_capacity(4 + extra.len());
    args.push(INCUS.to_string());
    args.push(verb.to_string());
    args.push(name.to_string());
    args.extend(extra.iter().map(|arg| (*arg).to_string()));
    Ok(args)
}

/// Confirms an argv targets the only supported container runtime.
///
/// A caller that receives an argv hands it to `SafeCommand` only after this
/// check, so a future builder cannot smuggle `lxc`/`lxd` back in.
pub fn validate_runtime(argv: &[String]) -> Result<(), IncusArgvError> {
    match argv.first().map(String::as_str) {
        Some(INCUS) => Ok(()),
        Some(other) => Err(IncusArgvError::UnexpectedRuntime(other.to_string())),
        None => Err(IncusArgvError::UnexpectedRuntime(String::new())),
    }
}

/// `incus launch images:ubuntu/24.04 term-<slug> --ephemeral`
pub fn launch_argv(name: &str) -> Result<Vec<String>, IncusArgvError> {
    validate_terminal_container(name)?;
    Ok(vec![
        INCUS.to_string(),
        "launch".to_string(),
        TEMPLATE.to_string(),
        name.to_string(),
        "--ephemeral".to_string(),
    ])
}

/// `incus exec term-<slug> -- bash -l` — the shell always runs **inside** the
/// container, never on the host.
pub fn exec_argv(name: &str) -> Result<Vec<String>, IncusArgvError> {
    argv("exec", name, &["--", "bash", "-l"])
}

/// `incus stop term-<slug> --force`
pub fn stop_argv(name: &str) -> Result<Vec<String>, IncusArgvError> {
    argv("stop", name, &["--force"])
}

/// `incus delete term-<slug> --force`
pub fn delete_argv(name: &str) -> Result<Vec<String>, IncusArgvError> {
    argv("delete", name, &["--force"])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rejected(name: &str) -> bool {
        validate_terminal_container(name).is_err()
    }

    #[test]
    fn every_argv_starts_with_incus() {
        let name = session_container_name("sess-42").expect("valid session id");
        for built in [
            launch_argv(&name),
            exec_argv(&name),
            stop_argv(&name),
            delete_argv(&name),
        ] {
            let args = built.expect("argv builds for a terminal container");
            assert_eq!(args.first().map(String::as_str), Some(INCUS));
            assert!(args.len() > 2);
            assert!(!args.iter().any(|arg| arg == "lxc" || arg == "lxd"));
        }
    }

    #[test]
    fn exec_argv_runs_bash_inside_the_container() {
        let args = exec_argv("term-abc123").expect("valid argv");
        assert_eq!(args, ["incus", "exec", "term-abc123", "--", "bash", "-l"]);
    }

    #[test]
    fn launch_argv_uses_the_incus_image_reference() {
        let args = launch_argv("term-abc123").expect("valid argv");
        assert_eq!(args[1], "launch");
        assert_eq!(args[2], TEMPLATE);
        assert!(args[3].starts_with(CONTAINER_PREFIX));
    }

    #[test]
    fn only_incus_is_accepted_as_runtime() {
        let argv = exec_argv("term-abc123").expect("valid argv");
        assert!(validate_runtime(&argv).is_ok());
        assert_eq!(
            validate_runtime(&["lxc".to_string(), "exec".to_string()]),
            Err(IncusArgvError::UnexpectedRuntime("lxc".to_string()))
        );
        assert!(validate_runtime(&[]).is_err());
    }

    #[test]
    fn invalid_container_names_are_rejected() {
        for name in [
            "",
            "term-",
            "/etc/passwd",
            "term-../etc",
            "term-a b",
            "term-a;b",
            "term-$(id)",
            "term-`id`",
            "term-a|b",
            "term-a&&b",
            "Term-Upper",
            "term-a_b",
            "-term-a",
            "term-a-",
            "term-a--b",
            "system",
            "tables",
            "term-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        ] {
            assert!(rejected(name), "{name} must be rejected");
        }
    }

    #[test]
    fn protected_containers_are_never_terminal_targets() {
        assert!(is_protected_container_name("system"));
        assert!(is_protected_container_name("Vault"));
        assert!(!is_protected_container_name("term-abc123"));
    }

    #[test]
    fn session_ids_are_slugged_into_terminal_names() {
        assert_eq!(
            session_container_name("Sess_42/../x").expect("slugged"),
            "term-sess-42-x"
        );
        assert_eq!(
            session_container_name("term-1759999999999").expect("slugged"),
            "term-term-1759999"
        );
        assert!(session_container_name("///").is_err());
        assert!(session_container_name("").is_err());
    }
}
