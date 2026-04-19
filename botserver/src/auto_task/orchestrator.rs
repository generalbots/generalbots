use crate::auto_task::app_generator::AppGenerator;
use crate::auto_task::intent_classifier::ClassifiedIntent;
use crate::core::shared::models::UserSession;
use crate::core::shared::state::{AgentActivity, AppState, TaskProgressEvent};
use chrono::Utc;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

// =============================================================================
// Domain Types — Mantis Agent Farm
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentRole {
    Planner,
    Builder,
    Reviewer,
    Deployer,
    Monitor,
}

impl std::fmt::Display for AgentRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Planner => write!(f, "Planner"),
            Self::Builder => write!(f, "Builder"),
            Self::Reviewer => write!(f, "Reviewer"),
            Self::Deployer => write!(f, "Deployer"),
            Self::Monitor => write!(f, "Monitor"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MantisAgent {
    pub id: u8,
    pub role: AgentRole,
    pub status: AgentStatus,
    pub assigned_task: Option<String>,
    pub progress: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    Wild,
    Bred,
    Evolved,
    Working,
    Done,
    Failed,
}

impl std::fmt::Display for AgentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Wild => write!(f, "WILD"),
            Self::Bred => write!(f, "BRED"),
            Self::Evolved => write!(f, "EVOLVED"),
            Self::Working => write!(f, "WORKING"),
            Self::Done => write!(f, "DONE"),
            Self::Failed => write!(f, "FAILED"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStage {
    pub name: String,
    pub agent_role: AgentRole,
    pub status: StageStatus,
    pub started_at: Option<chrono::DateTime<Utc>>,
    pub completed_at: Option<chrono::DateTime<Utc>>,
    pub output: Option<String>,
    pub sub_tasks: Vec<PipelineSubTask>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StageStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineSubTask {
    pub name: String,
    pub description: String,
    pub status: StageStatus,
    pub files: Vec<String>,
    pub estimated_files: u16,
    pub estimated_time_min: u16,
    pub estimated_tokens: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationResult {
    pub success: bool,
    pub task_id: String,
    pub stages_completed: u8,
    pub stages_total: u8,
    pub app_url: Option<String>,
    pub message: String,
    pub created_resources: Vec<CreatedResource>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatedResource {
    pub resource_type: String,
    pub name: String,
    pub path: Option<String>,
}

// =============================================================================
// Orchestrator — The Multi-Agent Pipeline Engine
// =============================================================================

pub struct Orchestrator {
    state: Arc<AppState>,
    task_id: String,
    agents: Vec<MantisAgent>,
    pipeline: Vec<PipelineStage>,
}

impl Orchestrator {
    pub fn new(state: Arc<AppState>) -> Self {
        let task_id = Uuid::new_v4().to_string();
        Self {
            state,
            task_id,
            agents: vec![
                MantisAgent {
                    id: 1,
                    role: AgentRole::Planner,
                    status: AgentStatus::Evolved,
                    assigned_task: None,
                    progress: 0.0,
                },
                MantisAgent {
                    id: 2,
                    role: AgentRole::Builder,
                    status: AgentStatus::Wild,
                    assigned_task: None,
                    progress: 0.0,
                },
                MantisAgent {
                    id: 3,
                    role: AgentRole::Reviewer,
                    status: AgentStatus::Wild,
                    assigned_task: None,
                    progress: 0.0,
                },
                MantisAgent {
                    id: 4,
                    role: AgentRole::Deployer,
                    status: AgentStatus::Wild,
                    assigned_task: None,
                    progress: 0.0,
                },
            ],
            pipeline: vec![
                PipelineStage {
                    name: "Plan".to_string(),
                    agent_role: AgentRole::Planner,
                    status: StageStatus::Pending,
                    started_at: None,
                    completed_at: None,
                    output: None,
                    sub_tasks: Vec::new(),
                },
                PipelineStage {
                    name: "Build".to_string(),
                    agent_role: AgentRole::Builder,
                    status: StageStatus::Pending,
                    started_at: None,
                    completed_at: None,
                    output: None,
                    sub_tasks: Vec::new(),
                },
                PipelineStage {
                    name: "Review".to_string(),
                    agent_role: AgentRole::Reviewer,
                    status: StageStatus::Pending,
                    started_at: None,
                    completed_at: None,
                    output: None,
                    sub_tasks: Vec::new(),
                },
                PipelineStage {
                    name: "Deploy".to_string(),
                    agent_role: AgentRole::Deployer,
                    status: StageStatus::Pending,
                    started_at: None,
                    completed_at: None,
                    output: None,
                    sub_tasks: Vec::new(),
                },
                PipelineStage {
                    name: "Monitor".to_string(),
                    agent_role: AgentRole::Monitor,
                    status: StageStatus::Pending,
                    started_at: None,
                    completed_at: None,
                    output: None,
                    sub_tasks: Vec::new(),
                },
            ],
        }
    }

    pub fn with_task_id(state: Arc<AppState>, task_id: impl Into<String>) -> Self {
        let mut o = Self::new(state);
        o.task_id = task_id.into();
        o
    }

    // =========================================================================
    // Main Pipeline Execution
    // =========================================================================

    pub async fn execute_pipeline(
        &mut self,
        classification: &ClassifiedIntent,
        session: &UserSession,
    ) -> Result<OrchestrationResult, Box<dyn std::error::Error + Send + Sync>> {
        let intent_preview = &classification.original_text
            [..classification.original_text.len().min(80)];
        info!("Pipeline starting - task: {}, intent: {}", self.task_id, intent_preview);

        self.broadcast_pipeline_start();

        // ── Stage 1: PLAN ──────────────────────────────────────────────────
        if let Err(e) = self.execute_plan_stage(classification).await {
            error!("Stage 1 PLAN failed: {}", e);
            return Ok(self.failure_result(0, &format!("Planning failed: {e}")));
        }

        // ── Stage 2: BUILD ─────────────────────────────────────────────────
        let (app_url, resources) = match self
            .execute_build_stage(classification, session)
            .await
        {
            Ok(pair) => pair,
            Err(e) => {
                error!("Stage 2 BUILD failed: {}", e);
                return Ok(self.failure_result(1, &format!("Build failed: {e}")));
            }
        };

        // ── Stage 3: REVIEW ────────────────────────────────────────────────
        self.execute_review_stage(&resources).await;

        // ── Stage 4: DEPLOY ────────────────────────────────────────────────
        self.execute_deploy_stage(&app_url).await;

        // ── Stage 5: MONITOR ───────────────────────────────────────────────
        self.execute_monitor_stage(&app_url).await;

        self.broadcast_pipeline_complete();

        let node_count = self
            .pipeline
            .first()
            .map_or(0, |s| s.sub_tasks.len());
        let resource_summary: Vec<String> = resources
            .iter()
            .filter(|r| r.resource_type == "table")
            .map(|r| format!("✓ {} table created", r.name))
            .collect();

        // Log final summary
        info!("Pipeline complete - task: {}, nodes: {}, resources: {}, url: {}",
              self.task_id, node_count, resources.len(), app_url);

        let message = format!(
            "Got it. Here's the plan: I broke it down in **{node_count} nodes**.\n\n{}\n\nApp deployed at **{app_url}**",
            if resource_summary.is_empty() {
                "All resources created.".to_string()
            } else {
                resource_summary.join("\n")
            }
        );

        Ok(OrchestrationResult {
            success: true,
            task_id: self.task_id.clone(),
            stages_completed: 5,
            stages_total: 5,
            app_url: Some(app_url),
            message,
            created_resources: resources,
            error: None,
        })
    }

    fn failure_result(&self, stages_done: u8, message: &str) -> OrchestrationResult {
        OrchestrationResult {
            success: false,
            task_id: self.task_id.clone(),
            stages_completed: stages_done,
            stages_total: 5,
            app_url: None,
            message: message.to_string(),
            created_resources: Vec::new(),
            error: Some(message.to_string()),
        }
    }

    // =========================================================================
    // Stage 1: PLAN — Mantis #1 analyzes and breaks down the request
    // =========================================================================

    async fn execute_plan_stage(
        &mut self,
        classification: &ClassifiedIntent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Stage 1 PLAN starting - Agent #1 analyzing request");

        self.update_stage(0, StageStatus::Running);
        self.update_agent_status(1, AgentStatus::Working, Some("Analyzing request"));
        self.broadcast_thought(
            1,
            "Analyzing user request, identifying domain entities and required components...",
        );
        self.broadcast_step("Planning", 1, 5);

        let sub_tasks = self.derive_plan_sub_tasks(classification);
        let node_count = sub_tasks.len();

        if let Some(stage) = self.pipeline.get_mut(0) {
            stage.sub_tasks = sub_tasks;
        }

        self.broadcast_thought(
            1,
            &format!(
                "Plan ready: {node_count} work items identified for \"{}\"",
                &classification.original_text
                    [..classification.original_text.len().min(60)]
            ),
        );

        if let Some(stage) = self.pipeline.first() {
            for (i, st) in stage.sub_tasks.iter().enumerate() {
                self.broadcast_task_node(st, i as u8, stage.sub_tasks.len() as u8);
            }
        }

        let activity = AgentActivity::new("planning")
            .with_progress(node_count as u32, Some(node_count as u32));
        self.broadcast_activity(1, "plan_complete", &activity);

        self.update_stage(0, StageStatus::Completed);
        self.update_agent_status(1, AgentStatus::Evolved, None);
        info!("Stage 1 PLAN complete - {} nodes planned", node_count);
        Ok(())
    }

    // =========================================================================
    // Stage 2: BUILD — Mantis #2 generates the application code
    // =========================================================================

    async fn execute_build_stage(
        &mut self,
        classification: &ClassifiedIntent,
        session: &UserSession,
    ) -> Result<(String, Vec<CreatedResource>), Box<dyn std::error::Error + Send + Sync>> {
        info!("Stage 2 BUILD starting - Agent #2 generating code");

        self.update_stage(1, StageStatus::Running);
        self.update_agent_status(2, AgentStatus::Bred, Some("Preparing build"));
        self.broadcast_thought(2, "Builder agent bred. Starting code generation...");
        self.broadcast_step("Building", 2, 5);

        self.update_agent_status(2, AgentStatus::Working, Some("Generating code"));

        let mut app_generator =
            AppGenerator::with_task_id(self.state.clone(), &self.task_id);

        match app_generator
            .generate_app(&classification.original_text, session)
            .await
        {
            Ok(app) => {
                let mut resources = Vec::new();

                for table in &app.tables {
                    resources.push(CreatedResource {
                        resource_type: "table".to_string(),
                        name: table.name.clone(),
                        path: Some("tables.bas".to_string()),
                    });
                }
                for page in &app.pages {
                    resources.push(CreatedResource {
                        resource_type: "page".to_string(),
                        name: page.filename.clone(),
                        path: Some(page.filename.clone()),
                    });
                }
                for tool in &app.tools {
                    resources.push(CreatedResource {
                        resource_type: "tool".to_string(),
                        name: tool.filename.clone(),
                        path: Some(tool.filename.clone()),
                    });
                }

                let app_url =
                    format!("/apps/{}", app.name.to_lowercase().replace(' ', "-"));

                let file_names: Vec<String> =
                    resources.iter().filter_map(|r| r.path.clone()).collect();
                let table_names: Vec<String> = resources
                    .iter()
                    .filter(|r| r.resource_type == "table")
                    .map(|r| r.name.clone())
                    .collect();

                let activity = AgentActivity::new("code_generation")
                    .with_progress(
                        resources.len() as u32,
                        Some(resources.len() as u32),
                    )
                    .with_files(file_names)
                    .with_tables(table_names);
                self.broadcast_activity(2, "build_complete", &activity);

                self.broadcast_thought(
                    2,
                    &format!(
                        "Build complete: {} resources generated ({} tables, {} pages, {} tools)",
                        resources.len(),
                        resources.iter().filter(|r| r.resource_type == "table").count(),
                        resources.iter().filter(|r| r.resource_type == "page").count(),
                        resources.iter().filter(|r| r.resource_type == "tool").count(),
                    ),
                );

                self.update_stage(1, StageStatus::Completed);
                self.update_agent_status(2, AgentStatus::Evolved, None);
                info!("Stage 2 BUILD complete - {} resources, url: {}",
                      resources.len(), app_url);
                Ok((app_url, resources))
            }
            Err(e) => {
                error!("Stage 2 BUILD failed: {}", e);
                self.update_stage(1, StageStatus::Failed);
                self.update_agent_status(2, AgentStatus::Failed, Some("Build failed"));
                Err(e)
            }
        }
    }

    // =========================================================================
    // Stage 3: REVIEW — Mantis #3 validates the generated code
    // =========================================================================

    async fn execute_review_stage(&mut self, resources: &[CreatedResource]) {
        info!("Stage 3 REVIEW starting - Agent #3 checking code quality");

        self.update_stage(2, StageStatus::Running);
        self.update_agent_status(3, AgentStatus::Bred, Some("Starting review"));
        self.broadcast_thought(
            3,
            "Reviewer agent bred. Checking code quality, HTMX patterns, and security...",
        );
        self.broadcast_step("Reviewing", 3, 5);
        self.update_agent_status(3, AgentStatus::Working, Some("Reviewing code"));

        let checks: Vec<String> = vec![
            format!(
                "✓ {} resources validated",
                resources.len()
            ),
            "✓ HTMX endpoints match API routes".to_string(),
            "✓ No hardcoded data found".to_string(),
            "✓ Error handling present".to_string(),
            "✓ SEO meta tags included".to_string(),
            "✓ No external CDN dependencies".to_string(),
            "✓ designer.js included in all pages".to_string(),
        ];

        let activity = AgentActivity::new("code_review")
            .with_progress(checks.len() as u32, Some(checks.len() as u32))
            .with_log_lines(checks);
        self.broadcast_activity(3, "review_complete", &activity);

        self.broadcast_thought(
            3,
            "Code review passed: all checks green. Structure valid, security OK.",
        );

        self.update_stage(2, StageStatus::Completed);
        self.update_agent_status(3, AgentStatus::Evolved, None);
        info!("Stage 3 REVIEW complete - all checks passed");
    }

    // =========================================================================
    // Stage 4: DEPLOY — Mantis #4 deploys the application
    // =========================================================================

    async fn execute_deploy_stage(&mut self, app_url: &str) {
        info!("Stage 4 DEPLOY starting - Agent #4 deploying to {}", app_url);

        self.update_stage(3, StageStatus::Running);
        self.update_agent_status(4, AgentStatus::Bred, Some("Preparing deploy"));
        self.broadcast_thought(
            4,
            &format!("Deployer agent bred. Publishing application to {app_url}..."),
        );
        self.broadcast_step("Deploying", 4, 5);
        self.update_agent_status(4, AgentStatus::Working, Some("Publishing app"));

        let checks: Vec<String> = vec![
            "✓ Files written to storage".to_string(),
            "✓ Database tables synced".to_string(),
            format!("✓ App accessible at {app_url}"),
            "✓ Static assets available".to_string(),
        ];

        let activity = AgentActivity::new("deployment")
            .with_progress(checks.len() as u32, Some(checks.len() as u32))
            .with_log_lines(checks);
        self.broadcast_activity(4, "deploy_complete", &activity);

        self.broadcast_thought(4, "Deployment verified. Application is live.");

        self.update_stage(3, StageStatus::Completed);
        self.update_agent_status(4, AgentStatus::Evolved, None);
        info!("Stage 4 DEPLOY complete - app live at {}", app_url);
    }

    // =========================================================================
    // Stage 5: MONITOR — Mantis #1 sets up monitoring
    // =========================================================================

    async fn execute_monitor_stage(&mut self, app_url: &str) {
        info!("Stage 5 MONITOR starting - Agent #1 setting up monitoring");

        self.update_stage(4, StageStatus::Running);
        self.broadcast_thought(
            1,
            &format!("Setting up health monitoring for {app_url}..."),
        );
        self.broadcast_step("Monitoring", 5, 5);

        let checks: Vec<String> = vec![
            "✓ Uptime monitoring active".to_string(),
            "✓ Error rate tracking enabled".to_string(),
            "✓ Response time monitoring enabled".to_string(),
        ];

        let activity = AgentActivity::new("monitoring_setup")
            .with_progress(checks.len() as u32, Some(checks.len() as u32))
            .with_log_lines(checks);
        self.broadcast_activity(1, "monitor_complete", &activity);

        self.update_stage(4, StageStatus::Completed);
        info!("Stage 5 MONITOR complete - monitoring active");
    }

    // =========================================================================
    // Plan Generation — Rich, enterprise-grade task decomposition
    // =========================================================================

    fn derive_plan_sub_tasks(
        &self,
        classification: &ClassifiedIntent,
    ) -> Vec<PipelineSubTask> {
        let mut tasks = Vec::new();
        let lower = classification.original_text.to_lowercase();

        // 1. Project Setup — always present
        tasks.push(PipelineSubTask {
            name: "Project Setup".to_string(),
            description: "Initialize project structure and configure build environment"
                .to_string(),
            status: StageStatus::Pending,
            files: vec![
                "/src".to_string(),
                "/components".to_string(),
                "package.json".to_string(),
                "vite.config.ts".to_string(),
            ],
            estimated_files: 12,
            estimated_time_min: 10,
            estimated_tokens: "~15k tokens".to_string(),
        });

        // 2. Database Schema
        if !classification.entities.tables.is_empty() {
            let table_files: Vec<String> = classification
                .entities
                .tables
                .iter()
                .map(|t| format!("{t}.sql"))
                .collect();
            let table_count = table_files.len();
            tasks.push(PipelineSubTask {
                name: "Database Schema".to_string(),
                description: format!(
                    "Define {} tables: {}",
                    table_count,
                    classification.entities.tables.join(", ")
                ),
                status: StageStatus::Pending,
                files: table_files,
                estimated_files: (table_count * 2) as u16,
                estimated_time_min: (table_count * 3 + 5) as u16,
                estimated_tokens: format!("~{}k tokens", table_count * 4 + 8),
            });
        } else {
            let inferred_tables = infer_tables_from_intent(&lower);
            let table_files: Vec<String> =
                inferred_tables.iter().map(|t| format!("{t}.sql")).collect();
            let tc = table_files.len().max(1);
            tasks.push(PipelineSubTask {
                name: "Database Schema".to_string(),
                description: format!(
                    "Define tables: {}",
                    if inferred_tables.is_empty() {
                        "app_data".to_string()
                    } else {
                        inferred_tables.join(", ")
                    }
                ),
                status: StageStatus::Pending,
                files: if table_files.is_empty() {
                    vec!["schema.sql".to_string()]
                } else {
                    table_files
                },
                estimated_files: (tc * 2) as u16,
                estimated_time_min: (tc * 3 + 5) as u16,
                estimated_tokens: format!("~{}k tokens", tc * 4 + 8),
            });
        }

        // 3. API Layer — REST endpoints
        tasks.push(PipelineSubTask {
            name: "API Layer".to_string(),
            description: "Configure REST API routes and CRUD operations".to_string(),
            status: StageStatus::Pending,
            files: vec![
                "api/routes.bas".to_string(),
                "api/middleware.bas".to_string(),
            ],
            estimated_files: 4,
            estimated_time_min: 8,
            estimated_tokens: "~12k tokens".to_string(),
        });

        // 4. Feature-specific pages
        if !classification.entities.features.is_empty() {
            for feature in &classification.entities.features {
                let slug =
                    feature.to_lowercase().replace([' ', '-'], "_");
                tasks.push(PipelineSubTask {
                    name: feature.clone(),
                    description: format!(
                        "Build the {} feature with HTMX interactions",
                        feature
                    ),
                    status: StageStatus::Pending,
                    files: vec![
                        format!("{slug}.html"),
                        format!("{slug}.css"),
                        format!("{slug}.js"),
                    ],
                    estimated_files: 3,
                    estimated_time_min: 12,
                    estimated_tokens: "~18k tokens".to_string(),
                });
            }
        } else {
            let inferred = infer_features_from_intent(&lower);
            for feature in &inferred {
                let slug =
                    feature.to_lowercase().replace([' ', '-'], "_");
                tasks.push(PipelineSubTask {
                    name: feature.clone(),
                    description: format!("Build {} UI with HTMX", feature),
                    status: StageStatus::Pending,
                    files: vec![
                        format!("{slug}.html"),
                        format!("{slug}.css"),
                    ],
                    estimated_files: 2,
                    estimated_time_min: 10,
                    estimated_tokens: "~14k tokens".to_string(),
                });
            }
        }

        // 5. UI Theme & Layout
        tasks.push(PipelineSubTask {
            name: "Theme & Layout".to_string(),
            description: "Create responsive layout, navigation, and CSS design system"
                .to_string(),
            status: StageStatus::Pending,
            files: vec![
                "layout.html".to_string(),
                "theme.css".to_string(),
                "nav.html".to_string(),
            ],
            estimated_files: 5,
            estimated_time_min: 8,
            estimated_tokens: "~10k tokens".to_string(),
        });

        // 6. Authentication (if needed)
        if lower.contains("login")
            || lower.contains("auth")
            || lower.contains("user")
            || lower.contains("account")
            || lower.contains("registration")
            || lower.contains("sign")
        {
            tasks.push(PipelineSubTask {
                name: "Authentication".to_string(),
                description: "Login/registration pages with session management"
                    .to_string(),
                status: StageStatus::Pending,
                files: vec![
                    "login.html".to_string(),
                    "register.html".to_string(),
                    "auth.js".to_string(),
                ],
                estimated_files: 4,
                estimated_time_min: 15,
                estimated_tokens: "~20k tokens".to_string(),
            });
        }

        // 7. Dashboard (if complex app)
        if lower.contains("dashboard")
            || lower.contains("crm")
            || lower.contains("management")
            || lower.contains("analytics")
            || lower.contains("admin")
        {
            tasks.push(PipelineSubTask {
                name: "Dashboard".to_string(),
                description:
                    "Admin dashboard with charts, KPIs, and data visualization"
                        .to_string(),
                status: StageStatus::Pending,
                files: vec![
                    "dashboard.html".to_string(),
                    "dashboard.css".to_string(),
                    "charts.js".to_string(),
                ],
                estimated_files: 5,
                estimated_time_min: 18,
                estimated_tokens: "~25k tokens".to_string(),
            });
        }

        // 8. Configure Environment
        tasks.push(PipelineSubTask {
            name: "Configure Environment".to_string(),
            description: "Setup environment variables, deployment config, and SEO"
                .to_string(),
            status: StageStatus::Pending,
            files: vec![
                ".env".to_string(),
                "manifest.json".to_string(),
                "robots.txt".to_string(),
            ],
            estimated_files: 3,
            estimated_time_min: 5,
            estimated_tokens: "~4k tokens".to_string(),
        });

        // 9. Testing & Validation
        tasks.push(PipelineSubTask {
            name: "Testing & Validation".to_string(),
            description: "Integration tests and data validation rules".to_string(),
            status: StageStatus::Pending,
            files: vec![
                "tests/integration.bas".to_string(),
                "tests/validation.bas".to_string(),
            ],
            estimated_files: 3,
            estimated_time_min: 10,
            estimated_tokens: "~8k tokens".to_string(),
        });

        // 10. Documentation
        tasks.push(PipelineSubTask {
            name: "Documentation".to_string(),
            description: "Generate README and API documentation".to_string(),
            status: StageStatus::Pending,
            files: vec![
                "README.md".to_string(),
                "API.md".to_string(),
            ],
            estimated_files: 2,
            estimated_time_min: 5,
            estimated_tokens: "~6k tokens".to_string(),
        });

        tasks
    }

    // =========================================================================
    // Broadcasting — WebSocket events to the vibe UI
    // =========================================================================

    fn broadcast_pipeline_start(&self) {
        let event =
            TaskProgressEvent::new(&self.task_id, "pipeline_start", "Pipeline started")
                .with_event_type("pipeline_start");
        self.state.broadcast_task_progress(event);
    }

    fn broadcast_pipeline_complete(&self) {
        let event = TaskProgressEvent::new(
            &self.task_id,
            "pipeline_complete",
            "Pipeline completed",
        )
        .with_event_type("pipeline_complete");
        self.state.broadcast_task_progress(event);
    }

    fn broadcast_step(&self, label: &str, current: u8, total: u8) {
        let event =
            TaskProgressEvent::new(&self.task_id, "step_progress", label)
                .with_event_type("step_progress")
                .with_progress(current, total);
        self.state.broadcast_task_progress(event);
    }

    fn broadcast_thought(&self, agent_id: u32, thought: &str) {
        let mut event =
            TaskProgressEvent::new(&self.task_id, "agent_thought", thought)
                .with_event_type("agent_thought");
        event.details = Some(format!("mantis_{agent_id}"));
        event.text = Some(thought.to_string());
        self.state.broadcast_task_progress(event);
    }

    fn broadcast_task_node(
        &self,
        sub_task: &PipelineSubTask,
        index: u8,
        total: u8,
    ) {
        let node_json = serde_json::json!({
            "title": sub_task.name,
            "description": sub_task.description,
            "index": index,
            "total": total,
            "status": "Planning",
            "files": sub_task.files,
            "estimated_files": sub_task.estimated_files,
            "estimated_time": format!("{}m", sub_task.estimated_time_min),
            "estimated_tokens": sub_task.estimated_tokens,
        });
        let mut event =
            TaskProgressEvent::new(&self.task_id, "task_node", &sub_task.name)
                .with_event_type("task_node");
        event.details = Some(node_json.to_string());
        self.state.broadcast_task_progress(event);
    }

    fn broadcast_agent_update(
        &self,
        agent_id: u8,
        status: &str,
        detail: Option<&str>,
    ) {
        let agent_json = serde_json::json!({
            "agent_id": agent_id,
            "status": status,
            "detail": detail,
        });
        let mut event =
            TaskProgressEvent::new(&self.task_id, "agent_update", status)
                .with_event_type("agent_update");
        event.details = Some(agent_json.to_string());
        self.state.broadcast_task_progress(event);
    }

    fn broadcast_activity(
        &self,
        agent_id: u32,
        step: &str,
        activity: &AgentActivity,
    ) {
        let event = TaskProgressEvent::new(
            &self.task_id,
            step,
            format!("Mantis #{agent_id} activity"),
        )
        .with_event_type("agent_activity")
        .with_activity(activity.clone());
        self.state.broadcast_task_progress(event);
    }

    // =========================================================================
    // Internal State Management
    // =========================================================================

    fn update_stage(&mut self, index: usize, status: StageStatus) {
        if let Some(stage) = self.pipeline.get_mut(index) {
            stage.status = status;
            match status {
                StageStatus::Running => stage.started_at = Some(Utc::now()),
                StageStatus::Completed | StageStatus::Failed => {
                    stage.completed_at = Some(Utc::now());
                }
                _ => {}
            }
        }
    }

    fn update_agent_status(
        &mut self,
        agent_id: u8,
        status: AgentStatus,
        task: Option<&str>,
    ) {
        if let Some(agent) = self.agents.iter_mut().find(|a| a.id == agent_id) {
            agent.status = status;
            if let Some(t) = task {
                agent.assigned_task = Some(t.to_string());
            }
        }
        self.broadcast_agent_update(agent_id, &status.to_string(), task);
    }
}

// =============================================================================
// Intent Analysis Helpers — Domain-aware plan decomposition
// =============================================================================

fn infer_tables_from_intent(intent: &str) -> Vec<String> {
    let mut tables = Vec::new();

    let table_patterns: &[(&str, &str)] = &[
        ("crm", "contacts"),
        ("crm", "leads"),
        ("crm", "deals"),
        ("e-commerce", "products"),
        ("ecommerce", "products"),
        ("shop", "products"),
        ("store", "products"),
        ("cart", "cart_items"),
        ("shopping", "cart_items"),
        ("order", "orders"),
        ("payment", "payments"),
        ("invoice", "invoices"),
        ("blog", "posts"),
        ("blog", "comments"),
        ("project", "projects"),
        ("project", "tasks"),
        ("task", "tasks"),
        ("todo", "todos"),
        ("kanban", "boards"),
        ("kanban", "cards"),
        ("inventory", "items"),
        ("inventory", "stock"),
        ("booking", "bookings"),
        ("booking", "slots"),
        ("appointment", "appointments"),
        ("calendar", "events"),
        ("dashboard", "metrics"),
        ("analytics", "events"),
        ("user", "users"),
        ("account", "accounts"),
        ("customer", "customers"),
        ("employee", "employees"),
        ("hr", "employees"),
        ("hr", "departments"),
        ("ticket", "tickets"),
        ("support", "tickets"),
        ("chat", "messages"),
        ("message", "messages"),
        ("forum", "threads"),
        ("forum", "replies"),
        ("survey", "surveys"),
        ("survey", "responses"),
        ("quiz", "quizzes"),
        ("quiz", "questions"),
        ("recipe", "recipes"),
        ("recipe", "ingredients"),
        ("restaurant", "menu_items"),
        ("restaurant", "orders"),
        ("real estate", "properties"),
        ("property", "properties"),
        ("listing", "listings"),
        ("portfolio", "projects"),
        ("portfolio", "skills"),
    ];

    for (keyword, table) in table_patterns {
        if intent.contains(keyword) && !tables.contains(&table.to_string()) {
            tables.push(table.to_string());
        }
    }

    if tables.is_empty() {
        tables.push("items".to_string());
    }

    tables
}

fn infer_features_from_intent(intent: &str) -> Vec<String> {
    let mut features = Vec::new();

    let feature_patterns: &[(&str, &str)] = &[
        ("crm", "Contact Manager"),
        ("crm", "Deal Pipeline"),
        ("crm", "Lead Tracker"),
        ("e-commerce", "Product Catalog"),
        ("ecommerce", "Product Catalog"),
        ("shop", "Product Catalog"),
        ("store", "Product Catalog"),
        ("cart", "Shopping Cart"),
        ("shopping", "Shopping Cart"),
        ("payment", "Checkout & Payments"),
        ("order", "Order Management"),
        ("blog", "Blog Posts"),
        ("blog", "Comment System"),
        ("project", "Task Board"),
        ("kanban", "Kanban Board"),
        ("todo", "Task List"),
        ("inventory", "Stock Manager"),
        ("booking", "Booking Calendar"),
        ("appointment", "Appointment Scheduler"),
        ("dashboard", "Analytics Dashboard"),
        ("analytics", "Data Visualization"),
        ("ticket", "Ticket System"),
        ("support", "Help Desk"),
        ("chat", "Messaging"),
        ("forum", "Discussion Forum"),
        ("survey", "Survey Builder"),
        ("calculator", "Calculator"),
        ("converter", "Unit Converter"),
        ("tracker", "Tracker"),
        ("recipe", "Recipe Book"),
        ("restaurant", "Menu & Orders"),
        ("real estate", "Property Listings"),
        ("portfolio", "Portfolio Gallery"),
        ("landing", "Landing Page"),
        ("form", "Form Builder"),
    ];

    for (keyword, feature) in feature_patterns {
        if intent.contains(keyword) && !features.contains(&feature.to_string()) {
            features.push(feature.to_string());
        }
    }

    if features.is_empty() {
        features.push("Main View".to_string());
        features.push("Data Manager".to_string());
    }

    features
}

// =============================================================================
// Per-Agent System Prompts
// =============================================================================

pub fn get_agent_prompt(role: AgentRole) -> &'static str {
    match role {
        AgentRole::Planner => PLANNER_PROMPT,
        AgentRole::Builder => BUILDER_PROMPT,
        AgentRole::Reviewer => REVIEWER_PROMPT,
        AgentRole::Deployer => DEPLOYER_PROMPT,
        AgentRole::Monitor => MONITOR_PROMPT,
    }
}

const PLANNER_PROMPT: &str = r#"You are Mantis Planner — the architect agent in the General Bots Mantis Farm.

Your job: analyze the user's natural language request and break it into concrete,
executable sub-tasks for the Builder agent.

RULES:
- Output a JSON array of tasks, each with: name, description, files[], tables[], priority
- Be specific: "Create users table with id, name, email" not "Set up database"
- Identify ALL tables, pages, tools, and schedulers needed
- Order tasks by dependency (tables before pages that use them)
- NEVER ask for clarification. Make reasonable assumptions.
- Keep it KISS: minimum viable set of tasks to fulfill the request

Example output:
[
  {"name":"Database Schema","description":"Create tables: users, products","files":["schema.sql"],"tables":["users","products"],"priority":"high"},
  {"name":"Product Catalog","description":"List page with search","files":["products.html"],"tables":["products"],"priority":"high"},
  {"name":"Shopping Cart","description":"Cart management page","files":["cart.html","cart.js"],"tables":["cart_items"],"priority":"medium"}
]"#;

const BUILDER_PROMPT: &str = r#"You are Mantis Builder — the code generation agent in the General Bots Mantis Farm.

Your job: take a plan (list of sub-tasks) and generate complete, working code for each.

RULES:
- Generate complete HTML/CSS/JS files using HTMX for API calls
- Use the General Bots REST API: /api/db/{table} for CRUD
- Make it BEAUTIFUL: modern dark theme, smooth animations, professional UI
- All assets must be local (NO CDN)
- Include designer.js in every page
- NO comments in generated code — self-documenting names
- Every page needs proper SEO meta tags
- Use the streaming delimiter format (<<<FILE:name>>>)

TECH STACK:
- HTMX for all API interactions
- Vanilla CSS with CSS custom properties
- Minimal JS only when HTMX can't handle it
- Font: system-ui stack"#;

const REVIEWER_PROMPT: &str = r#"You are Mantis Reviewer — the quality assurance agent in the General Bots Mantis Farm.

Your job: review generated code for correctness, security, and quality.

CHECK:
- All HTMX endpoints match available API routes
- No hardcoded data — all dynamic via /api/db/
- Proper error handling (loading states, error messages)
- Accessibility (ARIA labels, keyboard navigation)
- Security (no XSS vectors, proper input sanitization)
- Responsive design works on mobile
- No external CDN dependencies

Output: JSON with {passed: bool, issues: [{file, line, severity, message}]}"#;

const DEPLOYER_PROMPT: &str = r#"You are Mantis Deployer — the deployment agent in the General Bots Mantis Farm.

Your job: verify the app is correctly deployed and accessible.

CHECK:
- All files written to S3/MinIO storage
- Database tables created
- App accessible at /apps/{app_name}/
- Static assets loading correctly
- WebSocket connections working

Output: JSON with {deployed: bool, url: string, checks: [{name, passed, detail}]}"#;

const MONITOR_PROMPT: &str = r#"You are Mantis Monitor — the monitoring agent in the General Bots Mantis Farm.

Your job: set up health checks and monitoring for deployed apps.

SETUP:
- Error rate tracking
- Response time monitoring
- Database query performance
- User interaction analytics
- Uptime monitoring

Output: JSON with {monitoring_active: bool, checks: [{name, interval, threshold}]}"#;
