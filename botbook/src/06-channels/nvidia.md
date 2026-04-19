# NVIDIA GPU Module

System monitoring for NVIDIA GPU utilization and performance metrics.

## Overview

This module provides GPU monitoring capabilities when NVIDIA hardware is available, useful for tracking resource usage during LLM inference and multimodal generation tasks.

## Feature Flag

Enabled via Cargo feature:

```toml
[features]
nvidia = []
```

## Functions

### has_nvidia_gpu()

Check if NVIDIA GPU is available:

```rust
if nvidia::has_nvidia_gpu() {
    // GPU acceleration available
}
```

Returns `true` if `nvidia-smi` command succeeds.

### get_gpu_utilization()

Get current GPU and memory utilization:

```rust
let util = nvidia::get_gpu_utilization()?;
let gpu_percent = util.get("gpu");      // GPU compute utilization %
let mem_percent = util.get("memory");   // GPU memory utilization %
```

### get_system_metrics()

Get combined CPU and GPU metrics:

```rust
let metrics = nvidia::get_system_metrics()?;
println!("CPU: {}%", metrics.cpu_usage);
if let Some(gpu) = metrics.gpu_usage {
    println!("GPU: {}%", gpu);
}
```

## SystemMetrics Struct

| Field | Type | Description |
|-------|------|-------------|
| `cpu_usage` | `f32` | CPU utilization percentage |
| `gpu_usage` | `Option<f32>` | GPU utilization (None if no NVIDIA GPU) |

## Requirements

- NVIDIA GPU with driver installed
- `nvidia-smi` command available in PATH

## Use Cases

- Monitor GPU during image/video generation
- Track resource usage for LLM inference
- Capacity planning for bot deployments
- Performance dashboards

## See Also

- [Multimodal Module](./multimodal.md)
- [Time-Series Database](./timeseries.md) - Store GPU metrics over time