//! `tool_executor::registry` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub requires_approval: bool,
    pub allowed_use_cases: Vec<VibeUseCase>,
}

impl ToolSchema {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters: serde_json::json!({"type": "object", "properties": {}}),
            requires_approval: false,
            allowed_use_cases: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_parameters(mut self, params: serde_json::Value) -> Self {
        self.parameters = params;
        self
    }

    #[must_use]
    pub fn with_approval(mut self) -> Self {
        self.requires_approval = true;
        self
    }

    #[must_use]
    pub fn with_use_cases(mut self, cases: Vec<VibeUseCase>) -> Self {
        self.allowed_use_cases = cases;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDescriptor {
    pub schema: ToolSchema,
    pub category: ToolCategory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolCategory {
    Autotask,
    Deployment,
    Crm,
    Sources,
    File,
    Analysis,
}

pub type ToolFuture = std::pin::Pin<Box<dyn std::future::Future<Output = VibeToolResult> + Send>>;

pub struct ToolRegistry {
    pub(crate) tools: RwLock<HashMap<String, RegisteredTool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let mut tools = HashMap::new();
        Self::register_builtin_tools(&mut tools);
        Self::register_harness_tools(&mut tools);
        Self {
            tools: RwLock::new(tools),
        }
    }

    /// #747 — real harness tools: file/shell/git/logs/test operating on the
    /// project workspace, all sandboxed.
    pub(crate) fn register_harness_tools(tools: &mut HashMap<String, RegisteredTool>) {
        use crate::harness;

        let set_title_schema = ToolSchema::new(
            "file/set-title",
            "Set the HTML document title in a project workspace",
        )
        .with_parameters(serde_json::json!({
            "type": "object",
            "properties": {
                "project": {"type": "string", "minLength": 1, "description": "Vibe project id (name)"},
                "title": {"type": "string", "minLength": 1, "description": "New browser document title"}
            },
            "required": ["project", "title"]
        }))
        .with_approval()
        .with_use_cases(vec![VibeUseCase::SoftwareDevelopment]);
        tools.insert(
            "file/set-title".to_string(),
            RegisteredTool {
                descriptor: ToolDescriptor {
                    schema: set_title_schema,
                    category: ToolCategory::File,
                },
                handler: harness::file_tools::file_set_title(),
            },
        );

        let entries: Vec<(String, String, bool, ToolHandler)> = vec![
            (
                "file/read".into(),
                "Read a file from the project workspace".into(),
                false,
                harness::file_tools::file_read(),
            ),
            (
                "file/write".into(),
                "Write complete file content into the project workspace; path must be a real project-relative file name".into(),
                true,
                harness::file_tools::file_write(),
            ),
            (
                "file/replace".into(),
                "Replace exact text in an existing project file; use this for focused edits instead of rewriting the whole file".into(),
                true,
                harness::file_tools::file_replace(),
            ),
            (
                "file/list".into(),
                "List files in the project workspace".into(),
                false,
                harness::file_tools::file_list(),
            ),
            (
                "file/delete".into(),
                "Delete a file from the project workspace".into(),
                true,
                harness::file_tools::file_delete(),
            ),
            (
                "file/exists".into(),
                "Check whether a workspace path exists".into(),
                false,
                harness::file_tools::file_exists(),
            ),
            (
                "shell/run".into(),
                "Run an allowlisted command inside the project workspace".into(),
                true,
                harness::run_tools::run_command(),
            ),
            (
                "git/status".into(),
                "Show git status of the project workspace".into(),
                false,
                harness::git_tools::git_status(),
            ),
            (
                "git/log".into(),
                "Show recent commits of the project".into(),
                false,
                harness::git_tools::git_log_tool(),
            ),
            (
                "git/diff".into(),
                "Show the working tree diff of the project".into(),
                false,
                harness::git_tools::git_diff_tool(),
            ),
            (
                "git/commit".into(),
                "Stage all changes and commit in the project".into(),
                true,
                harness::git_tools::git_commit_tool(),
            ),
            (
                "git/init".into(),
                "Initialize or clone the project repository into the workspace".into(),
                true,
                harness::git_tools::git_init_tool(),
            ),
            (
                "git/snapshot-previous".into(),
                "Snapshot the currently deployed commit into a release/prev-<ts> branch before publishing (rollback point)".into(),
                true,
                harness::git_tools::git_snapshot_previous_tool(),
            ),
            (
                "logs/read".into(),
                "Read the tail of a project log file".into(),
                false,
                harness::log_tools::logs_read(),
            ),
            (
                "logs/list".into(),
                "List available project log files".into(),
                false,
                harness::log_tools::logs_list(),
            ),
            (
                "test/run".into(),
                "Run the project test suite".into(),
                true,
                harness::test_tools::test_run(),
            ),
            (
                "test/list".into(),
                "Detect the test frameworks present in the project".into(),
                false,
                harness::test_tools::test_list(),
            ),
        ];

        for (name, description, requires_approval, handler) in entries {
            let required = match name.as_str() {
                "file/read" | "file/delete" | "file/exists" => vec!["project", "path"],
                "file/write" => vec!["project", "path", "content"],
                "file/replace" => vec!["project", "path", "old", "new"],
                "shell/run" => vec!["project", "command"],
                "git/commit" => vec!["project", "message"],
                _ => vec!["project"],
            };
            let tool_schema = ToolSchema::new(name.clone(), description)
                .with_parameters(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "project": {"type": "string", "minLength": 1, "description": "Vibe project id (name)"},
                        "project_id": {"type": "string", "description": "Vibe project UUID (injected by the deploy pipeline)"},
                        "path": {"type": "string", "minLength": 1, "description": "Required project-relative path such as index.js; never use an absolute path or placeholder such as ..."},
                        "content": {"type": "string", "description": "Complete replacement file content, not a patch or isolated value"},
                        "old": {"type": "string", "minLength": 1, "description": "Exact existing text to replace"},
                        "new": {"type": "string", "description": "Replacement text; may be empty to remove old text"},
                        "all": {"type": "boolean", "description": "Replace every occurrence when true; defaults to false"},
                        "command": {"type": "string", "description": "Allowlisted command to run"},
                        "args": {"type": "array", "items": {"type": "string"}, "description": "Command arguments"},
                        "message": {"type": "string", "description": "Commit message"},
                        "limit": {"type": "integer", "description": "Line/commit limit"},
                        "timeout_secs": {"type": "integer", "description": "Command timeout in seconds"}
                    },
                    "required": required
                }))
                .with_approval_if(requires_approval)
                .with_use_cases(vec![VibeUseCase::SoftwareDevelopment]);
            tools.insert(
                name,
                RegisteredTool {
                    descriptor: ToolDescriptor {
                        schema: tool_schema,
                        category: ToolCategory::File,
                    },
                    handler,
                },
            );
        }
    }

    pub async fn get_descriptor(&self, name: &str) -> Option<ToolDescriptor> {
        let tools = self.tools.read().await;
        tools.get(name).map(|t| t.descriptor.clone())
    }

    pub async fn list_tools(&self) -> Vec<ToolDescriptor> {
        let tools = self.tools.read().await;
        tools.values().map(|t| t.descriptor.clone()).collect()
    }

    pub async fn list_tools_for_use_case(&self, use_case: VibeUseCase) -> Vec<ToolDescriptor> {
        let tools = self.tools.read().await;
        tools
            .values()
            .filter(|t| {
                t.descriptor.schema.allowed_use_cases.is_empty()
                    || t.descriptor.schema.allowed_use_cases.contains(&use_case)
            })
            .map(|t| t.descriptor.clone())
            .collect()
    }

    pub async fn validate_arguments(
        &self,
        tool_name: &str,
        arguments: &serde_json::Value,
    ) -> Result<(), String> {
        let tools = self.tools.read().await;
        let tool = tools
            .get(tool_name)
            .ok_or_else(|| format!("Ferramenta '{tool_name}' não encontrada"))?;

        if tool
            .descriptor
            .schema
            .parameters
            .get("properties")
            .is_none()
        {
            return Ok(());
        }

        if let Some(props) = tool
            .descriptor
            .schema
            .parameters
            .get("properties")
            .and_then(|p| p.as_object())
        {
            let empty_map = serde_json::Map::new();
            let args_map = arguments.as_object().unwrap_or(&empty_map);
            if let Some(required) = tool
                .descriptor
                .schema
                .parameters
                .get("required")
                .and_then(|r| r.as_array())
            {
                for req in required {
                    let key = req.as_str().unwrap_or("");
                    let Some(value) = args_map.get(key) else {
                        return Err(format!("Parâmetro obrigatório ausente: '{key}'"));
                    };
                    if value.is_null() {
                        return Err(format!("Required argument '{key}' cannot be null"));
                    }
                }
            }
            for key in args_map.keys() {
                if !props.contains_key(key) {
                    return Err(format!("Parâmetro desconhecido: '{key}'"));
                }
            }
            for (key, value) in args_map {
                let Some(spec) = props.get(key) else {
                    continue;
                };
                if spec.get("type").and_then(|value| value.as_str()) == Some("string") {
                    let Some(text) = value.as_str() else {
                        return Err(format!("Argument '{key}' must be a string"));
                    };
                    let min_length = spec
                        .get("minLength")
                        .and_then(|value| value.as_u64())
                        .unwrap_or(0) as usize;
                    if text.trim().chars().count() < min_length {
                        return Err(format!("Argument '{key}' must not be empty"));
                    }
                    if key == "path" && matches!(text.trim(), "..." | "…") {
                        return Err(
                            "Argument 'path' must name a real project-relative file".to_string()
                        );
                    }
                }
            }
        }

        Ok(())
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub trait ToolSchemaExt {
    fn with_approval_if(self, needs_approval: bool) -> Self;
}

impl ToolSchemaExt for ToolSchema {
    fn with_approval_if(mut self, needs_approval: bool) -> Self {
        self.requires_approval = needs_approval;
        self
    }
}
