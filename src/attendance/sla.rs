use crate::core::shared::schema::people::attendance_sla_events;
use crate::core::shared::schema::people::attendance_sla_policies;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::time::{interval, Duration};
use uuid::Uuid;

pub struct AppState {
    pub conn: diesel_async::Pool<diesel_async::AsyncPgConnection>,
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

pub async fn start_sla_breach_monitor(state: Arc<AppState>) {
    let mut interval_timer = interval(Duration::from_secs(30));
    
    info!("Starting SLA breach monitor");
    
    loop {
        interval_timer.tick().await;
        
        if let Err(e) = check_sla_breaches(&state).await {
            error!("SLA breach check failed: {}", e);
        }
    }
}

async fn check_sla_breaches(state: &Arc<AppState>) -> Result<(), String> {
    let mut conn = state.conn.get().await.map_err(|e| format!("DB pool error: {e}"))?;
    
    let pending_events = attendance_sla_events::table
        .filter(attendance_sla_events::status.eq("pending"))
        .filter(attendance_sla_events::due_at.le(diesel::dsl::now))
        .load::<(Uuid, String, chrono::DateTime<chrono::Utc>)>(&mut conn)
        .await
        .map_err(|e| format!("Query error: {e}"))?;

    if !pending_events.is_empty() {
        info!("Found {} SLA breaches to process", pending_events.len());
    }

    for (event_id, session_id, due_at) in pending_events {
        let breached_at = chrono::Utc::now();
        
        diesel::update(attendance_sla_events::table.filter(attendance_sla_events::id.eq(event_id)))
            .set((
                attendance_sla_events::status.eq("breached"),
                attendance_sla_events::breached_at.eq(Some(breached_at)),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| format!("Update error: {e}"))?;

        info!("SLA breached for session {} (event {})", session_id, event_id);

        let webhook_data = serde_json::json!({
            "event_id": event_id,
            "session_id": session_id,
            "due_at": due_at,
            "breached_at": breached_at
        });

        if let Ok(mut db_conn) = state.conn.get().await {
            crate::attendance::webhooks::emit_webhook_event(
                &mut db_conn,
                uuid::Uuid::nil(),
                "sla.breached",
                webhook_data,
            );
        }
    }

    Ok(())
}

    for (event_id, session_id, due_at) in pending_events {
        let breached_at = chrono::Utc::now();
        
        diesel::update(attendance_sla_events::table.filter(attendance_sla_events::id.eq(event_id)))
            .set((
                attendance_sla_events::status.eq("breached"),
                attendance_sla_events::breached_at.eq(Some(breached_at)),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| format!("Update error: {}", e))?;

        info!("SLA breached for session {} (event {})", session_id, event_id);
    }

    Ok(())
}

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
