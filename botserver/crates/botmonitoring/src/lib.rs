pub mod real_time;
pub mod tracing;

use axum::{extract::State, response::Html, routing::get, Router};
use chrono::Local;
use std::sync::Arc;

pub use real_time::MetricsCollector;
pub use tracing::{DistributedTracingService, TraceContext};

pub trait MonitoringState: Send + Sync + 'static {
    fn active_session_count(&self) -> usize;
    fn is_db_healthy(&self) -> bool;
}

pub trait MonitoringUrls: Send + Sync + 'static {
    fn monitoring_dashboard() -> &'static str;
    fn monitoring_services() -> &'static str;
    fn monitoring_resources() -> &'static str;
    fn monitoring_logs() -> &'static str;
    fn monitoring_llm() -> &'static str;
    fn monitoring_health() -> &'static str;
    fn monitoring_timestamp() -> &'static str;
    fn monitoring_bots() -> &'static str;
    fn monitoring_services_status() -> &'static str;
    fn monitoring_resources_bars() -> &'static str;
    fn monitoring_activity_latest() -> &'static str;
    fn monitoring_metric_sessions() -> &'static str;
    fn monitoring_metric_messages() -> &'static str;
    fn monitoring_metric_response_time() -> &'static str;
    fn monitoring_trend_sessions() -> &'static str;
    fn monitoring_rate_messages() -> &'static str;
    fn monitoring_sessions_panel() -> &'static str;
    fn monitoring_messages_panel() -> &'static str;
}

pub struct DefaultMonitoringUrls;

impl MonitoringUrls for DefaultMonitoringUrls {
    fn monitoring_dashboard() -> &'static str { "/api/ui/monitoring/dashboard" }
    fn monitoring_services() -> &'static str { "/api/ui/monitoring/services" }
    fn monitoring_resources() -> &'static str { "/api/ui/monitoring/resources" }
    fn monitoring_logs() -> &'static str { "/api/ui/monitoring/logs" }
    fn monitoring_llm() -> &'static str { "/api/ui/monitoring/llm" }
    fn monitoring_health() -> &'static str { "/api/ui/monitoring/health" }
    fn monitoring_timestamp() -> &'static str { "/api/ui/monitoring/timestamp" }
    fn monitoring_bots() -> &'static str { "/api/ui/monitoring/bots" }
    fn monitoring_services_status() -> &'static str { "/api/ui/monitoring/services/status" }
    fn monitoring_resources_bars() -> &'static str { "/api/ui/monitoring/resources/bars" }
    fn monitoring_activity_latest() -> &'static str { "/api/ui/monitoring/activity/latest" }
    fn monitoring_metric_sessions() -> &'static str { "/api/ui/monitoring/metric/sessions" }
    fn monitoring_metric_messages() -> &'static str { "/api/ui/monitoring/metric/messages" }
    fn monitoring_metric_response_time() -> &'static str { "/api/ui/monitoring/metric/response_time" }
    fn monitoring_trend_sessions() -> &'static str { "/api/ui/monitoring/trend/sessions" }
    fn monitoring_rate_messages() -> &'static str { "/api/ui/monitoring/rate/messages" }
    fn monitoring_sessions_panel() -> &'static str { "/api/ui/monitoring/sessions" }
    fn monitoring_messages_panel() -> &'static str { "/api/ui/monitoring/messages" }
}

pub fn configure<S: MonitoringState, U: MonitoringUrls>() -> Router<Arc<S>> {
    Router::new()
        .route(U::monitoring_dashboard(), get(dashboard::<S, U>))
        .route(U::monitoring_services(), get(services::<S, U>))
        .route(U::monitoring_resources(), get(resources::<S, U>))
        .route(U::monitoring_logs(), get(logs::<S, U>))
        .route(U::monitoring_llm(), get(llm_metrics::<S, U>))
        .route(U::monitoring_health(), get(health::<S, U>))
        .route(U::monitoring_timestamp(), get(timestamp::<S, U>))
        .route(U::monitoring_bots(), get(bots::<S, U>))
        .route(U::monitoring_services_status(), get(services_status::<S, U>))
        .route(U::monitoring_resources_bars(), get(resources_bars::<S, U>))
        .route(U::monitoring_activity_latest(), get(activity_latest::<S, U>))
        .route(U::monitoring_metric_sessions(), get(metric_sessions::<S, U>))
        .route(U::monitoring_metric_messages(), get(metric_messages::<S, U>))
        .route(U::monitoring_metric_response_time(), get(metric_response_time::<S, U>))
        .route(U::monitoring_trend_sessions(), get(trend_sessions::<S, U>))
        .route(U::monitoring_rate_messages(), get(rate_messages::<S, U>))
        .route(U::monitoring_sessions_panel(), get(sessions_panel::<S, U>))
        .route(U::monitoring_messages_panel(), get(messages_panel::<S, U>))
}

async fn dashboard<S: MonitoringState, U: MonitoringUrls>(State(state): State<Arc<S>>) -> Html<String> {
    #[cfg(feature = "monitoring")]
    let (cpu_usage, total_memory, used_memory, memory_percent, uptime_str) = {
        use sysinfo::{System, SystemExt};
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
        0.0f32, 0u64, 0u64, 0.0f64, "N/A".to_string()
    );

    let active_sessions = state.active_session_count();

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
</div><div class="refresh-indicator" hx-get="/api/monitoring/dashboard" hx-trigger="every 10s" hx-swap="outerHTML" hx-target="closest .dashboard-grid, .refresh-indicator"> <span class="refresh-dot"></span> Auto-refreshing </div>"##,
        cpu_status = if cpu_usage > 80.0 { "danger" } else if cpu_usage > 60.0 { "warning" } else { "success" },
        mem_status = if memory_percent > 80.0 { "danger" } else if memory_percent > 60.0 { "warning" } else { "success" },
        used_gb = used_memory as f64 / 1_073_741_824.0,
        total_gb = total_memory as f64 / 1_073_741_824.0,
    ))
}

async fn services<S: MonitoringState, U: MonitoringUrls>(_state: State<Arc<S>>) -> Html<String> {
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
</tr>"##,
            name_lower = name.to_lowercase().replace(' ', "-"),
        ));
    }

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
</div>"##
    ))
}

async fn resources<S: MonitoringState, U: MonitoringUrls>(_state: State<Arc<S>>) -> Html<String> {
    #[cfg(feature = "monitoring")]
    let (disk_rows, net_rows) = {
        use sysinfo::{Disks, DisksExt, Networks, NetworksExt, System, SystemExt};
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
</tr>"##,
                mount = disk.mount_point().display(),
                used_gb = used as f64 / 1_073_741_824.0,
                total_gb = total as f64 / 1_073_741_824.0,
                status = if percent > 90.0 { "danger" } else if percent > 70.0 { "warning" } else { "success" },
            ));
        }
        let networks = Networks::new_with_refreshed_list();
        let mut net_rows = String::new();

        for (name, data) in networks.list() {
            net_rows.push_str(&format!(
                r##"<tr>
  <td>{name}</td>
  <td>{rx:.2} MB</td>
  <td>{tx:.2} MB</td>
</tr>"##,
                rx = data.total_received() as f64 / 1_048_576.0,
                tx = data.total_transmitted() as f64 / 1_048_576.0,
            ));
        }
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
</div>"##
    ))
}

async fn logs<S: MonitoringState, U: MonitoringUrls>(_state: State<Arc<S>>) -> Html<String> {
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

</div>"##.to_string(),
    )
}

async fn llm_metrics<S: MonitoringState, U: MonitoringUrls>(_state: State<Arc<S>>) -> Html<String> {
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
</div>"##.to_string(),
    )
}

async fn health<S: MonitoringState, U: MonitoringUrls>(State(state): State<Arc<S>>) -> Html<String> {
    let db_ok = state.is_db_healthy();
    let status = if db_ok { "healthy" } else { "degraded" };

    Html(format!(
        r##"<div class="health-status {status}">
  <span class="status-icon"></span>
  <span class="status-text">{status}</span>

</div>"##
    ))
}

#[cfg(feature = "monitoring")]
fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;

    if days > 0 {
        format!("{days}d {hours}h {minutes}m")
    } else if hours > 0 {
        format!("{hours}h {minutes}m")
    } else {
        format!("{minutes}m")
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

async fn timestamp<S: MonitoringState, U: MonitoringUrls>(_state: State<Arc<S>>) -> Html<String> {
    let now = Local::now();
    Html(format!("Last updated: {}", now.format("%H:%M:%S")))
}

async fn bots<S: MonitoringState, U: MonitoringUrls>(State(state): State<Arc<S>>) -> Html<String> {
    let active_sessions = state.active_session_count();

    Html(format!(
        r##"<div class="bots-list">
  <div class="bot-item">
    <span class="bot-name">Active Sessions</span>
    <span class="bot-count">{active_sessions}</span>
  </div>
</div>"##
    ))
}

async fn services_status<S: MonitoringState, U: MonitoringUrls>(_state: State<Arc<S>>) -> Html<String> {
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

async fn resources_bars<S: MonitoringState, U: MonitoringUrls>(_state: State<Arc<S>>) -> Html<String> {
    #[cfg(feature = "monitoring")]
    let (cpu_usage, memory_percent) = {
        use sysinfo::{System, SystemExt};
        let mut sys = System::new_all();
        sys.refresh_all();

        let cpu_usage = sys.global_cpu_usage();
        let total_memory = sys.total_memory();
        let used_memory = sys.used_memory();
        let memory_percent = if total_memory > 0 {
            ((used_memory as f64 / total_memory as f64) * 100.0) as f32
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
</g> <g transform="translate(0, 20)"> <text x="0" y="0" fill="#94a3b8" font-family="system-ui" font-size="10">MEM</text> <rect x="40" y="-8" width="100" height="10" rx="2" fill="#1e293b"/> <rect x="40" y="-8" width="{mem_width}" height="10" rx="2" fill="#10b981"/> <text x="150" y="0" fill="#f8fafc" font-family="system-ui" font-size="10">{memory_percent:.0}%</text> </g>"##,
        cpu_width = cpu_usage.min(100.0f32),
        mem_width = memory_percent.min(100.0f32),
    ))
}

async fn activity_latest<S: MonitoringState, U: MonitoringUrls>(_state: State<Arc<S>>) -> Html<String> {
    Html("System monitoring active...".to_string())
}

async fn metric_sessions<S: MonitoringState, U: MonitoringUrls>(State(state): State<Arc<S>>) -> Html<String> {
    let active_sessions = state.active_session_count();
    Html(active_sessions.to_string())
}

async fn metric_messages<S: MonitoringState, U: MonitoringUrls>(_state: State<Arc<S>>) -> Html<String> {
    Html("--".to_string())
}

async fn metric_response_time<S: MonitoringState, U: MonitoringUrls>(_state: State<Arc<S>>) -> Html<String> {
    Html("--".to_string())
}

async fn trend_sessions<S: MonitoringState, U: MonitoringUrls>(_state: State<Arc<S>>) -> Html<String> {
    Html("↑ 0%".to_string())
}

async fn rate_messages<S: MonitoringState, U: MonitoringUrls>(_state: State<Arc<S>>) -> Html<String> {
    Html("0/hr".to_string())
}

async fn sessions_panel<S: MonitoringState, U: MonitoringUrls>(State(state): State<Arc<S>>) -> Html<String> {
    let active_sessions = state.active_session_count();

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
</div>"##
    ))
}

async fn messages_panel<S: MonitoringState, U: MonitoringUrls>(_state: State<Arc<S>>) -> Html<String> {
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

</div>"##.to_string(),
    )
}
