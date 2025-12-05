//! Transfer to Human Keyword
//!
//! Provides the TRANSFER TO HUMAN keyword for bot-to-human handoff in conversations.
//! This is a critical feature for hybrid bot/human support workflows.
//!
//! ## Features
//!
//! - Transfer to any available attendant
//! - Transfer to specific person by name or alias
//! - Transfer to specific department
//! - Priority-based queue placement
//! - Context passing for seamless handoff
//!
//! ## Configuration
//!
//! Requires `crm-enabled = true` in the bot's config.csv file.
//! Attendants are configured in attendant.csv in the bot's .gbai folder.
//!
//! ## Usage in BASIC
//!
//! ```basic
//! ' Transfer to any available human
//! TRANSFER TO HUMAN
//!
//! ' Transfer to specific person
//! TRANSFER TO HUMAN "John Smith"
//!
//! ' Transfer to department
//! TRANSFER TO HUMAN department: "sales"
//!
//! ' Transfer with priority and context
//! TRANSFER TO HUMAN "support", "high", "Customer needs help with billing"
//! ```
//!
//! ## As LLM Tool
//!
//! This keyword is also registered as an LLM tool, allowing the AI to
//! automatically transfer conversations when appropriate.

use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use chrono::Utc;
use diesel::prelude::*;
use log::{debug, error, info, warn};
use rhai::{Dynamic, Engine, Map};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

/// Transfer request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferToHumanRequest {
    /// Optional name or alias of the person to transfer to
    pub name: Option<String>,
    /// Optional department to transfer to
    pub department: Option<String>,
    /// Priority level: "normal", "high", "urgent"
    pub priority: Option<String>,
    /// Reason for the transfer (passed to attendant)
    pub reason: Option<String>,
    /// Additional context from the conversation
    pub context: Option<String>,
}

/// Transfer result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferResult {
    pub success: bool,
    pub status: TransferStatus,
    pub queue_position: Option<i32>,
    pub assigned_to: Option<String>,
    pub assigned_to_name: Option<String>,
    pub estimated_wait_seconds: Option<i32>,
    pub message: String,
}

/// Transfer status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferStatus {
    /// Queued for next available attendant
    Queued,
    /// Assigned to specific attendant
    Assigned,
    /// Attendant is online and ready
    Connected,
    /// No attendants available
    NoAttendants,
    /// CRM not enabled
    CrmDisabled,
    /// Specified attendant not found
    AttendantNotFound,
    /// Error during transfer
    Error,
}

/// Attendant information from CSV
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attendant {
    pub id: String,
    pub name: String,
    pub channel: String,
    pub preferences: String,
    pub department: Option<String>,
    pub aliases: Vec<String>,
    pub status: AttendantStatus,
}

/// Attendant status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AttendantStatus {
    Online,
    Busy,
    Away,
    Offline,
}

impl Default for AttendantStatus {
    fn default() -> Self {
        AttendantStatus::Offline
    }
}

/// Check if CRM is enabled in bot's config.csv
pub fn is_crm_enabled(bot_id: Uuid, work_path: &str) -> bool {
    let config_path = PathBuf::from(work_path)
        .join(format!("{}.gbai", bot_id))
        .join("config.csv");

    if !config_path.exists() {
        // Also try without UUID prefix
        let alt_path = PathBuf::from(work_path).join("config.csv");
        if alt_path.exists() {
            return check_config_for_crm(&alt_path);
        }
        warn!("Config file not found: {:?}", config_path);
        return false;
    }

    check_config_for_crm(&config_path)
}

fn check_config_for_crm(config_path: &PathBuf) -> bool {
    match std::fs::read_to_string(config_path) {
        Ok(content) => {
            for line in content.lines() {
                let line_lower = line.to_lowercase();
                // Check for crm-enabled = true or crm_enabled = true
                if (line_lower.contains("crm-enabled") || line_lower.contains("crm_enabled"))
                    && line_lower.contains("true")
                {
                    return true;
                }
                // Also support legacy transfer = true
                if line_lower.contains("transfer") && line_lower.contains("true") {
                    return true;
                }
            }
            false
        }
        Err(e) => {
            error!("Failed to read config file: {}", e);
            false
        }
    }
}

/// Read attendants from attendant.csv
pub fn read_attendants(bot_id: Uuid, work_path: &str) -> Vec<Attendant> {
    let attendant_path = PathBuf::from(work_path)
        .join(format!("{}.gbai", bot_id))
        .join("attendant.csv");

    if !attendant_path.exists() {
        // Try alternate path
        let alt_path = PathBuf::from(work_path).join("attendant.csv");
        if alt_path.exists() {
            return parse_attendants_csv(&alt_path);
        }
        warn!("Attendant file not found: {:?}", attendant_path);
        return Vec::new();
    }

    parse_attendants_csv(&attendant_path)
}

fn parse_attendants_csv(path: &PathBuf) -> Vec<Attendant> {
    match std::fs::read_to_string(path) {
        Ok(content) => {
            let mut attendants = Vec::new();
            let mut lines = content.lines();

            // Skip header
            let header = lines.next().unwrap_or("");
            let headers: Vec<String> = header.split(',').map(|s| s.trim().to_lowercase()).collect();

            // Find column indices
            let id_idx = headers.iter().position(|h| h == "id").unwrap_or(0);
            let name_idx = headers.iter().position(|h| h == "name").unwrap_or(1);
            let channel_idx = headers.iter().position(|h| h == "channel").unwrap_or(2);
            let pref_idx = headers.iter().position(|h| h == "preferences").unwrap_or(3);
            let dept_idx = headers.iter().position(|h| h == "department");
            let alias_idx = headers.iter().position(|h| h == "aliases" || h == "alias");

            for line in lines {
                if line.trim().is_empty() {
                    continue;
                }

                let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                if parts.len() >= 4 {
                    let department = dept_idx.and_then(|i| parts.get(i).map(|s| s.to_string()));
                    let aliases = alias_idx
                        .and_then(|i| parts.get(i))
                        .map(|s| s.split(';').map(|a| a.trim().to_lowercase()).collect())
                        .unwrap_or_default();

                    attendants.push(Attendant {
                        id: parts.get(id_idx).unwrap_or(&"").to_string(),
                        name: parts.get(name_idx).unwrap_or(&"").to_string(),
                        channel: parts.get(channel_idx).unwrap_or(&"all").to_string(),
                        preferences: parts.get(pref_idx).unwrap_or(&"").to_string(),
                        department,
                        aliases,
                        status: AttendantStatus::Online, // Default to online, will be updated from DB
                    });
                }
            }

            info!("Loaded {} attendants from CSV", attendants.len());
            attendants
        }
        Err(e) => {
            error!("Failed to read attendant file: {}", e);
            Vec::new()
        }
    }
}

/// Find attendant by name, alias, or department
pub fn find_attendant<'a>(
    attendants: &'a [Attendant],
    name: Option<&str>,
    department: Option<&str>,
) -> Option<&'a Attendant> {
    if let Some(search_name) = name {
        let search_lower = search_name.to_lowercase();

        // First try exact name match
        if let Some(att) = attendants
            .iter()
            .find(|a| a.name.to_lowercase() == search_lower)
        {
            return Some(att);
        }

        // Try partial name match
        if let Some(att) = attendants
            .iter()
            .find(|a| a.name.to_lowercase().contains(&search_lower))
        {
            return Some(att);
        }

        // Try alias match
        if let Some(att) = attendants
            .iter()
            .find(|a| a.aliases.contains(&search_lower))
        {
            return Some(att);
        }

        // Try ID match
        if let Some(att) = attendants
            .iter()
            .find(|a| a.id.to_lowercase() == search_lower)
        {
            return Some(att);
        }
    }

    if let Some(dept) = department {
        let dept_lower = dept.to_lowercase();

        // Find first online attendant in department
        if let Some(att) = attendants.iter().find(|a| {
            a.department
                .as_ref()
                .map(|d| d.to_lowercase() == dept_lower)
                .unwrap_or(false)
                && a.status == AttendantStatus::Online
        }) {
            return Some(att);
        }

        // Try preferences match for department
        if let Some(att) = attendants.iter().find(|a| {
            a.preferences.to_lowercase().contains(&dept_lower)
                && a.status == AttendantStatus::Online
        }) {
            return Some(att);
        }
    }

    // Return first online attendant if no specific match
    attendants
        .iter()
        .find(|a| a.status == AttendantStatus::Online)
}

/// Priority to integer for queue ordering
fn priority_to_int(priority: Option<&str>) -> i32 {
    match priority.map(|p| p.to_lowercase()).as_deref() {
        Some("urgent") => 3,
        Some("high") => 2,
        Some("normal") | None => 1,
        Some("low") => 0,
        _ => 1,
    }
}

/// Execute the transfer to human
pub async fn execute_transfer(
    state: Arc<AppState>,
    session: &UserSession,
    request: TransferToHumanRequest,
) -> TransferResult {
    let work_path = "./work";
    let bot_id = session.bot_id;

    // Check if CRM is enabled
    if !is_crm_enabled(bot_id, work_path) {
        return TransferResult {
            success: false,
            status: TransferStatus::CrmDisabled,
            queue_position: None,
            assigned_to: None,
            assigned_to_name: None,
            estimated_wait_seconds: None,
            message: "CRM features are not enabled. Add 'crm-enabled,true' to config.csv"
                .to_string(),
        };
    }

    // Load attendants
    let attendants = read_attendants(bot_id, work_path);
    if attendants.is_empty() {
        return TransferResult {
            success: false,
            status: TransferStatus::NoAttendants,
            queue_position: None,
            assigned_to: None,
            assigned_to_name: None,
            estimated_wait_seconds: None,
            message: "No attendants configured. Create attendant.csv in your .gbai folder"
                .to_string(),
        };
    }

    // Find matching attendant
    let attendant = find_attendant(
        &attendants,
        request.name.as_deref(),
        request.department.as_deref(),
    );

    // If specific name was requested but not found
    if request.name.is_some() && attendant.is_none() {
        return TransferResult {
            success: false,
            status: TransferStatus::AttendantNotFound,
            queue_position: None,
            assigned_to: None,
            assigned_to_name: None,
            estimated_wait_seconds: None,
            message: format!(
                "Attendant '{}' not found. Available attendants: {}",
                request.name.as_ref().unwrap(),
                attendants
                    .iter()
                    .map(|a| a.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        };
    }

    // Update session to mark as needing human attention
    let priority = priority_to_int(request.priority.as_deref());
    let transfer_context = serde_json::json!({
        "transfer_requested_at": Utc::now().to_rfc3339(),
        "transfer_priority": priority,
        "transfer_reason": request.reason.clone().unwrap_or_default(),
        "transfer_context": request.context.clone().unwrap_or_default(),
        "transfer_to_name": request.name.clone(),
        "transfer_to_department": request.department.clone(),
        "needs_human": true,
        "assigned_to": attendant.as_ref().map(|a| a.id.clone()),
        "assigned_to_name": attendant.as_ref().map(|a| a.name.clone()),
        "status": if attendant.is_some() { "assigned" } else { "queued" },
    });

    // Update session in database
    let session_id = session.id;
    let conn = state.conn.clone();
    let ctx_data = transfer_context.clone();

    let update_result = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn
            .get()
            .map_err(|e| format!("DB connection error: {}", e))?;

        use crate::shared::models::schema::user_sessions;

        diesel::update(user_sessions::table.filter(user_sessions::id.eq(session_id)))
            .set(user_sessions::context_data.eq(ctx_data))
            .execute(&mut db_conn)
            .map_err(|e| format!("Failed to update session: {}", e))
    })
    .await;

    match update_result {
        Ok(Ok(_)) => {
            if let Some(att) = attendant {
                info!(
                    "Transfer: Session {} assigned to {} ({})",
                    session.id, att.name, att.id
                );
                TransferResult {
                    success: true,
                    status: TransferStatus::Assigned,
                    queue_position: Some(1),
                    assigned_to: Some(att.id.clone()),
                    assigned_to_name: Some(att.name.clone()),
                    estimated_wait_seconds: Some(30),
                    message: format!(
                        "You have been connected to {}. They will be with you shortly.",
                        att.name
                    ),
                }
            } else {
                info!(
                    "Transfer: Session {} queued for next available attendant",
                    session.id
                );
                TransferResult {
                    success: true,
                    status: TransferStatus::Queued,
                    queue_position: Some(1), // TODO: Calculate actual position
                    assigned_to: None,
                    assigned_to_name: None,
                    estimated_wait_seconds: Some(120),
                    message: "You have been added to the queue. The next available attendant will assist you.".to_string(),
                }
            }
        }
        Ok(Err(e)) => {
            error!("Transfer failed: {}", e);
            TransferResult {
                success: false,
                status: TransferStatus::Error,
                queue_position: None,
                assigned_to: None,
                assigned_to_name: None,
                estimated_wait_seconds: None,
                message: format!("Transfer failed: {}", e),
            }
        }
        Err(e) => {
            error!("Transfer task failed: {:?}", e);
            TransferResult {
                success: false,
                status: TransferStatus::Error,
                queue_position: None,
                assigned_to: None,
                assigned_to_name: None,
                estimated_wait_seconds: None,
                message: format!("Transfer task failed: {:?}", e),
            }
        }
    }
}

/// Convert TransferResult to Rhai Dynamic
impl TransferResult {
    pub fn to_dynamic(&self) -> Dynamic {
        let mut map = Map::new();
        map.insert("success".into(), Dynamic::from(self.success));
        map.insert(
            "status".into(),
            Dynamic::from(format!("{:?}", self.status).to_lowercase()),
        );
        map.insert("message".into(), Dynamic::from(self.message.clone()));

        if let Some(pos) = self.queue_position {
            map.insert("queue_position".into(), Dynamic::from(pos));
        }
        if let Some(ref id) = self.assigned_to {
            map.insert("assigned_to".into(), Dynamic::from(id.clone()));
        }
        if let Some(ref name) = self.assigned_to_name {
            map.insert("assigned_to_name".into(), Dynamic::from(name.clone()));
        }
        if let Some(wait) = self.estimated_wait_seconds {
            map.insert("estimated_wait_seconds".into(), Dynamic::from(wait));
        }

        Dynamic::from(map)
    }
}

/// Register the TRANSFER TO HUMAN keyword with the Rhai engine
pub fn register_transfer_to_human_keyword(
    state: Arc<AppState>,
    user: UserSession,
    engine: &mut Engine,
) {
    // TRANSFER TO HUMAN (no arguments - any available)
    let state_clone = state.clone();
    let user_clone = user.clone();
    engine.register_fn("transfer_to_human", move || -> Dynamic {
        let state = state_clone.clone();
        let session = user_clone.clone();

        let rt = tokio::runtime::Handle::current();
        let result = rt.block_on(async {
            execute_transfer(
                state,
                &session,
                TransferToHumanRequest {
                    name: None,
                    department: None,
                    priority: None,
                    reason: None,
                    context: None,
                },
            )
            .await
        });

        result.to_dynamic()
    });

    // TRANSFER TO HUMAN "name"
    let state_clone = state.clone();
    let user_clone = user.clone();
    engine.register_fn("transfer_to_human", move |name: &str| -> Dynamic {
        let state = state_clone.clone();
        let session = user_clone.clone();
        let name_str = name.to_string();

        let rt = tokio::runtime::Handle::current();
        let result = rt.block_on(async {
            execute_transfer(
                state,
                &session,
                TransferToHumanRequest {
                    name: Some(name_str),
                    department: None,
                    priority: None,
                    reason: None,
                    context: None,
                },
            )
            .await
        });

        result.to_dynamic()
    });

    // TRANSFER TO HUMAN "department", "priority"
    let state_clone = state.clone();
    let user_clone = user.clone();
    engine.register_fn(
        "transfer_to_human",
        move |department: &str, priority: &str| -> Dynamic {
            let state = state_clone.clone();
            let session = user_clone.clone();
            let dept = department.to_string();
            let prio = priority.to_string();

            let rt = tokio::runtime::Handle::current();
            let result = rt.block_on(async {
                execute_transfer(
                    state,
                    &session,
                    TransferToHumanRequest {
                        name: None,
                        department: Some(dept),
                        priority: Some(prio),
                        reason: None,
                        context: None,
                    },
                )
                .await
            });

            result.to_dynamic()
        },
    );

    // TRANSFER TO HUMAN "department", "priority", "reason"
    let state_clone = state.clone();
    let user_clone = user.clone();
    engine.register_fn(
        "transfer_to_human",
        move |department: &str, priority: &str, reason: &str| -> Dynamic {
            let state = state_clone.clone();
            let session = user_clone.clone();
            let dept = department.to_string();
            let prio = priority.to_string();
            let rsn = reason.to_string();

            let rt = tokio::runtime::Handle::current();
            let result = rt.block_on(async {
                execute_transfer(
                    state,
                    &session,
                    TransferToHumanRequest {
                        name: None,
                        department: Some(dept),
                        priority: Some(prio),
                        reason: Some(rsn),
                        context: None,
                    },
                )
                .await
            });

            result.to_dynamic()
        },
    );

    // TRANSFER TO HUMAN with Map (for named parameters)
    let state_clone = state.clone();
    let user_clone = user.clone();
    engine.register_fn("transfer_to_human_ex", move |params: Map| -> Dynamic {
        let state = state_clone.clone();
        let session = user_clone.clone();

        let name = params
            .get("name")
            .and_then(|v| v.clone().try_cast::<String>());
        let department = params
            .get("department")
            .and_then(|v| v.clone().try_cast::<String>());
        let priority = params
            .get("priority")
            .and_then(|v| v.clone().try_cast::<String>());
        let reason = params
            .get("reason")
            .and_then(|v| v.clone().try_cast::<String>());
        let context = params
            .get("context")
            .and_then(|v| v.clone().try_cast::<String>());

        let rt = tokio::runtime::Handle::current();
        let result = rt.block_on(async {
            execute_transfer(
                state,
                &session,
                TransferToHumanRequest {
                    name,
                    department,
                    priority,
                    reason,
                    context,
                },
            )
            .await
        });

        result.to_dynamic()
    });

    debug!("Registered TRANSFER TO HUMAN keywords");
}

/// Tool schema for LLM integration
pub const TRANSFER_TO_HUMAN_TOOL_SCHEMA: &str = r#"{
    "name": "transfer_to_human",
    "description": "Transfer the conversation to a human attendant. Use this when the customer explicitly asks to speak with a person, when the issue is too complex for automated handling, or when emotional support is needed.",
    "parameters": {
        "type": "object",
        "properties": {
            "name": {
                "type": "string",
                "description": "If someone wants to talk to somebody specific, provide their name or alias. Leave empty for any available attendant."
            },
            "department": {
                "type": "string",
                "description": "Department to transfer to: sales, support, technical, billing, etc.",
                "enum": ["sales", "support", "technical", "billing", "general"]
            },
            "priority": {
                "type": "string",
                "description": "Priority level for the transfer",
                "enum": ["normal", "high", "urgent"],
                "default": "normal"
            },
            "reason": {
                "type": "string",
                "description": "Brief reason for the transfer to help the attendant understand the context"
            }
        },
        "required": []
    }
}"#;

/// Get the tool definition for registration with LLM
pub fn get_tool_definition() -> serde_json::Value {
    serde_json::json!({
        "type": "function",
        "function": {
            "name": "transfer_to_human",
            "description": "Transfer the conversation to a human attendant. Use this when the customer explicitly asks to speak with a person, when the issue is too complex for automated handling, or when emotional support is needed.",
            "parameters": {
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "If someone wants to talk to somebody specific, provide their name or alias. Leave empty for any available attendant."
                    },
                    "department": {
                        "type": "string",
                        "description": "Department to transfer to: sales, support, technical, billing, etc."
                    },
                    "priority": {
                        "type": "string",
                        "description": "Priority level for the transfer: normal, high, or urgent",
                        "default": "normal"
                    },
                    "reason": {
                        "type": "string",
                        "description": "Brief reason for the transfer to help the attendant understand the context"
                    }
                },
                "required": []
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_to_int() {
        assert_eq!(priority_to_int(Some("urgent")), 3);
        assert_eq!(priority_to_int(Some("high")), 2);
        assert_eq!(priority_to_int(Some("normal")), 1);
        assert_eq!(priority_to_int(Some("low")), 0);
        assert_eq!(priority_to_int(None), 1);
    }

    #[test]
    fn test_find_attendant_by_name() {
        let attendants = vec![
            Attendant {
                id: "att-001".to_string(),
                name: "John Smith".to_string(),
                channel: "all".to_string(),
                preferences: "sales".to_string(),
                department: Some("commercial".to_string()),
                aliases: vec!["johnny".to_string(), "js".to_string()],
                status: AttendantStatus::Online,
            },
            Attendant {
                id: "att-002".to_string(),
                name: "Jane Doe".to_string(),
                channel: "web".to_string(),
                preferences: "support".to_string(),
                department: Some("customer-service".to_string()),
                aliases: vec![],
                status: AttendantStatus::Online,
            },
        ];

        // Find by exact name
        let found = find_attendant(&attendants, Some("John Smith"), None);
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, "att-001");

        // Find by partial name
        let found = find_attendant(&attendants, Some("john"), None);
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, "att-001");

        // Find by alias
        let found = find_attendant(&attendants, Some("johnny"), None);
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, "att-001");

        // Find by department
        let found = find_attendant(&attendants, None, Some("customer-service"));
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, "att-002");
    }

    #[test]
    fn test_transfer_result_to_dynamic() {
        let result = TransferResult {
            success: true,
            status: TransferStatus::Assigned,
            queue_position: Some(1),
            assigned_to: Some("att-001".to_string()),
            assigned_to_name: Some("John Smith".to_string()),
            estimated_wait_seconds: Some(30),
            message: "Connected to John".to_string(),
        };

        let dynamic = result.to_dynamic();
        let map = dynamic.try_cast::<Map>().unwrap();

        assert_eq!(
            map.get("success")
                .unwrap()
                .clone()
                .try_cast::<bool>()
                .unwrap(),
            true
        );
        assert_eq!(
            map.get("assigned_to_name")
                .unwrap()
                .clone()
                .try_cast::<String>()
                .unwrap(),
            "John Smith"
        );
    }
}
