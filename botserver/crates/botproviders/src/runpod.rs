use crate::{ComputeProvider, MachineSpec, ProvisionResult, ProviderError, ProviderInfo};
use async_trait::async_trait;

pub struct RunPodProvider;

impl RunPodProvider {
    pub fn new() -> Self {
        Self
    }

    fn gpu_type_to_runpod(&self, gpu: &str) -> String {
        let lower = gpu.to_lowercase();
        match lower.as_str() {
            "rtx 4090" | "rtx4090" => "NVIDIA GeForce RTX 4090",
            "rtx 3090" | "rtx3090" => "NVIDIA GeForce RTX 3090",
            "a100" => "NVIDIA A100 80GB PCIe",
            "a6000" => "NVIDIA RTX A6000",
            "h100" => "NVIDIA H100 80GB HBM3",
            "l40s" => "NVIDIA L40S",
            _ => gpu,
        }
        .to_string()
    }

    fn runpod_gpu_to_type(runpod_id: &str) -> Option<String> {
        let lower = runpod_id.to_lowercase();
        if lower.contains("4090") {
            Some("RTX 4090".into())
        } else if lower.contains("3090") {
            Some("RTX 3090".into())
        } else if lower.contains("a100") {
            Some("A100".into())
        } else if lower.contains("a6000") {
            Some("RTX A6000".into())
        } else if lower.contains("h100") {
            Some("H100".into())
        } else if lower.contains("l40s") {
            Some("L40S".into())
        } else {
            None
        }
    }
}

#[async_trait]
impl ComputeProvider for RunPodProvider {
    fn name(&self) -> &str {
        "runpod"
    }

    fn info(&self) -> ProviderInfo {
        ProviderInfo {
            name: "runpod".into(),
            display_name: "RunPod".into(),
            regions: vec![
                "US-CA".into(), "US-TX".into(), "US-VA".into(),
                "EU-UK".into(), "EU-RO".into(), "SG-SIN".into(),
            ],
            available_gpus: vec![
                "RTX 4090".into(), "RTX 3090".into(), "A100".into(),
                "RTX A6000".into(), "H100".into(), "L40S".into(),
            ],
            supports_spot: true,
        }
    }

    async fn provision(
        &self,
        spec: &MachineSpec,
        region: &str,
        api_key: &str,
    ) -> Result<ProvisionResult, ProviderError> {
        let client = reqwest::Client::new();

        let gpu_type_id = if let Some(ref gpu) = spec.gpu_type {
            self.gpu_type_to_runpod(gpu.as_str())
        } else {
            return Err(ProviderError::Api("GPU type required for RunPod".into()));
        };

        let body = serde_json::json!({
            "name": format!("gb-{}-{}", self.name(), chrono::Utc::now().format("%Y%m%d%H%M%S")),
            "imageName": "runpod/base:latest",
            "gpuTypeId": gpu_type_id,
            "gpuCount": spec.gpu_count.max(1),
            "containerDiskSizeGb": spec.disk_gb.max(10),
            "volumeInGb": 0,
            "minVcpuCount": spec.cpu_cores.max(2),
            "minMemoryInGb": spec.ram_gb.max(8),
            "country": region,
        });

        let resp = client
            .post("https://api.runpod.io/v2/gpu/request")
            .header("Authorization", format!("Bearer {api_key}"))
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    ProviderError::Api("RunPod request timed out".into())
                } else {
                    ProviderError::Request(e)
                }
            })?;

        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();

        if !status.is_success() {
            return Err(if status.as_u16() == 401 {
                ProviderError::Auth("Invalid RunPod API key".into())
            } else if status.as_u16() == 503 {
                ProviderError::Capacity("RunPod: no capacity in region".into())
            } else {
                ProviderError::Api(format!("RunPod HTTP {status}: {text}"))
            });
        }

        let parsed: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| ProviderError::Api(format!("RunPod parse error: {e}")))?;

        let instance_id = parsed["id"]
            .as_str()
            .or_else(|| parsed["gpuRequestId"].as_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(ProvisionResult {
            provider: "runpod".into(),
            instance_id,
            status: "provisioning".into(),
            ip_address: None,
            region: region.into(),
            spec: spec.clone(),
            hourly_cost: 0.0,
        })
    }

    async fn terminate(&self, instance_id: &str, api_key: &str) -> Result<(), ProviderError> {
        let client = reqwest::Client::new();
        let resp = client
            .post(format!("https://api.runpod.io/v2/gpu/request/{instance_id}/stop"))
            .header("Authorization", format!("Bearer {api_key}"))
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(ProviderError::Api(format!(
                "RunPod terminate failed: HTTP {}",
                resp.status()
            )));
        }
        Ok(())
    }

    async fn get_status(&self, instance_id: &str, api_key: &str) -> Result<String, ProviderError> {
        let client = reqwest::Client::new();
        let resp = client
            .get(format!("https://api.runpod.io/v2/gpu/request/{instance_id}"))
            .header("Authorization", format!("Bearer {api_key}"))
            .send()
            .await?;

        let text = resp.text().await.unwrap_or_default();
        let parsed: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| ProviderError::Api(format!("RunPod parse: {e}")))?;

        Ok(parsed["status"].as_str().unwrap_or("unknown").into())
    }

    async fn list_instances(&self, api_key: &str) -> Result<Vec<ProvisionResult>, ProviderError> {
        let client = reqwest::Client::new();
        let resp = client
            .get("https://api.runpod.io/v2/gpu/requests")
            .header("Authorization", format!("Bearer {api_key}"))
            .send()
            .await?;

        let text = resp.text().await.unwrap_or_default();
        let parsed: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| ProviderError::Api(format!("RunPod parse: {e}")))?;

        let items = parsed.as_array().ok_or_else(|| {
            ProviderError::Api("RunPod returned non-array".into())
        })?;

        Ok(items
            .iter()
            .map(|item| {
                let gpu_type = item["gpuTypeId"]
                    .as_str()
                    .and_then(Self::runpod_gpu_to_type);
                ProvisionResult {
                    provider: "runpod".into(),
                    instance_id: item["id"].as_str().unwrap_or("").into(),
                    status: item["status"].as_str().unwrap_or("unknown").into(),
                    ip_address: item["ip"].as_str().map(|s| s.into()),
                    region: item["country"].as_str().unwrap_or("").into(),
                    spec: MachineSpec {
                        cpu_cores: item["minVcpuCount"].as_u64().unwrap_or(2) as u32,
                        ram_gb: item["minMemoryInGb"].as_u64().unwrap_or(8) as u32,
                        disk_gb: item["containerDiskSizeGb"].as_u64().unwrap_or(10) as u32,
                        gpu_type,
                        gpu_count: item["gpuCount"].as_u64().unwrap_or(1) as u32,
                        bandwidth_tb: 0,
                    },
                    hourly_cost: item["costPerHr"].as_f64().unwrap_or(0.0),
                }
            })
            .collect())
    }
}
