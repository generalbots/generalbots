use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put},
    Router,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::shared::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IncidentSeverity {
    Critical,
    High,
    Medium,
    Low,
    Informational,
}

impl IncidentSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Critical => "critical",
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
            Self::Informational => "informational",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "critical" => Some(Self::Critical),
            "high" => Some(Self::High),
            "medium" => Some(Self::Medium),
            "low" => Some(Self::Low),
            "informational" | "info" => Some(Self::Informational),
            _ => None,
        }
    }

    pub fn response_time_minutes(&self) -> i64 {
        match self {
            Self::Critical => 15,
            Self::High => 60,
            Self::Medium => 240,
            Self::Low => 1440,
            Self::Informational => 10080,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IncidentStatus {
    New,
    Triaging,
    Investigating,
    Containing,
    Eradicating,
    Recovering,
    PostIncident,
    Resolved,
    Closed,
}

impl IncidentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Triaging => "triaging",
            Self::Investigating => "investigating",
            Self::Containing => "containing",
            Self::Eradicating => "eradicating",
            Self::Recovering => "recovering",
            Self::PostIncident => "post_incident",
            Self::Resolved => "resolved",
            Self::Closed => "closed",
        }
    }

    pub fn is_active(&self) -> bool {
        !matches!(self, Self::Resolved | Self::Closed)
    }

    pub fn next_status(&self) -> Option<Self> {
        match self {
            Self::New => Some(Self::Triaging),
            Self::Triaging => Some(Self::Investigating),
            Self::Investigating => Some(Self::Containing),
            Self::Containing => Some(Self::Eradicating),
            Self::Eradicating => Some(Self::Recovering),
            Self::Recovering => Some(Self::PostIncident),
            Self::PostIncident => Some(Self::Resolved),
            Self::Resolved => Some(Self::Closed),
            Self::Closed => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IncidentCategory {
    SecurityBreach,
    DataLeak,
    ServiceOutage,
    PerformanceDegradation,
    UnauthorizedAccess,
    Malware,
    PhishingAttack,
    DenialOfService,
    ConfigurationError,
    ComplianceViolation,
    Other,
}

impl IncidentCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SecurityBreach => "security_breach",
            Self::DataLeak => "data_leak",
            Self::ServiceOutage => "service_outage",
            Self::PerformanceDegradation => "performance_degradation",
            Self::UnauthorizedAccess => "unauthorized_access",
            Self::Malware => "malware",
            Self::PhishingAttack => "phishing_attack",
            Self::DenialOfService => "denial_of_service",
            Self::ConfigurationError => "configuration_error",
            Self::ComplianceViolation => "compliance_violation",
            Self::Other => "other",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Incident {
    pub id: Uuid,
    pub incident_number: String,
    pub title: String,
    pub description: String,
    pub severity: IncidentSeverity,
    pub status: IncidentStatus,
    pub category: IncidentCategory,
    pub affected_systems: Vec<String>,
    pub affected_users_count: Option<u64>,
    pub reported_by: String,
    pub assigned_to: Option<String>,
    pub team: Option<String>,
    pub source: IncidentSource,
    pub timeline: Vec<TimelineEntry>,
    pub actions_taken: Vec<ActionTaken>,
    pub evidence: Vec<Evidence>,
    pub related_incidents: Vec<Uuid>,
    pub tags: Vec<String>,
    pub sla_breach: bool,
    pub sla_breach_at: Option<DateTime<Utc>>,
    pub response_due_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
    pub root_cause: Option<String>,
    pub lessons_learned: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IncidentSource {
    Manual,
    Automated,
    Alert,
    CustomerReport,
    SecurityScan,
    AuditFinding,
    ThirdParty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event_type: TimelineEventType,
    pub description: String,
    pub user: Option<String>,
    pub automated: bool,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TimelineEventType {
    Created,
    StatusChanged,
    SeverityChanged,
    Assigned,
    CommentAdded,
    EvidenceAdded,
    ActionTaken,
    EscalationTriggered,
    NotificationSent,
    SlaBreached,
    Resolved,
    Closed,
    Reopened,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionTaken {
    pub id: Uuid,
    pub action_type: ActionType,
    pub description: String,
    pub performed_by: String,
    pub performed_at: DateTime<Utc>,
    pub automated: bool,
    pub success: bool,
    pub result: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActionType {
    IsolateSystem,
    BlockIpAddress,
    DisableAccount,
    RevokeCredentials,
    RestoreFromBackup,
    ApplyPatch,
    UpdateFirewallRules,
    EnableEnhancedLogging,
    NotifyStakeholders,
    EngageVendor,
    CollectEvidence,
    RunForensics,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub id: Uuid,
    pub evidence_type: EvidenceType,
    pub title: String,
    pub description: Option<String>,
    pub file_path: Option<String>,
    pub url: Option<String>,
    pub hash: Option<String>,
    pub collected_by: String,
    pub collected_at: DateTime<Utc>,
    pub chain_of_custody: Vec<CustodyEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EvidenceType {
    LogFile,
    Screenshot,
    NetworkCapture,
    MemoryDump,
    DiskImage,
    Configuration,
    Email,
    Document,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustodyEntry {
    pub timestamp: DateTime<Utc>,
    pub action: String,
    pub person: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationHook {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub trigger: HookTrigger,
    pub conditions: Vec<HookCondition>,
    pub actions: Vec<HookAction>,
    pub enabled: bool,
    pub priority: i32,
    pub cooldown_minutes: Option<i64>,
    pub last_triggered: Option<DateTime<Utc>>,
    pub trigger_count: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HookTrigger {
    IncidentCreated,
    StatusChanged,
    SeverityChanged,
    SlaBreachImminent,
    SlaBreach,
    Escalation,
    PatternDetected,
    ThresholdExceeded,
    Scheduled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookCondition {
    pub field: String,
    pub operator: ConditionOperator,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConditionOperator {
    Equals,
    NotEquals,
    Contains,
    GreaterThan,
    LessThan,
    In,
    NotIn,
    Matches,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookAction {
    pub action_type: HookActionType,
    pub parameters: HashMap<String, serde_json::Value>,
    pub retry_on_failure: bool,
    pub max_retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HookActionType {
    SendNotification,
    CreateTicket,
    ExecuteRunbook,
    CallWebhook,
    UpdateIncident,
    AssignToTeam,
    EscalateToManager,
    BlockIpAddress,
    DisableUser,
    CollectLogs,
    TriggerBackup,
    SendSlackMessage,
    SendEmail,
    CreateJiraIssue,
    PageOnCall,
    RunScript,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationPolicy {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub levels: Vec<EscalationLevel>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationLevel {
    pub level: u32,
    pub delay_minutes: i64,
    pub notify_users: Vec<String>,
    pub notify_teams: Vec<String>,
    pub channels: Vec<NotificationChannel>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NotificationChannel {
    Email,
    Slack,
    MsTeams,
    PagerDuty,
    Sms,
    Webhook,
    InApp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Runbook {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub category: IncidentCategory,
    pub severity_levels: Vec<IncidentSeverity>,
    pub steps: Vec<RunbookStep>,
    pub automated_steps: Vec<AutomatedStep>,
    pub estimated_duration_minutes: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunbookStep {
    pub order: u32,
    pub title: String,
    pub description: String,
    pub responsible: String,
    pub verification: Option<String>,
    pub rollback: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomatedStep {
    pub order: u32,
    pub name: String,
    pub action: HookActionType,
    pub parameters: HashMap<String, serde_json::Value>,
    pub timeout_seconds: u64,
    pub continue_on_failure: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateIncidentRequest {
    pub title: String,
    pub description: String,
    pub severity: String,
    pub category: String,
    pub affected_systems: Option<Vec<String>>,
    pub reported_by: String,
    pub source: Option<String>,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateIncidentRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub severity: Option<String>,
    pub status: Option<String>,
    pub assigned_to: Option<String>,
    pub team: Option<String>,
    pub root_cause: Option<String>,
    pub lessons_learned: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddTimelineEntryRequest {
    pub description: String,
    pub user: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddActionRequest {
    pub action_type: String,
    pub description: String,
    pub performed_by: String,
    pub success: bool,
    pub result: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListIncidentsQuery {
    pub status: Option<String>,
    pub severity: Option<String>,
    pub category: Option<String>,
    pub assigned_to: Option<String>,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct IncidentListResponse {
    pub incidents: Vec<IncidentSummary>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

#[derive(Debug, Serialize)]
pub struct IncidentSummary {
    pub id: Uuid,
    pub incident_number: String,
    pub title: String,
    pub severity: String,
    pub status: String,
    pub category: String,
    pub assigned_to: Option<String>,
    pub sla_breach: bool,
    pub response_due_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct IncidentStats {
    pub total_incidents: u64,
    pub open_incidents: u64,
    pub resolved_incidents: u64,
    pub sla_breaches: u64,
    pub by_severity: HashMap<String, u64>,
    pub by_category: HashMap<String, u64>,
    pub by_status: HashMap<String, u64>,
    pub mean_time_to_resolve_hours: f64,
    pub mean_time_to_respond_minutes: f64,
}

pub struct IncidentResponseService {
    incidents: Arc<RwLock<HashMap<Uuid, Incident>>>,
    hooks: Arc<RwLock<Vec<AutomationHook>>>,
    escalation_policies: Arc<RwLock<HashMap<Uuid, EscalationPolicy>>>,
    runbooks: Arc<RwLock<HashMap<Uuid, Runbook>>>,
    incident_counter: std::sync::atomic::AtomicU64,
}

impl Default for IncidentResponseService {
    fn default() -> Self {
        Self::new()
    }
}

impl IncidentResponseService {
    pub fn new() -> Self {
        Self {
            incidents: Arc::new(RwLock::new(HashMap::new())),
            hooks: Arc::new(RwLock::new(Vec::new())),
            escalation_policies: Arc::new(RwLock::new(HashMap::new())),
            runbooks: Arc::new(RwLock::new(HashMap::new())),
            incident_counter: std::sync::atomic::AtomicU64::new(1000),
        }
    }

    fn generate_incident_number(&self) -> String {
        let num = self
            .incident_counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let now = Utc::now();
        format!("INC-{}{:02}{:02}-{:04}", now.format("%Y"), now.format("%m"), now.format("%d"), num)
    }

    pub async fn create_incident(
        &self,
        title: String,
        description: String,
        severity: IncidentSeverity,
        category: IncidentCategory,
        affected_systems: Vec<String>,
        reported_by: String,
        source: IncidentSource,
        tags: Vec<String>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Incident {
        let now = Utc::now();
        let incident_id = Uuid::new_v4();
        let incident_number = self.generate_incident_number();
        let response_due_at = now + Duration::minutes(severity.response_time_minutes());

        let initial_timeline = TimelineEntry {
            id: Uuid::new_v4(),
            timestamp: now,
            event_type: TimelineEventType::Created,
            description: format!("Incident created: {title}"),
            user: Some(reported_by.clone()),
            automated: matches!(source, IncidentSource::Automated | IncidentSource::Alert),
            metadata: HashMap::new(),
        };

        let incident = Incident {
            id: incident_id,
            incident_number,
            title,
            description,
            severity,
            status: IncidentStatus::New,
            category,
            affected_systems,
            affected_users_count: None,
            reported_by,
            assigned_to: None,
            team: None,
            source,
            timeline: vec![initial_timeline],
            actions_taken: Vec::new(),
            evidence: Vec::new(),
            related_incidents: Vec::new(),
            tags,
            sla_breach: false,
            sla_breach_at: None,
            response_due_at,
            created_at: now,
            updated_at: now,
            resolved_at: None,
            closed_at: None,
            root_cause: None,
            lessons_learned: None,
            metadata,
        };

        {
            let mut incidents = self.incidents.write().await;
            incidents.insert(incident_id, incident.clone());
        }

        self.trigger_hooks(HookTrigger::IncidentCreated, &incident).await;

        log::info!(
            "Created incident {}: {} (severity: {:?})",
            incident.incident_number,
            incident.title,
            incident.severity
        );

        incident
    }

    pub async fn update_status(
        &self,
        incident_id: Uuid,
        new_status: IncidentStatus,
        user: &str,
    ) -> Result<Incident, String> {
        let mut incidents = self.incidents.write().await;
        let incident = incidents
            .get_mut(&incident_id)
            .ok_or("Incident not found")?;

        let old_status = incident.status.clone();
        incident.status = new_status.clone();
        incident.updated_at = Utc::now();

        if new_status == IncidentStatus::Resolved {
            incident.resolved_at = Some(Utc::now());
        } else if new_status == IncidentStatus::Closed {
            incident.closed_at = Some(Utc::now());
        }

        incident.timeline.push(TimelineEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type: TimelineEventType::StatusChanged,
            description: format!("Status changed from {} to {}", old_status.as_str(), new_status.as_str()),
            user: Some(user.to_string()),
            automated: false,
            metadata: HashMap::new(),
        });

        let incident_clone = incident.clone();
        drop(incidents);

        self.trigger_hooks(HookTrigger::StatusChanged, &incident_clone).await;

        Ok(incident_clone)
    }

    pub async fn update_severity(
        &self,
        incident_id: Uuid,
        new_severity: IncidentSeverity,
        user: &str,
    ) -> Result<Incident, String> {
        let mut incidents = self.incidents.write().await;
        let incident = incidents
            .get_mut(&incident_id)
            .ok_or("Incident not found")?;

        let old_severity = incident.severity.clone();
        incident.severity = new_severity.clone();
        incident.updated_at = Utc::now();
        incident.response_due_at = incident.created_at + Duration::minutes(new_severity.response_time_minutes());

        incident.timeline.push(TimelineEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type: TimelineEventType::SeverityChanged,
            description: format!("Severity changed from {} to {}", old_severity.as_str(), new_severity.as_str()),
            user: Some(user.to_string()),
            automated: false,
            metadata: HashMap::new(),
        });

        let incident_clone = incident.clone();
        drop(incidents);

        self.trigger_hooks(HookTrigger::SeverityChanged, &incident_clone).await;

        Ok(incident_clone)
    }

    pub async fn assign_incident(
        &self,
        incident_id: Uuid,
        assignee: &str,
        assigner: &str,
    ) -> Result<Incident, String> {
        let mut incidents = self.incidents.write().await;
        let incident = incidents
            .get_mut(&incident_id)
            .ok_or("Incident not found")?;

        incident.assigned_to = Some(assignee.to_string());
        incident.updated_at = Utc::now();

        incident.timeline.push(TimelineEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type: TimelineEventType::Assigned,
            description: format!("Assigned to {assignee}"),
            user: Some(assigner.to_string()),
            automated: false,
            metadata: HashMap::new(),
        });

        Ok(incident.clone())
    }

    pub async fn add_action(
        &self,
        incident_id: Uuid,
        action_type: ActionType,
        description: String,
        performed_by: String,
        automated: bool,
        success: bool,
        result: Option<String>,
    ) -> Result<ActionTaken, String> {
        let mut incidents = self.incidents.write().await;
        let incident = incidents
            .get_mut(&incident_id)
            .ok_or("Incident not found")?;

        let action = ActionTaken {
            id: Uuid::new_v4(),
            action_type,
            description: description.clone(),
            performed_by: performed_by.clone(),
            performed_at: Utc::now(),
            automated,
            success,
            result,
        };

        incident.actions_taken.push(action.clone());
        incident.updated_at = Utc::now();

        incident.timeline.push(TimelineEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type: TimelineEventType::ActionTaken,
            description,
            user: Some(performed_by),
            automated,
            metadata: HashMap::new(),
        });

        Ok(action)
    }

    pub async fn add_evidence(
        &self,
        incident_id: Uuid,
        evidence_type: EvidenceType,
        title: String,
        description: Option<String>,
        file_path: Option<String>,
        url: Option<String>,
        collected_by: String,
    ) -> Result<Evidence, String> {
        let mut incidents = self.incidents.write().await;
        let incident = incidents
            .get_mut(&incident_id)
            .ok_or("Incident not found")?;

        let evidence = Evidence {
            id: Uuid::new_v4(),
            evidence_type,
            title: title.clone(),
            description,
            file_path,
            url,
            hash: None,
            collected_by: collected_by.clone(),
            collected_at: Utc::now(),
            chain_of_custody: vec![CustodyEntry {
                timestamp: Utc::now(),
                action: "Collected".to_string(),
                person: collected_by.clone(),
                notes: None,
            }],
        };

        incident.evidence.push(evidence.clone());
        incident.updated_at = Utc::now();

        incident.timeline.push(TimelineEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type: TimelineEventType::EvidenceAdded,
            description: format!("Evidence added: {title}"),
            user: Some(collected_by),
            automated: false,
            metadata: HashMap::new(),
        });

        Ok(evidence)
    }

    pub async fn check_sla_breaches(&self) {
        let now = Utc::now();
        let mut incidents = self.incidents.write().await;

        for incident in incidents.values_mut() {
            if incident.status.is_active() && !incident.sla_breach && now > incident.response_due_at {
                incident.sla_breach = true;
                incident.sla_breach_at = Some(now);
                incident.updated_at = now;

                incident.timeline.push(TimelineEntry {
                    id: Uuid::new_v4(),
                    timestamp: now,
                    event_type: TimelineEventType::SlaBreached,
                    description: "SLA response time breached".to_string(),
                    user: None,
                    automated: true,
                    metadata: HashMap::new(),
                });

                log::warn!(
                    "SLA breach for incident {}: {}",
                    incident.incident_number,
                    incident.title
                );
            }
        }
    }

    pub async fn get_incident(&self, incident_id: Uuid) -> Option<Incident> {
        let incidents = self.incidents.read().await;
        incidents.get(&incident_id).cloned()
    }

    pub async fn list_incidents(
        &self,
        status_filter: Option<IncidentStatus>,
        severity_filter: Option<IncidentSeverity>,
        page: u32,
        per_page: u32,
    ) -> IncidentListResponse {
        let incidents = self.incidents.read().await;

        let mut filtered: Vec<_> = incidents
            .values()
            .filter(|i| {
                if let Some(ref status) = status_filter {
                    if &i.status != status {
                        return false;
                    }
                }
                if let Some(ref severity) = severity_filter {
                    if &i.severity != severity {
                        return false;
                    }
                }
                true
            })
            .collect();

        filtered.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        let total = filtered.len() as u64;
        let total_pages = ((total as f64) / (per_page as f64)).ceil() as u32;
        let start = ((page - 1) * per_page) as usize;
        let end = (start + per_page as usize).min(filtered.len());

        let summaries: Vec<IncidentSummary> = filtered[start..end]
            .iter()
            .map(|i| IncidentSummary {
                id: i.id,
                incident_number: i.incident_number.clone(),
                title: i.title.clone(),
                severity: i.severity.as_str().to_string(),
                status: i.status.as_str().to_string(),
                category: i.category.as_str().to_string(),
                assigned_to: i.assigned_to.clone(),
                sla_breach: i.sla_breach,
                response_due_at: i.response_due_at,
                created_at: i.created_at,
            })
            .collect();

        IncidentListResponse {
            incidents: summaries,
            total,
            page,
            per_page,
            total_pages,
        }
    }

    pub async fn get_stats(&self) -> IncidentStats {
        let incidents = self.incidents.read().await;

        let mut by_severity: HashMap<String, u64> = HashMap::new();
        let mut by_category: HashMap<String, u64> = HashMap::new();
        let mut by_status: HashMap<String, u64> = HashMap::new();
        let mut total_resolve_time: i64 = 0;
        let mut resolved_count: u64 = 0;
        let mut sla_breaches: u64 = 0;
        let mut open_incidents: u64 = 0;

        for incident in incidents.values() {
            *by_severity.entry(incident.severity.as_str().to_string()).or_insert(0) += 1;
            *by_category.entry(incident.category.as_str().to_string()).or_insert(0) += 1;
            *by_status.entry(incident.status.as_str().to_string()).or_insert(0) += 1;

            if incident.sla_breach {
                sla_breaches += 1;
            }

            if incident.status.is_active() {
                open_incidents += 1;
            }

            if let Some(resolved_at) = incident.resolved_at {
                let duration = resolved_at - incident.created_at;
                total_resolve_time += duration.num_minutes();
                resolved_count += 1;
            }
        }

        let mean_time_to_resolve_hours = if resolved_count > 0 {
            (total_resolve_time as f64 / resolved_count as f64) / 60.0
        } else {
            0.0
        };

        IncidentStats {
            total_incidents: incidents.len() as u64,
            open_incidents,
            resolved_incidents: resolved_count,
            sla_breaches,
            by_severity,
            by_category,
            by_status,
            mean_time_to_resolve_hours,
            mean_time_to_respond_minutes: 0.0,
        }
    }

    pub async fn register_hook(&self, hook: AutomationHook) {
        let mut hooks = self.hooks.write().await;
        hooks.push(hook);
