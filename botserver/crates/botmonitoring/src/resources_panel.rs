use crate::MonitoringState;
use crate::MonitoringUrls;
use axum::extract::{Query, State};
use axum::response::Html;
use serde::Deserialize;
use std::sync::Arc;

#[cfg(feature = "monitoring")]
use sysinfo::{Disks, Networks, Process, System};

#[derive(Deserialize)]
pub struct SortQuery {
    pub sort: Option<String>,
}

fn system_snapshot() -> Option<(f32, u64, u64, u64, f64)> {
    #[cfg(feature = "monitoring")]
    {
        let mut sys = System::new_all();
        sys.refresh_all();
        let cpu = sys.global_cpu_usage();
        let total = sys.total_memory();
        let used = sys.used_memory();
        let uptime = System::uptime();
        let mem_percent = if total > 0 {
            (used as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        Some((cpu, total, used, uptime, mem_percent))
    }
    #[cfg(not(feature = "monitoring"))]
    {
        let _ = ();
        None
    }
}

pub async fn cpu_card<S: MonitoringState, U: MonitoringUrls>(_state: State<Arc<S>>) -> Html<String> {
    let (cpu, _, _, _, _) = system_snapshot().unwrap_or((0.0, 0, 0, 0, 0.0));
    let cores = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);

    Html(format!(
        r##"<div class="card-icon">
    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="4" y="4" width="16" height="16" rx="2"></rect>
        <rect x="9" y="9" width="6" height="6"></rect>
        <line x1="9" y1="1" x2="9" y2="4"></line>
        <line x1="15" y1="1" x2="15" y2="4"></line>
        <line x1="9" y1="20" x2="9" y2="23"></line>
        <line x1="15" y1="20" x2="15" y2="23"></line>
        <line x1="20" y1="9" x2="23" y2="9"></line>
        <line x1="20" y1="14" x2="23" y2="14"></line>
        <line x1="1" y1="9" x2="4" y2="9"></line>
        <line x1="1" y1="14" x2="4" y2="14"></line>
    </svg>
</div>
<div class="card-content">
    <span class="card-label">CPU Usage</span>
    <span class="card-value">{cpu:.0}%</span>
    <div class="progress-bar">
        <div class="progress-fill cpu" style="width: {cpu:.0}%"></div>
    </div>
    <span class="card-detail">{cores} cores</span>
</div>"##
    ))
}

pub async fn memory_card<S: MonitoringState, U: MonitoringUrls>(_state: State<Arc<S>>) -> Html<String> {
    let (_, total, used, _, mem_percent) = system_snapshot().unwrap_or((0.0, 0, 0, 0, 0.0));

    Html(format!(
        r##"<div class="card-icon">
    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="2" y="6" width="20" height="12" rx="2"></rect>
        <line x1="6" y1="12" x2="6" y2="12"></line>
        <line x1="10" y1="12" x2="10" y2="12"></line>
        <line x1="14" y1="12" x2="14" y2="12"></line>
        <line x1="18" y1="12" x2="18" y2="12"></line>
    </svg>
</div>
<div class="card-content">
    <span class="card-label">Memory Usage</span>
    <span class="card-value">{mem_percent:.0}%</span>
    <div class="progress-bar">
        <div class="progress-fill memory" style="width: {mem_percent:.0}%"></div>
    </div>
    <span class="card-detail">{used_gb:.1} GB / {total_gb:.1} GB</span>
</div>"##,
        used_gb = used as f64 / 1_073_741_824.0,
        total_gb = total as f64 / 1_073_741_824.0,
    ))
}

pub async fn disk_card<S: MonitoringState, U: MonitoringUrls>(_state: State<Arc<S>>) -> Html<String> {
    #[cfg(feature = "monitoring")]
    let (total, used, percent) = {
        let disks = Disks::new_with_refreshed_list();
        let total: u64 = disks.list().iter().map(|d| d.total_space()).sum();
        let available: u64 = disks.list().iter().map(|d| d.available_space()).sum();
        let used = total.saturating_sub(available);
        let percent = if total > 0 {
            (used as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        (total, used, percent)
    };

    #[cfg(not(feature = "monitoring"))]
    let (total, used, percent) = (0u64, 0u64, 0.0);

    Html(format!(
        r##"<div class="card-icon">
    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <ellipse cx="12" cy="5" rx="9" ry="3"></ellipse>
        <path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"></path>
        <path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"></path>
    </svg>
</div>
<div class="card-content">
    <span class="card-label">Disk Usage</span>
    <span class="card-value">{percent:.0}%</span>
    <div class="progress-bar">
        <div class="progress-fill disk" style="width: {percent:.0}%"></div>
    </div>
    <span class="card-detail">{used_gb:.1} GB / {total_gb:.1} GB</span>
</div>"##,
        used_gb = used as f64 / 1_073_741_824.0,
        total_gb = total as f64 / 1_073_741_824.0,
    ))
}

pub async fn network_card<S: MonitoringState, U: MonitoringUrls>(_state: State<Arc<S>>) -> Html<String> {
    #[cfg(feature = "monitoring")]
    let (rx, tx) = {
        let networks = Networks::new_with_refreshed_list();
        let mut rx = 0u64;
        let mut tx = 0u64;
        for (_, data) in networks.list() {
            rx += data.total_received();
            tx += data.total_transmitted();
        }
        (rx, tx)
    };

    #[cfg(not(feature = "monitoring"))]
    let (rx, tx) = (0u64, 0u64);

    Html(format!(
        r##"<div class="card-icon">
    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M5 12.55a11 11 0 0 1 14.08 0"></path>
        <path d="M1.42 9a16 16 0 0 1 21.16 0"></path>
        <path d="M8.53 16.11a6 6 0 0 1 6.95 0"></path>
        <line x1="12" y1="20" x2="12.01" y2="20"></line>
    </svg>
</div>
<div class="card-content">
    <span class="card-label">Network I/O</span>
    <span class="card-value">{rx_mb:.1}/{tx_mb:.1} MB</span>
    <div class="network-stats">
        <span class="net-stat">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="18 15 12 9 6 15"></polyline>
            </svg>
            {rx_mb:.1} MB
        </span>
        <span class="net-stat">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="6 9 12 15 18 9"></polyline>
            </svg>
            {tx_mb:.1} MB
        </span>
    </div>
    <span class="card-detail">Total since boot</span>
</div>"##,
        rx_mb = rx as f64 / 1_048_576.0,
        tx_mb = tx as f64 / 1_048_576.0,
    ))
}

pub async fn disk_partitions<S: MonitoringState, U: MonitoringUrls>(_state: State<Arc<S>>) -> Html<String> {
    #[cfg(feature = "monitoring")]
    let rows = {
        let disks = Disks::new_with_refreshed_list();
        let mut rows = String::new();
        for disk in disks.list() {
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total.saturating_sub(available);
            let percent = if total > 0 {
                (used as f64 / total as f64) * 100.0
            } else {
                0.0
            };
            rows.push_str(&format!(
                r##"<div class="partition-row">
    <div class="partition-info">
        <span class="partition-name">{mount}</span>
        <span class="partition-type">{fs}</span>
    </div>
    <div class="partition-usage">
        <div class="usage-bar">
            <div class="usage-fill" style="width: {percent:.0}%"></div>
        </div>
        <span class="usage-text">{used_gb:.1} GB / {total_gb:.1} GB</span>
    </div>
</div>"##,
                mount = disk.mount_point().display(),
                fs = disk.file_system().to_string_lossy(),
            ));
        }
        rows
    };

    #[cfg(not(feature = "monitoring"))]
    let rows = String::new();

    Html(rows)
}

pub async fn network_interfaces<S: MonitoringState, U: MonitoringUrls>(_state: State<Arc<S>>) -> Html<String> {
    #[cfg(feature = "monitoring")]
    let rows = {
        let networks = Networks::new_with_refreshed_list();
        let mut rows = String::new();
        for (name, data) in networks.list() {
            rows.push_str(&format!(
                r##"<div class="interface-row">
    <div class="interface-info">
        <span class="interface-name">{name}</span>
        <span class="interface-ip">--</span>
    </div>
    <div class="interface-stats">
        <span class="stat-in">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="18 15 12 9 6 15"></polyline>
            </svg>
            {rx:.2} MB/s
        </span>
        <span class="stat-out">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="6 9 12 15 18 9"></polyline>
            </svg>
            {tx:.2} MB/s
        </span>
    </div>
</div>"##,
                rx = data.total_received() as f64 / 1_048_576.0,
                tx = data.total_transmitted() as f64 / 1_048_576.0,
            ));
        }
        rows
    };

    #[cfg(not(feature = "monitoring"))]
    let rows = String::new();

    Html(rows)
}

pub async fn processes<S: MonitoringState, U: MonitoringUrls>(State(_state): State<Arc<S>>, Query(query): Query<SortQuery>) -> Html<String> {
    let sort = query.sort.clone().unwrap_or_else(|| "cpu".to_string());
    let rows = sorted_process_rows(&sort);

    Html(format!(
        r##"<table class="process-table">
    <thead>
        <tr>
            <th>PID</th>
            <th>Process</th>
            <th>CPU</th>
            <th>Memory</th>
            <th>Status</th>
        </tr>
    </thead>
    <tbody>
        {rows}
    </tbody>
</table>"##
    ))
}

fn sorted_process_rows(sort: &str) -> String {
    #[cfg(feature = "monitoring")]
    {
        let mut sys = System::new_all();
        sys.refresh_all();
        let mut procs: Vec<(sysinfo::Pid, &Process)> = sys.processes().iter().map(|(pid, p)| (*pid, p)).collect();

        match sort {
            "memory" => procs.sort_by(|a, b| b.1.memory().cmp(&a.1.memory())),
            "name" => procs.sort_by(|a, b| a.1.name().cmp(b.1.name())),
            _ => procs.sort_by(|a, b| b.1.cpu_usage().total_cmp(&a.1.cpu_usage())),
        }

        let mut rows = String::new();
        for (pid, proc) in procs.iter().take(20) {
            rows.push_str(&format!(
                r##"<tr>
    <td>{pid}</td>
    <td>{name}</td>
    <td>{cpu:.1}%</td>
    <td>{mem_gb:.2} GB</td>
    <td><span class="status-pill">{status}</span></td>
</tr>"##,
                name = proc.name().to_string_lossy(),
                cpu = proc.cpu_usage(),
                mem_gb = proc.memory() as f64 / 1_073_741_824.0,
                status = proc.status(),
            ));
        }
        rows
    }

    #[cfg(not(feature = "monitoring"))]
    {
        let _ = sort;
        String::new()
    }
}

pub async fn system_info<S: MonitoringState, U: MonitoringUrls>(_state: State<Arc<S>>) -> Html<String> {    #[cfg(feature = "monitoring")]
    let (hostname, os, kernel, uptime, load) = {
        let hostname = System::host_name().unwrap_or_else(|| "unknown".to_string());
        let os = System::long_os_version().unwrap_or_else(|| "unknown".to_string());
        let kernel = System::kernel_version().unwrap_or_else(|| "unknown".to_string());
        let uptime = System::uptime();
        let load = std::fs::read_to_string("/proc/loadavg")
            .ok()
            .and_then(|s| s.split_whitespace().take(3).collect::<Vec<_>>().join(" ").into())
            .unwrap_or_else(|| "--".to_string());
        (hostname, os, kernel, uptime, load)
    };

    #[cfg(not(feature = "monitoring"))]
    let (hostname, os, kernel, uptime, load) = (
        "unknown".to_string(),
        "unknown".to_string(),
        "unknown".to_string(),
        0u64,
        "--".to_string(),
    );

    Html(format!(
        r##"<div class="info-row">
    <span class="info-label">Hostname</span>
    <span class="info-value">{hostname}</span>
</div>
<div class="info-row">
    <span class="info-label">OS</span>
    <span class="info-value">{os}</span>
</div>
<div class="info-row">
    <span class="info-label">Kernel</span>
    <span class="info-value">{kernel}</span>
</div>
<div class="info-row">
    <span class="info-label">Uptime</span>
    <span class="info-value">{uptime}</span>
</div>
<div class="info-row">
    <span class="info-label">Load Average</span>
    <span class="info-value">{load}</span>
</div>"##,
        uptime = crate::format_uptime_duration(uptime),
    ))
}

pub async fn cpu_chart<S: MonitoringState, U: MonitoringUrls>(_state: State<Arc<S>>) -> Html<String> {
    let (cpu, _, _, _, _) = system_snapshot().unwrap_or((0.0, 0, 0, 0, 0.0));
    chart_svg(cpu as f64, "cpu-gradient", "#3b82f6")
}

pub async fn memory_chart<S: MonitoringState, U: MonitoringUrls>(_state: State<Arc<S>>) -> Html<String> {
    let (_, _, _, _, mem) = system_snapshot().unwrap_or((0.0, 0, 0, 0, 0.0));
    chart_svg(mem, "memory-gradient", "#10b981")
}

fn chart_svg(value: f64, gradient_id: &str, color: &str) -> Html<String> {
    let value = value.clamp(0.0, 100.0);
    let y = 150.0 - (value / 100.0 * 120.0);
    let baseline = 150.0;

    Html(format!(
        r##"<svg viewBox="0 0 400 150" class="sparkline-chart">
    <defs>
        <linearGradient id="{gradient_id}" x1="0%" y1="0%" x2="0%" y2="100%">
            <stop offset="0%" style="stop-color:{color};stop-opacity:0.3"/>
            <stop offset="100%" style="stop-color:{color};stop-opacity:0"/>
        </linearGradient>
    </defs>
    <path d="M0,{baseline} L0,{y} Q100,{y} 200,{y} T400,{y} L400,{baseline} Z" fill="url(#{gradient_id})"/>
    <path d="M0,{y} Q100,{y} 200,{y} T400,{y}" fill="none" stroke="{color}" stroke-width="2"/>
    <text x="350" y="20" fill="{color}" font-size="11" text-anchor="end">{value:.0}%</text>
</svg>
<div class="chart-axis-y">
    <span>100%</span>
    <span>50%</span>
    <span>0%</span>
</div>"##
    ))
}
