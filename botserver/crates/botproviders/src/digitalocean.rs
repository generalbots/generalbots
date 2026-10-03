//! DigitalOcean adapter (issue #1470).
//!
//! Strong droplet lifecycle API and simple billing; the smallest and most
//! predictable of the new hosts, which suits the `vps-small` and `vps-medium`
//! SKUs.

use async_trait::async_trait;

use crate::{
    ComputeProvider, MachineSpec, ProviderError, ProviderInfo, ProvisionResult, classify_status,
    read_body,
};

const API: &str = "https://api.digitalocean.com/v2";

#[derive(Debug, Default)]
pub struct DigitalOceanProvider;

impl DigitalOceanProvider {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Largest published slug. Nothing above this exists to fall back to.
    const LARGEST: &'static str = "s-16vcpu-32gb";

    /// Maps a spec to a droplet slug.
    ///
    /// DigitalOcean publishes fixed slugs per size, so a spec rounds up to the
    /// next slug that covers both CPU and RAM. A spec above [`Self::LARGEST`] is
    /// an error rather than a substitution — handing a 32-core request the
    /// biggest 16-vCPU droplet would under-provision the tenant silently.
    pub fn slug(spec: &MachineSpec) -> Result<&'static str, ProviderError> {
        let (cores, ram) = (spec.cpu_cores.max(1), spec.ram_gb.max(1));
        Ok(match (cores, ram) {
            (1..=1, ..=1) => "s-1vcpu-1gb",
            (1..=1, ..=2) => "s-1vcpu-2gb",
            (..=2, ..=4) => "s-2vcpu-4gb",
            (..=4, ..=8) => "s-4vcpu-8gb",
            (..=8, ..=16) => "s-8vcpu-16gb",
            (..=16, ..=32) => Self::LARGEST,
            _ => {
                return Err(ProviderError::Capacity(format!(
                    "DigitalOcean has no droplet for {cores} vCPU/{ram} GB; the largest is {}",
                    Self::LARGEST
                )))
            }
        })
    }

    /// Maps a spec to a region slug.
    #[must_use]
    pub fn region_slug(region: &str) -> String {
        match region.trim().to_uppercase().as_str() {
            "AMS" | "EU" | "EUROPE" => "ams3",
            "LON" | "UK" => "lon1",
            "FRA" | "DE" => "fra1",
            "TOR" | "CA" => "tor1",
            "SGP" | "SG" | "ASIA" => "sgp1",
            "BLR" | "IN" => "blr1",
            "SYD" | "AU" => "syd1",
            _ => "nyc3",
        }
        .to_string()
    }

    /// Disk in GB for a slug — DigitalOcean bundles it and bills overage, so the
    /// plan size is what the tenant actually gets at the quoted price.
    #[must_use]
    pub fn disk_gb_for(slug: &str) -> u32 {
        match slug {
            "s-1vcpu-1gb" | "s-1vcpu-2gb" => 25,
            "s-2vcpu-4gb" => 80,
            "s-4vcpu-8gb" => 160,
            "s-8vcpu-16gb" => 320,
            _ => 640,
        }
    }

    fn client() -> Result<reqwest::Client, ProviderError> {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| ProviderError::Api(format!("DigitalOcean: HTTP client unavailable: {e}")))
    }

    async fn get(&self, path: &str, api_key: &str) -> Result<serde_json::Value, ProviderError> {
        let client = Self::client()?;
        let resp = client
            .get(format!("{API}{path}"))
            .header("Authorization", format!("Bearer {api_key}"))
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            let headers = resp.headers().clone();
            let body = read_body(resp).await.unwrap_or_default();
            return Err(classify_status("DigitalOcean", status, &headers, &body));
        }
        let text = read_body(resp).await?;
        serde_json::from_str(&text)
            .map_err(|e| ProviderError::Api(format!("DigitalOcean: unreadable response: {e}")))
    }
}

#[async_trait]
impl ComputeProvider for DigitalOceanProvider {
    fn name(&self) -> &str {
        "digitalocean"
    }

    fn info(&self) -> ProviderInfo {
        ProviderInfo {
            name: "digitalocean".into(),
            display_name: "DigitalOcean".into(),
            regions: vec![
                "nyc3".into(),
                "ams3".into(),
                "fra1".into(),
                "lon1".into(),
                "tor1".into(),
                "sgp1".into(),
                "blr1".into(),
                "syd1".into(),
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
                "DigitalOcean has no GPU droplets; use a GPU provider for this SKU".into(),
            ));
        }
        let slug = Self::slug(spec)?;
        let region_slug = Self::region_slug(region);
        let name = format!("gb-{}", chrono::Utc::now().format("%Y%m%d%H%M%S"));
        // Spot requests carry no SSH key material, so the instance is created
        // with agent-based provisioning: a bootstrap droplet can enrol it.
        let body = serde_json::json!({
            "name": name,
            "region": region_slug,
            "size": slug,
            "image": "ubuntu-24-04-x64",
            "tags": ["general-bots"],
            "enable_ipv6": true,
            "with_droplet_agent": !spec.use_spot,
        });

        let client = Self::client()?;
        let resp = client
            .post(format!("{API}/droplets"))
            .header("Authorization", format!("Bearer {api_key}"))
            .json(&body)
            .send()
            .await?;
        let status = resp.status();
        let headers = resp.headers().clone();
        let text = read_body(resp).await?;
        if !status.is_success() {
            return Err(classify_status("DigitalOcean", status, &headers, &text));
        }
        let parsed: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| ProviderError::Api(format!("DigitalOcean: unreadable create response: {e}")))?;

        let droplet = &parsed["droplet"];
        let instance_id = droplet["id"].as_i64().map(|v| v.to_string()).ok_or_else(|| {
            ProviderError::Api("DigitalOcean created a droplet but returned no id".into())
        })?;

        Ok(ProvisionResult {
            provider: "digitalocean".into(),
            instance_id,
            status: droplet["status"].as_str().unwrap_or("new").into(),
            ip_address: droplet["networks"]["v4"]
                .as_array()
                .and_then(|nets| nets.first())
                .and_then(|net| net["ip_address"].as_str())
                .map(str::to_string),
            region: region_slug,
            spec: MachineSpec {
                disk_gb: Self::disk_gb_for(slug),
                ..spec.clone()
            },
            // DigitalOcean does not return a price on create; the caller
            // reconciles the plan slug against the catalogue.
            hourly_cost: 0.0,
        })
    }

    async fn terminate(&self, instance_id: &str, api_key: &str) -> Result<(), ProviderError> {
        let client = Self::client()?;
        let resp = client
            .delete(format!("{API}/droplets/{instance_id}"))
            .header("Authorization", format!("Bearer {api_key}"))
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            let headers = resp.headers().clone();
            let body = read_body(resp).await.unwrap_or_default();
            return Err(classify_status("DigitalOcean", status, &headers, &body));
        }
        Ok(())
    }

    async fn get_status(&self, instance_id: &str, api_key: &str) -> Result<String, ProviderError> {
        let parsed = self.get(&format!("/droplets/{instance_id}"), api_key).await?;
        Ok(parsed["droplet"]["status"].as_str().unwrap_or("unknown").into())
    }

    async fn list_instances(&self, api_key: &str) -> Result<Vec<ProvisionResult>, ProviderError> {
        let parsed = self.get("/droplets?per_page=50", api_key).await?;
        let droplets = parsed["droplets"]
            .as_array()
            .ok_or_else(|| ProviderError::Api("DigitalOcean: response carried no droplets array".into()))?;

        Ok(droplets
            .iter()
            .map(|droplet| {
                let slug = droplet["size_slug"].as_str().unwrap_or_default();
                ProvisionResult {
                    provider: "digitalocean".into(),
                    instance_id: droplet["id"].as_i64().unwrap_or_default().to_string(),
                    status: droplet["status"].as_str().unwrap_or("unknown").into(),
                    ip_address: droplet["networks"]["v4"]
                        .as_array()
                        .and_then(|nets| nets.first())
                        .and_then(|net| net["ip_address"].as_str())
                        .map(str::to_string),
                    region: droplet["region"]["slug"].as_str().unwrap_or_default().to_string(),
                    spec: MachineSpec {
                        cpu_cores: droplet["vcpus"].as_u64().unwrap_or(1) as u32,
                        ram_gb: droplet["memory_mib"].as_u64().unwrap_or(1024) as u32 / 1024,
                        disk_gb: Self::disk_gb_for(slug),
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
#[path = "digitalocean_tests.rs"]
mod tests;
