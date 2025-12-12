//! Auto Task System - Self-Executing Intelligent Tasks
//!
//! This module provides the "Auto Task" functionality that enables tasks to
//! automatically execute themselves using LLM-generated BASIC programs.
//! It integrates with the Intent Compiler, Safety Layer, and MCP servers
//! to create a fully autonomous task execution system.
//!
//! # Architecture
//!
//! ```text
//! User Intent → Auto Task → Intent Compiler → Execution Plan → Safety Check → Execute
//!      ↓            ↓              ↓                ↓              ↓           ↓
//! "Build CRM"   Create task   Generate BASIC   Validate plan   Simulate   Run steps
//!               with metadata  program          & approve       impact     with audit
//! ```
//!
//! # Features
//!
//! - **Automatic Execution**: Tasks execute themselves when conditions are met
//! - **Safety First**: All actions are simulated and validated before execution
//! - **Decision Framework**: Ambiguous situations generate options for user choice
//! - **Audit Trail**: Complete logging of all actions and decisions
//! - **MCP Integration**: Leverage registered MCP servers for extended capabilities
//! - **Rollback Support**: Automatic rollback on failure when possible

use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use chrono::{DateTime, Duration, Utc};
use diesel::prelude::*;
use log::{error, info, trace, warn};
use rhai::{Dynamic, Engine};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

// ============================================================================
// AUTO TASK DATA STRUCTURES
// ============================================================================

/// Represents an auto-executing task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoTask {
    /// Unique task identifier
    pub id: String,
    /// Human-readable task title
    pub title: String,
    /// Original intent/description
    pub intent: String,
    /// Current task status
    pub status: AutoTaskStatus,
    /// Execution mode
    pub mode: ExecutionMode,
    /// Priority level
    pub priority: TaskPriority,
    /// Generated execution plan ID
    pub plan_id: Option<String>,
    /// Generated BASIC program
    pub basic_program: Option<String>,
    /// Current execution step (0 = not started)
    pub current_step: i32,
    /// Total steps in the plan
    pub total_steps: i32,
    /// Execution progress (0.0 - 1.0)
    pub progress: f64,
    /// Step execution results
    pub step_results: Vec<StepExecutionResult>,
    /// Pending decisions requiring user input
    pub pending_decisions: Vec<PendingDecision>,
    /// Active approvals waiting
    pub pending_approvals: Vec<PendingApproval>,
    /// Risk assessment summary
    pub risk_summary: Option<RiskSummary>,
    /// Resource usage tracking
    pub resource_usage: ResourceUsage,
    /// Error information if failed
    pub error: Option<TaskError>,
    /// Rollback state if available
    pub rollback_state: Option<RollbackState>,
    /// Session that created this task
    pub session_id: String,
    /// Bot executing this task
    pub bot_id: String,
    /// User who created the task
    pub created_by: String,
    /// Assigned executor (user or "auto")
    pub assigned_to: String,
    /// Scheduling information
    pub schedule: Option<TaskSchedule>,
    /// Tags for organization
    pub tags: Vec<String>,
    /// Parent task ID if this is a subtask
    pub parent_task_id: Option<String>,
    /// Child task IDs
    pub subtask_ids: Vec<String>,
    /// Dependencies on other tasks
    pub depends_on: Vec<String>,
    /// Tasks that depend on this one
    pub dependents: Vec<String>,
    /// MCP servers being used
    pub mcp_servers: Vec<String>,
    /// External APIs being called
    pub external_apis: Vec<String>,
    /// Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    /// Estimated completion time
    pub estimated_completion: Option<DateTime<Utc>>,
}

/// Auto task status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AutoTaskStatus {
    /// Task created, not yet analyzed
    Draft,
    /// Intent being compiled to BASIC program
    Compiling,
    /// Plan generated, waiting for approval
    PendingApproval,
    /// Simulating execution impact
    Simulating,
    /// Waiting for user decision on options
    WaitingDecision,
    /// Ready to execute
    Ready,
    /// Currently executing
    Running,
    /// Paused by user or system
    Paused,
    /// Waiting for external resource
    Blocked,
    /// Completed successfully
    Completed,
    /// Failed with error
    Failed,
    /// Cancelled by user
    Cancelled,
    /// Rolling back changes
    RollingBack,
    /// Rollback completed
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

/// Execution mode for the task
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionMode {
    /// Fully automatic - execute without user intervention
    FullyAutomatic,
    /// Semi-automatic - pause for approvals on high-risk steps
    SemiAutomatic,
    /// Supervised - pause after each step for review
    Supervised,
    /// Manual - user triggers each step
    Manual,
    /// Dry run - simulate only, don't execute
    DryRun,
}

impl Default for ExecutionMode {
    fn default() -> Self {
        ExecutionMode::SemiAutomatic
    }
}

/// Task priority
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

/// Result of executing a single step
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

/// Status of a single step
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StepStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
    RolledBack,
}

/// Execution log entry
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

/// A decision point requiring user input
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
    /// Choose between multiple approaches
    ApproachSelection,
    /// Confirm a high-risk action
    RiskConfirmation,
    /// Resolve ambiguous intent
    AmbiguityResolution,
    /// Provide missing information
    InformationRequest,
    /// Handle an error
    ErrorRecovery,
    /// Custom decision type
    Custom(String),
}

/// An option in a decision
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

/// A pending approval request
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

/// Risk assessment summary
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

#
