//! #747 — Command runner behind the Vibe tools.
//!
//! Mirrors the `SafeCommand` discipline: an explicit binary allowlist, no
//! shell metacharacters in arguments, no environment overrides, a hard
//! timeout, and bounded output capture. This is the only place in the
//! harness that spawns processes.

use std::collections::HashSet;
use std::process::Stdio;
use std::sync::LazyLock;

static ALLOWED_COMMANDS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    HashSet::from([
        "ls", "cat", "head", "tail", "grep", "find", "wc", "diff", "stat",
        "git", "mkdir", "touch", "cp", "mv", "rm",
        "node", "npm", "npx", "cargo", "python3", "python", "sh",
        "botserver", "botc", "caddy", "incus", "dig", "nslookup",
    ])
});

const FORBIDDEN_SHELL_CHARS: [char; 9] = [';', '|', '&', '$', '`', '<', '>', '\n', '\0'];
const MAX_OUTPUT_BYTES: usize = 512 * 1024;
const MAX_ARGS: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuardError {
    CommandNotAllowed(String),
    InvalidArgument(String),
    ShellInjection(String),
    Spawn(String),
    Timeout,
    Io(String),
}

impl std::fmt::Display for GuardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CommandNotAllowed(c) => write!(f, "command not in allowlist: {c}"),
            Self::InvalidArgument(a) => write!(f, "invalid argument: {a}"),
            Self::ShellInjection(s) => write!(f, "shell injection attempt: {s}"),
            Self::Spawn(m) => write!(f, "spawn failed: {m}"),
            Self::Timeout => write!(f, "command exceeded time limit"),
            Self::Io(m) => write!(f, "io: {m}"),
        }
    }
}

#[derive(Debug)]
pub struct RunOutput {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

/// Validate a single argument: no shell metacharacters, bounded length this
/// shell-char check mirrors `command_guard` semantics without needing the
/// botcore crate.
fn validate_arg(arg: &str) -> Result<(), GuardError> {
    if arg.is_empty() {
        return Err(GuardError::InvalidArgument("empty argument".into()));
    }
    if arg.chars().count() > 4096 {
        return Err(GuardError::InvalidArgument("argument too long".into()));
    }
    for ch in arg.chars() {
        if FORBIDDEN_SHELL_CHARS.contains(&ch) {
            return Err(GuardError::ShellInjection(format!("forbidden character '{ch}' in argument")));
        }
    }
    Ok(())
}

/// Run `program args` in `cwd` with a timeout. No `-c` shell strings are
/// composed here: every argument is passed verbatim to `std::process`.
pub fn run(
    program: &str,
    args: &[String],
    cwd: &std::path::Path,
    timeout_secs: u64,
) -> Result<RunOutput, GuardError> {
    if !ALLOWED_COMMANDS.contains(program) {
        return Err(GuardError::CommandNotAllowed(program.into()));
    }
    if args.len() > MAX_ARGS {
        return Err(GuardError::InvalidArgument("too many arguments".into()));
    }
    for arg in args {
        validate_arg(arg)?;
    }

    let mut child = std::process::Command::new(program)
        .args(args)
        .current_dir(cwd)
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| GuardError::Spawn(e.to_string()))?;

    let (stdout, stderr) = {
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let stdout_join = stdout.map(|mut o| {
            std::thread::spawn(move || {
                let mut buf = Vec::new();
                use std::io::Read;
                let _ = o.read_to_end(&mut buf);
                buf.truncate(MAX_OUTPUT_BYTES);
                buf
            })
        });
        let stderr_join = stderr.map(|mut o| {
            std::thread::spawn(move || {
                let mut buf = Vec::new();
                use std::io::Read;
                let _ = o.read_to_end(&mut buf);
                buf.truncate(MAX_OUTPUT_BYTES);
                buf
            })
        });
        (
            stdout_join.map(|j| j.join().map_err(|_| GuardError::Io("stdout thread".into())))
                .transpose()?.unwrap_or_default(),
            stderr_join.map(|j| j.join().map_err(|_| GuardError::Io("stderr thread".into())))
                .transpose()?.unwrap_or_default(),
        )
    };

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout_secs);
    let exit_code = loop {
        if let Some(status) = child.try_wait().map_err(|e| GuardError::Io(e.to_string()))? {
            break status.code();
        }
        if std::time::Instant::now() > deadline {
            let _ = child.kill();
            return Err(GuardError::Timeout);
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    };

    Ok(RunOutput {
        exit_code,
        stdout: String::from_utf8_lossy(&stdout).into_owned(),
        stderr: String::from_utf8_lossy(&stderr).into_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_allowlisted_commands() {
        let cwd = std::env::temp_dir();
        let err = run("docker", &["ps".to_string()], &cwd, 5);
        assert!(matches!(err, Err(GuardError::CommandNotAllowed(name)) if name == "docker"));
    }

    #[test]
    fn rejects_shell_metacharacters_in_args() {
        let cwd = std::env::temp_dir();
        let args = vec!["-c".to_string(), "echo hi; rm -rf /".to_string()];
        assert!(matches!(run("sh", &args, &cwd, 5), Err(GuardError::ShellInjection(_))));
    }

    #[test]
    fn runs_allowlisted_command() {
        let cwd = std::env::temp_dir();
        let out = run("git", &["--version".to_string()], &cwd, 30);
        let out = out.expect("git should run");
        assert_eq!(out.exit_code, Some(0));
        assert!(out.stdout.contains("git version"));
    }

    #[test]
    fn allowlist_contains_harness_commands() {
        for cmd in ["git", "cat", "ls", "tail", "npm", "cargo", "python3"] {
            assert!(ALLOWED_COMMANDS.contains(cmd), "{cmd} must be allowlisted");
        }
    }
}
#[cfg(test)]
mod harness_cmd_tests {
    use super::*;

    #[test]
    fn runs_node_expression() {
        // Self-contained: write the fixture into a temp dir instead of
        // depending on a pre-seeded /tmp/vibe-workspaces/calculator tree.
        let dir = std::env::temp_dir().join(format!("vibe-cmd-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("index.js"), "console.log(eval(process.argv[2]));").unwrap();
        let out = run("node", &["index.js".to_string(), "2+3".to_string()], &dir, 10);
        let _ = std::fs::remove_dir_all(&dir);
        match out {
            Ok(o) => {
                assert_eq!(o.exit_code, Some(0));
                assert_eq!(o.stdout.trim(), "5");
            }
            Err(e) => panic!("node run failed: {e:?}"),
        }
    }
}
