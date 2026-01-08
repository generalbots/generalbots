use anyhow::Result;
use std::collections::HashMap;
use sysinfo::System;
use crate::security::command_guard::SafeCommand;

#[derive(Debug, Default)]
pub struct SystemMetrics {
    pub gpu_usage: Option<f32>,
    pub cpu_usage: f32,
}

pub fn get_system_metrics() -> Result<SystemMetrics> {
    let mut sys = System::new();
    sys.refresh_cpu_usage();
    let cpu_usage = sys.global_cpu_usage();
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

#[must_use]
pub fn has_nvidia_gpu() -> bool {
    let cmd = SafeCommand::new("nvidia-smi")
        .and_then(|c| c.arg("--query-gpu=utilization.gpu"))
        .and_then(|c| c.arg("--format=csv,noheader,nounits"));

    match cmd {
        Ok(safe_cmd) => match safe_cmd.execute() {
            Ok(output) => output.status.success(),
            Err(_) => false,
        },
        Err(_) => false,
    }
}

pub fn get_gpu_utilization() -> Result<HashMap<String, f32>> {
    let cmd = SafeCommand::new("nvidia-smi")
        .and_then(|c| c.arg("--query-gpu=utilization.gpu,utilization.memory"))
        .and_then(|c| c.arg("--format=csv,noheader,nounits"))
        .map_err(|e| anyhow::anyhow!("Failed to build nvidia-smi command: {}", e))?;

    let output = cmd.execute()
        .map_err(|e| anyhow::anyhow!("Failed to execute nvidia-smi: {}", e))?;

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
