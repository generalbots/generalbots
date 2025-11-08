use anyhow::Result;
use std::collections::HashMap;
use sysinfo::{System};

/// System monitoring data
#[derive(Default)]
pub struct SystemMetrics {
    pub gpu_usage: Option<f32>,
    pub cpu_usage: f32,
}

/// Gets current system metrics
pub fn get_system_metrics(_current_tokens: usize, _max_tokens: usize) -> Result<SystemMetrics> {
    let mut sys = System::new();
    sys.refresh_cpu_usage();
    
    // Get CPU usage (average across all cores)
    let cpu_usage = sys.global_cpu_usage();

    // Get GPU usage if available
    let gpu_usage = if has_nvidia_gpu() {
        get_gpu_utilization()?.get("gpu").copied()
    } else {
        None
    };


    Ok(SystemMetrics {
        gpu_usage,
        cpu_usage,
    })
}

/// Checks if NVIDIA GPU is available
pub fn has_nvidia_gpu() -> bool {
    match std::process::Command::new("nvidia-smi")
        .arg("--query-gpu=utilization.gpu")
        .arg("--format=csv,noheader,nounits")
        .output()
    {
        Ok(output) => output.status.success(),
        Err(_) => {
            false
        }
    }
}

/// Gets current GPU utilization percentages
pub fn get_gpu_utilization() -> Result<HashMap<String, f32>> {
    let output = std::process::Command::new("nvidia-smi")
        .arg("--query-gpu=utilization.gpu,utilization.memory")
        .arg("--format=csv,noheader,nounits")
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to query GPU utilization"));
    }

    let output_str = String::from_utf8(output.stdout)?;
    let mut util = HashMap::new();

    for line in output_str.lines() {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 2 {
            util.insert(
                "gpu".to_string(),
                parts[0].trim().parse::<f32>().unwrap_or(0.0),
            );
            util.insert(
                "memory".to_string(),
                parts[1].trim().parse::<f32>().unwrap_or(0.0),
            );
        }
    }

    Ok(util)
}
