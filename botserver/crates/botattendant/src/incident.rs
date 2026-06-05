use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IncidentStatus {
    New,
    Assigned,
    InProgress,
    Resolved,
    Closed,
}

impl IncidentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            IncidentStatus::New => "New",
            IncidentStatus::Assigned => "Assigned",
            IncidentStatus::InProgress => "InProgress",
            IncidentStatus::Resolved => "Resolved",
            IncidentStatus::Closed => "Closed",
        }
    }

    pub fn from_str(s: &str) -> Option<IncidentStatus> {
        match s {
            "New" => Some(IncidentStatus::New),
            "Assigned" => Some(IncidentStatus::Assigned),
            "InProgress" => Some(IncidentStatus::InProgress),
            "Resolved" => Some(IncidentStatus::Resolved),
            "Closed" => Some(IncidentStatus::Closed),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IncidentPriority {
    Critical,
    High,
    Medium,
    Low,
}

impl IncidentPriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            IncidentPriority::Critical => "Critical",
            IncidentPriority::High => "High",
            IncidentPriority::Medium => "Medium",
            IncidentPriority::Low => "Low",
        }
    }

    pub fn from_str(s: &str) -> Option<IncidentPriority> {
        match s {
            "Critical" => Some(IncidentPriority::Critical),
            "High" => Some(IncidentPriority::High),
            "Medium" => Some(IncidentPriority::Medium),
            "Low" => Some(IncidentPriority::Low),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Incident {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub priority: IncidentPriority,
    pub status: IncidentStatus,
    pub category: String,
    pub assignee: Option<String>,
    pub sla_deadline: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateIncidentRequest {
    pub title: String,
    pub description: String,
    pub priority: String,
    pub category: String,
    pub assignee: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateIncidentRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub priority: Option<String>,
    pub status: Option<String>,
    pub category: Option<String>,
    pub assignee: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct IncidentQuery {
    pub status: Option<String>,
    pub priority: Option<String>,
    pub category: Option<String>,
    pub assignee: Option<String>,
}

type Storage = Arc<Mutex<HashMap<Uuid, Incident>>>;

#[derive(Clone)]
pub struct IncidentService {
    storage: Storage,
}

impl IncidentService {
    pub fn new() -> Self {
        IncidentService {
            storage: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn create(&self, req: CreateIncidentRequest) -> Result<Incident, String> {
        let priority = IncidentPriority::from_str(&req.priority)
            .ok_or_else(|| format!("Invalid priority: {}", req.priority))?;
        let id = Uuid::new_v4();
        let now = Utc::now();
        let sla_deadline = match priority {
            IncidentPriority::Critical => Some(now + chrono::Duration::hours(4)),
            IncidentPriority::High => Some(now + chrono::Duration::hours(8)),
            IncidentPriority::Medium => Some(now + chrono::Duration::hours(24)),
            IncidentPriority::Low => Some(now + chrono::Duration::hours(72)),
        };
        let incident = Incident {
            id,
            title: req.title,
            description: req.description,
            priority,
            status: IncidentStatus::New,
            category: req.category,
            assignee: req.assignee,
            sla_deadline,
            created_at: now,
            resolved_at: None,
        };
        let mut store = self.storage.lock().map_err(|e| format!("Lock error: {e}"))?;
        store.insert(id, incident.clone());
        Ok(incident)
    }

    pub fn get(&self, id: Uuid) -> Result<Incident, String> {
        let store = self.storage.lock().map_err(|e| format!("Lock error: {e}"))?;
        store.get(&id).cloned().ok_or_else(|| format!("Incident not found: {id}"))
    }

    pub fn list(&self, query: IncidentQuery) -> Result<Vec<Incident>, String> {
        let store = self.storage.lock().map_err(|e| format!("Lock error: {e}"))?;
        let incidents: Vec<Incident> = store
            .values()
            .filter(|inc| {
                if let Some(ref status_str) = query.status {
                    if let Some(s) = IncidentStatus::from_str(status_str) {
                        if inc.status != s {
                            return false;
                        }
                    }
                }
                if let Some(ref priority_str) = query.priority {
                    if let Some(p) = IncidentPriority::from_str(priority_str) {
                        if inc.priority != p {
                            return false;
                        }
                    }
                }
                if let Some(ref cat) = query.category {
                    if &inc.category != cat {
                        return false;
                    }
                }
                if let Some(ref assignee) = query.assignee {
                    if inc.assignee.as_deref() != Some(assignee) {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();
        Ok(incidents)
    }

    pub fn update(&self, id: Uuid, req: UpdateIncidentRequest) -> Result<Incident, String> {
        let mut store = self.storage.lock().map_err(|e| format!("Lock error: {e}"))?;
        let incident = store.get_mut(&id).ok_or_else(|| format!("Incident not found: {id}"))?;
        if let Some(title) = req.title {
            incident.title = title;
        }
        if let Some(description) = req.description {
            incident.description = description;
        }
        if let Some(ref priority_str) = req.priority {
            let priority = IncidentPriority::from_str(priority_str)
                .ok_or_else(|| format!("Invalid priority: {priority_str}"))?;
            incident.priority = priority;
        }
        if let Some(ref status_str) = req.status {
            let status = IncidentStatus::from_str(status_str)
                .ok_or_else(|| format!("Invalid status: {status_str}"))?;
            incident.status = status;
            if status == IncidentStatus::Resolved || status == IncidentStatus::Closed {
                incident.resolved_at = Some(Utc::now());
            }
        }
        if let Some(category) = req.category {
            incident.category = category;
        }
        if let Some(assignee) = req.assignee {
            incident.assignee = Some(assignee);
        }
        Ok(incident.clone())
    }

    pub fn delete(&self, id: Uuid) -> Result<(), String> {
        let mut store = self.storage.lock().map_err(|e| format!("Lock error: {e}"))?;
        store.remove(&id).ok_or_else(|| format!("Incident not found: {id}"))?;
        Ok(())
    }

    pub fn check_sla_violation(&self, id: Uuid) -> Result<(bool, Option<DateTime<Utc>>), String> {
        let store = self.storage.lock().map_err(|e| format!("Lock error: {e}"))?;
        let incident = store.get(&id).ok_or_else(|| format!("Incident not found: {id}"))?;
        match incident.sla_deadline {
            Some(deadline) => {
                let now = Utc::now();
                let violated = now > deadline
                    && incident.status != IncidentStatus::Resolved
                    && incident.status != IncidentStatus::Closed;
                Ok((violated, Some(deadline)))
            }
            None => Ok((false, None)),
        }
    }
}
