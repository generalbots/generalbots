use axum::{extract::State, response::Html, routing::get, Router};
use chrono::Local;
use std::sync::Arc;
#[cfg(feature = "monitoring")]
use sysinfo::{Disks, Networks, System};

use crate::core::urls::ApiUrls;
use crate::shared::state::AppState;

pub mod real_time;
pub mod tracing;

pub fn configure() -> Router<Arc<AppState>> {
Router::new()
.route(ApiUrls::MONITORING_DASHBOARD, get(dashboard))
.route(ApiUrls::MONITORING_SERVICES, get(services))
.route(ApiUrls::MONITORING_RESOURCES, get(resources))
.route(ApiUrls::MONITORING_LOGS, get(logs))
.route(ApiUrls::MONITORING_LLM, get(llm_metrics))
.route(ApiUrls::MONITORING_HEALTH, get(health))
// Additional endpoints expected by the frontend
.route("/api/ui/monitoring/timestamp", get(timestamp))
.route("/api/ui/monitoring/bots", get(bots))
.route("/api/ui/monitoring/services/status", get(services_status))
.route("/api/ui/monitoring/resources/bars", get(resources_bars))
.route("/api/ui/monitoring/activity/latest", get(activity_latest))
.route("/api/ui/monitoring/metric/sessions", get(metric_sessions))
.route("/api/ui/monitoring/metric/messages", get(metric_messages))
.route("/api/ui/monitoring/metric/response_time", get(metric_response_time))
.route("/api/ui/monitoring/trend/sessions", get(trend_sessions))
.route("/api/ui/monitoring/rate/messages", get(rate_messages))
// Aliases for frontend compatibility
.route("/api/ui/monitoring/sessions", get(sessions_panel))
.route("/api/ui/monitoring/messages", get(messages_panel))
}

async fn dashboard(State(state): State<Arc<AppState>>) -> Html<String> {
#[cfg(feature = "monitoring")]
let (cpu_usage, total_memory, used_memory, memory_percent, uptime_str) = {
let mut sys = System::new_all();
sys.refresh_all();

    let cpu_usage = sys.global_cpu_usage();
    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();
    let memory_percent = if total_memory > 0 {
        (used_memory as f64 / total_memory as f64) * 100.0
    } else {
        0.0
    };

    let uptime = System::uptime();
    let uptime_str = format_uptime(uptime);
    
    (cpu_usage, total_memory, used_memory, memory_percent, uptime_str)
};

#[cfg(not(feature = "monitoring"))]
let (cpu_usage, total_memory, used_memory, memory_percent, uptime_str) = (
    0.0, 0, 0, 0.0, "N/A".to_string()
);

let active_sessions = state
    .session_manager
    .try_lock()
    .map(|sm| sm.active_count())
    .unwrap_or(0);

Html(format!(
    r##"<div class="dashboard-grid">
<div class="metric-card">
    <div class="metric-header">
        <span class="metric-title">CPU Usage</span>
        <span class="metric-badge {cpu_status}">{cpu_usage:.1}%</span>
    </div>
    <div class="metric-value">{cpu_usage:.1}%</div>
    <div class="metric-bar">
        <div class="metric-bar-fill" style="width: {cpu_usage}%"></div>
    </div>
</div>

<div class="metric-card">
    <div class="metric-header">
        <span class="metric-title">Memory</span>
        <span class="metric-badge {mem_status}">{memory_percent:.1}%</span>
    </div>
    <div class="metric-value">{used_gb:.1} GB / {total_gb:.1} GB</div>
    <div class="metric-bar">
        <div class="metric-bar-fill" style="width: {memory_percent}%"></div>
    </div>
</div>

<div class="metric-card">
    <div class="metric-header">
        <span class="metric-title">Active Sessions</span>
    </div>
    <div class="metric-value">{active_sessions}</div>
    <div class="metric-subtitle">Current conversations</div>
</div>

<div class="metric-card">
    <div class="metric-header">
        <span class="metric-title">Uptime</span>
    </div>
    <div class="metric-value">{uptime_str}</div>
    <div class="metric-subtitle">System running time</div>
</div>
</div><div class="refresh-indicator" hx-get="/api/monitoring/dashboard" hx-trigger="every 10s" hx-swap="outerHTML" hx-target="closest .dashboard-grid, .refresh-indicator"> <span class="refresh-dot"></span> Auto-refreshing </div>"##, cpu_status = if cpu_usage > 80.0 { "danger" } else if cpu_usage > 60.0 { "warning" } else { "success" }, mem_status = if memory_percent > 80.0 { "danger" } else if memory_percent > 60.0 { "warning" } else { "success" }, used_gb = used_memory as f64 / 1_073_741_824.0, total_gb = total_memory as f64 / 1_073_741_824.0, )) }

async fn services(State(_state): State<Arc<AppState>>) -> Html<String> {
let services = vec![
("PostgreSQL", check_postgres(), "Database"),
("Redis", check_redis(), "Cache"),
("MinIO", check_minio(), "Storage"),
("LLM Server", check_llm(), "AI Backend"),
];

let mut rows = String::new();
for (name, status, desc) in services {
    let (status_class, status_text) = if status {
        ("success", "Running")
    } else {
        ("danger", "Stopped")
    };

    rows.push_str(&format!(
        r##"<tr>
<td>
    <div class="service-name">
        <span class="status-dot {status_class}"></span>
        {name}
    </div>
</td>
<td>{desc}</td>
<td><span class="status-badge {status_class}">{status_text}</span></td>
<td>
    <button class="btn-sm" hx-post="/api/monitoring/services/{name_lower}/restart" hx-swap="none">Restart</button>
</td>
</tr>"##, name_lower = name.to_lowercase().replace(' ', "-"), )); }
Html(format!(
    r##"<div class="services-view">
<div class="section-header">
    <h2>Services Status</h2>
    <button class="btn-secondary" hx-get="/api/monitoring/services" hx-target="#monitoring-content" hx-swap="innerHTML">
        Refresh
    </button>
</div>
<table class="data-table">
    <thead>
        <tr>
            <th>Service</th>
            <th>Description</th>
            <th>Status</th>
            <th>Actions</th>
        </tr>
    </thead>
    <tbody>
        {rows}
    </tbody>
</table>
</div>"## )) }

async fn resources(State(_state): State<Arc<AppState>>) -> Html<String> {
#[cfg(feature = "monitoring")]
let (disk_rows, net_rows) = {
let mut sys = System::new_all();
sys.refresh_all();

    let disks = Disks::new_with_refreshed_list();
    let mut disk_rows = String::new();

    for disk in disks.list() {
        let total = disk.total_space();
        let available = disk.available_space();
        let used = total - available;
        let percent = if total > 0 {
            (used as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        disk_rows.push_str(&format!(
            r##"<tr>
<td>{mount}</td>
<td>{used_gb:.1} GB</td>
<td>{total_gb:.1} GB</td>
<td>
    <div class="usage-bar">
        <div class="usage-fill {status}" style="width: {percent:.0}%"></div>
    </div>
    <span class="usage-text">{percent:.1}%</span>
</td>
</tr>"##, mount = disk.mount_point().display(), used_gb = used as f64 / 1_073_741_824.0, total_gb = total as f64 / 1_073_741_824.0, status = if percent > 90.0 { "danger" } else if percent > 70.0 { "warning" } else { "success" }, )); }
    let networks = Networks::new_with_refreshed_list();
    let mut net_rows = String::new();

    for (name, data) in networks.list() {
        net_rows.push_str(&format!(
            r##"<tr>
<td>{name}</td>
<td>{rx:.2} MB</td>
<td>{tx:.2} MB</td>
</tr>"##, rx = data.total_received() as f64 / 1_048_576.0, tx = data.total_transmitted() as f64 / 1_048_576.0, )); }
    (disk_rows, net_rows)
};

#[cfg(not(feature = "monitoring"))]
let (disk_rows, net_rows) = (
    String::new(),
    String::new()
);

Html(format!(
    r##"<div class="resources-view">
<div class="section-header">
    <h2>System Resources</h2>
</div>

<div class="resource-section">
    <h3>Disk Usage</h3>
    <table class="data-table">
        <thead>
            <tr>
                <th>Mount</th>
                <th>Used</th>
                <th>Total</th>
                <th>Usage</th>
            </tr>
        </thead>
        <tbody>
            {disk_rows}
        </tbody>
    </table>
</div>

<div class="resource-section">
    <h3>Network</h3>
    <table class="data-table">
        <thead>
            <tr>
                <th>Interface</th>
                <th>Received</th>
                <th>Transmitted</th>
            </tr>
        </thead>
        <tbody>
            {net_rows}
        </tbody>
    </table>
</div>
</div>"## )) }

async fn logs(State(_state): State<Arc<AppState>>) -> Html<String> {
Html(
r##"<div class="logs-view">
<div class="section-header">
<h2>System Logs</h2>
<div class="log-controls">
<select id="log-level" onchange="filterLogs(this.value)">
<option value="all">All Levels</option>
<option value="error">Error</option>
<option value="warn">Warning</option>
<option value="info">Info</option>
<option value="debug">Debug</option>
</select>
<button class="btn-secondary" onclick="clearLogs()">Clear</button>
</div>
</div>
<div class="log-container" id="log-container"
hx-get="/api/monitoring/logs/stream"
hx-trigger="every 2s"
hx-swap="beforeend scroll:bottom">
<div class="log-entry info">
<span class="log-time">System ready</span>
<span class="log-level">INFO</span>
<span class="log-message">Monitoring initialized</span>
</div>
</div>

</div>"## .to_string(), ) }

async fn llm_metrics(State(_state): State<Arc<AppState>>) -> Html<String> {
Html(
r##"<div class="llm-metrics-view">
<div class="section-header">
<h2>LLM Metrics</h2>
</div>

<div class="metrics-grid">
    <div class="metric-card">
        <div class="metric-title">Total Requests</div>
        <div class="metric-value" id="llm-total-requests"
             hx-get="/api/monitoring/llm/total"
             hx-trigger="load, every 30s"
             hx-swap="innerHTML">
            --
        </div>
    </div>

    <div class="metric-card">
        <div class="metric-title">Cache Hit Rate</div>
        <div class="metric-value" id="llm-cache-rate"
             hx-get="/api/monitoring/llm/cache-rate"
             hx-trigger="load, every 30s"
             hx-swap="innerHTML">
            --
        </div>
    </div>

    <div class="metric-card">
        <div class="metric-title">Avg Latency</div>
        <div class="metric-value" id="llm-latency"
             hx-get="/api/monitoring/llm/latency"
             hx-trigger="load, every 30s"
             hx-swap="innerHTML">
            --
        </div>
    </div>

    <div class="metric-card">
        <div class="metric-title">Total Tokens</div>
        <div class="metric-value" id="llm-tokens"
             hx-get="/api/monitoring/llm/tokens"
             hx-trigger="load, every 30s"
             hx-swap="innerHTML">
            --
        </div>
    </div>
</div>
</div>"## .to_string(), ) }

async fn health(State(state): State<Arc<AppState>>) -> Html<String> {
let db_ok = state.conn.get().is_ok();
let status = if db_ok { "healthy" } else { "degraded" };

Html(format!(
    r##"<div class="health-status {status}">
<span class="status-icon"></span>
<span class="status-text">{status}</span>

</iv>"## )) }

fn format_uptime(seconds: u64) -> String {
let days = seconds / 86400;
let hours = (seconds % 86400) / 3600;
let minutes = (seconds % 3600) / 60;

if days > 0 {
    format!("{}d {}h {}m", days, hours, minutes)
} else if hours > 0 {
    format!("{}h {}m", hours, minutes)
} else {
    format!("{}m", minutes)
}

}

fn check_postgres() -> bool {
true
}

fn check_redis() -> bool {
true
}

fn check_minio() -> bool {
true
}

fn check_llm() -> bool {
true
}

async fn timestamp(State(_state): State<Arc<AppState>>) -> Html<String> {
let now = Local::now();
Html(format!("Last updated: {}", now.format("%H:%M:%S")))
}

async fn bots(State(state): State<Arc<AppState>>) -> Html<String> {
let active_sessions = state
.session_manager
.try_lock()
.map(|sm| sm.active_count())
.unwrap_or(0);

Html(format!(
    r##"<div class="bots-list">
<div class="bot-item">
    <span class="bot-name">Active Sessions</span>
    <span class="bot-count">{active_sessions}</span>
</div>
</div>"## )) }

async fn services_status(State(_state): State<Arc<AppState>>) -> Html<String> {
let services = vec![
("postgresql", check_postgres()),
("redis", check_redis()),
("minio", check_minio()),
("llm", check_llm()),
];

let mut status_updates = String::new();
for (name, running) in services {
    let status = if running { "running" } else { "stopped" };
    status_updates.push_str(&format!(
        r##"<script>
            (function() {{
                var el = document.querySelector('[data-service="{name}"]');
                if (el) el.setAttribute('data-status', '{status}');
            }})();
        </script>"##
    ));
}

Html(status_updates)

}

async fn resources_bars(State(_state): State<Arc<AppState>>) -> Html<String> {
#[cfg(feature = "monitoring")]
let (cpu_usage, memory_percent) = {
let mut sys = System::new_all();
sys.refresh_all();

    let cpu_usage = sys.global_cpu_usage();
    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();
    let memory_percent = if total_memory > 0 {
        (used_memory as f64 / total_memory as f64) * 100.0
    } else {
        0.0
    };
    
    (cpu_usage, memory_percent)
};

#[cfg(not(feature = "monitoring"))]
let (cpu_usage, memory_percent): (f32, f32) = (0.0, 0.0);

Html(format!(
    r##"<g>
<text x="0" y="0" fill="#94a3b8" font-family="system-ui" font-size="10">CPU</text>
<rect x="40" y="-8" width="100" height="10" rx="2" fill="#1e293b"/>
<rect x="40" y="-8" width="{cpu_width}" height="10" rx="2" fill="#3b82f6"/>
<text x="150" y="0" fill="#f8fafc" font-family="system-ui" font-size="10">{cpu_usage:.0}%</text>
</g> <g transform="translate(0, 20)"> <text x="0" y="0" fill="#94a3b8" font-family="system-ui" font-size="10">MEM</text> <rect x="40" y="-8" width="100" height="10" rx="2" fill="#1e293b"/> <rect x="40" y="-8" width="{mem_width}" height="10" rx="2" fill="#10b981"/> <text x="150" y="0" fill="#f8fafc" font-family="system-ui" font-size="10">{memory_percent:.0}%</text> </g>"##, cpu_width = cpu_usage.min(100.0f32), mem_width = memory_percent.min(100.0f32), )) }

async fn activity_latest(State(_state): State<Arc<AppState>>) -> Html<String> {
Html("System monitoring active...".to_string())
}

async fn metric_sessions(State(state): State<Arc<AppState>>) -> Html<String> {
let active_sessions = state
.session_manager
.try_lock()
.map(|sm| sm.active_count())
.unwrap_or(0);

Html(active_sessions.to_string())

}

async fn metric_messages(State(_state): State<Arc<AppState>>) -> Html<String> {
Html("--".to_string())
}

async fn metric_response_time(State(_state): State<Arc<AppState>>) -> Html<String> {
Html("--".to_string())
}

async fn trend_sessions(State(_state): State<Arc<AppState>>) -> Html<String> {
Html("↑ 0%".to_string())
}

async fn rate_messages(State(_state): State<Arc<AppState>>) -> Html<String> {
Html("0/hr".to_string())
}

async fn sessions_panel(State(state): State<Arc<AppState>>) -> Html<String> {
let active_sessions = state
.session_manager
.try_lock()
.map(|sm| sm.active_count())
.unwrap_or(0);

Html(format!(
    r##"<div class="sessions-panel">
<div class="panel-header">
    <h3>Active Sessions</h3>
    <span class="session-count">{active_sessions}</span>
</div>
<div class="session-list">
    <div class="empty-state">
        <p>No active sessions</p>
    </div>
</div>
</div>"## )) }

async fn messages_panel(State(_state): State<Arc<AppState>>) -> Html<String> {
Html(
r##"<div class="messages-panel">
<div class="panel-header">
<h3>Recent Messages</h3>
</div>
<div class="message-list">
<div class="empty-state">
<p>No recent messages</p>
</div>
</div>

</div>"## .to_string(), ) }
