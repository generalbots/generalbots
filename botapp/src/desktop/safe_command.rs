use std::collections::HashSet;
use std::path::PathBuf;
use std::process::{Child, Command, Output, Stdio};
use std::sync::LazyLock;

static ALLOWED_COMMANDS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    HashSet::from([
        "rclone",
        "notify-send",
        "osascript",
    ])
});

static FORBIDDEN_SHELL_CHARS: LazyLock<HashSet<char>> = LazyLock::new(|| {
    HashSet::from([
        ';', '|', '&', '$', '`', '(', ')', '{', '}', '<', '>', '\n', '\r', '\0',
    ])
});

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SafeCommandError {
    CommandNotAllowed(String),
    InvalidArgument(String),
    ExecutionFailed(String),
    ShellInjectionAttempt(String),
}

impl std::fmt::Display for SafeCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CommandNotAllowed(cmd) => write!(f, "Command not in allowlist: {cmd}"),
            Self::InvalidArgument(arg) => write!(f, "Invalid argument: {arg}"),
            Self::ExecutionFailed(msg) => write!(f, "Command execution failed: {msg}"),
            Self::ShellInjectionAttempt(input) => {
                write!(f, "Shell injection attempt detected: {input}")
            }
        }
    }
}

impl std::error::Error for SafeCommandError {}

pub struct SafeCommand {
    command: String,
    args: Vec<String>,
    working_dir: Option<PathBuf>,
    stdout: Option<Stdio>,
    stderr: Option<Stdio>,
}

impl SafeCommand {
    pub fn new(command: &str) -> Result<Self, SafeCommandError> {
        let cmd_name = std::path::Path::new(command)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(command);

        if !ALLOWED_COMMANDS.contains(cmd_name) {
            return Err(SafeCommandError::CommandNotAllowed(command.to_string()));
        }

        Ok(Self {
            command: command.to_string(),
            args: Vec::new(),
            working_dir: None,
            stdout: None,
            stderr: None,
        })
    }

    pub fn arg(mut self, arg: &str) -> Result<Self, SafeCommandError> {
        validate_argument(arg)?;
        self.args.push(arg.to_string());
        Ok(self)
    }

    #[must_use]
    pub fn stdout(mut self, stdout: Stdio) -> Self {
        self.stdout = Some(stdout);
        self
    }

    #[must_use]
    pub fn stderr(mut self, stderr: Stdio) -> Self {
        self.stderr = Some(stderr);
        self
    }

    pub fn output(self) -> Result<Output, SafeCommandError> {
        let mut cmd = Command::new(&self.command);
        cmd.args(&self.args);

        if let Some(ref dir) = self.working_dir {
            cmd.current_dir(dir);
        }

        if let Some(stdout) = self.stdout {
            cmd.stdout(stdout);
        }

        if let Some(stderr) = self.stderr {
            cmd.stderr(stderr);
        }

        cmd.output()
            .map_err(|e| SafeCommandError::ExecutionFailed(e.to_string()))
    }

    pub fn spawn(self) -> Result<Child, SafeCommandError> {
        let mut cmd = Command::new(&self.command);
        cmd.args(&self.args);

        if let Some(ref dir) = self.working_dir {
            cmd.current_dir(dir);
        }

        if let Some(stdout) = self.stdout {
            cmd.stdout(stdout);
        }

        if let Some(stderr) = self.stderr {
            cmd.stderr(stderr);
        }

        cmd.spawn()
            .map_err(|e| SafeCommandError::ExecutionFailed(e.to_string()))
    }
}

fn validate_argument(arg: &str) -> Result<(), SafeCommandError> {
    if arg.is_empty() {
        return Err(SafeCommandError::InvalidArgument(
            "Empty argument".to_string(),
        ));
    }

    if arg.len() > 4096 {
        return Err(SafeCommandError::InvalidArgument(
            "Argument too long".to_string(),
        ));
    }

    for c in arg.chars() {
        if FORBIDDEN_SHELL_CHARS.contains(&c) {
            return Err(SafeCommandError::ShellInjectionAttempt(format!(
                "Forbidden character '{}' in argument",
                c.escape_default()
            )));
        }
    }

    let dangerous_patterns = ["$(", "`", "&&", "||", ">>", "<<"];

    for pattern in dangerous_patterns {
        if arg.contains(pattern) {
            return Err(SafeCommandError::ShellInjectionAttempt(format!(
                "Dangerous pattern '{pattern}' detected"
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allowed_command() {
        assert!(SafeCommand::new("rclone").is_ok());
        assert!(SafeCommand::new("notify-send").is_ok());
        assert!(SafeCommand::new("osascript").is_ok());
    }

    #[test]
    fn test_disallowed_command() {
        assert!(SafeCommand::new("rm").is_err());
        assert!(SafeCommand::new("bash").is_err());
        assert!(SafeCommand::new("sh").is_err());
    }

    #[test]
    fn test_valid_arguments() {
        let cmd = SafeCommand::new("rclone")
            .unwrap()
            .arg("sync")
            .unwrap()
            .arg("/home/user/data")
            .unwrap()
            .arg("remote:bucket");
        assert!(cmd.is_ok());
    }

    #[test]
    fn test_injection_attempts() {
        let cmd = SafeCommand::new("rclone").unwrap();
        assert!(cmd.arg("; rm -rf /").is_err());

        let cmd = SafeCommand::new("rclone").unwrap();
        assert!(cmd.arg("$(whoami)").is_err());

        let cmd = SafeCommand::new("rclone").unwrap();
        assert!(cmd.arg("test`id`").is_err());

        let cmd = SafeCommand::new("rclone").unwrap();
        assert!(cmd.arg("a && b").is_err());
    }
}
