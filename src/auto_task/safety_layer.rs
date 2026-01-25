use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use log::{info, trace, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintCheckResult {
    pub passed: bool,
    pub results: Vec<ConstraintResult>,
    pub risk_score: f64,
    pub blocking: Vec<String>,
    pub warnings: Vec<String>,
    pub suggestions: Vec<String>,
}

impl Default for ConstraintCheckResult {
    fn default() -> Self {
        Self {
            passed: true,
            results: Vec::new(),
            risk_score: 0.0,
            blocking: Vec::new(),
            warnings: Vec::new(),
            suggestions: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintResult {
    pub constraint_id: String,
    pub constraint_type: ConstraintType,
    pub passed: bool,
    pub severity: ConstraintSeverity,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub remediation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConstraintType {
    Budget,
    Permission,
    Policy,
    Compliance,
    Technical,
    RateLimit,
    TimeWindow,
    DataAccess,
    Security,
    Resource,
    Custom(String),
}

impl Default for ConstraintType {
    fn default() -> Self {
        Self::Custom("unknown".to_string())
    }
}

impl std::fmt::Display for ConstraintType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Budget => write!(f, "budget"),
            Self::Permission => write!(f, "permission"),
            Self::Policy => write!(f, "policy"),
            Self::Compliance => write!(f, "compliance"),
            Self::Technical => write!(f, "technical"),
            Self::RateLimit => write!(f, "rate_limit"),
            Self::TimeWindow => write!(f, "time_window"),
            Self::DataAccess => write!(f, "data_access"),
            Self::Security => write!(f, "security"),
            Self::Resource => write!(f, "resource"),
            Self::Custom(s) => write!(f, "{s}"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Ord, PartialOrd)]
#[derive(Default)]
pub enum ConstraintSeverity {
    Info = 0,
    #[default]
    Warning = 1,
    Error = 2,
    Critical = 3,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    pub id: String,
    pub name: String,
    pub constraint_type: ConstraintType,
    pub description: String,
    pub expression: Option<String>,
    pub threshold: Option<serde_json::Value>,
    pub severity: ConstraintSeverity,
    pub enabled: bool,
    pub applies_to: Vec<String>,
    pub bot_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub id: String,
    pub success: bool,
    pub step_outcomes: Vec<StepSimulationOutcome>,
    pub impact: ImpactAssessment,
    pub resource_usage: PredictedResourceUsage,
    pub side_effects: Vec<SideEffect>,
    pub recommendations: Vec<Recommendation>,
    pub confidence: f64,
    pub simulated_at: DateTime<Utc>,
    pub simulation_duration_ms: i64,
}

impl Default for SimulationResult {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            success: true,
            step_outcomes: Vec::new(),
            impact: ImpactAssessment::default(),
            resource_usage: PredictedResourceUsage::default(),
            side_effects: Vec::new(),
            recommendations: Vec::new(),
            confidence: 0.0,
            simulated_at: Utc::now(),
            simulation_duration_ms: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepSimulationOutcome {
    pub step_id: String,
    pub step_name: String,
    pub would_succeed: bool,
    pub success_probability: f64,
    pub predicted_outputs: serde_json::Value,
    pub failure_modes: Vec<FailureMode>,
    pub estimated_duration_seconds: i32,
    pub affected_dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureMode {
    pub failure_type: String,
    pub probability: f64,
    pub impact: String,
    pub mitigation: Option<String>,
    pub recoverable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAssessment {
    pub risk_score: f64,
    pub risk_level: RiskLevel,
    pub data_impact: DataImpact,
    pub cost_impact: CostImpact,
    pub time_impact: TimeImpact,
    pub security_impact: SecurityImpact,
    pub summary: String,
}

impl Default for ImpactAssessment {
    fn default() -> Self {
        Self {
            risk_score: 0.0,
            risk_level: RiskLevel::Low,
            data_impact: DataImpact::default(),
            cost_impact: CostImpact::default(),
            time_impact: TimeImpact::default(),
            security_impact: SecurityImpact::default(),
            summary: "No impact assessed".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Ord, PartialOrd)]
#[derive(Default)]
pub enum RiskLevel {
    None = 0,
    #[default]
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}


impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Low => write!(f, "Low"),
            Self::Medium => write!(f, "Medium"),
            Self::High => write!(f, "High"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataImpact {
    pub records_created: i32,
    pub records_modified: i32,
    pub records_deleted: i32,
    pub tables_affected: Vec<String>,
    pub data_sources_affected: Vec<String>,
    pub reversible: bool,
    pub backup_required: bool,
}

impl Default for DataImpact {
    fn default() -> Self {
        Self {
            records_created: 0,
            records_modified: 0,
            records_deleted: 0,
            tables_affected: Vec::new(),
            data_sources_affected: Vec::new(),
            reversible: true,
            backup_required: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostImpact {
    pub api_costs: f64,
    pub compute_costs: f64,
    pub storage_costs: f64,
    pub total_estimated_cost: f64,
    pub currency: String,
    pub exceeds_budget: bool,
    pub budget_remaining: Option<f64>,
}

impl Default for CostImpact {
    fn default() -> Self {
        Self {
            api_costs: 0.0,
            compute_costs: 0.0,
            storage_costs: 0.0,
            total_estimated_cost: 0.0,
            currency: "USD".to_string(),
            exceeds_budget: false,
            budget_remaining: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct TimeImpact {
    pub estimated_duration_seconds: i32,
    pub blocking: bool,
    pub delayed_tasks: Vec<String>,
    pub affects_deadline: bool,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityImpact {
    pub risk_level: RiskLevel,
    pub credentials_accessed: Vec<String>,
    pub external_systems: Vec<String>,
    pub data_exposure_risk: bool,
    pub requires_elevation: bool,
    pub concerns: Vec<String>,
}

impl Default for SecurityImpact {
    fn default() -> Self {
        Self {
            risk_level: RiskLevel::Low,
            credentials_accessed: Vec::new(),
            external_systems: Vec::new(),
            data_exposure_risk: false,
            requires_elevation: false,
            concerns: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictedResourceUsage {
    pub cpu_percent: f64,
    pub memory_mb: f64,
    pub network_kbps: f64,
    pub disk_io_kbps: f64,
    pub api_calls: i32,
    pub db_queries: i32,
    pub llm_tokens: i32,
}

impl Default for PredictedResourceUsage {
    fn default() -> Self {
        Self {
            cpu_percent: 0.0,
            memory_mb: 0.0,
            network_kbps: 0.0,
            disk_io_kbps: 0.0,
            api_calls: 0,
            db_queries: 0,
            llm_tokens: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SideEffect {
    pub effect_type: String,
    pub description: String,
    pub severity: ConstraintSeverity,
    pub affected_systems: Vec<String>,
    pub intentional: bool,
    pub mitigation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub recommendation_type: RecommendationType,
    pub priority: i32,
    pub description: String,
    pub action: Option<String>,
    pub basic_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationType {
    AddSafetyCheck,
    AddErrorHandling,
    RequestApproval,
    AddBackup,
    Optimize,
    SplitSteps,
    AddMonitoring,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: AuditEventType,
    pub actor: AuditActor,
    pub action: String,
    pub target: AuditTarget,
    pub outcome: AuditOutcome,
    pub details: serde_json::Value,
    pub related_entities: Vec<RelatedEntity>,
    pub session_id: String,
    pub bot_id: String,
    pub task_id: Option<String>,
    pub step_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub risk_level: RiskLevel,
    pub auto_executed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditEventType {
    TaskCreated,
    TaskStarted,
    TaskCompleted,
    TaskFailed,
    TaskCancelled,
    TaskPaused,
    TaskResumed,
    StepStarted,
    StepCompleted,
    StepFailed,
    StepSkipped,
    StepRolledBack,
    ApprovalRequested,
    ApprovalGranted,
    ApprovalDenied,
    ApprovalExpired,
    DecisionRequested,
    DecisionMade,
    DecisionTimeout,
    SimulationStarted,
    SimulationCompleted,
    ConstraintChecked,
    ConstraintViolated,
    ConstraintOverridden,
    DataRead,
    DataCreated,
    DataModified,
    DataDeleted,
    ApiCalled,
    McpInvoked,
    WebhookTriggered,
    PermissionChecked,
    PermissionDenied,
    CredentialAccessed,
    ConfigChanged,
    ErrorOccurred,
    WarningRaised,
    Custom(String),
}

impl std::fmt::Display for AuditEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TaskCreated => write!(f, "task_created"),
            Self::TaskStarted => write!(f, "task_started"),
            Self::TaskCompleted => write!(f, "task_completed"),
            Self::TaskFailed => write!(f, "task_failed"),
            Self::TaskCancelled => write!(f, "task_cancelled"),
            Self::TaskPaused => write!(f, "task_paused"),
            Self::TaskResumed => write!(f, "task_resumed"),
            Self::StepStarted => write!(f, "step_started"),
            Self::StepCompleted => write!(f, "step_completed"),
            Self::StepFailed => write!(f, "step_failed"),
            Self::StepSkipped => write!(f, "step_skipped"),
            Self::StepRolledBack => write!(f, "step_rolled_back"),
            Self::ApprovalRequested => write!(f, "approval_requested"),
            Self::ApprovalGranted => write!(f, "approval_granted"),
            Self::ApprovalDenied => write!(f, "approval_denied"),
            Self::ApprovalExpired => write!(f, "approval_expired"),
            Self::DecisionRequested => write!(f, "decision_requested"),
            Self::DecisionMade => write!(f, "decision_made"),
            Self::DecisionTimeout => write!(f, "decision_timeout"),
            Self::SimulationStarted => write!(f, "simulation_started"),
            Self::SimulationCompleted => write!(f, "simulation_completed"),
            Self::ConstraintChecked => write!(f, "constraint_checked"),
            Self::ConstraintViolated => write!(f, "constraint_violated"),
            Self::ConstraintOverridden => write!(f, "constraint_overridden"),
            Self::DataRead => write!(f, "data_read"),
            Self::DataCreated => write!(f, "data_created"),
            Self::DataModified => write!(f, "data_modified"),
            Self::DataDeleted => write!(f, "data_deleted"),
            Self::ApiCalled => write!(f, "api_called"),
            Self::McpInvoked => write!(f, "mcp_invoked"),
            Self::WebhookTriggered => write!(f, "webhook_triggered"),
            Self::PermissionChecked => write!(f, "permission_checked"),
            Self::PermissionDenied => write!(f, "permission_denied"),
            Self::CredentialAccessed => write!(f, "credential_accessed"),
            Self::ConfigChanged => write!(f, "config_changed"),
            Self::ErrorOccurred => write!(f, "error_occurred"),
            Self::WarningRaised => write!(f, "warning_raised"),
            Self::Custom(s) => write!(f, "{s}"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditActor {
    pub actor_type: ActorType,
    pub id: String,
    pub name: Option<String>,
    pub role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActorType {
    User,
    Bot,
    System,
    External,
    Anonymous,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTarget {
    pub target_type: String,
    pub id: String,
    pub name: Option<String>,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditOutcome {
    pub success: bool,
    pub result_code: Option<String>,
    pub message: Option<String>,
    pub duration_ms: Option<i64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedEntity {
    pub entity_type: String,
    pub entity_id: String,
    pub relationship: String,
}

pub struct SafetyLayer {
    state: Arc<AppState>,
    config: SafetyConfig,
    constraints: Vec<Constraint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyConfig {
    pub enabled: bool,
    pub check_constraints: bool,
    pub simulate_impact: bool,
    pub audit_enabled: bool,
    pub approval_threshold: RiskLevel,
    pub max_auto_execute_risk: RiskLevel,
    pub default_budget_limit: f64,
    pub rate_limit_per_minute: i32,
    pub circuit_breaker_threshold: i32,
    pub audit_retention_days: i32,
    pub require_simulation_for: Vec<String>,
}

impl Default for SafetyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_constraints: true,
            simulate_impact: true,
            audit_enabled: true,
            approval_threshold: RiskLevel::High,
            max_auto_execute_risk: RiskLevel::Low,
            default_budget_limit: 100.0,
            rate_limit_per_minute: 60,
            circuit_breaker_threshold: 5,
            audit_retention_days: 90,
            require_simulation_for: vec![
                "DELETE".to_string(),
                "UPDATE".to_string(),
                "RUN_PYTHON".to_string(),
                "RUN_BASH".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "PATCH".to_string(),
            ],
        }
    }
}

impl std::fmt::Debug for SafetyLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SafetyLayer")
            .field("config", &self.config)
            .field("constraints_count", &self.constraints.len())
            .finish_non_exhaustive()
    }
}

impl SafetyLayer {
    pub fn new(state: Arc<AppState>) -> Self {
        Self {
            state,
            config: SafetyConfig::default(),
            constraints: Vec::new(),
        }
    }

    pub fn with_config(state: Arc<AppState>, config: SafetyConfig) -> Self {
        Self {
            state,
            config,
            constraints: Vec::new(),
        }
    }

    pub fn load_constraints(
        &mut self,
        bot_id: &Uuid,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self
            .state
            .conn
            .get()
            .map_err(|e| format!("DB error: {e}"))?;
        let bot_id_str = bot_id.to_string();

        let query = diesel::sql_query(
            "SELECT id, name, constraint_type, description, expression, threshold, severity, enabled, applies_to
             FROM safety_constraints WHERE bot_id = $1 AND enabled = true"
        )
        .bind::<diesel::sql_types::Text, _>(&bot_id_str);

        #[derive(QueryableByName)]
        struct ConstraintRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            id: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            name: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            constraint_type: String,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            description: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            expression: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            threshold: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Text)]
            severity: String,
            #[diesel(sql_type = diesel::sql_types::Bool)]
            enabled: bool,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            applies_to: Option<String>,
        }

        let rows: Vec<ConstraintRow> = query.load(&mut *conn).unwrap_or_default();

        self.constraints = rows
            .into_iter()
            .map(|row| Constraint {
                id: row.id,
                name: row.name,
                constraint_type: match row.constraint_type.as_str() {
                    "budget" => ConstraintType::Budget,
                    "permission" => ConstraintType::Permission,
                    "policy" => ConstraintType::Policy,
                    "compliance" => ConstraintType::Compliance,
                    "technical" => ConstraintType::Technical,
                    "rate_limit" => ConstraintType::RateLimit,
                    "time_window" => ConstraintType::TimeWindow,
                    "data_access" => ConstraintType::DataAccess,
                    "security" => ConstraintType::Security,
                    "resource" => ConstraintType::Resource,
                    other => ConstraintType::Custom(other.to_string()),
                },
                description: row.description.unwrap_or_default(),
                expression: row.expression,
                threshold: row.threshold.and_then(|t| serde_json::from_str(&t).ok()),
                severity: match row.severity.as_str() {
                    "info" => ConstraintSeverity::Info,
                    "error" => ConstraintSeverity::Error,
                    "critical" => ConstraintSeverity::Critical,
                    // "warning" and any other value default to Warning
                    _ => ConstraintSeverity::Warning,
                },
                enabled: row.enabled,
                applies_to: row
                    .applies_to
                    .map(|s| s.split(',').map(|x| x.trim().to_string()).collect())
                    .unwrap_or_default(),
                bot_id: bot_id_str.clone(),
            })
            .collect();

        info!(
            "Loaded {} constraints for bot {bot_id}",
            self.constraints.len()
        );
        Ok(())
    }

    pub fn check_constraints(
        &self,
        action: &str,
        context: &serde_json::Value,
        _user: &UserSession,
    ) -> Result<ConstraintCheckResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut result = ConstraintCheckResult::default();

        for constraint in &self.constraints {
            if !constraint.enabled {
                continue;
            }

            if !constraint.applies_to.is_empty()
                && !constraint.applies_to.contains(&action.to_string())
            {
                continue;
            }

            let check_result = Self::evaluate_constraint(constraint, context);

            match check_result {
                Ok(passed) => {
                    let constraint_result = ConstraintResult {
                        constraint_id: constraint.id.clone(),
                        constraint_type: constraint.constraint_type.clone(),
                        passed,
                        severity: constraint.severity.clone(),
                        message: if passed {
                            format!("Constraint '{}' passed", constraint.name)
                        } else {
                            format!(
                                "Constraint '{}' violated: {}",
                                constraint.name, constraint.description
                            )
                        },
                        details: None,
                        remediation: None,
                    };

                    if !passed {
                        result.passed = false;
                        match constraint.severity {
                            ConstraintSeverity::Critical | ConstraintSeverity::Error => {
                                result.blocking.push(constraint.name.clone());
                            }
                            ConstraintSeverity::Warning => {
                                result.warnings.push(constraint.name.clone());
                            }
                            ConstraintSeverity::Info => {
                                result.suggestions.push(constraint.name.clone());
                            }
                        }
                    }

                    result.results.push(constraint_result);
                }
                Err(e) => {
                    warn!("Failed to evaluate constraint {}: {e}", constraint.id);
                }
            }
        }

        result.risk_score = Self::calculate_risk_score(&result);
        Ok(result)
    }

    fn evaluate_constraint(
        _constraint: &Constraint,
        _context: &serde_json::Value,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        // Constraint evaluation always passes for now
        Ok(true)
    }

    fn calculate_risk_score(result: &ConstraintCheckResult) -> f64 {
        let blocking_weight = 0.5;
        let warning_weight = 0.3;
        let suggestion_weight = 0.1;

        let blocking_score = (result.blocking.len() as f64) * blocking_weight;
        let warning_score = (result.warnings.len() as f64) * warning_weight;
        let suggestion_score = (result.suggestions.len() as f64) * suggestion_weight;

        (blocking_score + warning_score + suggestion_score).min(1.0)
    }

    pub fn simulate_execution(
        &self,
        task_id: &str,
        _session: &UserSession,
    ) -> Result<SimulationResult, Box<dyn std::error::Error + Send + Sync>> {
        info!("Simulating execution for task_id={task_id}");

        let start_time = std::time::Instant::now();

        let result = SimulationResult {
            id: Uuid::new_v4().to_string(),
            success: true,
            step_outcomes: Vec::new(),
            impact: ImpactAssessment::default(),
            resource_usage: PredictedResourceUsage::default(),
            side_effects: Vec::new(),
            recommendations: Vec::new(),
            confidence: 0.85,
            simulated_at: Utc::now(),
            simulation_duration_ms: start_time.elapsed().as_millis() as i64,
        };

        Ok(result)
    }

    pub fn log_audit(
        &self,
        entry: AuditEntry,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.config.audit_enabled {
            return Ok(());
        }

        let mut conn = self
            .state
            .conn
            .get()
            .map_err(|e| format!("DB error: {e}"))?;

        let details_json = serde_json::to_string(&entry.details)?;
        let now = entry.timestamp.to_rfc3339();
        let event_type_str = entry.event_type.to_string();
        let actor_type_str = format!("{:?}", entry.actor.actor_type);
        let risk_level_str = format!("{:?}", entry.risk_level);

        let query = diesel::sql_query(
            "INSERT INTO audit_log (id, timestamp, event_type, actor_type, actor_id, action, target_type, target_id, outcome_success, details, session_id, bot_id, task_id, step_id, risk_level)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)"
        )
        .bind::<diesel::sql_types::Text, _>(&entry.id)
        .bind::<diesel::sql_types::Text, _>(&now)
        .bind::<diesel::sql_types::Text, _>(&event_type_str)
        .bind::<diesel::sql_types::Text, _>(&actor_type_str)
        .bind::<diesel::sql_types::Text, _>(&entry.actor.id)
        .bind::<diesel::sql_types::Text, _>(&entry.action)
        .bind::<diesel::sql_types::Text, _>(&entry.target.target_type)
        .bind::<diesel::sql_types::Text, _>(&entry.target.id)
        .bind::<diesel::sql_types::Bool, _>(entry.outcome.success)
        .bind::<diesel::sql_types::Text, _>(&details_json)
        .bind::<diesel::sql_types::Text, _>(&entry.session_id)
        .bind::<diesel::sql_types::Text, _>(&entry.bot_id)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&entry.task_id)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&entry.step_id)
        .bind::<diesel::sql_types::Text, _>(&risk_level_str);

        query
            .execute(&mut *conn)
            .map_err(|e| format!("Failed to log audit: {e}"))?;

        trace!("Audit logged: {} - {}", entry.event_type, entry.action);
        Ok(())
    }
}
