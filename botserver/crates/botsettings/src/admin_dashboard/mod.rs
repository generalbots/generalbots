pub mod activity;
pub mod audit;
pub mod bots;
pub mod export;
pub mod health;
pub mod invitations;
pub mod stats;
pub mod teams;

use axum::{routing::{delete, get, post}, Router};
use diesel::RunQueryDsl;
use std::sync::Arc;

use botcore::shared::state::AppState;

pub fn configure_admin_dashboard_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/admin/dashboard/stats", get(stats::dashboard_stats))
        .route("/api/admin/dashboard/health", get(health::dashboard_health))
        .route("/api/admin/dashboard/activity", get(activity::dashboard_activity))
        .route("/api/admin/dashboard/members", get(teams::dashboard_members))
        .route("/api/admin/dashboard/roles", get(teams::dashboard_roles))
        .route("/api/admin/dashboard/bots", get(teams::dashboard_bots))
        .route("/api/admin/dashboard/invitations", get(invitations::dashboard_invitations))
        .route("/api/admin/invitations/resend/:id", post(invitations::resend_invitation))
        .route("/api/admin/invitations/bulk", post(invitations::bulk_invite))
        .route("/api/admin/invitations/:id", delete(invitations::revoke_invitation))
        .route("/api/admin/invitations", post(invitations::create_invitation))
        .route("/api/admin/export-report", get(export::export_report))
        .route("/api/admin/health", get(health::admin_health))
        .route("/api/admin/activity/recent", get(activity::activity_recent))
        .route("/api/admin/stats/users", get(export::stats_users))
        .route("/api/admin/stats/bots", get(export::stats_bots))
        .route("/api/admin/stats/groups", get(export::stats_groups))
        .route("/api/admin/stats/storage", get(export::stats_storage))
        .route("/api/admin/users", get(export::list_users))
        .route("/api/admin/groups", get(export::list_groups))
        .route("/api/admin/dns", get(export::list_dns))
        .route("/api/admin/invitations", get(invitations::admin_invitations))
        .route("/api/admin/bots", get(bots::admin_bots))
        .route("/api/admin/audit", get(audit::admin_audit))
}

#[derive(Debug, diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CountResult {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub count: i64,
}

pub fn count_query(conn: &mut diesel::PgConnection, sql: &str) -> i64 {
    diesel::sql_query(sql)
        .get_result::<CountResult>(conn)
        .map(|r| r.count)
        .unwrap_or(0)
}

pub fn get_conn(
    state: &Arc<AppState>,
) -> Option<diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>> {
    state.conn.get().ok()
}

pub fn format_number(value: i64) -> String {
    if value >= 1_000_000 {
        format!("{:.1}M", value as f64 / 1_000_000.0)
    } else if value >= 1_000 {
        format!("{:.1}K", value as f64 / 1_000.0)
    } else {
        value.to_string()
    }
}
