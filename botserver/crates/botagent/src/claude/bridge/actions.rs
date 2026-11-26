//! What a Claude Code tool use does, in the terms of the permission card.
//! A tool the bridge does not describe here is denied without asking.

use serde_json::Value;

use super::protocol::ToolRequest;

/// One tool use, described for the person. Paths are absolute (a relative
/// one is resolved against the session's working directory) so the card,
/// and the protected-path check, see the real target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClaudeAction {
    /// `Bash`: the exact command line.
    Command { command: String },
    /// `Bash` running nothing but a package install.
    Install { manager: String, packages: Vec<String>, command: String },
    /// `Read`.
    ReadFile { path: String },
    /// `Write` (`edit: false`), `Edit`, `MultiEdit` or `NotebookEdit`.
    WriteFile { path: String, edit: bool },
    /// `WebFetch`.
    Fetch { url: String },
    /// `WebSearch`: the query (the search runs on Anthropic's side).
    Search { query: String },
}

/// The built-in tools a bridged session may use (`--tools`): each one asks
/// (see `SETTINGS_JSON`) and is described by [`ClaudeAction::from_request`].
pub(super) const TOOLS: &str = "Bash,Read,Write,Edit,NotebookEdit,WebFetch,WebSearch";

fn field(input: &Value, key: &str) -> Option<String> {
    input
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
}

fn absolute(path: String, cwd: &str) -> String {
    if path.starts_with('/') {
        return path;
    }
    let tail = path.strip_prefix("./").unwrap_or(&path);
    format!("{}/{tail}", cwd.trim_end_matches('/'))
}

/// Install flags that change nothing about what is fetched or from where;
/// any other flag (an index URL, a registry) keeps the command a command.
const QUIET_FLAGS: &[&str] = &[
    "-y", "--yes", "-g", "--global", "-q", "--quiet", "--user", "-U", "--upgrade", "--no-input",
    "-D", "--save-dev", "--no-install-recommends", "--break-system-packages",
];

/// `pip install x y`, `npm install -g z`, `apt-get install -y w`: the whole
/// command is one install with plain package names and no shell operators.
fn install(command: &str) -> Option<(String, Vec<String>)> {
    if command.chars().any(|c| ";&|<>`$(){}\\\n'\"".contains(c)) {
        return None;
    }
    let words: Vec<&str> = command.split_whitespace().collect();
    let words = words.strip_prefix(&["sudo"]).unwrap_or(&words);
    let (manager, rest) = match words {
        ["pip" | "pip3", "install", rest @ ..] => ("pip", rest),
        ["python" | "python3", "-m", "pip", "install", rest @ ..] => ("pip", rest),
        ["npm", "install" | "i", rest @ ..] => ("npm", rest),
        ["apt-get" | "apt", "install", rest @ ..] => ("apt", rest),
        _ => return None,
    };
    let (flags, packages): (Vec<&str>, Vec<&str>) = rest.iter().partition(|w| w.starts_with('-'));
    if !flags.iter().all(|f| QUIET_FLAGS.contains(f)) {
        return None;
    }
    let packages: Vec<String> = packages.iter().map(|w| (*w).to_string()).collect();
    let plain = packages
        .iter()
        .all(|p| p.chars().all(|c| c.is_ascii_alphanumeric() || "-_.@/=<>~^[],:+".contains(c)));
    (!packages.is_empty() && plain).then(|| (manager.to_string(), packages))
}

impl ClaudeAction {
    /// The action a `can_use_tool` request asks for, or `None` for a tool
    /// (or argument shape) the bridge does not know: that one is denied.
    #[must_use]
    pub fn from_request(request: &ToolRequest, cwd: &str) -> Option<Self> {
        let input = &request.input;
        match request.tool_name.as_str() {
            "Bash" => {
                let command = field(input, "command")?;
                Some(match install(&command) {
                    Some((manager, packages)) => Self::Install { manager, packages, command },
                    None => Self::Command { command },
                })
            }
            "Read" => Some(Self::ReadFile { path: absolute(field(input, "file_path")?, cwd) }),
            "Write" => Some(Self::WriteFile { path: absolute(field(input, "file_path")?, cwd), edit: false }),
            "Edit" | "MultiEdit" => {
                Some(Self::WriteFile { path: absolute(field(input, "file_path")?, cwd), edit: true })
            }
            "NotebookEdit" => {
                Some(Self::WriteFile { path: absolute(field(input, "notebook_path")?, cwd), edit: true })
            }
            "WebFetch" => {
                let url = field(input, "url")?;
                (url.starts_with("https://") || url.starts_with("http://")).then_some(Self::Fetch { url })
            }
            "WebSearch" => Some(Self::Search { query: field(input, "query")? }),
            _ => None,
        }
    }

    /// Files the action writes, for its step row.
    #[must_use]
    pub fn files(&self) -> Vec<String> {
        match self {
            Self::WriteFile { path, .. } => vec![path.clone()],
            _ => Vec::new(),
        }
    }

    #[must_use]
    pub fn is_command(&self) -> bool {
        matches!(self, Self::Command { .. } | Self::Install { .. })
    }
}

/// The exit code a `Bash` tool result states: `Exit code N` on its first
/// line when it failed, 0 when it succeeded.
#[must_use]
pub fn exit_code(is_error: bool, text: &str) -> Option<i32> {
    if !is_error {
        return Some(0);
    }
    text.lines()
        .next()?
        .trim()
        .strip_prefix("Exit code ")?
        .trim()
        .parse()
        .ok()
}

/// The result text without the code line and the CLI's own reminders.
#[must_use]
pub fn output_text(text: &str) -> String {
    let body = text.split("<system-reminder>").next().unwrap_or_else(|| {
        tracing::debug!("Claude output has no content before the reminder; using the whole text");
        text
    });
    let mut lines = body.lines().peekable();
    if lines.peek().is_some_and(|l| l.trim().starts_with("Exit code ")) {
        lines.next();
    }
    lines.collect::<Vec<_>>().join("\n").trim().to_string()
}
