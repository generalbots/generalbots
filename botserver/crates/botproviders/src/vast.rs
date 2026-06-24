use crate::{ComputeProvider, MachineSpec, ProvisionResult, ProviderError, ProviderInfo};
use async_trait::async_trait;

pub struct VastAiProvider;

impl VastAiProvider {
    pub fn new() -> Self {
        Self
    }

    fn gpu_query(gpu_type: &str) -> String {
        let gpu = gpu_type.to_lowercase().replace(' ', "-");
        format!(
            "{{\"num_gpus\":{{\"gte\":1}},\"gpu_name\":{{\"eq\":\"{gpu}\"}},\"verified\":{{\"eq\":true}}}}",
        )
    }
}

#[async_trait]
impl ComputeProvider for VastAiProvider {
    fn name(&self) -> &str {
        "vast"
    }

    fn info(&self) -> ProviderInfo {
        ProviderInfo {
            name: "vast".into(),
            display_name: "Vast.ai".into(),
            regions: vec![
                "US".into(), "EU".into(), "ASIA".into(), "ANY".into(),
            ],
            available_gpus: vec![
                "RTX 4090".into(), "RTX 3090".into(), "RTX 3060".into(),
                "A100".into(), "H100".into(),
            ],
            supports_spot: true,
        }
    }

    async fn provision(
        &self,
        spec: &MachineSpec,
        _region: &str,
        api_key: &str,
    ) -> Result<ProvisionResult, ProviderError> {
        let client = reqwest::Client::new();

        let gpu_name = spec
            .gpu_type
            .as_deref()
            .unwrap_or("RTX 4090")
            .to_lowercase()
            .replace(' ', "-");

        let search_query = Self::gpu_query(&gpu_name);

        let search_resp = client
            .post("https://console.vast.ai/api/v0/bundles")
            .header("Accept", "application/json")
            .query(&[("q", &search_query as &str)])
            .send()
            .await?;

        let search_text = search_resp.text().await.unwrap_or_default();
        let search_parsed: serde_json::Value = serde_json::from_str(&search_text)
            .map_err(|e| ProviderError::Api(format!("Vast.ai search parse: {e}")))?;

        let offers = search_parsed["offers"]
            .as_array()
            .ok_or_else(|| ProviderError::Capacity("Vast.ai: no offers found".into()))?;

        let best = offers
            .iter()
            .filter(|o| o["num_gpus"].as_u64().unwrap_or(0) >= spec.gpu_count as u64)
            .min_by_key(|o| {
                (o["dph_total"]
                    .as_f64()
                    .unwrap_or(f64::MAX)
                    * 1000.0) as u64
            })
            .ok_or_else(|| ProviderError::Capacity("Vast.ai: no matching GPU offers".into()))?;

        let bundle_id = best["id"]
            .as_u64()
            .ok_or_else(|| ProviderError::Api("Vast.ai: missing bundle id".into()))?;

        let label = format!("gb-{}-{}", self.name(), chrono::Utc::now().format("%Y%m%d%H%M%S"));

        let create_body = serde_json::json!({
            "client_id": "me",
            "image": "nvidia/cuda:12.4.0-base-ubuntu22.04",
            "env": "",
            "disk": spec.disk_gb.max(10),
            "label": label,
            "extra": "",
            "onstart": "sleep 3600",
            "bundle_id": bundle_id,
            "price": best["dph_total"].as_f64().unwrap_or(0.5),
        });

        let instance_resp = client
            .put("https://console.vast.ai/api/v0/instances")
            .header("Accept", "application/json")
            .header("Authorization", format!("Bearer {api_key}"))
            .json(&create_body)
            .send()
            .await?;

        let instance_text = instance_resp.text().await.unwrap_or_default();
        let parsed: serde_json::Value = serde_json::from_str(&instance_text)
            .map_err(|e| ProviderError::Api(format!("Vast.ai create parse: {e}")))?;

        let instance_id = parsed["new_instance"]
            .as_u64()
            .or_else(|| parsed["id"].as_u64())
            .map(|id| id.to_string())
            .ok_or_else(|| {
                ProviderError::Api(format!(
                    "Vast.ai create failed: {instance_text}"
                ))
            })?;

        Ok(ProvisionResult {
            provider: "vast".into(),
            instance_id,
            status: "provisioning".into(),
            ip_address: None,
            region: best["geolocation"]
                .as_str()
                .unwrap_or("ANY")
                .into(),
            spec: spec.clone(),
            hourly_cost: best["dph_total"].as_f64().unwrap_or(0.0),
        })
    }

    async fn terminate(&self, instance_id: &str, api_key: &str) -> Result<(), ProviderError> {
        let client = reqwest::Client::new();
        let resp = client
            .delete(format!(
                "https://console.vast.ai/api/v0/instances/{instance_id}"
            ))
            .header("Authorization", format!("Bearer {api_key}"))
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(ProviderError::Api(format!(
                "Vast.ai terminate failed: HTTP {}",
                resp.status()
            )));
        }
        Ok(())
    }

    async fn get_status(&self, instance_id: &str, api_key: &str) -> Result<String, ProviderError> {
        let client = reqwest::Client::new();
        let resp = client
            .get("https://console.vast.ai/api/v0/instances")
            .header("Authorization", format!("Bearer {api_key}"))
            .send()
            .await?;

        let text = resp.text().await.unwrap_or_default();
        let parsed: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| ProviderError::Api(format!("Vast.ai parse: {e}")))?;

        let instances = parsed["instances"].as_array().ok_or_else(|| {
            ProviderError::Api("Vast.ai: missing instances".into())
        })?;

        Ok(instances
            .iter()
            .find(|i| {
                i["id"].as_u64().map(|id| id.to_string()) == Some(instance_id.to_string())
                    || i["label"].as_str() == Some(instance_id)
            })
            .and_then(|i| i["actual_status"].as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "not_found".into()))
    }

    async fn list_instances(&self, api_key: &str) -> Result<Vec<ProvisionResult>, ProviderError> {
        let client = reqwest::Client::new();
        let resp = client
            .get("https://console.vast.ai/api/v0/instances")
            .header("Authorization", format!("Bearer {api_key}"))
            .send()
            .await?;

        let text = resp.text().await.unwrap_or_default();
        let parsed: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| ProviderError::Api(format!("Vast.ai parse: {e}")))?;

        let instances = parsed["instances"].as_array().ok_or_else(|| {
            ProviderError::Api("Vast.ai: missing instances".into())
        })?;

        Ok(instances
            .iter()
            .map(|inst| {
                let gpu_name = inst["gpu_name"].as_str().map(|s| s.into());
                ProvisionResult {
                    provider: "vast".into(),
                    instance_id: inst["id"].as_u64().unwrap_or(0).to_string(),
                    status: inst["actual_status"]
                        .as_str()
                        .unwrap_or("unknown")
                        .into(),
                    ip_address: inst["ssh_host"]
                        .as_str()
                        .map(|s| format!("{}:{}", s, inst["ssh_port"].as_u64().unwrap_or(22))),
                    region: inst["geolocation"].as_str().unwrap_or("ANY").into(),
                    spec: MachineSpec {
                        cpu_cores: inst["cpu_cores"].as_u64().unwrap_or(2) as u32,
                        ram_gb: (inst["ram"].as_u64().unwrap_or(8192) / 1024) as u32,
                        disk_gb: inst["disk_space"].as_u64().unwrap_or(10) as u32,
                        gpu_type: gpu_name,
                        gpu_count: inst["num_gpus"].as_u64().unwrap_or(1) as u32,
                        bandwidth_tb: 0,
                    },
                    hourly_cost: inst["dph_total"].as_f64().unwrap_or(0.0),
                }
            })
            .collect())
    }
}
