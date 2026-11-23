use crate::real_time::MonitoringMessage;
use crate::MonitoringState;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use botsecurity_auth::auth_api::{AuthenticatedUser, Role};
use serde_json::json;
use std::sync::Arc;

/// Pure authorization predicate for the `/ws/logs` upgrade (#2858).
///
/// Kept free of I/O and framework types so the authorization boundary can be
/// unit-tested without a server, a database, or a token. `authenticated` is the
/// auth middleware's verified identity (JWT/session signature and expiry
/// already checked) and `roles` are the DB-resolved RBAC roles on that
/// identity. A socket is admitted only for an authenticated Admin/SuperAdmin:
/// an anonymous or non-admin caller never gets a log stream merely because
/// the RBAC route table marks `/ws/**` GET anonymous.
fn ws_logs_authorized(authenticated: bool, roles: &[Role]) -> bool {
    authenticated
        && roles
            .iter()
            .any(|role| matches!(role, Role::Admin | Role::SuperAdmin))
}

/// Authenticates and authorizes the handshake *before* the upgrade.
///
/// The `AuthenticatedUser` extension is only populated by the auth middleware
/// after it verified the caller's JWT/session, and the RBAC role resolver has
/// already upgraded DB-backed admins by the time this handler runs. Relying on
/// the HTTP path gate alone is not enough here: `/ws/**` is anonymous in the
/// RBAC table, so any authenticated user otherwise reached this stream.
pub async fn ws_logs<S: MonitoringState>(
    State(state): State<Arc<S>>,
    user: Option<Extension<AuthenticatedUser>>,
    ws: WebSocketUpgrade,
) -> Response {
    match user {
        None => (StatusCode::UNAUTHORIZED, "Authentication required").into_response(),
        Some(Extension(user)) if !ws_logs_authorized(user.is_authenticated(), &user.roles) => {
            log::warn!("/ws/logs handshake refused: an authenticated admin is required");
            (
                StatusCode::FORBIDDEN,
                "Admin access required for the monitoring log stream",
            )
                .into_response()
        }
        Some(_) => ws.on_upgrade(move |socket| handle_log_socket(socket, state)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admin_roles_are_admitted_when_authenticated() {
        assert!(ws_logs_authorized(true, &[Role::Admin]));
        assert!(ws_logs_authorized(true, &[Role::SuperAdmin]));
        assert!(ws_logs_authorized(true, &[Role::User, Role::Admin]));
    }

    #[test]
    fn non_admin_roles_are_refused() {
        assert!(!ws_logs_authorized(true, &[]));
        assert!(!ws_logs_authorized(true, &[Role::User]));
        assert!(!ws_logs_authorized(true, &[Role::Moderator]));
        assert!(!ws_logs_authorized(true, &[Role::User, Role::Moderator]));
        assert!(!ws_logs_authorized(true, &[Role::Service, Role::Bot]));
    }

    #[test]
    fn unauthenticated_callers_are_refused_even_with_an_admin_role() {
        assert!(!ws_logs_authorized(false, &[Role::Admin]));
        assert!(!ws_logs_authorized(false, &[Role::SuperAdmin]));
        assert!(!ws_logs_authorized(false, &[]));
    }
}

async fn handle_log_socket<S: MonitoringState>(mut socket: WebSocket, state: Arc<S>) {
    let collector = state.metrics_collector();
    let mut rx = collector.subscribe();

    let welcome = json!({
        "id": "monitoring-0",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "level": "info",
        "service": "monitoring",
        "message": "Monitoring log stream connected",
    });
    if socket
        .send(Message::Text(welcome.to_string()))
        .await
        .is_err()
    {
        return;
    }

    loop {
        tokio::select! {
            incoming = socket.recv() => {
                match incoming {
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
            event = rx.recv() => {
                match event {
                    Ok(MonitoringMessage::AlertFired { alert }) => {
                        let entry = json!({
                            "id": format!("alert-{}", alert.id),
                            "timestamp": alert.started_at.to_rfc3339(),
                            "level": "error",
                            "service": "alerts",
                            "message": format!("[{}] {}", alert.rule_name, alert.message),
                            "context": {
                                "severity": format!("{:?}", alert.severity),
                                "metric": alert.metric_name,
                                "value": alert.metric_value,
                            },
                        });
                        if socket.send(Message::Text(entry.to_string())).await.is_err() {
                            break;
                        }
                    }
                    Ok(MonitoringMessage::HealthUpdate { health }) => {
                        let status = format!("{:?}", health.status);
                        let level = if health.status == crate::HealthStatus::Healthy {
                            "info"
                        } else {
                            "warn"
                        };
                        let entry = json!({
                            "id": format!("health-{}", health.last_check.timestamp_millis()),
                            "timestamp": health.last_check.to_rfc3339(),
                            "level": level,
                            "service": "monitoring",
                            "message": format!(
                                "System health: {} (CPU {:.1}%, Mem {:.1}%, Disk {:.1}%)",
                                status, health.cpu_usage_percent, health.memory_usage_percent, health.disk_usage_percent
                            ),
                            "context": {
                                "cpu": health.cpu_usage_percent,
                                "memory": health.memory_usage_percent,
                                "disk": health.disk_usage_percent,
                                "latency_ms": health.average_latency_ms,
                            },
                        });
                        if socket.send(Message::Text(entry.to_string())).await.is_err() {
                            break;
                        }
                    }
                    Ok(MonitoringMessage::AlertResolved { alert_id }) => {
                        let entry = json!({
                            "id": format!("resolve-{}", alert_id),
                            "timestamp": chrono::Utc::now().to_rfc3339(),
                            "level": "info",
                            "service": "alerts",
                            "message": format!("Alert {alert_id} resolved"),
                        });
                        if socket.send(Message::Text(entry.to_string())).await.is_err() {
                            break;
                        }
                    }
                    Ok(_) => {}
                    Err(_) => break,
                }
            }
        }
    }
}
