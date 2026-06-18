use crate::{ComputeProvider, MachineSpec, ProvisionResult, ProviderError, ProviderInfo};
use async_trait::async_trait;

pub struct VultrProvider;

impl VultrProvider {
    pub fn new() -> Self {
        Self
    }

    fn map_plan(spec: &MachineSpec) -> &'static str {
        match (spec.cpu_cores, spec.ram_gb, spec.gpu_type.is_some()) {
            (2, 4, _) => "vhp-2c-4gb",
            (2, 8, _) => "vhp-2c-8gb",
            (4, 16, _) => "vhp-4c-16gb",
            (6, 24, _) => "vhp-6c-24gb",
            (8, 32, _) => "vhp-8c-32gb",
            (16, 64, _) => "vhp-16c-64gb",
            (c, r, true) if c >= 4 && r >= 16 => "gpu-4c-16gb",
            (c, r, true) if c >= 8 && r >= 32 => "gpu-8c-32gb",
            (_, _, true) => "gpu-2c-8gb",
            _ => "vhp-2c-4gb",
        }
    }
}

#[async_trait]
impl ComputeProvider for VultrProvider {
    fn name(&self) -> &str {
        "vultr"
    }

    fn info(&self) -> ProviderInfo {
        ProviderInfo {
            name: "vultr".into(),
            display_name: "Vultr".into(),
            regions: vec![
                "sea".into(), "lax".into(), "ord".into(), "ewr".into(),
                "fra".into(), "lon".into(), "ams".into(), "sgp".into(),
                "tok".into(), "syd".into(),
            ],
            available_gpus: vec![
                "RTX 4090".into(), "RTX 3090".into(), "A100".into(),
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

        let plan = Self::map_plan(spec);
        let label = format!("gb-{}-{}", self.name(), chrono::Utc::now().format("%Y%m%d%H%M%S"));

        let body = serde_json::json!({
            "region": region,
            "plan": plan,
            "label": label,
            "backups": "disabled",
            "enable_ipv6": false,
        });

        let resp = client
            .post("https://api.vultr.com/v2/instances")
            .header("Authorization", format!("Bearer {api_key}"))
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();

        if !status.is_success() {
            return Err(if status.as_u16() == 401 {
                ProviderError::Auth("Invalid Vultr API key".into())
            } else if status.as_u16() == 409 {
                ProviderError::Capacity("Vultr: no capacity in region".into())
            } else {
                ProviderError::Api(format!("Vultr HTTP {status}: {text}"))
            });
        }

        let parsed: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| ProviderError::Api(format!("Vultr parse: {e}")))?;

        let instance = &parsed["instance"];
        let instance_id = instance["id"].as_str().unwrap_or("").to_string();

        Ok(ProvisionResult {
            provider: "vultr".into(),
            instance_id,
            status: "provisioning".into(),
            ip_address: instance["main_ip"].as_str().map(|s| s.into()),
            region: region.into(),
            spec: spec.clone(),
            hourly_cost: 0.0,
        })
    }

    async fn terminate(&self, instance_id: &str, api_key: &str) -> Result<(), ProviderError> {
        let client = reqwest::Client::new();
        let resp = client
            .delete(format!("https://api.vultr.com/v2/instances/{instance_id}"))
            .header("Authorization", format!("Bearer {api_key}"))
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(ProviderError::Api(format!(
                "Vultr terminate failed: HTTP {}",
                resp.status()
            )));
        }
        Ok(())
    }

    async fn get_status(&self, instance_id: &str, api_key: &str) -> Result<String, ProviderError> {
        let client = reqwest::Client::new();
        let resp = client
            .get(format!("https://api.vultr.com/v2/instances/{instance_id}"))
            .header("Authorization", format!("Bearer {api_key}"))
            .send()
            .await?;

        let text = resp.text().await.unwrap_or_default();
        let parsed: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| ProviderError::Api(format!("Vultr parse: {e}")))?;

        Ok(parsed["instance"]["status"]
            .as_str()
            .unwrap_or("unknown")
            .into())
    }

    async fn list_instances(&self, api_key: &str) -> Result<Vec<ProvisionResult>, ProviderError> {
        let client = reqwest::Client::new();
        let resp = client
            .get("https://api.vultr.com/v2/instances")
            .header("Authorization", format!("Bearer {api_key}"))
            .send()
            .await?;

        let text = resp.text().await.unwrap_or_default();
        let parsed: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| ProviderError::Api(format!("Vultr parse: {e}")))?;

        let instances = parsed["instances"]
            .as_array()
            .ok_or_else(|| ProviderError::Api("Vultr: missing instances array".into()))?;

        Ok(instances
            .iter()
            .map(|inst| ProvisionResult {
                provider: "vultr".into(),
                instance_id: inst["id"].as_str().unwrap_or("").into(),
                status: inst["status"].as_str().unwrap_or("unknown").into(),
                ip_address: inst["main_ip"].as_str().map(|s| s.into()),
                region: inst["region"].as_str().unwrap_or("").into(),
                spec: MachineSpec {
                    cpu_cores: inst["vcpu_count"].as_u64().unwrap_or(1) as u32,
                    ram_gb: (inst["ram"].as_u64().unwrap_or(1024) / 1024) as u32,
                    disk_gb: inst["disk"].as_u64().unwrap_or(25) as u32,
                    gpu_type: None,
                    gpu_count: 0,
                    bandwidth_tb: inst["allowed_bandwidth"]
                        .as_u64()
                        .unwrap_or(0) as u32,
                },
                hourly_cost: inst["cost_per_month"]
                    .as_f64()
                    .unwrap_or(0.0)
                    / 730.0,
            })
            .collect())
    }
}
