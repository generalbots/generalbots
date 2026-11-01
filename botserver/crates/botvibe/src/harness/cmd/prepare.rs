use super::*;

pub(super) fn prepare_command(
    program: &str,
    args: &[String],
    cwd: &std::path::Path,
) -> Result<std::process::Command, GuardError> {
    validate_program(program)?;
    if !program_is_allowed(program) {
        return Err(GuardError::CommandNotAllowed(program.into()));
    }
    // #1534: narrow the executable to the three harness commands before it
    // reaches a command line. `program_is_allowed` is the shared guard; this
    // explicit match keeps the taint analysis honest.
    if !matches!(program, "node" | "npm" | "npx") {
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
            // #1534: never fall back to executing the caller-named program.
            // Without an absolute node.exe path the harness cannot run npm/npx,
            // so the request fails instead of spawning an arbitrary binary.
            _ => {
                return Err(GuardError::CommandNotAllowed(format!(
                    "{program}: Node.js is not installed under ProgramFiles"
                )))
            }
        }
    };
    #[cfg(not(target_os = "windows"))]
    let mut command = std::process::Command::new(harness_executable(program)?);

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

/// The only executables the harness may spawn (#1534). Every arm is a string
/// literal, so request input selects from a fixed set instead of naming the
/// program that gets executed.
fn harness_executable(program: &str) -> Result<&'static str, GuardError> {
    match program {
        "node" => Ok("node"),
        "npm" => Ok("npm"),
        "npx" => Ok("npx"),
        _ => Err(GuardError::CommandNotAllowed(program.into())),
    }
}

#[cfg(test)]
mod allowlist_tests {
    use super::{harness_executable, prepare_command};

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn builds_only_allowlisted_commands() {
        for program in ["node", "npm", "npx"] {
            let command = prepare_command(program, &["--version".to_string()], std::path::Path::new("/tmp"));
            assert!(command.is_ok(), "{program} must be allowed");
        }
    }

    #[test]
    fn rejects_any_other_program() {
        for program in [
            "bash",
            "sh",
            "/bin/sh",
            "/usr/bin/node",
            "node; rm -rf /",
            "node ",
            "NODE",
            "",
        ] {
            assert!(
                harness_executable(program).is_err(),
                "{program:?} must be rejected"
            );
        }
    }

    #[test]
    fn rejects_shell_metacharacters_in_arguments() {
        let args = vec!["ok".to_string(), "; rm -rf /".to_string()];
        assert!(prepare_command("node", &args, std::path::Path::new("/tmp")).is_err());
    }
}
