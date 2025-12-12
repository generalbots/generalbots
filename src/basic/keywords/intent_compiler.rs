//! Intent Compiler - LLM to BASIC Program Translator
//!
//! This module provides the core "Intent Compiler" functionality that translates
//! natural language requests into executable BASIC programs using the General Bots
//! keyword system.
//!
//! # Architecture
//!
//! ```text
//! User Intent → Intent Analysis → Plan Generation → BASIC Program → Execution
//!      ↓              ↓                 ↓                ↓             ↓
//!  "Make CRM"   Extract entities   Generate steps   CREATE_TASK    Run with
//!               & requirements     with keywords    SET var        safety checks
//! ```
//!
//! # Example
//!
//! ```basic
//! ' Generated from: "Make a financial CRM for Deloitte"
//! PLAN_START "Financial CRM for Deloitte"
//!   STEP 1, "Create database schema", HIGH
//!   STEP 2, "Setup user authentication", HIGH
//!   STEP 3, "Create client management module", MEDIUM
//!   STEP 4, "Create financial tracking module", MEDIUM
//!   STEP 5, "Create reporting dashboard", LOW
//! PLAN_END
//!
//! REQUIRE_APPROVAL "create-database", "Creating database will cost ~$50/month"
//! IF approved THEN
//!   RUN_PYTHON "create_schema.py"
//! END IF
//! ```

use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use chrono::{DateTime, Utc};
use log::{info, trace, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

// ============================================================================
// CORE DATA STRUCTURES
// ============================================================================

/// Represents a compiled intent - the result of LLM analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledIntent {
    /// Unique identifier for this compiled intent
    pub id: String,
    /// Original user intent/request
    pub original_intent: String,
    /// Extracted entities from the intent
    pub entities: IntentEntities,
    /// Generated execution plan
    pub plan: ExecutionPlan,
    /// Generated BASIC program
    pub basic_program: String,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
    /// Alternative interpretations if ambiguous
    pub alternatives: Vec<AlternativeInterpretation>,
    /// Risk assessment
    pub risk_assessment: RiskAssessment,
    /// Estimated resources needed
    pub resource_estimate: ResourceEstimate,
    /// Timestamp of compilation
    pub compiled_at: DateTime<Utc>,
    /// Session that requested this compilation
    pub session_id: String,
    /// Bot that will execute this
    pub bot_id: String,
}

/// Entities extracted from the user's intent
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IntentEntities {
    /// Primary action (create, update, delete, analyze, etc.)
    pub action: String,
    /// Target object/system (CRM, website, report, etc.)
    pub target: String,
    /// Domain/industry (financial, healthcare, retail, etc.)
    pub domain: Option<String>,
    /// Client/company name if mentioned
    pub client: Option<String>,
    /// Specific features requested
    pub features: Vec<String>,
    /// Constraints mentioned (budget, timeline, etc.)
    pub constraints: Vec<Constraint>,
    /// Technologies/tools mentioned
    pub technologies: Vec<String>,
    /// Data sources mentioned
    pub data_sources: Vec<String>,
    /// Integrations needed
    pub integrations: Vec<String>,
}

/// A constraint on the task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    pub constraint_type: ConstraintType,
    pub value: String,
    pub is_hard: bool, // Hard constraint = must be met, soft = preferred
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

/// Execution plan generated from the intent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<PlanStep>,
    pub dependencies: HashMap<String, Vec<String>>, // step_id -> depends_on[]
    pub estimated_duration_minutes: i32,
    pub requires_approval: bool,
    pub approval_levels: Vec<ApprovalLevel>,
    pub rollback_plan: Option<String>,
}

/// A single step in the execution plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    pub id: String,
    pub order: i32,
    pub name: String,
    pub description: String,
    pub keywords: Vec<String>, // BASIC keywords this step will use
    pub basic_code: String,    // Generated BASIC code for this step
    pub priority: StepPriority,
    pub risk_level: RiskLevel,
    pub estimated_minutes: i32,
    pub requires_approval: bool,
    pub can_rollback: bool,
    pub dependencies: Vec<String>,
    pub outputs: Vec<String>,     // Variables/resources this step produces
    pub mcp_servers: Vec<String>, // MCP servers this step needs
    pub api_calls: Vec<ApiCallSpec>, // External APIs this step calls
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StepPriority {
    Critical, // Must complete for any success
    High,     // Important for core functionality
    Medium,   // Adds significant value
    Low,      // Nice to have
    Optional, // Can be skipped if needed
}

impl Default for StepPriority {
    fn default() -> Self {
        StepPriority::Medium
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum RiskLevel {
    None,     // No risk, reversible
    Low,      // Minor impact if fails
    Medium,   // Moderate impact, recoverable
    High,     // Significant impact, difficult recovery
    Critical, // Severe impact, may not be recoverable
}

impl Default for RiskLevel {
    fn default() -> Self {
        RiskLevel::Low
    }
}

/// API call specification for external integrations
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
        AuthType::None
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
        RetryConfig {
            max_retries: 3,
            backoff_ms: 1000,
            retry_on_status: vec![429, 500, 502, 503, 504],
        }
    }
}

/// Approval level for human-in-the-loop
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalLevel {
    pub level: i32,
    pub approver: String, // Role or specific user
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
        DefaultApprovalAction::Pause
    }
}

/// Alternative interpretation when intent is ambiguous
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

/// Risk assessment for the compiled intent
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
    pub probability: f64, // 0.0 - 1.0
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
    pub basic_code: Option<String>, // BASIC code to implement mitigation
    pub fallback_plan: Option<String>,
}

/// Resource estimate for the task
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
        ResourceEstimate {
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

// ============================================================================
// INTENT COMPILER ENGINE
// ============================================================================

/// The main Intent Compiler engine
pub struct IntentCompiler {
    _state: Arc<AppState>,
    config: IntentCompilerConfig,
}

/// Configuration for the Intent Compiler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentCompilerConfig {
    /// Enable/disable the compiler
    pub enabled: bool,
    /// LLM model to use for compilation
    pub model: String,
    /// Temperature for LLM (creativity vs determinism)
    pub temperature: f64,
    /// Maximum tokens for LLM response
    pub max_tokens: i32,
    /// Auto-execute low-risk tasks
    pub auto_execute_low_risk: bool,
    /// Always require approval for these risk levels
    pub require_approval_above: RiskLevel,
    /// Enable simulation before execution
    pub simulate_before_execute: bool,
    /// Maximum steps in a generated plan
    pub max_plan_steps: i32,
    /// Available keywords for code generation
    pub available_keywords: Vec<String>,
    /// Available MCP servers
    pub available_mcp_servers: Vec<String>,
}

impl Default for IntentCompilerConfig {
    fn default() -> Self {
        IntentCompilerConfig {
            enabled: true,
            model: "gpt-4".to_string(),
            temperature: 0.3, // Lower for more deterministic output
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
            .finish()
    }
}

impl IntentCompiler {
    pub fn new(state: Arc<AppState>) -> Self {
        IntentCompiler {
            _state: state,
            config: IntentCompilerConfig::default(),
        }
    }

    pub fn with_config(state: Arc<AppState>, config: IntentCompilerConfig) -> Self {
        IntentCompiler {
            _state: state,
            config,
        }
    }

    /// Main compilation method - translates intent to executable BASIC program
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

        // Step 1: Analyze the intent using LLM
        let entities = self.extract_entities(intent).await?;
        trace!("Extracted entities: {:?}", entities);

        // Step 2: Generate execution plan
        let plan = self.generate_plan(intent, &entities).await?;
        trace!("Generated plan with {} steps", plan.steps.len());

        // Step 3: Generate BASIC program from plan
        let basic_program = self.generate_basic_program(&plan, &entities).await?;
        trace!(
            "Generated BASIC program: {} lines",
            basic_program.lines().count()
        );

        // Step 4: Assess risks
        let risk_assessment = self.assess_risks(&plan).await?;

        // Step 5: Estimate resources
        let resource_estimate = self.estimate_resources(&plan).await?;

        // Step 6: Check for ambiguity and generate alternatives if needed
        let (confidence, alternatives) = self.check_ambiguity(intent, &entities, &plan).await?;

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

        // Store the compiled intent
        self.store_compiled_intent(&compiled).await?;

        Ok(compiled)
    }

    /// Extract entities from the user's intent using LLM
    async fn extract_entities(
        &self,
        intent: &str,
    ) -> Result<IntentEntities, Box<dyn std::error::Error + Send + Sync>> {
        let prompt = format!(
            r#"Analyze this user request and extract structured information.

User Request: "{}"

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

Respond ONLY with valid JSON, no explanation."#,
            intent
        );

        let response = self.call_llm(&prompt).await?;
        let entities: IntentEntities = serde_json::from_str(&response).unwrap_or_else(|e| {
            warn!("Failed to parse entity extraction response: {}", e);
            IntentEntities {
                action: "create".to_string(),
                target: intent.to_string(),
                ..Default::default()
            }
        });

        Ok(entities)
    }

    /// Generate an execution plan from the analyzed intent
    async fn generate_plan(
        &self,
        intent: &str,
        entities: &IntentEntities,
    ) -> Result<ExecutionPlan, Box<dyn std::error::Error + Send + Sync>> {
        let keywords_list = self.config.available_keywords.join(", ");
        let mcp_servers_list = self.config.available_mcp_servers.join(", ");

        let prompt = format!(
            r#"Generate an execution plan for this task.

Original Request: "{}"

Extracted Information:
- Action: {}
- Target: {}
- Domain: {}
- Client: {}
- Features: {:?}
- Technologies: {:?}
- Integrations: {:?}

Available BASIC Keywords: {}
Available MCP Servers: {}

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
            intent,
            entities.action,
            entities.target,
            entities.domain.as_deref().unwrap_or("general"),
            entities.client.as_deref().unwrap_or("none"),
            entities.features,
            entities.technologies,
            entities.integrations,
            keywords_list,
            mcp_servers_list,
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
                basic_code: String::new(), // Will be generated later
                priority: match s.priority.as_deref() {
                    Some("CRITICAL") => StepPriority::Critical,
                    Some("HIGH") => StepPriority::High,
                    Some("MEDIUM") => StepPriority::Medium,
                    Some("LOW") => StepPriority::Low,
                    Some("OPTIONAL") => StepPriority::Optional,
                    _ => StepPriority::Medium,
                },
                risk_level: match s.risk_level.as_deref() {
                    Some("NONE") => RiskLevel::None,
                    Some("LOW") => RiskLevel::Low,
                    Some("MEDIUM") => RiskLevel::Medium,
                    Some("HIGH") => RiskLevel::High,
                    Some("CRITICAL") => RiskLevel::Critical,
                    _ => RiskLevel::Low,
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

        // Build dependency map
        let mut dependencies: HashMap<String, Vec<String>> = HashMap::new();
        for step in &steps {
            dependencies.insert(step.id.clone(), step.dependencies.clone());
        }

        // Determine approval levels based on risk
        let approval_levels = self.determine_approval_levels(&steps);

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

    /// Generate BASIC program code from the execution plan
    async fn generate_basic_program(
        &self,
        plan: &ExecutionPlan,
        entities: &IntentEntities,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut program = String::new();

        // Header comment
        program.push_str(&format!(
            "' =============================================================================\n"
        ));
        program.push_str(&format!("' AUTO-GENERATED BASIC PROGRAM\n"));
        program.push_str(&format!("' Plan: {}\n", plan.name));
        program.push_str(&format!("' Description: {}\n", plan.description));
        program.push_str(&format!(
            "' Generated: {}\n",
            Utc::now().format("%Y-%m-%d %H:%M:%S")
        ));
        program.push_str(&format!(
            "' =============================================================================\n\n"
        ));

        // Plan declaration
        program.push_str(&format!(
            "PLAN_START \"{}\", \"{}\"\n",
            plan.name, plan.description
        ));

        // Declare steps
        for step in &plan.steps {
            let priority_str = match step.priority {
                StepPriority::Critical => "CRITICAL",
                StepPriority::High => "HIGH",
                StepPriority::Medium => "MEDIUM",
                StepPriority::Low => "LOW",
                StepPriority::Optional => "OPTIONAL",
            };
            program.push_str(&format!(
                "  STEP {}, \"{}\", {}\n",
                step.order, step.name, priority_str
            ));
        }
        program.push_str("PLAN_END\n\n");

        // Initialize variables
        program.push_str("' Initialize context variables\n");
        program.push_str(&format!("SET action = \"{}\"\n", entities.action));
        program.push_str(&format!("SET target = \"{}\"\n", entities.target));
        if let Some(ref client) = entities.client {
            program.push_str(&format!("SET client = \"{}\"\n", client));
        }
        if let Some(ref domain) = entities.domain {
            program.push_str(&format!("SET domain = \"{}\"\n", domain));
        }
        program.push_str("\n");

        // Set context for LLM operations
        program.push_str("' Set LLM context\n");
        program.push_str(&format!(
            "SET CONTEXT \"Task: {} {} for {}\"\n\n",
            entities.action,
            entities.target,
            entities.client.as_deref().unwrap_or("general use")
        ));

        // Generate code for each step
        for step in &plan.steps {
            program.push_str(&format!(
                "' -----------------------------------------------------------------------------\n"
            ));
            program.push_str(&format!("' STEP {}: {}\n", step.order, step.name));
            program.push_str(&format!("' {}\n", step.description));
            program.push_str(&format!(
                "' Risk: {:?}, Approval Required: {}\n",
                step.risk_level, step.requires_approval
            ));
            program.push_str(&format!(
                "' -----------------------------------------------------------------------------\n"
            ));

            // Generate step code
            let step_code = self.generate_step_code(step, entities).await?;
            program.push_str(&step_code);
            program.push_str("\n");
        }

        // Completion
        program.push_str("' Task completed\n");
        program.push_str("TALK \"Task completed successfully!\"\n");
        program.push_str(&format!(
            "AUDIT_LOG \"plan-complete\", \"{}\", \"success\"\n",
            plan.id
        ));

        Ok(program)
    }

    /// Generate BASIC code for a single step
    async fn generate_step_code(
        &self,
        step: &PlanStep,
        _entities: &IntentEntities,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut code = String::new();

        // Add approval check if needed
        if step.requires_approval {
            code.push_str(&format!(
                "REQUIRE_APPROVAL \"step-{}\", \"{}\"\n",
                step.order, step.description
            ));
            code.push_str("IF NOT approved THEN\n");
            code.push_str(&format!(
                "  TALK \"Step {} was not approved, skipping...\"\n",
                step.order
            ));
            code.push_str(&format!("  GOTO step_{}_end\n", step.order));
            code.push_str("END IF\n\n");
        }

        // Add simulation check for high-risk steps
        if matches!(step.risk_level, RiskLevel::High | RiskLevel::Critical) {
            code.push_str(&format!(
                "simulation_result = SIMULATE_IMPACT \"step-{}\"\n",
                step.order
            ));
            code.push_str("IF simulation_result.risk_score > 0.7 THEN\n");
            code.push_str(&format!(
                "  TALK \"High risk detected in step {}, requesting manual review...\"\n",
                step.order
            ));
            code.push_str("  REQUIRE_APPROVAL \"high-risk-override\", simulation_result.summary\n");
            code.push_str("END IF\n\n");
        }

        // Audit log start
        code.push_str(&format!(
            "AUDIT_LOG \"step-start\", \"step-{}\", \"{}\"\n",
            step.order, step.name
        ));

        // Generate code based on keywords
        for keyword in &step.keywords {
            match keyword.to_uppercase().as_str() {
                "CREATE_TASK" => {
                    code.push_str(&format!(
                        "task_{} = CREATE_TASK \"{}\", \"auto\", \"+1 day\", null\n",
                        step.order, step.name
                    ));
                }
                "LLM" => {
                    code.push_str(&format!(
                        "llm_result_{} = LLM \"{}\"\n",
                        step.order, step.description
                    ));
                }
                "RUN_PYTHON" => {
                    code.push_str(&format!(
                        "python_result_{} = RUN_PYTHON \"# {}\nprint('Step {} executed')\"\n",
                        step.order, step.description, step.order
                    ));
                }
                "RUN_JAVASCRIPT" => {
                    code.push_str(&format!(
                        "js_result_{} = RUN_JAVASCRIPT \"console.log('Step {} executed');\"\n",
                        step.order, step.order
                    ));
                }
                "GET" => {
                    code.push_str(&format!("data_{} = GET \"{}_data\"\n", step.order, step.id));
                }
                "SET" => {
                    code.push_str(&format!("SET step_{}_complete = true\n", step.order));
                }
                "SAVE" => {
                    code.push_str(&format!("SAVE step_{}_result TO \"results\"\n", step.order));
                }
                "POST" | "PUT" | "PATCH" | "DELETE HTTP" => {
                    for api_call in &step.api_calls {
                        code.push_str(&format!(
                            "{} \"{}\" INTO api_result_{}\n",
                            keyword, api_call.url_template, step.order
                        ));
                    }
                }
                "USE_MCP" => {
                    for mcp_server in &step.mcp_servers {
                        code.push_str(&format!(
                            "mcp_result_{} = USE_MCP \"{}\", \"{}\"\n",
                            step.order, mcp_server, step.description
                        ));
                    }
                }
                "SEND_MAIL" => {
                    code.push_str(&format!(
                        "SEND_MAIL \"status@bot.local\", \"Step {} Complete\", \"{}\"\n",
                        step.order, step.description
                    ));
                }
                _ => {
                    // Generic keyword usage
                    code.push_str(&format!("' Using keyword: {}\n", keyword));
                }
            }
        }

        // Mark outputs
        for output in &step.outputs {
            code.push_str(&format!("SET output_{} = result_{}\n", output, step.order));
        }

        // Audit log end
        code.push_str(&format!(
            "AUDIT_LOG \"step-end\", \"step-{}\", \"complete\"\n",
            step.order
        ));

        // Add step end label for GOTO
        code.push_str(&format!("step_{}_end:\n\n", step.order));

        Ok(code)
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

    async fn assess_risks(
        &self,
        plan: &ExecutionPlan,
    ) -> Result<RiskAssessment, Box<dyn std::error::Error + Send + Sync>> {
        let mut risks = Vec::new();
        let mut overall_risk = RiskLevel::Low;

        for step in &plan.steps {
            if step.risk_level >= RiskLevel::High {
                overall_risk = step.risk_level.clone();
                risks.push(IdentifiedRisk {
                    id: format!("risk-{}", step.id),
                    category: RiskCategory::DependencyFailure,
                    description: format!("Step '{}' has high risk level", step.name),
                    probability: 0.3,
                    impact: step.risk_level.clone(),
                    affected_steps: vec![step.id.clone()],
                });
            }
        }

        Ok(RiskAssessment {
            overall_risk,
            risks,
            mitigations: Vec::new(),
            requires_human_review: overall_risk >= RiskLevel::High,
            review_reason: if overall_risk >= RiskLevel::High {
                Some("High risk steps detected".to_string())
            } else {
                None
            },
        })
    }

    async fn estimate_resources(
        &self,
        plan: &ExecutionPlan,
    ) -> Result<ResourceEstimate, Box<dyn std::error::Error + Send + Sync>> {
        let mut estimate = ResourceEstimate::default();

        for step in &plan.steps {
            estimate.compute_hours += (step.estimated_minutes as f64) / 60.0;
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

        let llm_cost = (estimate.llm_tokens as f64) * 0.00002;
        estimate.estimated_cost_usd =
            estimate.compute_hours * 0.10 + (estimate.api_calls as f64) * 0.001 + llm_cost;

        Ok(estimate)
    }

    async fn check_ambiguity(
        &self,
        _intent: &str,
        _entities: &IntentEntities,
        _plan: &ExecutionPlan,
    ) -> Result<(f64, Vec<AlternativeInterpretation>), Box<dyn std::error::Error + Send + Sync>>
    {
        Ok((0.85, Vec::new()))
    }

    async fn store_compiled_intent(
        &self,
        _compiled: &CompiledIntent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Storing compiled intent (stub)");
        Ok(())
    }

    fn determine_approval_levels(&self, steps: &[PlanStep]) -> Vec<ApprovalLevel> {
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
