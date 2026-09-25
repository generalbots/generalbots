//! `tool_executor::executor` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

pub type ToolHandler = Arc<dyn Fn(serde_json::Value, &dyn VibeState) -> ToolFuture + Send + Sync>;

pub(crate) struct RegisteredTool {
    pub(crate) descriptor: ToolDescriptor,
    pub(crate) handler: ToolHandler,
}

impl ToolRegistry {
    pub(crate) fn register_builtin_tools(tools: &mut HashMap<String, RegisteredTool>) {
        // #796 — wired tools: real implementations (autotask, CRM, analysis).
        for (name, schema, handler) in crate::wired_tools::autotask::autotask_tools() {
            tools.insert(
                name.clone(),
                RegisteredTool {
                    descriptor: ToolDescriptor {
                        schema,
                        category: ToolCategory::Autotask,
                    },
                    handler,
                },
            );
        }

        let deploy_tools = vec![("deploy_app", "Realiza deploy de aplicação gerada", true)];

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
            tools.insert(
                name.to_string(),
                RegisteredTool {
                    descriptor: ToolDescriptor {
                        schema,
                        category: ToolCategory::Deployment,
                    },
                    handler: Arc::new(deploy_app_handler()),
                },
            );
        }

        let publish_schema = crate::publish::publish_project_schema();
        tools.insert(
            "publish/project".to_string(),
            RegisteredTool {
                descriptor: ToolDescriptor {
                    schema: publish_schema,
                    category: ToolCategory::Deployment,
                },
                handler: crate::publish::publish_project_tool(),
            },
        );

        for (name, schema, handler) in [
            (
                "domain/bind",
                crate::domains_tool::domain_bind_schema(),
                crate::domains_tool::domain_bind_tool(),
            ),
            (
                "domain/security",
                crate::domains_tool::domain_security_schema(),
                crate::domains_tool::domain_security_tool(),
            ),
            (
                "domain/verify",
                crate::domains_tool::domain_verify_schema(),
                crate::domains_tool::domain_verify_tool(),
            ),
            (
                "domain/tls",
                crate::domains_tool::domain_tls_schema(),
                crate::domains_tool::domain_tls_tool(),
            ),
        ] {
            tools.insert(
                name.to_string(),
                RegisteredTool {
                    descriptor: ToolDescriptor {
                        schema,
                        category: ToolCategory::Deployment,
                    },
                    handler,
                },
            );
        }

        for (name, schema, handler) in crate::ops_tools::ops_tools() {
            tools.insert(
                name.to_string(),
                RegisteredTool {
                    descriptor: ToolDescriptor {
                        schema,
                        category: ToolCategory::Deployment,
                    },
                    handler,
                },
            );
        }

        // #796 — CRM tools: contacts, deals, tickets, queued email.
        for (name, schema, handler) in crate::wired_tools::crm::crm_tools() {
            tools.insert(
                name.clone(),
                RegisteredTool {
                    descriptor: ToolDescriptor {
                        schema,
                        category: ToolCategory::Crm,
                    },
                    handler,
                },
            );
        }

        // #796 — analysis tools: market data, sentiment, reports, anomalies.
        for (name, schema, handler) in crate::wired_tools::analysis::analysis_tools() {
            tools.insert(
                name.clone(),
                RegisteredTool {
                    descriptor: ToolDescriptor {
                        schema,
                        category: ToolCategory::Analysis,
                    },
                    handler,
                },
            );
        }
    }

    pub async fn register(&self, descriptor: ToolDescriptor, handler: ToolHandler) {
        let name = descriptor.schema.name.clone();
        let mut tools = self.tools.write().await;
        tools.insert(
            name,
            RegisteredTool {
                descriptor,
                handler,
            },
        );
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
            tools.insert(
                name.clone(),
                RegisteredTool {
                    descriptor: ToolDescriptor {
                        schema,
                        category: ToolCategory::Deployment,
                    },
                    handler,
                },
            );
        }
        Ok(tools.len())
    }
}

pub(crate) fn deploy_app_handler(
) -> impl Fn(serde_json::Value, &dyn VibeState) -> ToolFuture + Send + Sync + 'static {
    move |args: serde_json::Value, _state: &dyn VibeState| {
        let args = args.clone();
        Box::pin(async move {
            let app_name = args
                .get("app_name")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let org = args
                .get("org")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let project_type = args
                .get("project_type")
                .and_then(|v| v.as_str())
                .unwrap_or("bot")
                .to_string();
            let environment = args
                .get("environment")
                .and_then(|v| v.as_str())
                .unwrap_or("development")
                .to_string();
            let framework = args
                .get("framework")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let custom_domain = args
                .get("custom_domain")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let (forgejo_url, alm_token, _org) = botcoresecrets::alm_config();
            let forgejo_token = if alm_token.is_empty() {
                None
            } else {
                Some(alm_token)
            };

            let (pt, dt) = match project_type.as_str() {
                "bot" => (
                    botdeployment::ProjectType::Bot,
                    botdeployment::DeployTarget::None,
                ),
                "site" => (
                    botdeployment::ProjectType::Site,
                    botdeployment::DeployTarget::CaddyStatic,
                ),
                app_pt if app_pt.starts_with("app-") => {
                    let fw = framework.clone().unwrap_or_else(|| {
                        app_pt.strip_prefix("app-").unwrap_or("unknown").to_string()
                    });
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
                organization: if org.is_empty() {
                    "generalbots".to_string()
                } else {
                    org
                },
                app_name,
                project_type: pt,
                deploy_target: dt,
                environment: env,
                custom_domain,
                ci_cd_enabled: true,
                database_url: None,
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
    pub(crate) registry: Arc<ToolRegistry>,
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
        let descriptor = self
            .registry
            .get_descriptor(&tool_call.tool_name)
            .await
            .ok_or_else(|| format!("Ferramenta '{}' não registrada", tool_call.tool_name))?;

        if !descriptor.schema.allowed_use_cases.is_empty()
            && !descriptor.schema.allowed_use_cases.contains(&use_case)
        {
            return Err(format!(
                "Ferramenta '{}' não disponível para caso de uso {}",
                tool_call.tool_name, use_case
            ));
        }

        self.registry
            .validate_arguments(&tool_call.tool_name, &tool_call.arguments)
            .await?;

        // Internal orchestration (deploy pipeline) may carry sanctioned
        // arguments the public schema deliberately hides (e.g. the
        // `publish/project` production stamp). They are injected here — AFTER
        // validation, so a client payload containing the key is still refused
        // as unknown, and only this server-side flag can add it.
        let arguments = if tool_call.internal {
            let mut args = tool_call.arguments.clone();
            if let Some(obj) = args.as_object_mut() {
                obj.insert(
                    crate::publish::PUBLISH_PRODUCTION_STAMP.to_string(),
                    serde_json::Value::Bool(true),
                );
            }
            args
        } else {
            tool_call.arguments.clone()
        };

        // #1400 — the approval concept was removed from Vibe: tools always
        // execute. Production safety for publish/project is enforced by the
        // deploy-role RBAC stamp inside the handler itself, not here.

        let start = std::time::Instant::now();
        let tools = self.registry.tools.read().await;
        let result = if let Some(registered) = tools.get(&tool_call.tool_name) {
            let handler = registered.handler.clone();
            drop(tools);
            (handler)(arguments, state).await
        } else {
            drop(tools);
            VibeToolResult {
                success: false,
                data: serde_json::Value::Null,
                error: Some(format!(
                    "Ferramenta '{}' não encontrada",
                    tool_call.tool_name
                )),
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
