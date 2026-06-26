use crate::{ComputeProvider, MachineSpec, ProvisionResult, ProviderError, ProviderInfo};
use async_trait::async_trait;

#[derive(Default)]
pub struct ContaboProvider;

impl ContaboProvider {
    pub fn new() -> Self {
        Self
    }

    fn map_region(region: &str) -> &str {
        match region.to_lowercase().as_str() {
            "us-east" | "us" => "US-EAST",
            "eu-west" | "eu" => "EU-WEST",
            "sg" | "asia" => "SINGAPORE",
            "au" | "australia" => "AUSTRALIA",
            _ => "EU-WEST",
        }
    }

    fn map_gpu(spec: &MachineSpec) -> &str {
        match spec.gpu_type.as_deref() {
            Some("H100") => "H100",
            Some("A100") | Some("A100 80GB") => "A100-80GB",
            Some("L40S") => "L40S",
            Some("RTX 4090") => "RTX-4090",
            Some("RTX 3090") | Some("RTX 3090 Ti") => "RTX-3090-Ti",
            _ => "RTX-4090",
        }
    }
}

#[async_trait]
impl ComputeProvider for ContaboProvider {
    fn name(&self) -> &str {
        "contabo"
    }

    fn info(&self) -> ProviderInfo {
        ProviderInfo {
            name: "contabo".into(),
            display_name: "Contabo".into(),
            regions: vec![
                "US-EAST".into(),
                "EU-WEST".into(),
                "SINGAPORE".into(),
                "AUSTRALIA".into(),
            ],
            available_gpus: vec![
                "A100 80GB".into(),
                "RTX 4090".into(),
                "RTX 3090".into(),
                "L40S".into(),
                "H100".into(),
            ],
            supports_spot: false,
        }
    }

    async fn provision(
        &self,
        spec: &MachineSpec,
        region: &str,
        api_key: &str,
    ) -> Result<ProvisionResult, ProviderError> {
        let client = reqwest::Client::new();
        let region_str = Self::map_region(region);
        let gpu_str = Self::map_gpu(spec);

        let label = format!("gb-{}-{}", self.name(), chrono::Utc::now().format("%Y%m%d%H%M%S"));

        let body = serde_json::json!({
            "displayName": label,
            "region": region_str,
            "productId": gpu_str,
            "imageId": "ubuntu-22.04",
            "sshKeys": [],
            "period": 1,
            "extraStorageGb": spec.disk_gb.max(50).saturating_sub(50),
            "ramMb": (spec.ram_gb * 1024).max(16384),
            "cpuCores": spec.cpu_cores.max(4),
        });

        let resp = client
            .post("https://api.contabo.com/v1/compute/instances")
            .header("Authorization", format!("Bearer {api_key}"))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(ProviderError::Api(format!("Contabo provision failed: {text}")));
        }

        let text = resp.text().await.unwrap_or_default();
        let parsed: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| ProviderError::Api(format!("Contabo parse: {e}")))?;

        let instance = parsed["data"][0]["instanceId"]
            .as_str()
            .or_else(|| parsed["data"][0]["id"].as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| ProviderError::Api(format!("Contabo: no instance in response: {text}")))?;

        Ok(ProvisionResult {
            provider: "contabo".into(),
            instance_id: instance,
            status: "provisioning".into(),
            ip_address: None,
            region: region_str.into(),
            spec: spec.clone(),
            hourly_cost: 0.0,
        })
    }

    async fn terminate(&self, instance_id: &str, api_key: &str) -> Result<(), ProviderError> {
        let client = reqwest::Client::new();
        let resp = client
            .delete(format!(
                "https://api.contabo.com/v1/compute/instances/{instance_id}"
            ))
            .header("Authorization", format!("Bearer {api_key}"))
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(ProviderError::Api(format!(
                "Contabo terminate failed: HTTP {}",
                resp.status()
            )));
        }
        Ok(())
    }

    async fn get_status(&self, instance_id: &str, api_key: &str) -> Result<String, ProviderError> {
        let client = reqwest::Client::new();
        let resp = client
            .get(format!(
                "https://api.contabo.com/v1/compute/instances/{instance_id}"
            ))
            .header("Authorization", format!("Bearer {api_key}"))
            .send()
            .await?;

        let text = resp.text().await.unwrap_or_default();
        let parsed: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| ProviderError::Api(format!("Contabo parse: {e}")))?;

        Ok(parsed["data"][0]["status"]
            .as_str()
            .unwrap_or("unknown")
            .to_string())
    }

    async fn list_instances(&self, api_key: &str) -> Result<Vec<ProvisionResult>, ProviderError> {
        let client = reqwest::Client::new();
        let resp = client
            .get("https://api.contabo.com/v1/compute/instances")
            .header("Authorization", format!("Bearer {api_key}"))
            .send()
            .await?;

        let text = resp.text().await.unwrap_or_default();
        let parsed: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| ProviderError::Api(format!("Contabo parse: {e}")))?;

        let instances = parsed["data"].as_array().ok_or_else(|| {
            ProviderError::Api("Contabo: missing data array".into())
        })?;

        Ok(instances
            .iter()
            .map(|inst| {
                let gpu_name = inst["productId"].as_str().map(|s| s.into());
                ProvisionResult {
                    provider: "contabo".into(),
                    instance_id: inst["instanceId"]
                        .as_str()
                        .or_else(|| inst["id"].as_str())
                        .unwrap_or("")
                        .to_string(),
                    status: inst["status"].as_str().unwrap_or("unknown").into(),
                    ip_address: inst["ipAddress"]
                        .as_str()
                        .or_else(|| inst["ipConfig"]["v4"]["ip"].as_str())
                        .map(|s| s.into()),
                    region: inst["region"].as_str().unwrap_or("EU-WEST").into(),
                    spec: MachineSpec {
                        cpu_cores: inst["cpuCores"].as_u64().unwrap_or(4) as u32,
                        ram_gb: (inst["ramMb"].as_u64().unwrap_or(16384) / 1024) as u32,
                        disk_gb: inst["extraStorageGb"].as_u64().unwrap_or(50) as u32,
                        gpu_type: gpu_name,
                        gpu_count: 1,
                        bandwidth_tb: inst["bandwidthLimitTb"].as_u64().unwrap_or(0) as u32,
                    },
                    hourly_cost: inst["hourlyCost"].as_f64().unwrap_or(0.0),
                }
            })
            .collect())
    }
}
