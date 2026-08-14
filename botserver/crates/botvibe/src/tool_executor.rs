use crate::types::{VibeState, VibeToolCall, VibeToolResult, VibeUseCase};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

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

pub type ToolHandler = Arc<dyn Fn(serde_json::Value, &dyn VibeState) -> ToolFuture + Send + Sync>;
pub type ToolFuture = std::pin::Pin<Box<dyn std::future::Future<Output = VibeToolResult> + Send>>;

pub struct ToolRegistry {
    tools: RwLock<HashMap<String, RegisteredTool>>,
}

struct RegisteredTool {
    descriptor: ToolDescriptor,
    handler: ToolHandler,
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

    fn register_builtin_tools(tools: &mut HashMap<String, RegisteredTool>) {

        // #796 — wired tools: real implementations (autotask, CRM, analysis).
        for (name, schema, handler) in crate::wired_tools::autotask::autotask_tools() {
            tools.insert(name.clone(), RegisteredTool {
                descriptor: ToolDescriptor { schema, category: ToolCategory::Autotask },
                handler,
            });
        }

        let deploy_tools = vec![
            ("deploy_app", "Realiza deploy de aplicação gerada", true),
        ];

        for (name, desc, approval) in deploy_tools {
            let schema = ToolSchema::new(name, desc)
                .with_parameters(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "app_name": {"type": "string", "description": "Name of the project/app"},
                        "org": {"type": "string", "description": "ALM organization name"},
                        "project_type": {"type": "string", "enum": ["bot", "app-htmx", "app-react", "app-vue", "site"], "description": "Project type: bot, app-*, or site"},
                        "environment": {"type": "string", "enum": ["development", "staging", "production"], "description": "Deployment environment"},
                        "framework": {"type": "string", "description": "Framework for apps (htmx, react, vue)"},
                        "custom_domain": {"type": "string", "description": "Optional custom domain"},
                        "files": {"type": "object", "description": "Files to deploy {path: content}"}
                    },
                    "required": ["app_name", "org", "project_type"]
                }))
                .with_approval_if(approval)
                .with_use_cases(vec![VibeUseCase::SoftwareDevelopment]);
            tools.insert(name.to_string(), RegisteredTool {
                descriptor: ToolDescriptor { schema, category: ToolCategory::Deployment },
                handler: Arc::new(deploy_app_handler()),
            });
        }

        let publish_schema = crate::publish::publish_project_schema();
        tools.insert("publish/project".to_string(), RegisteredTool {
            descriptor: ToolDescriptor {
                schema: publish_schema,
                category: ToolCategory::Deployment,
            },
            handler: crate::publish::publish_project_tool(),
        });

        for (name, schema, handler) in [
            ("domain/bind", crate::domains_tool::domain_bind_schema(), crate::domains_tool::domain_bind_tool()),
            ("domain/verify", crate::domains_tool::domain_verify_schema(), crate::domains_tool::domain_verify_tool()),
            ("domain/tls", crate::domains_tool::domain_tls_schema(), crate::domains_tool::domain_tls_tool()),
        ] {
            tools.insert(name.to_string(), RegisteredTool {
                descriptor: ToolDescriptor {
                    schema,
                    category: ToolCategory::Deployment,
                },
                handler,
            });
        }

        for (name, schema, handler) in crate::ops_tools::ops_tools() {
            tools.insert(name.to_string(), RegisteredTool {
                descriptor: ToolDescriptor {
                    schema,
                    category: ToolCategory::Deployment,
                },
                handler,
            });
        }

        // #796 — CRM tools: contacts, deals, tickets, queued email.
        for (name, schema, handler) in crate::wired_tools::crm::crm_tools() {
            tools.insert(name.clone(), RegisteredTool {
                descriptor: ToolDescriptor { schema, category: ToolCategory::Crm },
                handler,
            });
        }

        // #796 — analysis tools: market data, sentiment, reports, anomalies.
        for (name, schema, handler) in crate::wired_tools::analysis::analysis_tools() {
            tools.insert(name.clone(), RegisteredTool {
                descriptor: ToolDescriptor { schema, category: ToolCategory::Analysis },
                handler,
            });
        }
    }

    /// #747 — real harness tools: file/shell/git/logs/test operating on the
    /// project workspace, all sandboxed.
    fn register_harness_tools(tools: &mut HashMap<String, RegisteredTool>) {
        use crate::harness;

        let entries: Vec<(String, String, bool, ToolHandler)> = vec![
            ("file/read".into(), "Read a file from the project workspace".into(), false, harness::file_tools::file_read()),
            ("file/write".into(), "Write a file into the project workspace".into(), true, harness::file_tools::file_write()),
            ("file/list".into(), "List files in the project workspace".into(), false, harness::file_tools::file_list()),
            ("file/delete".into(), "Delete a file from the project workspace".into(), true, harness::file_tools::file_delete()),
            ("file/exists".into(), "Check whether a workspace path exists".into(), false, harness::file_tools::file_exists()),
            ("shell/run".into(), "Run an allowlisted command inside the project workspace".into(), true, harness::run_tools::run_command()),
            ("git/status".into(), "Show git status of the project workspace".into(), false, harness::git_tools::git_status()),
            ("git/log".into(), "Show recent commits of the project".into(), false, harness::git_tools::git_log_tool()),
            ("git/diff".into(), "Show the working tree diff of the project".into(), false, harness::git_tools::git_diff_tool()),
            ("git/commit".into(), "Stage all changes and commit in the project".into(), true, harness::git_tools::git_commit_tool()),
            ("git/init".into(), "Initialize or clone the project repository into the workspace".into(), true, harness::git_tools::git_init_tool()),
            ("logs/read".into(), "Read the tail of a project log file".into(), false, harness::log_tools::logs_read()),
            ("logs/list".into(), "List available project log files".into(), false, harness::log_tools::logs_list()),
            ("test/run".into(), "Run the project test suite".into(), true, harness::test_tools::test_run()),
            ("test/list".into(), "Detect the test frameworks present in the project".into(), false, harness::test_tools::test_list()),
        ];

        for (name, description, requires_approval, handler) in entries {
            let tool_schema = ToolSchema::new(name.clone(), description)
                .with_parameters(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "project": {"type": "string", "description": "Vibe project id (name)"},
                        "path": {"type": "string", "description": "Path relative to the project workspace"},
                        "content": {"type": "string", "description": "File content"},
                        "command": {"type": "string", "description": "Allowlisted command to run"},
                        "args": {"type": "array", "items": {"type": "string"}, "description": "Command arguments"},
                        "message": {"type": "string", "description": "Commit message"},
                        "limit": {"type": "integer", "description": "Line/commit limit"},
                        "timeout_secs": {"type": "integer", "description": "Command timeout in seconds"}
                    },
                    "required": ["project"]
                }))
                .with_approval_if(requires_approval)
                .with_use_cases(vec![VibeUseCase::SoftwareDevelopment]);
            tools.insert(name, RegisteredTool {
                descriptor: ToolDescriptor { schema: tool_schema, category: ToolCategory::File },
                handler,
            });
        }
    }

    pub async fn register(&self, descriptor: ToolDescriptor, handler: ToolHandler) {
        let name = descriptor.schema.name.clone();
        let mut tools = self.tools.write().await;
        tools.insert(name, RegisteredTool { descriptor, handler });
    }

    pub async fn register_m5_tools(
        &self,
        skills: Arc<crate::skills::SkillStore>,
        canvases: Arc<crate::canvases::CanvasStore>,
        issues: Arc<crate::issues::IssueStore>,
    ) -> Result<usize, String> {
        let mut tools = self.tools.write().await;

        let entries: Vec<(String, ToolSchema, ToolHandler)> = crate::skills::skill_tools(skills)
            .into_iter()
            .chain(crate::browser::browser_tools())
            .chain(crate::canvases::canvas_tools(canvases))
            .chain(crate::issues::issue_tools(issues))
            .chain(crate::websearch::websearch_tools())
            .chain(crate::gitflow::gitflow_tools())
            .collect();

        for (name, schema, handler) in entries {
            tools.insert(name.clone(), RegisteredTool {
                descriptor: ToolDescriptor { schema, category: ToolCategory::Deployment },
                handler,
            });
        }
        Ok(tools.len())
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
        tools.values()
            .filter(|t| {
                t.descriptor.schema.allowed_use_cases.is_empty()
                    || t.descriptor.schema.allowed_use_cases.contains(&use_case)
            })
            .map(|t| t.descriptor.clone())
            .collect()
    }

    pub async fn validate_arguments(&self, tool_name: &str, arguments: &serde_json::Value) -> Result<(), String> {
        let tools = self.tools.read().await;
        let tool = tools.get(tool_name).ok_or_else(|| format!("Ferramenta '{tool_name}' não encontrada"))?;

        if tool.descriptor.schema.parameters.get("properties").is_none() {
            return Ok(());
        }

        if let Some(props) = tool.descriptor.schema.parameters.get("properties").and_then(|p| p.as_object()) {
            let empty_map = serde_json::Map::new();
            let args_map = arguments.as_object().unwrap_or(&empty_map);
            if let Some(required) = tool.descriptor.schema.parameters.get("required").and_then(|r| r.as_array()) {
                for req in required {
                    let key = req.as_str().unwrap_or("");
                    if !args_map.contains_key(key) {
                        return Err(format!("Parâmetro obrigatório ausente: '{key}'"));
                    }
                }
            }
            for key in args_map.keys() {
                if !props.contains_key(key) {
                    return Err(format!("Parâmetro desconhecido: '{key}'"));
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

fn deploy_app_handler() -> impl Fn(serde_json::Value, &dyn VibeState) -> ToolFuture + Send + Sync + 'static {
    move |args: serde_json::Value, _state: &dyn VibeState| {
        let args = args.clone();
        Box::pin(async move {
            let app_name = args.get("app_name")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let org = args.get("org")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let project_type = args.get("project_type")
                .and_then(|v| v.as_str())
                .unwrap_or("bot")
                .to_string();
            let environment = args.get("environment")
                .and_then(|v| v.as_str())
                .unwrap_or("development")
                .to_string();
            let framework = args.get("framework")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let custom_domain = args.get("custom_domain")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let forgejo_url = std::env::var("FORGEJO_URL")
                .ok()
                .or_else(|| std::env::var("ALM_URL").ok())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "http://localhost:4747".to_string());
            let forgejo_token = std::env::var("FORGEJO_TOKEN").ok();

            let (pt, dt) = match project_type.as_str() {
                "bot" => (botdeployment::ProjectType::Bot, botdeployment::DeployTarget::None),
                "site" => (botdeployment::ProjectType::Site, botdeployment::DeployTarget::CaddyStatic),
                app_pt if app_pt.starts_with("app-") => {
                    let fw = framework.clone()
                        .unwrap_or_else(|| app_pt.strip_prefix("app-").unwrap_or("unknown").to_string());
                    let pt = botdeployment::ProjectType::App {
                        framework: fw,
                        node_version: None,
                        build_command: None,
                        output_directory: None,
                    };
                    let dt = botdeployment::DeployTarget::from(&pt);
                    (pt, dt)
                }
                _ => {
                    return VibeToolResult {
                        success: false,
                        data: serde_json::Value::Null,
                        error: Some(format!("Unknown project type: {project_type}")),
                        latency_ms: 0,
                    };
                }
            };

            let env = match environment.as_str() {
                "staging" => botdeployment::DeploymentEnvironment::Staging,
                "production" => botdeployment::DeploymentEnvironment::Production,
                _ => botdeployment::DeploymentEnvironment::Development,
            };

            let config = botdeployment::DeploymentConfig {
                organization: if org.is_empty() { "generalbots".to_string() } else { org },
                app_name,
                project_type: pt,
                deploy_target: dt,
                environment: env,
                custom_domain,
                ci_cd_enabled: true,
            };

            let router = botdeployment::DeploymentRouter::new(forgejo_url, forgejo_token);
            let generated_app = botdeployment::GeneratedApp::new(
                config.app_name.clone(),
                format!("{} project", config.project_type),
            );

            match router.deploy(config, generated_app).await {
                Ok(result) => VibeToolResult {
                    success: true,
                    data: serde_json::json!({
                        "url": result.url,
                        "repository": result.repository,
                        "project_type": result.project_type,
                        "deploy_target": result.deploy_target,
                        "status": format!("{:?}", result.status),
                    }),
                    error: None,
                    latency_ms: 0,
                },
                Err(e) => VibeToolResult {
                    success: false,
                    data: serde_json::Value::Null,
                    error: Some(e.to_string()),
                    latency_ms: 0,
                },
            }
        })
    }
}

pub struct VibeToolExecutor {
    registry: Arc<ToolRegistry>,
}

impl VibeToolExecutor {
    pub fn new(registry: Arc<ToolRegistry>) -> Self {
        Self { registry }
    }

    pub async fn execute(
        &self,
        tool_call: &mut VibeToolCall,
        use_case: VibeUseCase,
    state: &dyn VibeState,
    ) -> Result<(), String> {
        let descriptor = self.registry.get_descriptor(&tool_call.tool_name).await
            .ok_or_else(|| format!("Ferramenta '{}' não registrada", tool_call.tool_name))?;

        if !descriptor.schema.allowed_use_cases.is_empty()
            && !descriptor.schema.allowed_use_cases.contains(&use_case) {
            return Err(format!("Ferramenta '{}' não disponível para caso de uso {}", tool_call.tool_name, use_case));
        }

        self.registry.validate_arguments(&tool_call.tool_name, &tool_call.arguments).await?;

        if descriptor.schema.requires_approval && !tool_call.approved {
            tool_call.requires_approval = true;
            return Err("Aprovação requerida antes da execução".to_string());
        }

        let start = std::time::Instant::now();
        let tools = self.registry.tools.read().await;
        let result = if let Some(registered) = tools.get(&tool_call.tool_name) {
            let handler = registered.handler.clone();
            drop(tools);
            (handler)(tool_call.arguments.clone(), state).await
        } else {
            drop(tools);
            VibeToolResult {
                success: false,
                data: serde_json::Value::Null,
                error: Some(format!("Ferramenta '{}' não encontrada", tool_call.tool_name)),
                latency_ms: 0,
            }
        };

        let latency = start.elapsed().as_millis() as u64;
        tool_call.result = Some(VibeToolResult {
            latency_ms: latency,
            ..result
        });

        Ok(())
    }

    pub fn registry(&self) -> &Arc<ToolRegistry> {
        &self.registry
    }
}
