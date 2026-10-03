use crate::{ComputeProvider, MachineSpec, ProvisionResult, ProviderError, ProviderInfo};
use async_trait::async_trait;

#[derive(Debug, Default)]
pub struct VultrProvider;

impl VultrProvider {
    pub fn new() -> Self {
        Self
    }

    /// Maps a spec to a Vultr plan.
    ///
    /// The GPU flag is tested first: the previous ordering put the exact-match
    /// CPU arms above the GPU arms, so a 4-core/16-GB GPU request resolved to
    /// `vhp-4c-16gb` and the GPU plans were unreachable.
    /// Largest CPU plan, and the ceiling for a GPU request — Vultr publishes no
    /// accelerator larger than 8 vCPU/32 GB.
    const LARGEST_CPU: &'static str = "vhp-16c-64gb";
    const LARGEST_GPU: &'static str = "gpu-8c-32gb";

    /// Maps a spec to a Vultr plan.
    ///
    /// The GPU flag is tested first: the previous ordering put the exact-match
    /// CPU arms above the GPU arms, so a 4-core/16-GB GPU request resolved to
    /// `vhp-4c-16gb` and the GPU plans were unreachable.
    ///
    /// A spec above the largest plan of the relevant line is an error, not a
    /// substitution to the next plan down.
    fn map_plan(spec: &MachineSpec) -> Result<&'static str, ProviderError> {
        let (cores, ram) = (spec.cpu_cores.max(1), spec.ram_gb.max(1));
        if spec.gpu_type.is_some() {
            // RAM is the stronger constraint on these plans, so the tiers are
            // ordered by it: matching on cores first handed an 8-core/32-GB
            // request the 4-core/16-GB plan.
            return Ok(match (cores, ram) {
                (..=2, ..=8) => "gpu-2c-8gb",
                (..=4, ..=16) => "gpu-4c-16gb",
                (..=8, ..=32) => Self::LARGEST_GPU,
                _ => {
                    return Err(ProviderError::Capacity(format!(
                        "Vultr publishes no GPU plan above {} vCPU/32 GB; requested {cores}/{ram}",
                        Self::LARGEST_GPU
                    )))
                }
            });
        }
        Ok(match (cores, ram) {
            (..=2, ..=4) => "vhp-2c-4gb",
            (..=2, ..=8) => "vhp-2c-8gb",
            (..=4, ..=16) => "vhp-4c-16gb",
            (..=6, ..=24) => "vhp-6c-24gb",
            (..=8, ..=32) => "vhp-8c-32gb",
            (..=16, ..=64) => Self::LARGEST_CPU,
            _ => {
                return Err(ProviderError::Capacity(format!(
                    "Vultr publishes no plan above {}; requested {cores} cores/{ram} GB",
                    Self::LARGEST_CPU
                )))
            }
        })
    }

    /// Vultr v2 rejects a create without an OS id.
    const OS_ID: &'static str = "1743";
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
            // Spot is not offered on the cloud plans this adapter provisions.
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

        let plan = Self::map_plan(spec)?;
        let label = format!("gb-{}-{}", self.name(), chrono::Utc::now().format("%Y%m%d%H%M%S"));

        // `MachineSpec::use_spot` is deliberately ignored here: Vultr sells spot
        // capacity on accelerated and bare-metal products, not on the cloud plans
        // this adapter provisions, so the flag cannot be honoured and pretending
        // otherwise would quote a price the tenant does not get.
        let body = serde_json::json!({
            "region": region,
            "plan": plan,
            "label": label,
            // v2 rejects a create without an OS id; 1743 is Ubuntu 22.04 LTS.
            "os_id": Self::OS_ID,
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
        let instance_id = instance["id"].as_str().filter(|id| !id.is_empty()).map(str::to_string)
            .ok_or_else(|| {
                ProviderError::Api("Vultr created an instance but returned no id".into())
            })?;

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
                    // Vultr reports bandwidth in GB, not TB.
                    bandwidth_tb: inst["allowed_bandwidth"]
                        .as_u64()
                        .unwrap_or(0) as u32
                        / 1_000,
                    use_spot: false,
                },
                hourly_cost: inst["cost_per_month"]
                    .as_f64()
                    .unwrap_or(0.0)
                    / 730.0,
            })
            .collect())
    }
}

#[cfg(test)]
#[path = "vultr_tests.rs"]
mod tests;
