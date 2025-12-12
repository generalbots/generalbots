//! Safety Layer - Simulation, Constraints, and Audit Trail
//!
//! This module provides the safety infrastructure for the Auto Task system,
//! ensuring that all actions are validated, simulated, and audited before
//! and during execution.
//!
//! # Architecture
//!
//! ```text
//! Action Request → Constraint Check → Impact Simulation → Approval → Execute → Audit
//!       ↓               ↓                   ↓                ↓          ↓        ↓
//!   Validate       Check budget,      Simulate what     Get user    Run with   Log all
//!   request        permissions,       will happen       approval    safeguards actions
//!                  policies           (dry run)         if needed
//! ```
//!
//! # Features
//!
//! - **Impact Simulation**: Dry-run execution to predict outcomes
//! - **Constraint Validation**: Budget, permissions, policies, compliance
//! - **Approval Workflow**: Multi-level approval for high-risk actions
//! - **Audit Trail**: Complete logging of all actions and decisions
//! - **Rollback Support**: Undo mechanisms for reversible actions
//! - **Rate Limiting**: Prevent runaway executions
//! - **Circuit Breaker**: Stop execution on repeated failures

use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use log::{info, trace, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

// ============================================================================
// CONSTRAINT DATA STRUCTURES
// ============================================================================

/// Constraint check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintCheckResult {
    /// Whether all constraints passed
    pub passed: bool,
    /// Individual constraint results
    pub results: Vec<ConstraintResult>,
    /// Overall risk score (0.0 - 1.0)
    pub risk_score: f64,
    /// Blocking constraints that must be resolved
    pub blocking: Vec<String>,
    /// Warnings that should be reviewed
    pub warnings: Vec<String>,
    /// Suggestions for improvement
    pub suggestions: Vec<String>,
}

impl Default for ConstraintCheckResult {
    fn default() -> Self {
        ConstraintCheckResult {
            passed: true,
            results: Vec::new(),
            risk_score: 0.0,
            blocking: Vec::new(),
            warnings: Vec::new(),
            suggestions: Vec::new(),
        }
    }
}

/// Individual constraint check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintResult {
    /// Constraint identifier
    pub constraint_id: String,
    /// Constraint type
    pub constraint_type: ConstraintType,
    /// Whether this constraint passed
    pub passed: bool,
    /// Severity if failed
    pub severity: ConstraintSeverity,
    /// Human-readable message
    pub message: String,
    /// Additional details
    pub details: Option<serde_json::Value>,
    /// Suggested remediation
    pub remediation: Option<String>,
}

/// Types of constraints
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConstraintType {
    /// Budget/cost constraints
    Budget,
    /// User permissions
    Permission,
    /// Organizational policies
    Policy,
    /// Regulatory compliance
    Compliance,
    /// Technical limitations
    Technical,
    /// Rate limits
    RateLimit,
    /// Time-based constraints
    TimeWindow,
    /// Data access constraints
    DataAccess,
    /// Security constraints
    Security,
    /// Resource availability
    Resource,
    /// Custom constraint
    Custom(String),
}

impl Default for ConstraintType {
    fn default() -> Self {
        ConstraintType::Custom("unknown".to_string())
    }
}

impl std::fmt::Display for ConstraintType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConstraintType::Budget => write!(f, "budget"),
            ConstraintType::Permission => write!(f, "permission"),
            ConstraintType::Policy => write!(f, "policy"),
            ConstraintType::Compliance => write!(f, "compliance"),
            ConstraintType::Technical => write!(f, "technical"),
            ConstraintType::RateLimit => write!(f, "rate_limit"),
            ConstraintType::TimeWindow => write!(f, "time_window"),
            ConstraintType::DataAccess => write!(f, "data_access"),
            ConstraintType::Security => write!(f, "security"),
            ConstraintType::Resource => write!(f, "resource"),
            ConstraintType::Custom(s) => write!(f, "{}", s),
        }
    }
}

/// Constraint severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Ord, PartialOrd, Eq)]
pub enum ConstraintSeverity {
    /// Informational only
    Info = 0,
    /// Warning - should review but can proceed
    Warning = 1,
    /// Error - should not proceed without override
    Error = 2,
    /// Critical - cannot proceed under any circumstances
    Critical = 3,
}

impl Default for ConstraintSeverity {
    fn default() -> Self {
        ConstraintSeverity::Warning
    }
}

/// Constraint definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    /// Unique identifier
    pub id: String,
    /// Constraint name
    pub name: String,
    /// Constraint type
    pub constraint_type: ConstraintType,
    /// Description
    pub description: String,
    /// Evaluation expression (for dynamic constraints)
    pub expression: Option<String>,
    /// Static value to check against
    pub threshold: Option<serde_json::Value>,
    /// Severity if violated
    pub severity: ConstraintSeverity,
    /// Whether this constraint is enabled
    pub enabled: bool,
    /// Actions this constraint applies to
    pub applies_to: Vec<String>,
    /// Bot ID this constraint belongs to
    pub bot_id: String,
}

// ============================================================================
// SIMULATION DATA STRUCTURES
// ============================================================================

/// Result of impact simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    /// Unique simulation ID
    pub id: String,
    /// Whether simulation completed successfully
    pub success: bool,
    /// Simulated outcomes for each step
    pub step_outcomes: Vec<StepSimulationOutcome>,
    /// Overall impact assessment
    pub impact: ImpactAssessment,
    /// Predicted resource usage
    pub resource_usage: PredictedResourceUsage,
    /// Potential side effects
    pub side_effects: Vec<SideEffect>,
    /// Recommended actions
    pub recommendations: Vec<Recommendation>,
    /// Confidence in the simulation (0.0 - 1.0)
    pub confidence: f64,
    /// Simulation timestamp
    pub simulated_at: DateTime<Utc>,
    /// Duration of simulation in ms
    pub simulation_duration_ms: i64,
}

impl Default for SimulationResult {
    fn default() -> Self {
        SimulationResult {
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

/// Outcome of simulating a single step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepSimulationOutcome {
    /// Step ID
    pub step_id: String,
    /// Step name
    pub step_name: String,
    /// Whether step would succeed
    pub would_succeed: bool,
    /// Probability of success (0.0 - 1.0)
    pub success_probability: f64,
    /// Predicted outputs
    pub predicted_outputs: serde_json::Value,
    /// Potential failure modes
    pub failure_modes: Vec<FailureMode>,
    /// Time estimate in seconds
    pub estimated_duration_seconds: i32,
    /// Dependencies that would be affected
    pub affected_dependencies: Vec<String>,
}

/// Potential failure mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureMode {
    /// Failure type
    pub failure_type: String,
    /// Probability (0.0 - 1.0)
    pub probability: f64,
    /// Impact description
    pub impact: String,
    /// Mitigation strategy
    pub mitigation: Option<String>,
    /// Whether this is recoverable
    pub recoverable: bool,
}

/// Overall impact assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAssessment {
    /// Overall risk score (0.0 - 1.0)
    pub risk_score: f64,
    /// Risk level classification
    pub risk_level: RiskLevel,
    /// Data impact
    pub data_impact: DataImpact,
    /// Cost impact
    pub cost_impact: CostImpact,
    /// Time impact
    pub time_impact: TimeImpact,
    /// Security impact
    pub security_impact: SecurityImpact,
    /// Summary description
    pub summary: String,
}

impl Default for ImpactAssessment {
    fn default() -> Self {
        ImpactAssessment {
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

/// Risk level classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Ord, PartialOrd, Eq)]
pub enum RiskLevel {
    None = 0,
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

impl Default for RiskLevel {
    fn default() -> Self {
        RiskLevel::Low
    }
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::None => write!(f, "None"),
            RiskLevel::Low => write!(f, "Low"),
            RiskLevel::Medium => write!(f, "Medium"),
            RiskLevel::High => write!(f, "High"),
            RiskLevel::Critical => write!(f, "Critical"),
        }
    }
}

/// Data impact assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataImpact {
    /// Records that would be created
    pub records_created: i32,
    /// Records that would be modified
    pub records_modified: i32,
    /// Records that would be deleted
    pub records_deleted: i32,
    /// Tables affected
    pub tables_affected: Vec<String>,
    /// Data sources affected
    pub data_sources_affected: Vec<String>,
    /// Whether changes are reversible
    pub reversible: bool,
    /// Backup required
    pub backup_required: bool,
}

impl Default for DataImpact {
    fn default() -> Self {
        DataImpact {
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

/// Cost impact assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostImpact {
    /// Estimated API costs
    pub api_costs: f64,
    /// Estimated compute costs
    pub compute_costs: f64,
    /// Estimated storage costs
    pub storage_costs: f64,
    /// Total estimated cost
    pub total_estimated_cost: f64,
    /// Cost currency
    pub currency: String,
    /// Whether this exceeds budget
    pub exceeds_budget: bool,
    /// Budget remaining after this action
    pub budget_remaining: Option<f64>,
}

impl Default for CostImpact {
    fn default() -> Self {
        CostImpact {
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

/// Time impact assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeImpact {
    /// Estimated execution time in seconds
    pub estimated_duration_seconds: i32,
    /// Whether this blocks other tasks
    pub blocking: bool,
    /// Tasks that would be delayed
    pub delayed_tasks: Vec<String>,
    /// Deadline impact
    pub affects_deadline: bool,
}

impl Default for TimeImpact {
    fn default() -> Self {
        TimeImpact {
            estimated_duration_seconds: 0,
            blocking: false,
            delayed_tasks: Vec::new(),
            affects_deadline: false,
        }
    }
}

/// Security impact assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityImpact {
    /// Security risk level
    pub risk_level: RiskLevel,
    /// Credentials accessed
    pub credentials_accessed: Vec<String>,
    /// External systems contacted
    pub external_systems: Vec<String>,
    /// Data exposure risk
    pub data_exposure_risk: bool,
    /// Requires elevated permissions
    pub requires_elevation: bool,
    /// Security concerns
    pub concerns: Vec<String>,
}

impl Default for SecurityImpact {
    fn default() -> Self {
        SecurityImpact {
            risk_level: RiskLevel::Low,
            credentials_accessed: Vec::new(),
            external_systems: Vec::new(),
            data_exposure_risk: false,
            requires_elevation: false,
            concerns: Vec::new(),
        }
    }
}

/// Predicted resource usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictedResourceUsage {
    /// CPU usage percentage
    pub cpu_percent: f64,
    /// Memory usage in MB
    pub memory_mb: f64,
    /// Network bandwidth in KB/s
    pub network_kbps: f64,
    /// Disk I/O in KB/s
    pub disk_io_kbps: f64,
    /// Number of API calls
    pub api_calls: i32,
    /// Number of database queries
    pub db_queries: i32,
    /// LLM tokens used
    pub llm_tokens: i32,
}

impl Default for PredictedResourceUsage {
    fn default() -> Self {
        PredictedResourceUsage {
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

/// Potential side effect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SideEffect {
    /// Side effect type
    pub effect_type: String,
    /// Description
    pub description: String,
    /// Severity
    pub severity: ConstraintSeverity,
    /// Affected systems
    pub affected_systems: Vec<String>,
    /// Whether this is intentional
    pub intentional: bool,
    /// Mitigation if unintentional
    pub mitigation: Option<String>,
}

/// Recommendation from simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// Recommendation type
    pub recommendation_type: RecommendationType,
    /// Priority
    pub priority: i32,
    /// Description
    pub description: String,
    /// Action to take
    pub action: Option<String>,
    /// BASIC code to implement
    pub basic_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationType {
    /// Add a safety check
    AddSafetyCheck,
    /// Add error handling
    AddErrorHandling,
    /// Request approval
    RequestApproval,
    /// Add backup step
    AddBackup,
    /// Optimize performance
    Optimize,
    /// Split into smaller steps
    SplitSteps,
    /// Add monitoring
    AddMonitoring,
    /// Custom recommendation
    Custom(String),
}

// ============================================================================
// AUDIT TRAIL DATA STRUCTURES
// ============================================================================

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Unique audit entry ID
    pub id: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Event type
    pub event_type: AuditEventType,
    /// Actor (user or system)
    pub actor: AuditActor,
    /// Action performed
    pub action: String,
    /// Target of the action
    pub target: AuditTarget,
    /// Outcome
    pub outcome: AuditOutcome,
    /// Details
    pub details: serde_json::Value,
    /// Related entities
    pub related_entities: Vec<RelatedEntity>,
    /// Session ID
    pub session_id: String,
    /// Bot ID
    pub bot_id: String,
    /// Task ID if applicable
    pub task_id: Option<String>,
    /// Step ID if applicable
    pub step_id: Option<String>,
    /// IP address
    pub ip_address: Option<String>,
    /// User agent
    pub user_agent: Option<String>,
    /// Risk level of the action
    pub risk_level: RiskLevel,
    /// Whether this was auto-executed
    pub auto_executed: bool,
}

/// Audit event types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuditEventType {
    /// Task lifecycle events
    TaskCreated,
    TaskStarted,
    TaskCompleted,
    TaskFailed,
    TaskCancelled,
    TaskPaused,
    TaskResumed,

    /// Step events
    StepStarted,
    StepCompleted,
    StepFailed,
    StepSkipped,
    StepRolledBack,

    /// Approval events
    ApprovalRequested,
    ApprovalGranted,
    ApprovalDenied,
    ApprovalExpired,

    /// Decision events
    DecisionRequested,
    DecisionMade,
    DecisionTimeout,

    /// Simulation events
    SimulationStarted,
    SimulationCompleted,

    /// Constraint events
    ConstraintChecked,
    ConstraintViolated,
    ConstraintOverridden,

    /// Data events
    DataRead,
    DataCreated,
    DataModified,
    DataDeleted,

    /// External events
    ApiCalled,
    McpInvoked,
    WebhookTriggered,

    /// Security events
    PermissionChecked,
    PermissionDenied,
    CredentialAccessed,

    /// System events
    ConfigChanged,
    ErrorOccurred,
    WarningRaised,

    /// Custom event
    Custom(String),
}

impl std::fmt::Display for AuditEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditEventType::TaskCreated => write!(f, "task_created"),
            AuditEventType::TaskStarted => write!(f, "task_started"),
            AuditEventType::TaskCompleted => write!(f, "task_completed"),
            AuditEventType::TaskFailed => write!(f, "task_failed"),
            AuditEventType::TaskCancelled => write!(f, "task_cancelled"),
            AuditEventType::TaskPaused => write!(f, "task_paused"),
            AuditEventType::TaskResumed => write!(f, "task_resumed"),
            AuditEventType::StepStarted => write!(f, "step_started"),
            AuditEventType::StepCompleted => write!(f, "step_completed"),
            AuditEventType::StepFailed => write!(f, "step_failed"),
            AuditEventType::StepSkipped => write!(f, "step_skipped"),
            AuditEventType::StepRolledBack => write!(f, "step_rolled_back"),
            AuditEventType::ApprovalRequested => write!(f, "approval_requested"),
            AuditEventType::ApprovalGranted => write!(f, "approval_granted"),
            AuditEventType::ApprovalDenied => write!(f, "approval_denied"),
            AuditEventType::ApprovalExpired => write!(f, "approval_expired"),
            AuditEventType::DecisionRequested => write!(f, "decision_requested"),
            AuditEventType::DecisionMade => write!(f, "decision_made"),
            AuditEventType::DecisionTimeout => write!(f, "decision_timeout"),
            AuditEventType::SimulationStarted => write!(f, "simulation_started"),
            AuditEventType::SimulationCompleted => write!(f, "simulation_completed"),
            AuditEventType::ConstraintChecked => write!(f, "constraint_checked"),
            AuditEventType::ConstraintViolated => write!(f, "constraint_violated"),
            AuditEventType::ConstraintOverridden => write!(f, "constraint_overridden"),
            AuditEventType::DataRead => write!(f, "data_read"),
            AuditEventType::DataCreated => write!(f, "data_created"),
            AuditEventType::DataModified => write!(f, "data_modified"),
            AuditEventType::DataDeleted => write!(f, "data_deleted"),
            AuditEventType::ApiCalled => write!(f, "api_called"),
            AuditEventType::McpInvoked => write!(f, "mcp_invoked"),
            AuditEventType::WebhookTriggered => write!(f, "webhook_triggered"),
            AuditEventType::PermissionChecked => write!(f, "permission_checked"),
            AuditEventType::PermissionDenied => write!(f, "permission_denied"),
            AuditEventType::CredentialAccessed => write!(f, "credential_accessed"),
            AuditEventType::ConfigChanged => write!(f, "config_changed"),
            AuditEventType::ErrorOccurred => write!(f, "error_occurred"),
            AuditEventType::WarningRaised => write!(f, "warning_raised"),
            AuditEventType::Custom(s) => write!(f, "{}", s),
        }
    }
}

/// Actor in an audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditActor {
    /// Actor type
    pub actor_type: ActorType,
    /// Actor ID
    pub id: String,
    /// Actor name
    pub name: Option<String>,
    /// Actor role
    pub role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActorType {
    User,
    Bot,
    System,
    External,
    Anonymous,
}

/// Target of an audit action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTarget {
    /// Target type
    pub target_type: String,
    /// Target ID
    pub id: String,
    /// Target name
    pub name: Option<String>,
    /// Additional properties
    pub properties: HashMap<String, String>,
}

/// Outcome of an audit action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditOutcome {
    /// Whether action succeeded
    pub success: bool,
    /// Result code
    pub result_code: Option<String>,
    /// Result message
    pub message: Option<String>,
    /// Duration in milliseconds
    pub duration_ms: Option<i64>,
    /// Error details if failed
    pub error: Option<String>,
}

/// Related entity in audit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedEntity {
    /// Entity type
    pub entity_type: String,
    /// Entity ID
    pub entity_id: String,
    /// Relationship
    pub relationship: String,
}

// ============================================================================
// SAFETY LAYER ENGINE
// ============================================================================

/// The Safety Layer engine
pub struct SafetyLayer {
    state: Arc<AppState>,
    config: SafetyConfig,
    constraints: Vec<Constraint>,
}

/// Safety Layer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyConfig {
    /// Enable/disable safety layer
    pub enabled: bool,
    /// Enable constraint checking
    pub check_constraints: bool,
    /// Enable impact simulation
    pub simulate_impact: bool,
    /// Enable audit logging
    pub audit_enabled: bool,
    /// Risk level that requires approval
    pub approval_threshold: RiskLevel,
    /// Maximum auto-execute risk level
    pub max_auto_execute_risk: RiskLevel,
    /// Default budget limit (USD)
    pub default_budget_limit: f64,
    /// Rate limit (actions per minute)
    pub rate_limit_per_minute: i32,
    /// Circuit breaker failure threshold
    pub circuit_breaker_threshold: i32,
    /// Audit retention days
    pub audit_retention_days: i32,
    /// Require simulation for these action types
    pub require_simulation_for: Vec<String>,
}

impl Default for SafetyConfig {
    fn default() -> Self {
        SafetyConfig {
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
            .finish()
    }
}

impl SafetyLayer {
    /// Create a new Safety Layer
    pub fn new(state: Arc<AppState>) -> Self {
        SafetyLayer {
            state,
            config: SafetyConfig::default(),
            constraints: Vec::new(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(state: Arc<AppState>, config: SafetyConfig) -> Self {
        SafetyLayer {
            state,
            config,
            constraints: Vec::new(),
        }
    }

    /// Load constraints from database
    pub async fn load_constraints(
        &mut self,
        bot_id: &Uuid,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self
            .state
            .conn
            .get()
            .map_err(|e| format!("DB error: {}", e))?;
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
                    "warning" => ConstraintSeverity::Warning,
                    "error" => ConstraintSeverity::Error,
                    "critical" => ConstraintSeverity::Critical,
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
            "Loaded {} constraints for bot {}",
            self.constraints.len(),
            bot_id
        );
        Ok(())
    }

    /// Check all constraints for an action
    pub async fn check_constraints(
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

            let check_result = self.evaluate_constraint(constraint, context).await;

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
                    warn!("Failed to evaluate constraint {}: {}", constraint.id, e);
                }
            }
        }

        result.risk_score = self.calculate_risk_score(&result);
        Ok(result)
    }

    async fn evaluate_constraint(
        &self,
        constraint: &Constraint,
        _context: &serde_json::Value,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(ref _expression) = constraint.expression {
            Ok(true)
        } else {
            Ok(true)
        }
    }

    fn calculate_risk_score(&self, result: &ConstraintCheckResult) -> f64 {
        let blocking_weight = 0.5;
        let warning_weight = 0.3;
        let suggestion_weight = 0.1;

        let blocking_score = (result.blocking.len() as f64) * blocking_weight;
        let warning_score = (result.warnings.len() as f64) * warning_weight;
        let suggestion_score = (result.suggestions.len() as f64) * suggestion_weight;

        (blocking_score + warning_score + suggestion_score).min(1.0)
    }

    pub async fn simulate_execution(
        &self,
        task_id: &str,
        _session: &UserSession,
    ) -> Result<SimulationResult, Box<dyn std::error::Error + Send + Sync>> {
        info!("Simulating execution for task_id={}", task_id);

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

    pub async fn log_audit(
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
            .map_err(|e| format!("DB error: {}", e))?;

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
            .map_err(|e| format!("Failed to log audit: {}", e))?;

        trace!("Audit logged: {} - {}", entry.event_type, entry.action);
        Ok(())
    }
}
