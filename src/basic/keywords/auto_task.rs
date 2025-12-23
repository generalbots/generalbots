
























use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;




#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoTask {

    pub id: String,

    pub title: String,

    pub intent: String,

    pub status: AutoTaskStatus,

    pub mode: ExecutionMode,

    pub priority: TaskPriority,

    pub plan_id: Option<String>,

    pub basic_program: Option<String>,

    pub current_step: i32,

    pub total_steps: i32,

    pub progress: f64,

    pub step_results: Vec<StepExecutionResult>,

    pub pending_decisions: Vec<PendingDecision>,

    pub pending_approvals: Vec<PendingApproval>,

    pub risk_summary: Option<RiskSummary>,

    pub resource_usage: ResourceUsage,

    pub error: Option<TaskError>,

    pub rollback_state: Option<RollbackState>,

    pub session_id: String,

    pub bot_id: String,

    pub created_by: String,

    pub assigned_to: String,

    pub schedule: Option<TaskSchedule>,

    pub tags: Vec<String>,

    pub parent_task_id: Option<String>,

    pub subtask_ids: Vec<String>,

    pub depends_on: Vec<String>,

    pub dependents: Vec<String>,

    pub mcp_servers: Vec<String>,

    pub external_apis: Vec<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,

    pub estimated_completion: Option<DateTime<Utc>>,
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AutoTaskStatus {

    Draft,

    Compiling,

    PendingApproval,

    Simulating,

    WaitingDecision,

    Ready,

    Running,

    Paused,

    Blocked,

    Completed,

    Failed,

    Cancelled,

    RollingBack,

    RolledBack,
}

impl Default for AutoTaskStatus {
    fn default() -> Self {
        AutoTaskStatus::Draft
    }
}

impl std::fmt::Display for AutoTaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AutoTaskStatus::Draft => write!(f, "Draft"),
            AutoTaskStatus::Compiling => write!(f, "Compiling"),
            AutoTaskStatus::PendingApproval => write!(f, "Pending Approval"),
            AutoTaskStatus::Simulating => write!(f, "Simulating"),
            AutoTaskStatus::WaitingDecision => write!(f, "Waiting for Decision"),
            AutoTaskStatus::Ready => write!(f, "Ready"),
            AutoTaskStatus::Running => write!(f, "Running"),
            AutoTaskStatus::Paused => write!(f, "Paused"),
            AutoTaskStatus::Blocked => write!(f, "Blocked"),
            AutoTaskStatus::Completed => write!(f, "Completed"),
            AutoTaskStatus::Failed => write!(f, "Failed"),
            AutoTaskStatus::Cancelled => write!(f, "Cancelled"),
            AutoTaskStatus::RollingBack => write!(f, "Rolling Back"),
            AutoTaskStatus::RolledBack => write!(f, "Rolled Back"),
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionMode {

    FullyAutomatic,

    SemiAutomatic,

    Supervised,

    Manual,

    DryRun,
}

impl Default for ExecutionMode {
    fn default() -> Self {
        ExecutionMode::SemiAutomatic
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Ord, PartialOrd, Eq)]
pub enum TaskPriority {
    Critical = 4,
    High = 3,
    Medium = 2,
    Low = 1,
    Background = 0,
}

impl Default for TaskPriority {
    fn default() -> Self {
        TaskPriority::Medium
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepExecutionResult {
    pub step_id: String,
    pub step_order: i32,
    pub step_name: String,
    pub status: StepStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<i64>,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
    pub logs: Vec<ExecutionLog>,
    pub resources_used: ResourceUsage,
    pub can_rollback: bool,
    pub rollback_data: Option<serde_json::Value>,
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StepStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
    RolledBack,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionLog {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingDecision {
    pub id: String,
    pub decision_type: DecisionType,
    pub title: String,
    pub description: String,
    pub options: Vec<DecisionOption>,
    pub default_option: Option<String>,
    pub timeout_seconds: Option<i32>,
    pub timeout_action: TimeoutAction,
    pub context: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecisionType {

    ApproachSelection,

    RiskConfirmation,

    AmbiguityResolution,

    InformationRequest,

    ErrorRecovery,

    Custom(String),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionOption {
    pub id: String,
    pub label: String,
    pub description: String,
    pub pros: Vec<String>,
    pub cons: Vec<String>,
    pub estimated_impact: ImpactEstimate,
    pub recommended: bool,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactEstimate {
    pub cost_change: f64,
    pub time_change_minutes: i32,
    pub risk_change: f64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeoutAction {
    UseDefault,
    Pause,
    Cancel,
    Escalate,
}

impl Default for TimeoutAction {
    fn default() -> Self {
        TimeoutAction::Pause
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingApproval {
    pub id: String,
    pub approval_type: ApprovalType,
    pub title: String,
    pub description: String,
    pub risk_level: RiskLevel,
    pub approver: String,
    pub step_id: Option<String>,
    pub impact_summary: String,
    pub simulation_result: Option<SimulationResult>,
    pub timeout_seconds: i32,
    pub default_action: ApprovalDefault,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApprovalType {
    PlanApproval,
    StepApproval,
    HighRiskAction,
    ExternalApiCall,
    DataModification,
    CostOverride,
    SecurityOverride,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApprovalDefault {
    Approve,
    Reject,
    Pause,
    Escalate,
}

impl Default for ApprovalDefault {
    fn default() -> Self {
        ApprovalDefault::Pause
    }
}


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


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskSummary {
    pub overall_risk: RiskLevel,
    pub data_risk: RiskLevel,
    pub cost_risk: RiskLevel,
    pub security_risk: RiskLevel,
    pub compliance_risk: RiskLevel,
    pub risk_factors: Vec<RiskFactor>,
    pub mitigations_applied: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    pub id: String,
    pub category: RiskCategory,
    pub description: String,
    pub probability: f64,
    pub impact: RiskLevel,
    pub mitigation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskCategory {
    Data,
    Cost,
    Security,
    Compliance,
    Performance,
    Availability,
    Integration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub compute_hours: f64,
    pub storage_gb: f64,
    pub api_calls: i32,
    pub llm_tokens: i32,
    pub estimated_cost_usd: f64,
    pub mcp_servers_used: Vec<String>,
    pub external_services: Vec<String>,
}

impl Default for ResourceUsage {
    fn default() -> Self {
        ResourceUsage {
            compute_hours: 0.0,
            storage_gb: 0.0,
            api_calls: 0,
            llm_tokens: 0,
            estimated_cost_usd: 0.0,
            mcp_servers_used: Vec::new(),
            external_services: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskError {
    pub code: String,
    pub message: String,
    pub step_id: Option<String>,
    pub recoverable: bool,
    pub details: Option<serde_json::Value>,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackState {
    pub available: bool,
    pub steps_rolled_back: Vec<String>,
    pub rollback_data: HashMap<String, serde_json::Value>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl Default for RollbackState {
    fn default() -> Self {
        RollbackState {
            available: false,
            steps_rolled_back: Vec::new(),
            rollback_data: HashMap::new(),
            started_at: None,
            completed_at: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSchedule {
    pub schedule_type: ScheduleType,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub cron_expression: Option<String>,
    pub timezone: String,
    pub max_retries: i32,
    pub retry_delay_seconds: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScheduleType {
    Immediate,
    Scheduled,
    Recurring,
    OnDemand,
}

impl Default for TaskSchedule {
    fn default() -> Self {
        TaskSchedule {
            schedule_type: ScheduleType::Immediate,
            scheduled_at: None,
            cron_expression: None,
            timezone: "UTC".to_string(),
            max_retries: 3,
            retry_delay_seconds: 60,
        }
    }
}

use crate::basic::keywords::safety_layer::SimulationResult;
