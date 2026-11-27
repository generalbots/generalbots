//! Hardware probe for the on-device AI page (#2109).
//!
//! Detection is best effort: `nvidia-smi` gives the device name and VRAM when
//! a CUDA GPU is present; everything else (ROCm, Apple Metal, CPU-only) falls
//! back to system RAM with backend `cpu`. The probe never fails the request —
//! the page must render on a machine with no GPU at all.

use botsecurity::command_guard::SafeCommand;
use serde::Serialize;
use std::path::{Path, PathBuf};

/// What the page shows in its header and uses to pick the recommendation.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct Hardware {
    pub device: String,
    pub vram_gb: u16,
    pub vram_free_gb: u16,
    /// Free video memory in MiB, for the engine's launch plan.
    pub vram_free_mib: u32,
    pub driver: String,
    pub cuda_usable: bool,
    pub ram_gb: u64,
    pub backend: &'static str,
}

pub(crate) fn probe() -> Hardware {
    let ram_gb = system_ram_gb();
    match nvidia_device() {
        Some(gpu) => Hardware {
            device: gpu.name,
            vram_gb: gpu.vram_total_gb,
            vram_free_gb: gpu.vram_free_gb,
            vram_free_mib: gpu.vram_free_mib,
            driver: gpu.driver,
            cuda_usable: true,
            ram_gb,
            backend: "cuda",
        },
        None => Hardware {
            device: "CPU only".to_string(),
            vram_gb: 0,
            vram_free_gb: 0,
            vram_free_mib: 0,
            driver: String::new(),
            cuda_usable: false,
            ram_gb,
            backend: "cpu",
        },
    }
}

pub(crate) struct GpuInfo {
    pub name: String,
    pub vram_total_gb: u16,
    pub vram_free_gb: u16,
    pub vram_free_mib: u32,
    pub driver: String,
}

/// Queries `nvidia-smi` for the first GPU's name, VRAM and driver version.
fn nvidia_device() -> Option<GpuInfo> {
    match run_nvidia_smi("nvidia-smi") {
        Some(result) => return Some(result),
        None => log::debug!("[on_device_ai] nvidia-smi (bare name) did not produce a result"),
    }
    // On Windows the NVIDIA driver places `nvidia-smi.exe` in System32, but
    // `SafeCommand::env_clear()` reconstructs a minimal environment that may
    // not resolve the bare name. Try the canonical path as a fallback.
    #[cfg(windows)]
    {
        let system32 = std::env::var("SYSTEMROOT")
            .unwrap_or_else(|_| "C:\\Windows".to_string());
        let full = format!("{system32}\\System32\\nvidia-smi.exe");
        if std::path::Path::new(&full).exists() {
            log::debug!("[on_device_ai] trying full path: {full}");
            match run_nvidia_smi(&full) {
                Some(result) => return Some(result),
                None => log::warn!("[on_device_ai] nvidia-smi present at {full} but probe failed (NVML init error — check ProgramFiles env var)"),
            }
        } else {
            log::warn!("[on_device_ai] GPU probe: nvidia-smi.exe not found at {full}");
        }
    }
    #[cfg(not(windows))]
    log::warn!("[on_device_ai] GPU probe: nvidia-smi not found in PATH");
    None
}

/// Runs `nvidia-smi` (or its full path) and parses the output.
fn run_nvidia_smi(command: &str) -> Option<GpuInfo> {
    let cmd = match SafeCommand::new(command) {
        Ok(c) => c,
        Err(e) => {
            log::warn!("[on_device_ai] GPU probe: SafeCommand::new({command}) rejected: {e}");
            return None;
        }
    };
    let cmd = match cmd
        .arg("--query-gpu=name,memory.total,memory.free,driver_version")
        .and_then(|c| c.arg("--format=csv,noheader,nounits"))
    {
        Ok(c) => c,
        Err(e) => {
            log::warn!("[on_device_ai] GPU probe: nvidia-smi arg validation failed: {e}");
            return None;
        }
    };
    let output = match cmd.execute() {
        Ok(o) => o,
        Err(e) => {
            log::warn!("[on_device_ai] GPU probe: nvidia-smi execute failed: {e}");
            return None;
        }
    };
    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let detail = if !stdout.trim().is_empty() {
            stdout.chars().take(200).collect::<String>()
        } else {
            stderr.chars().take(200).collect::<String>()
        };
        log::warn!(
            "[on_device_ai] GPU probe: nvidia-smi exited {}: {}",
            output.status,
            detail,
        );
        return None;
    }
    parse_nvidia_line(&String::from_utf8_lossy(&output.stdout))
}

/// Parse `nvidia-smi` CSV output with 4 fields:
/// `name, memory.total, memory.free, driver_version`.
/// Falls back to 2-field format (name, memory.total) for compatibility.
pub(crate) fn parse_nvidia_line(stdout: &str) -> Option<GpuInfo> {
    let line = stdout.lines().map(str::trim).find(|line| !line.is_empty())?;
    let fields: Vec<&str> = line.split(',').map(str::trim).collect();
    if fields.len() < 2 {
        return None;
    }
    let name = fields[0];
    if name.is_empty() {
        return None;
    }
    let total_mb: f64 = fields[1].parse().ok()?;
    if !total_mb.is_finite() || total_mb <= 0.0 {
        return None;
    }
    let vram_total_gb = (total_mb / 1024.0).round().clamp(1.0, f64::from(u16::MAX)) as u16;
    let free_mb = fields
        .get(2)
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|v| v.is_finite() && *v >= 0.0)
        .unwrap_or(0.0);
    let vram_free_gb = (free_mb / 1024.0).round().clamp(0.0, f64::from(u16::MAX)) as u16;
    let vram_free_mib = free_mb.round().clamp(0.0, f64::from(u32::MAX)) as u32;
    let driver = fields.get(3).unwrap_or(&"").to_string();
    Some(GpuInfo {
        name: name.to_string(),
        vram_total_gb,
        vram_free_gb,
        vram_free_mib,
        driver,
    })
}

fn system_ram_gb() -> u64 {
    let mut system = sysinfo::System::new();
    system.refresh_memory();
    system.total_memory() / 1024 / 1024 / 1024
}

/// Where downloaded weights live: the same directory the local engine reads
/// (`{stack}/data/llm`), so a downloaded file is immediately usable.
pub(crate) fn models_dir() -> PathBuf {
    PathBuf::from(botcore::shared::utils::get_stack_path())
        .join("data")
        .join("llm")
}

/// Free bytes on the filesystem holding `path`, using the disk list the OS
/// panel already reads. `None` when the path cannot be matched to a mount.
pub(crate) fn free_bytes_for(path: &Path) -> Option<u64> {
    let target = path.to_string_lossy();
    crate::os_data::read_disks()
        .into_iter()
        .filter(|disk| target.starts_with(disk.mount.as_str()))
        .max_by_key(|disk| disk.mount.len())
        .map(|disk| disk.total_bytes.saturating_sub(disk.used_bytes))
}

#[cfg(test)]
mod hardware_tests {
    use super::{models_dir, parse_nvidia_line};

    #[test]
    fn parses_four_field_nvidia_smi_csv() {
        let gpu = parse_nvidia_line("NVIDIA GeForce RTX 4070 Laptop GPU, 8188, 4401, 621.14\n")
            .expect("should parse");
        assert_eq!(gpu.name, "NVIDIA GeForce RTX 4070 Laptop GPU");
        assert_eq!(gpu.vram_total_gb, 8);
        assert_eq!(gpu.vram_free_gb, 4);
        assert_eq!(gpu.vram_free_mib, 4401);
        assert_eq!(gpu.driver, "621.14");
    }

    #[test]
    fn parses_desktop_gpu_full_output() {
        let gpu = parse_nvidia_line("NVIDIA GeForce RTX 4090, 24564, 22000, 555.42\n")
            .expect("should parse");
        assert_eq!(gpu.name, "NVIDIA GeForce RTX 4090");
        assert_eq!(gpu.vram_total_gb, 24);
        assert_eq!(gpu.vram_free_gb, 21);
        assert_eq!(gpu.driver, "555.42");
    }

    #[test]
    fn parses_legacy_two_field_output() {
        let gpu = parse_nvidia_line("NVIDIA GeForce RTX 3060, 12288\n")
            .expect("should parse");
        assert_eq!(gpu.name, "NVIDIA GeForce RTX 3060");
        assert_eq!(gpu.vram_total_gb, 12);
        assert_eq!(gpu.vram_free_gb, 0);
        assert_eq!(gpu.driver, "");
    }

    #[test]
    fn parses_line_with_trailing_whitespace() {
        let gpu = parse_nvidia_line("  NVIDIA GeForce RTX 3060, 12288, 10000, 530.00  \n\n")
            .expect("should parse");
        assert_eq!(gpu.name, "NVIDIA GeForce RTX 3060");
        assert_eq!(gpu.vram_total_gb, 12);
    }

    #[test]
    fn ignores_empty_or_malformed_output() {
        assert!(parse_nvidia_line("").is_none());
        assert!(parse_nvidia_line("   \n").is_none());
        assert!(parse_nvidia_line("no comma here").is_none());
        assert!(parse_nvidia_line("GPU, not-a-number").is_none());
        assert!(parse_nvidia_line("GPU, 0").is_none());
    }

    #[test]
    fn models_dir_is_under_the_stack_data_directory() {
        assert!(models_dir().ends_with("data/llm"));
    }
}
