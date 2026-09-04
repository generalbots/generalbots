pub struct ServerCapacityConfig {
    pub cpu_pause_threshold: f32,
    pub ram_pause_threshold: f32,
    pub disk_pause_threshold: f32,
    pub ram_per_free_account_mb: f32,
    pub ram_per_shared_account_mb: f32,
    pub free_ram_allocation_fraction: f32,
}

impl Default for ServerCapacityConfig {
    fn default() -> Self {
        Self {
            cpu_pause_threshold: 80.0,
            ram_pause_threshold: 85.0,
            disk_pause_threshold: 90.0,
            ram_per_free_account_mb: 128.0,
            ram_per_shared_account_mb: 512.0,
            free_ram_allocation_fraction: 0.70,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum CapacityHealth {
    Healthy,
    Warning,
    Critical,
}

impl CapacityHealth {
    pub fn as_str(&self) -> &'static str {
        match self {
            CapacityHealth::Healthy => "healthy",
            CapacityHealth::Warning => "warning",
            CapacityHealth::Critical => "critical",
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ServerCapacityResult {
    pub cpu_cores: u32,
    pub cpu_usage_pct: f32,
    pub ram_total_gb: f32,
    pub ram_used_gb: f32,
    pub ram_available_gb: f32,
    pub disk_total_gb: f32,
    pub disk_used_gb: f32,
    pub disk_available_gb: f32,
    pub available_free_slots: u32,
    pub available_shared_slots: u32,
    pub pressure_index: f32,
    pub new_signups_allowed: bool,
    pub capacity_health: String,
}

pub fn calculate_server_capacity(
    config: &ServerCapacityConfig,
    active_free: u32,
    active_shared: u32,
) -> ServerCapacityResult {
    let mut system = sysinfo::System::new_all();
    system.refresh_all();

    let cpu_cores = sysinfo::System::physical_core_count().unwrap_or(1) as u32;
    // #1264 — a single System::new_all() sample reports CPU accumulated since
    // process start (sysinfo requires two refreshes to compute a real
    // interval). A busy bootstrap made every signup see ~95% and veto all
    // free signups forever. Take a three-sample measurement over ~2s so the
    // value reflects SUSTAINED load, not a single compile burst: the drive
    // monitor recompiles work files in sub-second bursts every scan, and a
    // 200ms window reliably landed inside one.
    system.refresh_cpu_usage();
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    system.refresh_cpu_usage();
    let mut samples = [system.global_cpu_usage()];
    for sample in samples.iter_mut().skip(1) {
        std::thread::sleep(std::time::Duration::from_millis(750));
        system.refresh_cpu_usage();
        *sample = system.global_cpu_usage();
    }
    let cpu_pct = samples.iter().sum::<f32>() / samples.len() as f32;
    let ram_total =
        system.total_memory() as f32 / 1024.0 / 1024.0 / 1024.0;
    let ram_used =
        system.used_memory() as f32 / 1024.0 / 1024.0 / 1024.0;
    let ram_available = ram_total - ram_used;
    let ram_free_mb = system.free_memory() as f32 / 1024.0 / 1024.0;
    let ram_pct = ram_used / ram_total * 100.0;

    let mut disk_total_gb = 0.0f32;
    let mut disk_used_gb = 0.0f32;
    let disks = sysinfo::Disks::new_with_refreshed_list();
    for disk in &disks {
        // #1264 follow-up: SUMMING every mounted filesystem double-counts
        // overlays/binds (dev runs services from botserver-stack on the same
        // root, container runtimes add more) and produced a phantom >90%
        // gate that blocked all local signups on a healthy disk. Only the
        // root filesystem carries the capacity signal we gate on.
        if disk.mount_point().to_str().is_some_and(|m| m == "/") {
            disk_total_gb = disk.total_space() as f32 / 1024.0 / 1024.0 / 1024.0;
            disk_used_gb = (disk.total_space() - disk.available_space()) as f32
                / 1024.0
                / 1024.0
                / 1024.0;
            break;
        }
    }
    let disk_available_gb = disk_total_gb - disk_used_gb;
    let disk_pct = if disk_total_gb > 0.0 {
        disk_used_gb / disk_total_gb * 100.0
    } else {
        0.0
    };

    let allocatable_ram_mb = ram_free_mb * config.free_ram_allocation_fraction;

    let raw_free_slots =
        (allocatable_ram_mb / config.ram_per_free_account_mb) as u32;
    let raw_shared_slots =
        (allocatable_ram_mb / config.ram_per_shared_account_mb) as u32;

    let available_free_slots = raw_free_slots.saturating_sub(active_free);
    let available_shared_slots = raw_shared_slots.saturating_sub(active_shared);

    let pressure_index = cpu_pct * 0.30 + ram_pct * 0.50 + disk_pct * 0.20;

    let new_signups_allowed = cpu_pct < config.cpu_pause_threshold
        && ram_pct < config.ram_pause_threshold
        && disk_pct < config.disk_pause_threshold;

    // The health label must reflect the SAME signal as the signup gate:
    // when any threshold veto is active the server is NOT healthy for new
    // work regardless of the weighted index (observed on prod: disk at
    // 90.04% vetoed every free signup while the index reported "healthy",
    // producing the contradictory server_at_capacity/healthy response).
    let capacity_health = if !new_signups_allowed {
        CapacityHealth::Critical
    } else if pressure_index < 60.0 {
        CapacityHealth::Healthy
    } else if pressure_index < 80.0 {
        CapacityHealth::Warning
    } else {
        CapacityHealth::Critical
    };

    ServerCapacityResult {
        cpu_cores,
        cpu_usage_pct: cpu_pct,
        ram_total_gb: ram_total,
        ram_used_gb: ram_used,
        ram_available_gb: ram_available,
        disk_total_gb,
        disk_used_gb,
        disk_available_gb,
        available_free_slots,
        available_shared_slots,
        pressure_index,
        new_signups_allowed,
        capacity_health: capacity_health.as_str().to_string(),
    }
}
