use crate::schema::*;
use crate::AttendanceConfig;
use diesel::prelude::*;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::time::{interval, Duration};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSlaPolicyRequest {
    pub name: String,
    pub channel: Option<String>,
    pub priority: Option<String>,
    pub first_response_minutes: Option<i32>,
    pub resolution_minutes: Option<i32>,
    pub escalate_on_breach: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSlaEventRequest {
    pub session_id: Uuid,
    pub sla_policy_id: Uuid,
    pub event_type: String,
    pub due_at: chrono::DateTime<chrono::Utc>,
}

pub async fn start_sla_breach_monitor(config: Arc<AttendanceConfig>) {
    let mut interval_timer = interval(Duration::from_secs(30));
    info!("Starting SLA breach monitor");
    loop {
        interval_timer.tick().await;
        if let Err(e) = check_sla_breaches(&config).await {
            error!("SLA breach check failed: {}", e);
        }
    }
}

async fn check_sla_breaches(config: &Arc<AttendanceConfig>) -> Result<(), String> {
    let pool = config.pool.clone();
    let pending_events = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| format!("DB pool error: {e}"))?;
        let events: Vec<(Uuid, String, chrono::DateTime<chrono::Utc>)> = attendance_sla_events::table
            .filter(attendance_sla_events::status.eq("pending"))
            .filter(attendance_sla_events::due_at.le(diesel::dsl::now))
            .select((
                attendance_sla_events::id,
                attendance_sla_events::event_type,
                attendance_sla_events::due_at,
            ))
            .load(&mut conn)
            .map_err(|e| format!("Query error: {e}"))?;
        Ok::<Vec<(Uuid, String, chrono::DateTime<chrono::Utc>)>, String>(events)
    })
    .await
    .map_err(|e| format!("Task error: {e}"))??;

    if !pending_events.is_empty() {
        info!("Found {} SLA breaches to process", pending_events.len());
    }

    for (event_id, _session_id, due_at) in pending_events {
        let breached_at = chrono::Utc::now();
        let pool = config.pool.clone();
        let update_result = tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|e| format!("DB pool error: {e}"))?;
            diesel::update(attendance_sla_events::table.filter(attendance_sla_events::id.eq(event_id)))
                .set((
                    attendance_sla_events::status.eq("breached"),
                    attendance_sla_events::breached_at.eq(Some(breached_at)),
                ))
                .execute(&mut conn)
                .map_err(|e| format!("Update error: {e}"))?;
            info!("SLA breached for event {} (due_at {})", event_id, due_at);
            Ok::<(), String>(())
        })
        .await;

        if let Err(e) = update_result {
            error!("SLA breach update failed for event {}: {:?}", event_id, e);
        }

        let pool = config.pool.clone();
        let _ = tokio::task::spawn_blocking(move || {
            if let Ok(mut db_conn) = pool.get() {
                crate::webhooks::emit_webhook_event(
                    &mut db_conn,
                    Uuid::nil(),
                    "sla.breached",
                    serde_json::json!({
                        "event_id": event_id,
                        "due_at": due_at,
                        "breached_at": breached_at
                    }),
                );
            }
        })
        .await;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_sla_policy_request_default_values() {
        let req = CreateSlaPolicyRequest {
            name: "Test Policy".to_string(),
            channel: None,
            priority: Some("high".to_string()),
            first_response_minutes: Some(15),
            resolution_minutes: Some(240),
            escalate_on_breach: Some(true),
        };
        assert_eq!(req.name, "Test Policy");
        assert_eq!(req.priority, Some("high".to_string()));
        assert_eq!(req.first_response_minutes, Some(15));
    }

    #[test]
    fn test_create_sla_event_request() {
        let req = CreateSlaEventRequest {
            session_id: Uuid::new_v4(),
            sla_policy_id: Uuid::new_v4(),
            event_type: "first_response".to_string(),
            due_at: chrono::Utc::now(),
        };
        assert_eq!(req.event_type, "first_response");
    }
}
