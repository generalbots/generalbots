use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use chrono::{DateTime, Utc};
use log::{info, trace, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Write;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledIntent {
    pub id: String,
    pub original_intent: String,
    pub entities: IntentEntities,
    pub plan: ExecutionPlan,
    pub basic_program: String,
    pub confidence: f64,
    pub alternatives: Vec<AlternativeInterpretation>,
    pub risk_assessment: RiskAssessment,
    pub resource_estimate: ResourceEstimate,
    pub compiled_at: DateTime<Utc>,
    pub session_id: String,
    pub bot_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IntentEntities {
    pub action: String,
    pub target: String,
    pub domain: Option<String>,
    pub client: Option<String>,
    pub features: Vec<String>,
    pub constraints: Vec<Constraint>,
    pub technologies: Vec<String>,
    pub data_sources: Vec<String>,
    pub integrations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    pub constraint_type: ConstraintType,
    pub value: String,
    pub is_hard: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConstraintType {
    Budget,
    Timeline,
    Technology,
    Security,
    Compliance,
    Performance,
    Scalability,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<PlanStep>,
    pub dependencies: HashMap<String, Vec<String>>,
    pub estimated_duration_minutes: i32,
    pub requires_approval: bool,
    pub approval_levels: Vec<ApprovalLevel>,
    pub rollback_plan: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    pub id: String,
    pub order: i32,
    pub name: String,
    pub description: String,
    pub keywords: Vec<String>,
    pub basic_code: String,
    pub priority: StepPriority,
    pub risk_level: RiskLevel,
    pub estimated_minutes: i32,
    pub requires_approval: bool,
    pub can_rollback: bool,
    pub dependencies: Vec<String>,
    pub outputs: Vec<String>,
    pub mcp_servers: Vec<String>,
    pub api_calls: Vec<ApiCallSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StepPriority {
    Critical,
    High,
    Medium,
    Low,
    Optional,
}

impl Default for StepPriority {
    fn default() -> Self {
        Self::Medium
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    None,
    Low,
    Medium,
    High,
    Critical,
}

impl Default for RiskLevel {
    fn default() -> Self {
        Self::Low
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiCallSpec {
    pub name: String,
    pub method: String,
    pub url_template: String,
    pub headers: HashMap<String, String>,
    pub body_template: Option<String>,
    pub auth_type: AuthType,
    pub retry_config: RetryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthType {
    None,
    ApiKey {
        header: String,
        key_ref: String,
    },
    Bearer {
        token_ref: String,
    },
    Basic {
        user_ref: String,
        pass_ref: String,
    },
    OAuth2 {
        client_id_ref: String,
        client_secret_ref: String,
    },
}

impl Default for AuthType {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_retries: i32,
    pub backoff_ms: i32,
    pub retry_on_status: Vec<i32>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            backoff_ms: 1000,
            retry_on_status: vec![429, 500, 502, 503, 504],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalLevel {
    pub level: i32,
    pub approver: String,
    pub reason: String,
    pub timeout_minutes: i32,
    pub default_action: DefaultApprovalAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DefaultApprovalAction {
    Approve,
    Reject,
    Escalate,
    Pause,
}

impl Default for DefaultApprovalAction {
    fn default() -> Self {
        Self::Pause
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeInterpretation {
    pub id: String,
    pub description: String,
    pub confidence: f64,
    pub plan_summary: String,
    pub pros: Vec<String>,
    pub cons: Vec<String>,
    pub estimated_cost: Option<f64>,
    pub estimated_time_hours: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub overall_risk: RiskLevel,
    pub risks: Vec<IdentifiedRisk>,
    pub mitigations: Vec<RiskMitigation>,
    pub requires_human_review: bool,
    pub review_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentifiedRisk {
    pub id: String,
    pub category: RiskCategory,
    pub description: String,
    pub probability: f64,
    pub impact: RiskLevel,
    pub affected_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskCategory {
    DataLoss,
    SecurityBreach,
    CostOverrun,
    TimelineSlip,
    IntegrationFailure,
    ComplianceViolation,
    PerformanceIssue,
    DependencyFailure,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMitigation {
    pub risk_id: String,
    pub strategy: String,
    pub basic_code: Option<String>,
    pub fallback_plan: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceEstimate {
    pub compute_hours: f64,
    pub storage_gb: f64,
    pub api_calls: i32,
    pub llm_tokens: i32,
    pub estimated_cost_usd: f64,
    pub human_hours: f64,
    pub mcp_servers_needed: Vec<String>,
    pub external_services: Vec<String>,
}

impl Default for ResourceEstimate {
    fn default() -> Self {
        Self {
            compute_hours: 0.0,
            storage_gb: 0.0,
            api_calls: 0,
            llm_tokens: 0,
            estimated_cost_usd: 0.0,
            human_hours: 0.0,
            mcp_servers_needed: Vec::new(),
            external_services: Vec::new(),
        }
    }
}

pub struct IntentCompiler {
    _state: Arc<AppState>,
    config: IntentCompilerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentCompilerConfig {
    pub enabled: bool,
    pub model: String,
    pub temperature: f64,
    pub max_tokens: i32,
    pub auto_execute_low_risk: bool,
    pub require_approval_above: RiskLevel,
    pub simulate_before_execute: bool,
    pub max_plan_steps: i32,
    pub available_keywords: Vec<String>,
    pub available_mcp_servers: Vec<String>,
}

impl Default for IntentCompilerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            model: "gpt-4".to_string(),
            temperature: 0.3,
            max_tokens: 4000,
            auto_execute_low_risk: false,
            require_approval_above: RiskLevel::Medium,
            simulate_before_execute: true,
            max_plan_steps: 50,
            available_keywords: get_all_keywords(),
            available_mcp_servers: Vec::new(),
        }
    }
}

impl std::fmt::Debug for IntentCompiler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IntentCompiler")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl IntentCompiler {
    pub fn new(state: Arc<AppState>) -> Self {
        Self {
            _state: state,
            config: IntentCompilerConfig::default(),
        }
    }

    pub fn with_config(state: Arc<AppState>, config: IntentCompilerConfig) -> Self {
        Self {
            _state: state,
            config,
        }
    }

    pub async fn compile(
        &self,
        intent: &str,
        session: &UserSession,
    ) -> Result<CompiledIntent, Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "Compiling intent for session {}: {}",
            session.id,
            &intent[..intent.len().min(100)]
        );

        let entities = self.extract_entities(intent).await?;
        trace!("Extracted entities: {entities:?}");

        let plan = self.generate_plan(intent, &entities).await?;
        trace!("Generated plan with {} steps", plan.steps.len());

        let basic_program = Self::generate_basic_program(&plan, &entities);
        trace!(
            "Generated BASIC program: {} lines",
            basic_program.lines().count()
        );

        let risk_assessment = Self::assess_risks(&plan);

        let resource_estimate = Self::estimate_resources(&plan);

        let (confidence, alternatives) = Self::check_ambiguity();

        let compiled = CompiledIntent {
            id: Uuid::new_v4().to_string(),
            original_intent: intent.to_string(),
            entities,
            plan,
            basic_program,
            confidence,
            alternatives,
            risk_assessment,
            resource_estimate,
            compiled_at: Utc::now(),
            session_id: session.id.to_string(),
            bot_id: session.bot_id.to_string(),
        };

        Self::store_compiled_intent(&compiled);

        Ok(compiled)
    }

    async fn extract_entities(
        &self,
        intent: &str,
    ) -> Result<IntentEntities, Box<dyn std::error::Error + Send + Sync>> {
        let prompt = format!(
            r#"Analyze this user request and extract structured information.

User Request: "{intent}"

Extract the following as JSON:
{{
    "action": "primary action (create/update/delete/analyze/report/integrate/automate)",
    "target": "what to create/modify (CRM, website, report, API, etc.)",
    "domain": "industry/domain if mentioned (financial, healthcare, retail, etc.) or null",
    "client": "client/company name if mentioned or null",
    "features": ["list of specific features requested"],
    "constraints": [
        {{"type": "budget|timeline|technology|security|compliance|performance", "value": "constraint value", "is_hard": true/false}}
    ],
    "technologies": ["specific technologies/tools mentioned"],
    "data_sources": ["data sources mentioned"],
    "integrations": ["external systems to integrate with"]
}}

Respond ONLY with valid JSON, no explanation."#
        );

        let response = self.call_llm(&prompt).await?;
        let entities: IntentEntities = serde_json::from_str(&response).unwrap_or_else(|e| {
            warn!("Failed to parse entity extraction response: {e}");
            IntentEntities {
                action: "create".to_string(),
                target: intent.to_string(),
                ..Default::default()
            }
        });

        Ok(entities)
    }

    async fn generate_plan(
        &self,
        intent: &str,
        entities: &IntentEntities,
    ) -> Result<ExecutionPlan, Box<dyn std::error::Error + Send + Sync>> {
        let keywords_list = self.config.available_keywords.join(", ");
        let mcp_servers_list = self.config.available_mcp_servers.join(", ");

        let prompt = format!(
            r#"Generate an execution plan for this task.

Original Request: "{intent}"

Extracted Information:
- Action: {}
- Target: {}
- Domain: {}
- Client: {}
- Features: {:?}
- Technologies: {:?}
- Integrations: {:?}

Available BASIC Keywords: {keywords_list}
Available MCP Servers: {mcp_servers_list}

Generate a detailed execution plan as JSON:
{{
    "name": "short plan name",
    "description": "brief description",
    "steps": [
        {{
            "id": "step-1",
            "order": 1,
            "name": "Step name",
            "description": "What this step does",
            "keywords": ["BASIC keywords this step uses"],
            "priority": "CRITICAL|HIGH|MEDIUM|LOW|OPTIONAL",
            "risk_level": "NONE|LOW|MEDIUM|HIGH|CRITICAL",
            "estimated_minutes": 5,
            "requires_approval": false,
            "can_rollback": true,
            "dependencies": [],
            "outputs": ["variables/resources produced"],
            "mcp_servers": ["MCP servers needed"],
            "api_calls": []
        }}
    ],
    "requires_approval": true/false,
    "estimated_duration_minutes": 60,
    "rollback_plan": "how to undo if needed"
}}

Maximum {} steps. Focus on practical, executable steps.
Respond ONLY with valid JSON."#,
            entities.action,
            entities.target,
            entities.domain.as_deref().unwrap_or("general"),
            entities.client.as_deref().unwrap_or("none"),
            entities.features,
            entities.technologies,
            entities.integrations,
            self.config.max_plan_steps
        );

        let response = self.call_llm(&prompt).await?;

        #[derive(Deserialize)]
        struct PlanResponse {
            name: String,
            description: String,
            steps: Vec<PlanStepResponse>,
            requires_approval: Option<bool>,
            estimated_duration_minutes: Option<i32>,
            rollback_plan: Option<String>,
        }

        #[derive(Deserialize)]
        struct PlanStepResponse {
            id: String,
            order: i32,
            name: String,
            description: String,
            keywords: Vec<String>,
            priority: Option<String>,
            risk_level: Option<String>,
            estimated_minutes: Option<i32>,
            requires_approval: Option<bool>,
            can_rollback: Option<bool>,
            dependencies: Option<Vec<String>>,
            outputs: Option<Vec<String>>,
            mcp_servers: Option<Vec<String>>,
            api_calls: Option<Vec<ApiCallSpec>>,
        }

        let plan_response: PlanResponse = serde_json::from_str(&response)?;

        let steps: Vec<PlanStep> = plan_response
            .steps
            .into_iter()
            .map(|s| PlanStep {
                id: s.id,
                order: s.order,
                name: s.name,
                description: s.description,
                keywords: s.keywords,
                basic_code: String::new(),
                priority: match s.priority.as_deref() {
                    Some("CRITICAL") => StepPriority::Critical,
                    Some("HIGH") => StepPriority::High,
                    Some("LOW") => StepPriority::Low,
                    Some("OPTIONAL") => StepPriority::Optional,
                    Some("MEDIUM") | None | _ => StepPriority::Medium,
                },
                risk_level: match s.risk_level.as_deref() {
                    Some("NONE") => RiskLevel::None,
                    Some("MEDIUM") => RiskLevel::Medium,
                    Some("HIGH") => RiskLevel::High,
                    Some("CRITICAL") => RiskLevel::Critical,
                    Some("LOW") | None | _ => RiskLevel::Low,
                },
                estimated_minutes: s.estimated_minutes.unwrap_or(5),
                requires_approval: s.requires_approval.unwrap_or(false),
                can_rollback: s.can_rollback.unwrap_or(true),
                dependencies: s.dependencies.unwrap_or_default(),
                outputs: s.outputs.unwrap_or_default(),
                mcp_servers: s.mcp_servers.unwrap_or_default(),
                api_calls: s.api_calls.unwrap_or_default(),
            })
            .collect();

        let mut dependencies: HashMap<String, Vec<String>> = HashMap::new();
        for step in &steps {
            dependencies.insert(step.id.clone(), step.dependencies.clone());
        }

        let approval_levels = Self::determine_approval_levels(&steps);

        Ok(ExecutionPlan {
            id: Uuid::new_v4().to_string(),
            name: plan_response.name,
            description: plan_response.description,
            steps,
            dependencies,
            estimated_duration_minutes: plan_response.estimated_duration_minutes.unwrap_or(60),
            requires_approval: plan_response.requires_approval.unwrap_or(false),
            approval_levels,
            rollback_plan: plan_response.rollback_plan,
        })
    }

    fn generate_basic_program(plan: &ExecutionPlan, entities: &IntentEntities) -> String {
        let mut program = String::new();

        let _ = writeln!(
            program,
            "' ============================================================================="
        );
        let _ = writeln!(program, "' AUTO-GENERATED BASIC PROGRAM");
        let _ = writeln!(program, "' Plan: {}", plan.name);
        let _ = writeln!(program, "' Description: {}", plan.description);
        let _ = writeln!(
            program,
            "' Generated: {}",
            Utc::now().format("%Y-%m-%d %H:%M:%S")
        );
        let _ = writeln!(
            program,
            "' =============================================================================\n"
        );

        let _ = writeln!(
            program,
            "PLAN_START \"{}\", \"{}\"",
            plan.name, plan.description
        );

        for step in &plan.steps {
            let priority_str = match step.priority {
                StepPriority::Critical => "CRITICAL",
                StepPriority::High => "HIGH",
                StepPriority::Medium => "MEDIUM",
                StepPriority::Low => "LOW",
                StepPriority::Optional => "OPTIONAL",
            };
            let _ = writeln!(
                program,
                "  STEP {}, \"{}\", {}",
                step.order, step.name, priority_str
            );
        }
        let _ = writeln!(program, "PLAN_END\n");

        let _ = writeln!(program, "' Initialize context variables");
        let _ = writeln!(program, "SET action = \"{}\"", entities.action);
        let _ = writeln!(program, "SET target = \"{}\"", entities.target);
        if let Some(ref client) = entities.client {
            let _ = writeln!(program, "SET client = \"{client}\"");
        }
        if let Some(ref domain) = entities.domain {
            let _ = writeln!(program, "SET domain = \"{domain}\"");
        }
        program.push('\n');

        let _ = writeln!(program, "' Set LLM context");
        let _ = writeln!(
            program,
            "SET CONTEXT \"Task: {} {} for {}\"\n",
            entities.action,
            entities.target,
            entities.client.as_deref().unwrap_or("general use")
        );

        for step in &plan.steps {
            let _ = writeln!(
                program,
                "' -----------------------------------------------------------------------------"
            );
            let _ = writeln!(program, "' STEP {}: {}", step.order, step.name);
            let _ = writeln!(program, "' {}", step.description);
            let _ = writeln!(
                program,
                "' Risk: {:?}, Approval Required: {}",
                step.risk_level, step.requires_approval
            );
            let _ = writeln!(
                program,
                "' -----------------------------------------------------------------------------"
            );

            let step_code = Self::generate_step_code(step);
            let _ = writeln!(program, "{step_code}");
        }

        let _ = writeln!(program, "' Task completed");
        let _ = writeln!(program, "TALK \"Task completed successfully!\"");
        let _ = writeln!(
            program,
            "AUDIT_LOG \"plan-complete\", \"{}\", \"success\"",
            plan.id
        );

        program
    }

    fn generate_step_code(step: &PlanStep) -> String {
        let mut code = String::new();

        if step.requires_approval {
            let _ = writeln!(
                code,
                "REQUIRE_APPROVAL \"step-{}\", \"{}\"",
                step.order, step.description
            );
            let _ = writeln!(code, "IF NOT approved THEN");
            let _ = writeln!(
                code,
                "  TALK \"Step {} was not approved, skipping...\"",
                step.order
            );
            let _ = writeln!(code, "  GOTO step_{}_end", step.order);
            let _ = writeln!(code, "END IF\n");
        }

        if matches!(step.risk_level, RiskLevel::High | RiskLevel::Critical) {
            let _ = writeln!(
                code,
                "simulation_result = SIMULATE_IMPACT \"step-{}\"",
                step.order
            );
            let _ = writeln!(code, "IF simulation_result.risk_score > 0.7 THEN");
            let _ = writeln!(
                code,
                "  TALK \"High risk detected in step {}, requesting manual review...\"",
                step.order
            );
            let _ = writeln!(
                code,
                "  REQUIRE_APPROVAL \"high-risk-override\", simulation_result.summary"
            );
            let _ = writeln!(code, "END IF\n");
        }

        let _ = writeln!(
            code,
            "AUDIT_LOG \"step-start\", \"step-{}\", \"{}\"",
            step.order, step.name
        );

        for keyword in &step.keywords {
            match keyword.to_uppercase().as_str() {
                "CREATE_TASK" => {
                    let _ = writeln!(
                        code,
                        "task_{} = CREATE_TASK \"{}\", \"auto\", \"+1 day\", null",
                        step.order, step.name
                    );
                }
                "LLM" => {
                    let _ = writeln!(
                        code,
                        "llm_result_{} = LLM \"{}\"",
                        step.order, step.description
                    );
                }
                "RUN_PYTHON" => {
                    let _ = writeln!(
                        code,
                        "python_result_{} = RUN_PYTHON \"# {}\nprint('Step {} executed')\"",
                        step.order, step.description, step.order
                    );
                }
                "RUN_JAVASCRIPT" => {
                    let _ = writeln!(
                        code,
                        "js_result_{} = RUN_JAVASCRIPT \"console.log('Step {} executed');\"",
                        step.order, step.order
                    );
                }
                "GET" => {
                    let _ = writeln!(code, "data_{} = GET \"{}_data\"", step.order, step.id);
                }
                "SET" => {
                    let _ = writeln!(code, "SET step_{}_complete = true", step.order);
                }
                "SAVE" => {
                    let _ = writeln!(code, "SAVE step_{}_result TO \"results\"", step.order);
                }
                "POST" | "PUT" | "PATCH" | "DELETE HTTP" => {
                    for api_call in &step.api_calls {
                        let _ = writeln!(
                            code,
                            "{keyword} \"{}\" INTO api_result_{}",
                            api_call.url_template, step.order
                        );
                    }
                }
                "USE_MCP" => {
                    for mcp_server in &step.mcp_servers {
                        let _ = writeln!(
                            code,
                            "mcp_result_{} = USE_MCP \"{}\", \"{}\"",
                            step.order, mcp_server, step.description
                        );
                    }
                }
                "SEND_MAIL" => {
                    let _ = writeln!(
                        code,
                        "SEND_MAIL \"status@bot.local\", \"Step {} Complete\", \"{}\"",
                        step.order, step.description
                    );
                }
                _ => {
                    let _ = writeln!(code, "' Using keyword: {keyword}");
                }
            }
        }

        for output in &step.outputs {
            let _ = writeln!(code, "SET output_{output} = result_{}", step.order);
        }

        let _ = writeln!(
            code,
            "AUDIT_LOG \"step-end\", \"step-{}\", \"complete\"",
            step.order
        );

        let _ = writeln!(code, "step_{}_end:\n", step.order);

        code
    }

    async fn call_llm(
        &self,
        prompt: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        trace!("Calling LLM with prompt length: {}", prompt.len());

        let response = serde_json::json!({
            "action": "create",
            "target": "system",
            "domain": null,
            "client": null,
            "features": [],
            "constraints": [],
            "technologies": [],
            "data_sources": [],
            "integrations": []
        });

        Ok(response.to_string())
    }

    fn assess_risks(plan: &ExecutionPlan) -> RiskAssessment {
        let mut risks = Vec::new();
        let mut overall_risk = RiskLevel::Low;

        for step in &plan.steps {
            if step.risk_level >= RiskLevel::High {
                overall_risk = step.risk_level;
                risks.push(IdentifiedRisk {
                    id: format!("risk-{}", step.id),
                    category: RiskCategory::DependencyFailure,
                    description: format!("Step '{}' has high risk level", step.name),
                    probability: 0.3,
                    impact: step.risk_level,
                    affected_steps: vec![step.id.clone()],
                });
            }
        }

        RiskAssessment {
            overall_risk,
            risks,
            mitigations: Vec::new(),
            requires_human_review: overall_risk >= RiskLevel::High,
            review_reason: if overall_risk >= RiskLevel::High {
                Some("High risk steps detected".to_string())
            } else {
                None
            },
        }
    }

    fn estimate_resources(plan: &ExecutionPlan) -> ResourceEstimate {
        let mut estimate = ResourceEstimate::default();

        for step in &plan.steps {
            estimate.compute_hours += f64::from(step.estimated_minutes) / 60.0;
            estimate.api_calls += step.api_calls.len() as i32;

            for keyword in &step.keywords {
                if keyword == "LLM" {
                    estimate.llm_tokens += 1000;
                }
            }

            for mcp in &step.mcp_servers {
                if !estimate.mcp_servers_needed.contains(mcp) {
                    estimate.mcp_servers_needed.push(mcp.clone());
                }
            }
        }

        let llm_cost = f64::from(estimate.llm_tokens).mul_add(0.00002, 0.0);
        estimate.estimated_cost_usd = estimate
            .compute_hours
            .mul_add(0.10, f64::from(estimate.api_calls) * 0.001)
            + llm_cost;

        estimate
    }

    fn check_ambiguity() -> (f64, Vec<AlternativeInterpretation>) {
        (0.85, Vec::new())
    }

    fn store_compiled_intent(_compiled: &CompiledIntent) {
        info!("Storing compiled intent (stub)");
    }

    fn determine_approval_levels(steps: &[PlanStep]) -> Vec<ApprovalLevel> {
        let mut levels = Vec::new();

        let has_high_risk = steps.iter().any(|s| s.risk_level >= RiskLevel::High);

        if has_high_risk {
            levels.push(ApprovalLevel {
                level: 1,
                approver: "admin".to_string(),
                reason: "High risk steps require approval".to_string(),
                timeout_minutes: 60,
                default_action: DefaultApprovalAction::Pause,
            });
        }

        levels
    }
}

fn get_all_keywords() -> Vec<String> {
    vec![
        "ADD BOT".to_string(),
        "ADD MEMBER".to_string(),
        "ADD SUGGESTION".to_string(),
        "ADD TOOL".to_string(),
        "AUDIT_LOG".to_string(),
        "BOOK".to_string(),
        "CLEAR KB".to_string(),
        "CLEAR TOOLS".to_string(),
        "CREATE DRAFT".to_string(),
        "CREATE SITE".to_string(),
        "CREATE_TASK".to_string(),
        "DELETE".to_string(),
        "DELETE HTTP".to_string(),
        "DOWNLOAD".to_string(),
        "FILL".to_string(),
        "FILTER".to_string(),
        "FIND".to_string(),
        "FIRST".to_string(),
        "GET".to_string(),
        "GRAPHQL".to_string(),
        "HEAR".to_string(),
        "INSERT".to_string(),
        "JOIN".to_string(),
        "LAST".to_string(),
        "LIST".to_string(),
        "LLM".to_string(),
        "MAP".to_string(),
        "MERGE".to_string(),
        "PATCH".to_string(),
        "PIVOT".to_string(),
        "POST".to_string(),
        "PRINT".to_string(),
        "PUT".to_string(),
        "REMEMBER".to_string(),
        "REQUIRE_APPROVAL".to_string(),
        "RUN_BASH".to_string(),
        "RUN_JAVASCRIPT".to_string(),
        "RUN_PYTHON".to_string(),
        "SAVE".to_string(),
        "SEND_MAIL".to_string(),
        "SEND_TEMPLATE".to_string(),
        "SET".to_string(),
        "SET CONTEXT".to_string(),
        "SET SCHEDULE".to_string(),
        "SET USER".to_string(),
        "SIMULATE_IMPACT".to_string(),
        "SMS".to_string(),
        "SOAP".to_string(),
        "TALK".to_string(),
        "UPDATE".to_string(),
        "UPLOAD".to_string(),
        "USE KB".to_string(),
        "USE MODEL".to_string(),
        "USE TOOL".to_string(),
        "USE WEBSITE".to_string(),
        "USE_MCP".to_string(),
        "WAIT".to_string(),
        "WEATHER".to_string(),
        "WEBHOOK".to_string(),
    ]
}
