//! Hetzner Cloud adapter (issue #1470).
//!
//! Fills the gap of a cheap European VPS with a strong REST API and no GPU
//! line, which makes it uncontested for the `vps-*` SKUs.

use async_trait::async_trait;

use crate::{
    ComputeProvider, MachineSpec, ProviderError, ProviderInfo, ProvisionResult,
    classify_status_or_capacity, read_body, truncate,
};

const API: &str = "https://api.hetzner.cloud/v1";

#[derive(Debug, Default)]
pub struct HetznerProvider;

impl HetznerProvider {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Largest published type. Nothing above this exists to fall back to.
    const LARGEST: &'static str = "ccx93";

    /// Maps a spec to a server type.
    ///
    /// Hetzner prices by named type, so a spec that does not land on one of the
    /// published shapes is rounded up to the next larger type. Rounding down
    /// would provision a machine smaller than the customer paid for.
    ///
    /// A spec larger than [`Self::LARGEST`] is an error rather than a
    /// substitution: silently handing a 32-core request the biggest 16-core plan
    /// is how a tenant ends up under-provisioned without knowing.
    pub fn server_type(spec: &MachineSpec) -> Result<&'static str, ProviderError> {
        let (cores, ram) = (spec.cpu_cores.max(1), spec.ram_gb.max(1));
        Ok(match (cores, ram) {
            (1..=1, ..=2) => "cx22",
            (..=2, ..=4) => "cx32",
            (..=2, ..=8) => "cx42",
            (..=3, ..=8) => "cpx31",
            (..=4, ..=8) => "cx52",
            (..=4, ..=16) => "cpx41",
            (..=8, ..=16) => "cx62",
            (..=8, ..=32) => "cpx51",
            (..=16, ..=64) => "ccx63",
            (..=16, ..=128) => Self::LARGEST,
            _ => {
                return Err(ProviderError::Capacity(format!(
                    "Hetzner has no type for {cores} cores/{ram} GB; the largest is {}",
                    Self::LARGEST
                )))
            }
        })
    }

    /// Maps a spec to a location id.
    ///
    /// The platform passes a region slug ("US", "EU"); Hetzner wants a two- to
    /// three-letter location. Unknown slugs fall back to Helsinki, the location
    /// with the widest stock.
    #[must_use]
    pub fn location_id(region: &str) -> &'static str {
        match region.trim().to_uppercase().as_str() {
            "US" | "NYC" | "NY" => "ash",
            "ASH" | "ASHBURN" => "ash",
            "HIL" | "HELSINKI" | "EU" => "hel1",
            "NBG" | "NUREMBERG" => "nbg1",
            "FSN" | "FALKENSTEIN" => "fsn1",
            "HEL" => "hel1",
            _ => "hel1",
        }
    }

    /// Hetzner ignores disk requests: every plan ships its own local NVMe and
    /// volumes are billed separately. Reporting the plan size keeps the recorded
    /// spec honest about what the customer received.
    #[must_use]
    pub fn disk_gb_for(server_type: &str) -> u32 {
        match server_type {
            "cx22" | "cx32" => 40,
            "cx42" | "cpx31" => 80,
            "cx52" | "cpx41" => 160,
            "cx62" | "cpx51" => 240,
            "ccx63" => 360,
            _ => 480,
        }
    }

    fn client() -> Result<reqwest::Client, ProviderError> {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| ProviderError::Api(format!("Hetzner: HTTP client unavailable: {e}")))
    }

    async fn fetch(
        method: reqwest::Method,
        path: &str,
        api_key: &str,
    ) -> Result<serde_json::Value, ProviderError> {
        let client = Self::client()?;
        let resp = client
            .request(method, format!("{API}{path}"))
            .header("Authorization", format!("Bearer {api_key}"))
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            let headers = resp.headers().clone();
            let body = read_body(resp).await.unwrap_or_default();
            return Err(classify_status_or_capacity("Hetzner", status, &headers, &body));
        }
        let text = read_body(resp).await?;
        serde_json::from_str(&text)
            .map_err(|e| ProviderError::Api(format!("Hetzner: unreadable response: {e}")))
    }
}

#[async_trait]
impl ComputeProvider for HetznerProvider {
    fn name(&self) -> &str {
        "hetzner"
    }

    fn info(&self) -> ProviderInfo {
        ProviderInfo {
            name: "hetzner".into(),
            display_name: "Hetzner Cloud".into(),
            regions: vec![
                "ash".into(),   // Ashburn, VA
                "hel1".into(),  // Helsinki
                "nbg1".into(),  // Nuremberg
                "fsn1".into(),  // Falkenstein
            ],
            available_gpus: vec![],
            supports_spot: true,
        }
    }

    async fn provision(
        &self,
        spec: &MachineSpec,
        region: &str,
        api_key: &str,
    ) -> Result<ProvisionResult, ProviderError> {
        if spec.gpu_type.is_some() {
            return Err(ProviderError::Capacity(
                "Hetzner Cloud has no GPU line; use a GPU provider for this SKU".into(),
            ));
        }
        let server_type = Self::server_type(spec)?;
        let location = Self::location_id(region);
        let name = format!("gb-{}", chrono::Utc::now().format("%Y%m%d%H%M%S"));
        let body = serde_json::json!({
            "name": name,
            "server_type": server_type,
            "location": location,
            "image": "ubuntu-24.04",
            "start_after_create": true,
            "labels": { "managed-by": "general-bots" },
        });

        let client = Self::client()?;
        let resp = client
            .post(format!("{API}/servers"))
            .header("Authorization", format!("Bearer {api_key}"))
            .json(&body)
            .send()
            .await?;
        let status = resp.status();
        let headers = resp.headers().clone();
        let text = read_body(resp).await?;
        if !status.is_success() {
            return Err(classify_status_or_capacity("Hetzner", status, &headers, &text));
        }
        let parsed: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| ProviderError::Api(format!("Hetzner: unreadable create response: {e}")))?;

        let id = parsed["server"]["id"].as_i64().map(|v| v.to_string());
        let instance_id = id.ok_or_else(|| {
            ProviderError::Api(format!(
                "Hetzner created a server but returned no id: {}",
                truncate(&text)
            ))
        })?;

        let ipv4 = parsed["server"]["public_net"]["ipv4"]["ip"]
            .as_str()
            .map(str::to_string);

        Ok(ProvisionResult {
            provider: "hetzner".into(),
            instance_id,
            status: parsed["server"]["status"].as_str().unwrap_or("initializing").into(),
            ip_address: ipv4,
            region: location.to_string(),
            spec: MachineSpec {
                disk_gb: Self::disk_gb_for(server_type),
                ..spec.clone()
            },
            // Hetzner returns no price on create; the price is a property of the
            // server type and is reconciled by the caller against the catalogue.
            hourly_cost: 0.0,
        })
    }

    async fn terminate(&self, instance_id: &str, api_key: &str) -> Result<(), ProviderError> {
        Self::fetch(reqwest::Method::DELETE, &format!("/servers/{instance_id}"), api_key).await?;
        Ok(())
    }

    async fn get_status(&self, instance_id: &str, api_key: &str) -> Result<String, ProviderError> {
        let parsed = Self::fetch(reqwest::Method::GET, &format!("/servers/{instance_id}"), api_key).await?;
        Ok(parsed["server"]["status"].as_str().unwrap_or("unknown").into())
    }

    async fn list_instances(&self, api_key: &str) -> Result<Vec<ProvisionResult>, ProviderError> {
        let parsed = Self::fetch(reqwest::Method::GET, "/servers?per_page=50", api_key).await?;
        let servers = parsed["servers"]
            .as_array()
            .ok_or_else(|| ProviderError::Api("Hetzner: response carried no servers array".into()))?;

        Ok(servers
            .iter()
            .map(|server| {
                let server_type = server["server_type"]["name"].as_str().unwrap_or_default();
                ProvisionResult {
                    provider: "hetzner".into(),
                    instance_id: server["id"].as_i64().unwrap_or_default().to_string(),
                    status: server["status"].as_str().unwrap_or("unknown").into(),
                    ip_address: server["public_net"]["ipv4"]["ip"].as_str().map(str::to_string),
                    region: server["datacenter"]["location"]["name"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string(),
                    spec: MachineSpec {
                        cpu_cores: server["server_type"]["cores"].as_u64().unwrap_or(1) as u32,
                        ram_gb: server["server_type"]["memory"].as_f64().unwrap_or(1.0) as u32,
                        disk_gb: Self::disk_gb_for(server_type),
                        gpu_type: None,
                        gpu_count: 0,
                        bandwidth_tb: 0,
                        use_spot: false,
                    },
                    hourly_cost: 0.0,
                }
            })
            .collect())
    }
}

#[cfg(test)]
#[path = "hetzner_tests.rs"]
mod tests;
