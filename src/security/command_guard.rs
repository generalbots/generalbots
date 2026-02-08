use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;
use std::process::{Child, Output};
use std::sync::LazyLock;

static ALLOWED_COMMANDS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    HashSet::from([
        "pdftotext",
        "pandoc",
        "nvidia-smi",
        "powershell",
        "clamscan",
        "freshclam",
        "mc",
        "ffmpeg",
        "ffprobe",
        "convert",
        "gs",
        "tesseract",
        "which",
        "where",
        "sh",
        "bash",
        "cmd",
        "pkill",
        "pgrep",
        "kill",
        "fuser",
        "curl",
        "tar",
        "unzip",
        "openssl",
        "pg_dump",
        "pg_isready",
        "lxc",
        "lxc-execute",
        "lxd",
        "docker",
        "apt-get",
        "brew",
        "rustc",
        "nvcc",
        // Desktop/sync commands
        "rclone",
        // Notification commands
        "notify-send",
        "osascript",
        // Shell utilities
        "true",
        "rm",
        "find",
        // Test/dev utilities
        "cargo",
        "redis-server",
        "redis-cli",
        "minio",
        "chromedriver",
        "chrome",
        "chromium",
        "brave",
        "diesel",
        "initdb",
        "pg_ctl",
        "createdb",
        "psql",
        // Security protection tools
        "lynis",
        "rkhunter",
        "chkrootkit",
        "suricata",
        "suricata-update",
        "maldet",
        "systemctl",
        "sudo",
        "visudo",
    ])
});

static FORBIDDEN_SHELL_CHARS: LazyLock<HashSet<char>> = LazyLock::new(|| {
    HashSet::from([
        ';', '|', '&', '$', '`', '(', ')', '{', '}', '<', '>', '\n', '\r', '\0',
    ])
});

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandGuardError {
    CommandNotAllowed(String),
    InvalidArgument(String),
    PathTraversal(String),
    ExecutionFailed(String),
    ShellInjectionAttempt(String),
}

impl std::fmt::Display for CommandGuardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CommandNotAllowed(cmd) => write!(f, "Command not in allowlist: {cmd}"),
            Self::InvalidArgument(arg) => write!(f, "Invalid argument: {arg}"),
            Self::PathTraversal(path) => write!(f, "Path traversal detected: {path}"),
            Self::ExecutionFailed(msg) => write!(f, "Command execution failed: {msg}"),
            Self::ShellInjectionAttempt(input) => {
                write!(f, "Shell injection attempt detected: {input}")
            }
        }
    }
}

impl std::error::Error for CommandGuardError {}

pub struct SafeCommand {
    command: String,
    args: Vec<String>,
    working_dir: Option<PathBuf>,
    allowed_paths: Vec<PathBuf>,
    envs: HashMap<String, String>,
}

impl SafeCommand {
    pub fn new(command: &str) -> Result<Self, CommandGuardError> {
        let cmd_name = std::path::Path::new(command)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(command);

        if !ALLOWED_COMMANDS.contains(cmd_name) {
            return Err(CommandGuardError::CommandNotAllowed(command.to_string()));
        }

        Ok(Self {
            command: command.to_string(),
            args: Vec::new(),
            working_dir: None,
            allowed_paths: vec![
                PathBuf::from("/tmp"),
                PathBuf::from("/var/tmp"),
                dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")),
                std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
            ],
            envs: HashMap::new(),
        })
    }

    pub fn arg(mut self, arg: &str) -> Result<Self, CommandGuardError> {
        validate_argument(arg)?;
        self.args.push(arg.to_string());
        Ok(self)
    }

    pub fn shell_script_arg(mut self, script: &str) -> Result<Self, CommandGuardError> {
        let is_unix_shell = self.command == "bash" || self.command == "sh";
        let is_windows_cmd = self.command == "cmd";
        if !is_unix_shell && !is_windows_cmd {
            return Err(CommandGuardError::InvalidArgument(
                "shell_script_arg only allowed for bash/sh/cmd commands".to_string(),
            ));
        }
        let valid_flag = if is_unix_shell {
            self.args.last().is_some_and(|a| a == "-c")
        } else {
            self.args.last().is_some_and(|a| a == "/C" || a == "/c")
        };
        if !valid_flag {
            return Err(CommandGuardError::InvalidArgument(
                "shell_script_arg requires -c (unix) or /C (windows) flag to be set first".to_string(),
            ));
        }
        if script.is_empty() {
            return Err(CommandGuardError::InvalidArgument(
                "Empty script".to_string(),
            ));
        }
        if script.len() > 8192 {
            return Err(CommandGuardError::InvalidArgument(
                "Script too long".to_string(),
            ));
        }
        let forbidden_patterns = ["$(", "`", ".."];
        for pattern in forbidden_patterns {
            if script.contains(pattern) {
                return Err(CommandGuardError::ShellInjectionAttempt(format!(
                    "Dangerous pattern '{}' in shell script",
                    pattern
                )));
            }
        }
        self.args.push(script.to_string());
        Ok(self)
    }

    pub fn trusted_shell_script_arg(mut self, script: &str) -> Result<Self, CommandGuardError> {
        let is_unix_shell = self.command == "bash" || self.command == "sh";
        let is_windows_cmd = self.command == "cmd";
        if !is_unix_shell && !is_windows_cmd {
            return Err(CommandGuardError::InvalidArgument(
                "trusted_shell_script_arg only allowed for bash/sh/cmd commands".to_string(),
            ));
        }
        let valid_flag = if is_unix_shell {
            self.args.last().is_some_and(|a| a == "-c")
        } else {
            self.args.last().is_some_and(|a| a == "/C" || a == "/c")
        };
        if !valid_flag {
            return Err(CommandGuardError::InvalidArgument(
                "trusted_shell_script_arg requires -c (unix) or /C (windows) flag to be set first".to_string(),
            ));
        }
        if script.is_empty() {
            return Err(CommandGuardError::InvalidArgument(
                "Empty script".to_string(),
            ));
        }
        if script.len() > 16384 {
            return Err(CommandGuardError::InvalidArgument(
                "Script too long".to_string(),
            ));
        }
        self.args.push(script.to_string());
        Ok(self)
    }

    pub fn args(mut self, args: &[&str]) -> Result<Self, CommandGuardError> {
        for arg in args {
            validate_argument(arg)?;
            self.args.push((*arg).to_string());
        }
        Ok(self)
    }

    pub fn path_arg(mut self, path: &std::path::Path) -> Result<Self, CommandGuardError> {
        let validated_path = validate_path(path, &self.allowed_paths)?;
        self.args.push(validated_path.to_string_lossy().to_string());
        Ok(self)
    }

    pub fn working_dir(mut self, dir: &std::path::Path) -> Result<Self, CommandGuardError> {
        let validated = validate_path(dir, &self.allowed_paths)?;
        self.working_dir = Some(validated);
        Ok(self)
    }

    pub fn allow_path(mut self, path: PathBuf) -> Self {
        self.allowed_paths.push(path);
        self
    }

    pub fn env(mut self, key: &str, value: &str) -> Result<Self, CommandGuardError> {
        validate_argument(key)?;
        validate_argument(value)?;
        self.envs.insert(key.to_string(), value.to_string());
        Ok(self)
    }

    pub fn execute(&self) -> Result<Output, CommandGuardError> {
        let mut cmd = std::process::Command::new(&self.command);
        cmd.args(&self.args);

        if let Some(ref dir) = self.working_dir {
            cmd.current_dir(dir);
        }

        cmd.env_clear();

        // Build PATH with standard locations plus botserver-stack/bin/shared
        let mut path_entries = vec![
            "/usr/local/bin".to_string(),
            "/usr/bin".to_string(),
            "/bin".to_string(),
        ];

        // Add botserver-stack/bin/shared to PATH if it exists
        let stack_path = std::env::var("BOTSERVER_STACK_PATH")
            .unwrap_or_else(|_| "./botserver-stack".to_string());
        let shared_bin = format!("{}/bin/shared", stack_path);
        if std::path::Path::new(&shared_bin).exists() {
            path_entries.insert(0, shared_bin);
        }

        cmd.env("PATH", path_entries.join(":"));
        cmd.env("HOME", dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp")));
        cmd.env("LANG", "C.UTF-8");

        for (key, value) in &self.envs {
            cmd.env(key, value);
        }

        cmd.output()
            .map_err(|e| CommandGuardError::ExecutionFailed(e.to_string()))
    }

    pub async fn execute_async(&self) -> Result<Output, CommandGuardError> {
        let mut cmd = std::process::Command::new(&self.command);
        cmd.args(&self.args);

        if let Some(ref dir) = self.working_dir {
            cmd.current_dir(dir);
        }

        cmd.env_clear();

        // Build PATH with standard locations plus botserver-stack/bin/shared
        let mut path_entries = vec![
            "/usr/local/bin".to_string(),
            "/usr/bin".to_string(),
            "/bin".to_string(),
        ];

        // Add botserver-stack/bin/shared to PATH if it exists
        let stack_path = std::env::var("BOTSERVER_STACK_PATH")
            .unwrap_or_else(|_| "./botserver-stack".to_string());
        let shared_bin = format!("{}/bin/shared", stack_path);
        if std::path::Path::new(&shared_bin).exists() {
            path_entries.insert(0, shared_bin);
        }

        cmd.env("PATH", path_entries.join(":"));
        cmd.env("HOME", dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp")));
        cmd.env("LANG", "C.UTF-8");

        for (key, value) in &self.envs {
            cmd.env(key, value);
        }

        cmd.output()
            .map_err(|e| CommandGuardError::ExecutionFailed(e.to_string()))
    }

    pub fn spawn(&self) -> Result<Child, CommandGuardError> {
        let mut cmd = std::process::Command::new(&self.command);
        cmd.args(&self.args);

        if let Some(ref dir) = self.working_dir {
            cmd.current_dir(dir);
        }

        cmd.env_clear();

        // Build PATH with standard locations plus botserver-stack/bin/shared
        let mut path_entries = vec![
            "/usr/local/bin".to_string(),
            "/usr/bin".to_string(),
            "/bin".to_string(),
        ];

        // Add botserver-stack/bin/shared to PATH if it exists
        let stack_path = std::env::var("BOTSERVER_STACK_PATH")
            .unwrap_or_else(|_| "./botserver-stack".to_string());
        let shared_bin = format!("{}/bin/shared", stack_path);
        if std::path::Path::new(&shared_bin).exists() {
            path_entries.insert(0, shared_bin);
        }

        cmd.env("PATH", path_entries.join(":"));
        cmd.env("HOME", dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp")));
        cmd.env("LANG", "C.UTF-8");

        for (key, value) in &self.envs {
            cmd.env(key, value);
        }

        cmd.spawn()
            .map_err(|e| CommandGuardError::ExecutionFailed(e.to_string()))
    }

    pub fn spawn_with_envs(&self, envs: &HashMap<String, String>) -> Result<Child, CommandGuardError> {
        let mut cmd = std::process::Command::new(&self.command);
        cmd.args(&self.args);

        if let Some(ref dir) = self.working_dir {
            cmd.current_dir(dir);
        }

        cmd.env_clear();

        // Build PATH with standard locations plus botserver-stack/bin/shared
        let mut path_entries = vec![
            "/usr/local/bin".to_string(),
            "/usr/bin".to_string(),
            "/bin".to_string(),
        ];

        // Add botserver-stack/bin/shared to PATH if it exists
        let stack_path = std::env::var("BOTSERVER_STACK_PATH")
            .unwrap_or_else(|_| "./botserver-stack".to_string());
        let shared_bin = format!("{}/bin/shared", stack_path);
        if std::path::Path::new(&shared_bin).exists() {
            path_entries.insert(0, shared_bin);
        }

        cmd.env("PATH", path_entries.join(":"));
        cmd.env("HOME", dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp")));
        cmd.env("LANG", "C.UTF-8");

        for (key, value) in &self.envs {
            cmd.env(key, value);
        }

        for (key, value) in envs {
            if validate_argument(key).is_ok() && validate_argument(value).is_ok() {
                cmd.env(key, value);
            }
        }

        cmd.spawn()
            .map_err(|e| CommandGuardError::ExecutionFailed(e.to_string()))
    }

    pub fn noop_child() -> Result<Child, CommandGuardError> {
        std::process::Command::new("true")
            .spawn()
            .map_err(|e| CommandGuardError::ExecutionFailed(e.to_string()))
    }
}

pub fn validate_argument(arg: &str) -> Result<(), CommandGuardError> {
    if arg.is_empty() {
        return Err(CommandGuardError::InvalidArgument(
            "Empty argument".to_string(),
        ));
    }

    if arg.len() > 4096 {
        return Err(CommandGuardError::InvalidArgument(
            "Argument too long".to_string(),
        ));
    }

    let is_url = arg.starts_with("http://") || arg.starts_with("https://");

    for c in arg.chars() {
        if FORBIDDEN_SHELL_CHARS.contains(&c) {
            if is_url && (c == '&' || c == '?' || c == '=') {
                continue;
            }
            return Err(CommandGuardError::ShellInjectionAttempt(format!(
                "Forbidden character '{}' in argument",
                c.escape_default()
            )));
        }
    }

    let dangerous_patterns = [
        "$(", "`", "&&", "||", ">>", "<<", "..", "//", "\\\\",
    ];

    for pattern in dangerous_patterns {
        if arg.contains(pattern) {
            if is_url && pattern == "//" {
                continue;
            }
            return Err(CommandGuardError::ShellInjectionAttempt(format!(
                "Dangerous pattern '{}' detected",
                pattern
            )));
        }
    }

    Ok(())
}

pub fn validate_path(
    path: &std::path::Path,
    allowed_roots: &[PathBuf],
) -> Result<PathBuf, CommandGuardError> {
    let canonical = path
        .canonicalize()
        .or_else(|_| {
            if let Some(parent) = path.parent() {
                parent.canonicalize().map(|p| p.join(path.file_name().unwrap_or_default()))
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Path not found",
                ))
            }
        })
        .map_err(|_| {
            CommandGuardError::PathTraversal(format!(
                "Cannot canonicalize path: {}",
                path.display()
            ))
        })?;

    let path_str = canonical.to_string_lossy();
    if path_str.contains("..") {
        return Err(CommandGuardError::PathTraversal(format!(
            "Path contains traversal: {}",
            path.display()
        )));
    }

    let is_allowed = allowed_roots
        .iter()
        .any(|root| canonical.starts_with(root));

    if !is_allowed {
        return Err(CommandGuardError::PathTraversal(format!(
            "Path outside allowed directories: {}",
            path.display()
        )));
    }

    Ok(canonical)
}

pub fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
        .collect::<String>()
        .trim_start_matches('.')
        .to_string()
}

pub fn safe_pdftotext(
    pdf_path: &std::path::Path,
    _allowed_paths: &[PathBuf],
) -> Result<String, CommandGuardError> {
    let output = SafeCommand::new("pdftotext")?
        .allow_path(pdf_path.parent().unwrap_or(std::path::Path::new("/tmp")).to_path_buf())
        .arg("-layout")?
        .path_arg(pdf_path)?
        .arg("-")?
        .execute()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(CommandGuardError::ExecutionFailed(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ))
    }
}

pub async fn safe_pdftotext_async(
    pdf_path: &std::path::Path,
) -> Result<String, CommandGuardError> {
    let parent = pdf_path.parent().unwrap_or(std::path::Path::new("/tmp")).to_path_buf();

    let output = SafeCommand::new("pdftotext")?
        .allow_path(parent)
        .arg("-layout")?
        .path_arg(pdf_path)?
        .arg("-")?
        .execute_async()
        .await?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(CommandGuardError::ExecutionFailed(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ))
    }
}

pub async fn safe_pandoc_async(
    input_path: &std::path::Path,
    from_format: &str,
    to_format: &str,
) -> Result<String, CommandGuardError> {
    validate_argument(from_format)?;
    validate_argument(to_format)?;

    let allowed_formats = ["docx", "plain", "html", "markdown", "rst", "latex", "txt"];
    if !allowed_formats.contains(&from_format) || !allowed_formats.contains(&to_format) {
        return Err(CommandGuardError::InvalidArgument(
            "Invalid format specified".to_string(),
        ));
    }

    let parent = input_path.parent().unwrap_or(std::path::Path::new("/tmp")).to_path_buf();

    let output = SafeCommand::new("pandoc")?
        .allow_path(parent)
        .arg("-f")?
        .arg(from_format)?
        .arg("-t")?
        .arg(to_format)?
        .path_arg(input_path)?
        .execute_async()
        .await?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(CommandGuardError::ExecutionFailed(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ))
    }
}

pub fn safe_nvidia_smi() -> Result<std::collections::HashMap<String, f32>, CommandGuardError> {
    let output = SafeCommand::new("nvidia-smi")?
        .arg("--query-gpu=utilization.gpu,utilization.memory")?
        .arg("--format=csv,noheader,nounits")?
        .execute()?;

    if !output.status.success() {
        return Err(CommandGuardError::ExecutionFailed(
            "Failed to query GPU utilization".to_string(),
        ));
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut util = std::collections::HashMap::new();

    for line in output_str.lines() {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 2 {
            util.insert(
                "gpu".to_string(),
                parts[0].trim().parse::<f32>().unwrap_or_default(),
            );
            util.insert(
                "memory".to_string(),
                parts[1].trim().parse::<f32>().unwrap_or_default(),
            );
        }
    }

    Ok(util)
}

pub fn has_nvidia_gpu_safe() -> bool {
    SafeCommand::new("nvidia-smi")
        .and_then(|cmd| {
            cmd.arg("--query-gpu=utilization.gpu")?
                .arg("--format=csv,noheader,nounits")
        })
        .and_then(|cmd| cmd.execute())
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_argument_valid() {
        assert!(validate_argument("hello").is_ok());
        assert!(validate_argument("-f").is_ok());
        assert!(validate_argument("--format=csv").is_ok());
        assert!(validate_argument("/path/to/file.txt").is_ok());
    }

    #[test]
    fn test_validate_argument_invalid() {
        assert!(validate_argument("hello; rm -rf /").is_err());
        assert!(validate_argument("$(whoami)").is_err());
        assert!(validate_argument("file | cat").is_err());
        assert!(validate_argument("test && echo").is_err());
        assert!(validate_argument("`id`").is_err());
        assert!(validate_argument("").is_err());
    }

    #[test]
    fn test_safe_command_allowed() {
        assert!(SafeCommand::new("pdftotext").is_ok());
        assert!(SafeCommand::new("pandoc").is_ok());
        assert!(SafeCommand::new("nvidia-smi").is_ok());
    }

    #[test]
    fn test_safe_command_disallowed() {
        assert!(SafeCommand::new("wget").is_err());
        assert!(SafeCommand::new("nc").is_err());
        assert!(SafeCommand::new("netcat").is_err());
        assert!(SafeCommand::new("dd").is_err());
        assert!(SafeCommand::new("mkfs").is_err());
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("test.pdf"), "test.pdf");
        assert_eq!(sanitize_filename("my-file_v1.txt"), "my-file_v1.txt");
        assert_eq!(sanitize_filename("../../../etc/passwd"), "etcpasswd");
        assert_eq!(sanitize_filename(".hidden"), "hidden");
        assert_eq!(sanitize_filename("file;rm -rf.txt"), "filerm-rf.txt");
    }

    #[test]
    fn test_path_traversal_detection() {
        let _allowed = vec![PathBuf::from("/tmp")];

        let result = validate_argument("../../../etc/passwd");
        assert!(result.is_err());
    }

    #[test]
    fn test_command_guard_error_display() {
        let err = CommandGuardError::CommandNotAllowed("bash".to_string());
        assert!(err.to_string().contains("bash"));

        let err2 = CommandGuardError::ShellInjectionAttempt("$(id)".to_string());
        assert!(err2.to_string().contains("injection"));
    }
}
