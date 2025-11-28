//! Audit Module
//!
//! Provides comprehensive audit logging and tracking capabilities
//! for compliance monitoring and security analysis.

use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Audit event types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum AuditEventType {
    UserLogin,
    UserLogout,
    PasswordChange,
    PermissionGranted,
    PermissionRevoked,
    DataAccess,
    DataModification,
    DataDeletion,
    ConfigurationChange,
    SecurityAlert,
    SystemError,
    ComplianceViolation,
}

/// Audit severity levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub enum AuditSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Audit event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event_type: AuditEventType,
    pub severity: AuditSeverity,
    pub user_id: Option<Uuid>,
    pub session_id: Option<String>,
    pub ip_address: Option<String>,
    pub resource_id: Option<String>,
    pub action: String,
    pub outcome: AuditOutcome,
    pub details: HashMap<String, String>,
    pub metadata: serde_json::Value,
}

/// Audit outcome
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum AuditOutcome {
    Success,
    Failure,
    Partial,
    Unknown,
}

/// Audit trail for tracking related events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrail {
    pub trail_id: Uuid,
    pub name: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub events: Vec<Uuid>,
    pub summary: String,
    pub tags: Vec<String>,
}

/// Audit retention policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub name: String,
    pub retention_days: i64,
    pub event_types: Vec<AuditEventType>,
    pub severity_threshold: Option<AuditSeverity>,
    pub archive_enabled: bool,
}

/// Audit statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStatistics {
    pub total_events: usize,
    pub events_by_type: HashMap<AuditEventType, usize>,
    pub events_by_severity: HashMap<AuditSeverity, usize>,
    pub events_by_outcome: HashMap<AuditOutcome, usize>,
    pub unique_users: usize,
    pub time_range: (DateTime<Utc>, DateTime<Utc>),
}

/// Audit service for managing audit logs
#[derive(Clone)]
pub struct AuditService {
    events: Arc<RwLock<VecDeque<AuditEvent>>>,
    trails: Arc<RwLock<HashMap<Uuid, AuditTrail>>>,
    retention_policies: Arc<RwLock<Vec<RetentionPolicy>>>,
    max_events: usize,
}

impl AuditService {
    /// Create new audit service
    pub fn new(max_events: usize) -> Self {
        Self {
            events: Arc::new(RwLock::new(VecDeque::new())),
            trails: Arc::new(RwLock::new(HashMap::new())),
            retention_policies: Arc::new(RwLock::new(vec![
                // Default retention policies
                RetentionPolicy {
                    name: "Security Events".to_string(),
                    retention_days: 365,
                    event_types: vec![
                        AuditEventType::SecurityAlert,
                        AuditEventType::ComplianceViolation,
                    ],
                    severity_threshold: Some(AuditSeverity::Warning),
                    archive_enabled: true,
                },
                RetentionPolicy {
                    name: "Access Logs".to_string(),
                    retention_days: 90,
                    event_types: vec![
                        AuditEventType::UserLogin,
                        AuditEventType::UserLogout,
                        AuditEventType::DataAccess,
                    ],
                    severity_threshold: None,
                    archive_enabled: false,
                },
            ])),
            max_events,
        }
    }

    /// Log an audit event
    pub async fn log_event(
        &self,
        event_type: AuditEventType,
        severity: AuditSeverity,
        user_id: Option<Uuid>,
        action: String,
        outcome: AuditOutcome,
        details: HashMap<String, String>,
    ) -> Result<Uuid> {
        let event = AuditEvent {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type: event_type.clone(),
            severity: severity.clone(),
            user_id,
            session_id: None,
            ip_address: None,
            resource_id: None,
            action,
            outcome,
            details,
            metadata: serde_json::json!({}),
        };

        let event_id = event.id;

        // Add event to the queue
        let mut events = self.events.write().await;
        events.push_back(event.clone());

        // Maintain max events limit
        while events.len() > self.max_events {
            events.pop_front();
        }

        log::info!(
            "Audit event logged: {} - {:?} - {:?}",
            event_id,
            event_type,
            severity
        );

        Ok(event_id)
    }

    /// Create an audit trail
    pub async fn create_trail(&self, name: String, tags: Vec<String>) -> Result<Uuid> {
        let trail = AuditTrail {
            trail_id: Uuid::new_v4(),
            name,
            started_at: Utc::now(),
            ended_at: None,
            events: Vec::new(),
            summary: String::new(),
            tags,
        };

        let trail_id = trail.trail_id;
        let mut trails = self.trails.write().await;
        trails.insert(trail_id, trail);

        Ok(trail_id)
    }

    /// Add event to trail
    pub async fn add_to_trail(&self, trail_id: Uuid, event_id: Uuid) -> Result<()> {
        let mut trails = self.trails.write().await;
        let trail = trails
            .get_mut(&trail_id)
            .ok_or_else(|| anyhow!("Trail not found"))?;

        if trail.ended_at.is_some() {
            return Err(anyhow!("Trail already ended"));
        }

        trail.events.push(event_id);
        Ok(())
    }

    /// End an audit trail
    pub async fn end_trail(&self, trail_id: Uuid, summary: String) -> Result<()> {
        let mut trails = self.trails.write().await;
        let trail = trails
            .get_mut(&trail_id)
            .ok_or_else(|| anyhow!("Trail not found"))?;

        trail.ended_at = Some(Utc::now());
        trail.summary = summary;

        Ok(())
    }

    /// Query audit events
    pub async fn query_events(&self, filter: AuditFilter) -> Result<Vec<AuditEvent>> {
        let events = self.events.read().await;

        let filtered: Vec<AuditEvent> = events
            .iter()
            .filter(|e| filter.matches(e))
            .cloned()
            .collect();

        Ok(filtered)
    }

    /// Get audit statistics
    pub async fn get_statistics(
        &self,
        since: Option<DateTime<Utc>>,
        until: Option<DateTime<Utc>>,
    ) -> AuditStatistics {
        let events = self.events.read().await;
        let since = since.unwrap_or(Utc::now() - Duration::days(30));
        let until = until.unwrap_or(Utc::now());

        let filtered_events: Vec<_> = events
            .iter()
            .filter(|e| e.timestamp >= since && e.timestamp <= until)
            .collect();

        let mut events_by_type = HashMap::new();
        let mut events_by_severity = HashMap::new();
        let mut events_by_outcome = HashMap::new();
        let mut unique_users = std::collections::HashSet::new();

        for event in &filtered_events {
            *events_by_type.entry(event.event_type.clone()).or_insert(0) += 1;
            *events_by_severity
                .entry(event.severity.clone())
                .or_insert(0) += 1;
            *events_by_outcome.entry(event.outcome.clone()).or_insert(0) += 1;

            if let Some(user_id) = event.user_id {
                unique_users.insert(user_id);
            }
        }

        AuditStatistics {
            total_events: filtered_events.len(),
            events_by_type,
            events_by_severity,
            events_by_outcome,
            unique_users: unique_users.len(),
            time_range: (since, until),
        }
    }

    /// Apply retention policies
    pub async fn apply_retention_policies(&self) -> Result<usize> {
        let policies = self.retention_policies.read().await;
        let mut events = self.events.write().await;
        let now = Utc::now();
        let mut removed_count = 0;

        for policy in policies.iter() {
            let cutoff = now - Duration::days(policy.retention_days);

            // Remove events older than retention period
            let initial_len = events.len();
            events.retain(|e| {
                if !policy.event_types.contains(&e.event_type) {
                    return true;
                }

                if let Some(threshold) = &policy.severity_threshold {
                    if e.severity < *threshold {
                        return true;
                    }
                }

                e.timestamp >= cutoff
            });

            removed_count += initial_len - events.len();
        }

        log::info!(
            "Applied retention policies, removed {} events",
            removed_count
        );
        Ok(removed_count)
    }

    /// Export audit logs
    pub async fn export_logs(
        &self,
        format: ExportFormat,
        filter: Option<AuditFilter>,
    ) -> Result<Vec<u8>> {
        let events = self.query_events(filter.unwrap_or_default()).await?;

        match format {
            ExportFormat::Json => {
                let json = serde_json::to_vec_pretty(&events)?;
                Ok(json)
            }
            ExportFormat::Csv => {
                let mut csv_writer = csv::Writer::from_writer(vec![]);

                // Write headers
                csv_writer.write_record(&[
                    "ID",
                    "Timestamp",
                    "Type",
                    "Severity",
                    "User",
                    "Action",
                    "Outcome",
                ])?;

                // Write records
                for event in events {
                    csv_writer.write_record(&[
                        event.id.to_string(),
                        event.timestamp.to_rfc3339(),
                        format!("{:?}", event.event_type),
                        format!("{:?}", event.severity),
                        event.user_id.map(|u| u.to_string()).unwrap_or_default(),
                        event.action,
                        format!("{:?}", event.outcome),
                    ])?;
                }

                Ok(csv_writer.into_inner()?)
            }
        }
    }

    /// Get compliance report
    pub async fn get_compliance_report(&self) -> ComplianceReport {
        let stats = self.get_statistics(None, None).await;
        let events = self.events.read().await;

        let security_incidents = events
            .iter()
            .filter(|e| e.event_type == AuditEventType::SecurityAlert)
            .count();

        let compliance_violations = events
            .iter()
            .filter(|e| e.event_type == AuditEventType::ComplianceViolation)
            .count();

        let failed_logins = events
            .iter()
            .filter(|e| {
                e.event_type == AuditEventType::UserLogin && e.outcome == AuditOutcome::Failure
            })
            .count();

        ComplianceReport {
            generated_at: Utc::now(),
            total_events: stats.total_events,
            security_incidents,
            compliance_violations,
            failed_logins,
            unique_users: stats.unique_users,
            critical_events: stats
                .events_by_severity
                .get(&AuditSeverity::Critical)
                .copied()
                .unwrap_or(0),
            audit_coverage: self.calculate_audit_coverage(&events),
        }
    }

    /// Calculate audit coverage percentage
    fn calculate_audit_coverage(&self, events: &VecDeque<AuditEvent>) -> f64 {
        // Calculate based on expected event types coverage
        let expected_types = 12; // Total number of event types
        let covered_types = events
            .iter()
            .map(|e| e.event_type.clone())
            .collect::<std::collections::HashSet<_>>()
            .len();

        (covered_types as f64 / expected_types as f64) * 100.0
    }
}

/// Audit filter for querying events
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditFilter {
    pub event_types: Option<Vec<AuditEventType>>,
    pub severity: Option<AuditSeverity>,
    pub user_id: Option<Uuid>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub outcome: Option<AuditOutcome>,
}

impl AuditFilter {
    fn matches(&self, event: &AuditEvent) -> bool {
        if let Some(types) = &self.event_types {
            if !types.contains(&event.event_type) {
                return false;
            }
        }

        if let Some(severity) = &self.severity {
            if event.severity < *severity {
                return false;
            }
        }

        if let Some(user_id) = &self.user_id {
            if event.user_id != Some(*user_id) {
                return false;
            }
        }

        if let Some(since) = &self.since {
            if event.timestamp < *since {
                return false;
            }
        }

        if let Some(until) = &self.until {
            if event.timestamp > *until {
                return false;
            }
        }

        if let Some(outcome) = &self.outcome {
            if event.outcome != *outcome {
                return false;
            }
        }

        true
    }
}

/// Export format for audit logs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Csv,
}

/// Compliance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub generated_at: DateTime<Utc>,
    pub total_events: usize,
    pub security_incidents: usize,
    pub compliance_violations: usize,
    pub failed_logins: usize,
    pub unique_users: usize,
    pub critical_events: usize,
    pub audit_coverage: f64,
}
