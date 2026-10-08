//! #747 — Command runner behind the Vibe tools.
//!
//! Mirrors the `SafeCommand` discipline: an explicit binary allowlist, no
//! shell metacharacters in arguments, no environment overrides, a hard
//! timeout, and bounded output capture. This is the only place in the
//! harness that spawns processes.

use std::collections::HashSet;
use std::path::Path;
use std::process::Stdio;
use std::sync::LazyLock;

static ALLOWED_COMMANDS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    HashSet::from([
        "ls",
        "cat",
        "head",
        "tail",
        "grep",
        "find",
        "wc",
        "diff",
        "stat",
        "git",
        "mkdir",
        "touch",
        "cp",
        "mv",
        "rm",
        // #917 — `sh` is deliberately absent: `sh -c "..."` executes arbitrary
        // code with no blocked metacharacter, and no harness tool needs it.
        "node",
        "npm",
        "npx",
        "cargo",
        "python3",
        "python",
        "botserver",
        "botc",
        "caddy",
        "incus",
        "wsl",
        "dig",
        "nslookup",
        // #1486 — CodeGraph: pre-indexed code knowledge graph (symbols, call
        // edges, blast radius). Installed into `apps`-kind project VMs; the
        // `code/*` wired tools shell out through here.
        "codegraph",
    ])
});

const FORBIDDEN_SHELL_CHARS: [char; 9] = [';', '|', '&', '$', '`', '<', '>', '\n', '\0'];
const MAX_OUTPUT_BYTES: usize = 512 * 1024;
const MAX_ARGS: usize = 64;

/// #1489 — `PATH` handed to the child process. `prepare_command` clears the
/// environment, so without this the child inherits nothing and `execvp` falls
/// back to `confstr(_CS_PATH)`, which on this host is only `/bin:/usr/bin`.
/// A binary installed by an npm global prefix (`/usr/local/bin`) was therefore
/// unreachable from `shell/run` while being perfectly reachable from a normal
/// shell — surfacing as `Spawn("No such file or directory")`, which reads like
/// a missing binary rather than a missing `PATH` entry.
///
/// Constant, never inherited from the botserver process: the guard decides what
/// the child can see.
const CHILD_PATH: &str = "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin";

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

fn prepare_command(
    program: &str,
    args: &[String],
    cwd: &std::path::Path,
) -> Result<std::process::Command, GuardError> {
    validate_program(program)?;
    if !program_is_allowed(program) {
        return Err(GuardError::CommandNotAllowed(program.into()));
    }
    if args.len() > MAX_ARGS {
        return Err(GuardError::InvalidArgument("too many arguments".into()));
    }
    for arg in args {
        validate_arg(arg)?;
    }

    #[cfg(target_os = "windows")]
    let mut command = {
        let node_root = std::env::var_os("ProgramFiles")
            .map(std::path::PathBuf::from)
            .map(|root| root.join("nodejs"));
        let npm_cli = match program {
            "npm" => Some("npm-cli.js"),
            "npx" => Some("npx-cli.js"),
            _ => None,
        };
        match (node_root, npm_cli) {
            (Some(root), Some(cli)) if root.join("node.exe").is_file() => {
                let mut command = std::process::Command::new(root.join("node.exe"));
                command.arg(root.join("node_modules").join("npm").join("bin").join(cli));
                command
            }
            _ => std::process::Command::new(program),
        }
    };
    #[cfg(not(target_os = "windows"))]
    let mut command = std::process::Command::new(program);

    command.args(args).current_dir(cwd).env_clear();

    #[cfg(not(target_os = "windows"))]
    command.env("PATH", CHILD_PATH);

    #[cfg(target_os = "windows")]
    for key in [
        "SystemRoot",
        "WINDIR",
        "COMSPEC",
        "PATH",
        "PATHEXT",
        "ProgramFiles",
        "USERPROFILE",
        "TEMP",
        "TMP",
    ] {
        if let Some(value) = std::env::var_os(key) {
            command.env(key, value);
        }
    }
    Ok(command)
}

/// #1489 — the allowlist is a list of binary names, so an explicit absolute
/// path is admitted when its final component is an allowed name. The `..`
/// rejection in `validate_program` still applies to the whole path, and a path
/// such as `/usr/local/bin/docker` is refused exactly like a bare `docker`.
fn program_is_allowed(program: &str) -> bool {
    if ALLOWED_COMMANDS.contains(program) {
        return true;
    }
    match Path::new(program).file_name().and_then(|n| n.to_str()) {
        Some(basename) => ALLOWED_COMMANDS.contains(basename),
        None => false,
    }
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
            return Err(GuardError::ShellInjection(format!(
                "forbidden character '{ch}' in argument"
            )));
        }
    }
    // Control characters can alter argument parsing semantics even without
    // shell metacharacters — reject anything non-printable (#1297).
    if arg
        .chars()
        .any(|ch| ch.is_control() || char::is_whitespace(ch) && (ch != ' ' && ch != '\t'))
    {
        return Err(GuardError::InvalidArgument(
            "argument contains control characters".into(),
        ));
    }
    Ok(())
}

/// The program name itself must be a plain executable name or absolute path
/// with no traversal or metacharacters (#1297).
fn validate_program(program: &str) -> Result<(), GuardError> {
    if program.is_empty() {
        return Err(GuardError::InvalidArgument("empty program".into()));
    }
    if program.chars().any(|ch| FORBIDDEN_SHELL_CHARS.contains(&ch)) {
        return Err(GuardError::ShellInjection(format!(
            "forbidden character in program name '{program}'"
        )));
    }
    if program.contains("..") {
        return Err(GuardError::InvalidArgument(
            "program path must not contain traversal sequences".into(),
        ));
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
    let mut command = prepare_command(program, args, cwd)?;
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .map_err(|e| GuardError::Spawn(e.to_string()))?;

    // #917 — drain stdout/stderr concurrently with a bounded buffer, then poll
    // the child with the timeout. The previous code joined the reader threads
    // *before* polling the child, so a child that never exited (and thus never
    // closed its pipes) deadlocked the runner and the timeout never fired.
    let stdout_join = child
        .stdout
        .take()
        .map(|o| std::thread::spawn(move || drain_reader(o)));
    let stderr_join = child
        .stderr
        .take()
        .map(|o| std::thread::spawn(move || drain_reader(o)));

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout_secs);
    let mut exit_code: Option<i32> = None;
    let mut timed_out = false;
    loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|e| GuardError::Io(e.to_string()))?
        {
            exit_code = status.code();
            break;
        }
        if std::time::Instant::now() > deadline {
            let _ = child.kill();
            let _ = child.wait();
            timed_out = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }

    let stdout = stdout_join
        .map(|j| j.join().map_err(|_| GuardError::Io("stdout thread".into())))
        .transpose()?
        .unwrap_or_default();
    let stderr = stderr_join
        .map(|j| j.join().map_err(|_| GuardError::Io("stderr thread".into())))
        .transpose()?
        .unwrap_or_default();

    if timed_out {
        return Err(GuardError::Timeout);
    }
    Ok(RunOutput {
        exit_code,
        stdout: String::from_utf8_lossy(&stdout).into_owned(),
        stderr: String::from_utf8_lossy(&stderr).into_owned(),
    })
}

/// Spawn a validated long-lived child without a command shell. The caller
/// owns the child handle and is responsible for monitoring its lifetime.
pub fn spawn_persistent(
    program: &str,
    args: &[String],
    cwd: &std::path::Path,
) -> Result<std::process::Child, GuardError> {
    prepare_command(program, args, cwd)?
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| GuardError::Spawn(e.to_string()))
}

/// Reads a child's pipe up to `MAX_OUTPUT_BYTES` and truncates, so an
/// untrusted command cannot exhaust memory by flooding its output (#917).
fn drain_reader<R: std::io::Read>(reader: R) -> Vec<u8> {
    use std::io::Read;
    let mut buf = Vec::new();
    let _ = reader
        .take(MAX_OUTPUT_BYTES as u64 + 1)
        .read_to_end(&mut buf);
    buf.truncate(MAX_OUTPUT_BYTES);
    buf
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
        let args = vec!["--help".to_string(), "echo hi; rm -rf /".to_string()];
        assert!(matches!(
            run("git", &args, &cwd, 5),
            Err(GuardError::ShellInjection(_))
        ));
    }

    #[test]
    fn sh_is_not_allowlisted() {
        assert!(
            !ALLOWED_COMMANDS.contains("sh"),
            "sh -c is arbitrary code execution"
        );
        // #1489 — admitting absolute paths must not become a way in.
        assert!(!program_is_allowed("/bin/sh"));
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

    // #1489 — the allowlist is a list of binary names, so an explicit absolute
    // path to an allowed binary must resolve while a path to a refused binary
    // must not.
    #[test]
    fn absolute_path_to_allowed_binary_is_accepted() {
        assert!(program_is_allowed("/usr/local/bin/codegraph"));
        assert!(program_is_allowed("/usr/bin/git"));
        assert!(program_is_allowed("git"));
    }

    #[test]
    fn absolute_path_to_refused_binary_is_rejected() {
        assert!(!program_is_allowed("/usr/local/bin/docker"));
        assert!(!program_is_allowed("/bin/sh"));
        assert!(!program_is_allowed("docker"));
    }

    #[test]
    fn traversal_in_program_path_is_still_rejected() {
        let cwd = std::env::temp_dir();
        let err = run("/usr/local/../bin/git", &[], &cwd, 5);
        assert!(matches!(err, Err(GuardError::InvalidArgument(_))));
    }

    // #1489 — the child must be able to resolve a binary that lives only under
    // `/usr/local/bin` (the npm global prefix), which `confstr(_CS_PATH)` does
    // not cover. `node` is used as the stand-in because it is allowlisted and
    // present on every supported host; the assertion is that PATH reaches the
    // child at all, which is what was broken.
    #[cfg(not(target_os = "windows"))]
    #[test]
    fn child_receives_a_path() {
        let cwd = std::env::temp_dir();
        let out = run("node", &["-p".to_string(), "process.env.PATH".to_string()], &cwd, 30);
        let out = out.expect("node should run");
        assert_eq!(out.exit_code, Some(0));
        let reported = out.stdout.trim();
        assert!(!reported.is_empty(), "child PATH must not be empty");
        assert_eq!(
            reported, CHILD_PATH,
            "child must get the constant guard PATH, not an inherited one"
        );
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn child_path_covers_the_npm_global_prefix() {
        assert!(
            CHILD_PATH.split(':').any(|dir| dir == "/usr/local/bin"),
            "npm global installs land in /usr/local/bin"
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn runs_npm_without_a_command_shell() {
        let cwd = std::env::temp_dir();
        let out = run("npm", &["--version".to_string()], &cwd, 30)
            .expect("npm should run through node.exe");
        assert_eq!(out.exit_code, Some(0));
        assert!(!out.stdout.trim().is_empty());
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
        let out = run(
            "node",
            &["index.js".to_string(), "2+3".to_string()],
            &dir,
            10,
        );
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
