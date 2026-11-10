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
    pub ram_gb: u64,
    pub backend: &'static str,
}

pub(crate) fn probe() -> Hardware {
    let ram_gb = system_ram_gb();
    match nvidia_device() {
        Some((device, vram_gb)) => Hardware {
            device,
            vram_gb,
            ram_gb,
            backend: "cuda",
        },
        None => Hardware {
            device: "CPU only".to_string(),
            vram_gb: 0,
            ram_gb,
            backend: "cpu",
        },
    }
}

/// `nvidia-smi --query-gpu=name,memory.total` → `("NVIDIA ...", vram_gb)`.
fn nvidia_device() -> Option<(String, u16)> {
    let output = SafeCommand::new("nvidia-smi")
        .ok()?
        .arg("--query-gpu=name,memory.total")
        .ok()?
        .arg("--format=csv,noheader,nounits")
        .ok()?
        .execute()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    parse_nvidia_line(&String::from_utf8_lossy(&output.stdout))
}

/// First GPU line → device name and whole-GB VRAM. Pure, so it is unit-tested
/// against the exact `nvidia-smi` output shape.
pub(crate) fn parse_nvidia_line(stdout: &str) -> Option<(String, u16)> {
    let line = stdout.lines().map(str::trim).find(|line| !line.is_empty())?;
    let (name, memory) = line.rsplit_once(',')?;
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    let total_mb: f64 = memory.trim().parse().ok()?;
    if !total_mb.is_finite() || total_mb <= 0.0 {
        return None;
    }
    let vram_gb = (total_mb / 1024.0).round().clamp(1.0, f64::from(u16::MAX)) as u16;
    Some((name.to_string(), vram_gb))
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
    fn parses_the_nvidia_smi_csv_line() {
        let parsed = parse_nvidia_line("NVIDIA GeForce RTX 4090, 24564\n");
        assert_eq!(parsed, Some(("NVIDIA GeForce RTX 4090".to_string(), 24)));
    }

    #[test]
    fn ignores_empty_or_malformed_output() {
        assert_eq!(parse_nvidia_line(""), None);
        assert_eq!(parse_nvidia_line("   \n"), None);
        assert_eq!(parse_nvidia_line("no comma here"), None);
        assert_eq!(parse_nvidia_line("GPU, not-a-number"), None);
        assert_eq!(parse_nvidia_line("GPU, 0"), None);
    }

    #[test]
    fn models_dir_is_under_the_stack_data_directory() {
        assert!(models_dir().ends_with("data/llm"));
    }

}
