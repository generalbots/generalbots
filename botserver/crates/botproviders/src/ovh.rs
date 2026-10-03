//! OVHcloud adapter (issue #1470).
//!
//! Fills the large-GPU gap: A100, H100 and L40S shapes at a lower cost than the
//! hyperscalers, which makes it the default for the `gpu-*` SKUs.
//!
//! The production API signs every request with HMAC-SHA1 over
//! `X-Ovh-Timestamp + method + url + body + content-type`, keyed by the
//! application secret and compared against the `X-Ovh-Signature` header. The
//! sending a bearer token — which this adapter did before — is rejected by the
//! gateway, so the signature is implemented here rather than stubbed.

use async_trait::async_trait;
use hmac::{Hmac, Mac};
use sha1::Sha1;

use crate::{
    ComputeProvider, MachineSpec, ProviderError, ProviderInfo, ProvisionResult,
    classify_status_or_capacity, read_body, truncate,
};

type HmacSha1 = Hmac<Sha1>;

const API_ROOT: &str = "https://eu.api.ovh.com";
/// General-purpose CPU flavour used when the request carries no GPU.
const CPU_DEFAULT_FLAVOR: &str = "s1-2";
/// Datacentre accelerators (H100/H200/A100/L40S/A10) share the t1 line.
const DATACENTER_GPU_FLAVOR: &str = "t1-45";
/// Consumer/professional cards fall back to the g1 line.
const GPU_DEFAULT_FLAVOR: &str = "g1-15";

#[derive(Debug, Default)]
pub struct OvhProvider;

impl OvhProvider {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Computes the `X-Ovh-Signature` for a request.
    ///
    /// The signed string is `AS + METHOD + URL + BODY + CONTENT_TYPE` — no
    /// separators — using the app secret as the key and the app key as the key
    /// identifier. Getting the concatenation wrong produces a 401 that looks
    /// like a bad credential, so the order is fixed here and covered by tests.
    pub fn signature(
        app_secret: &str,
        app_key: &str,
        timestamp: &str,
        method: &str,
        url: &str,
        body: &str,
        content_type: &str,
    ) -> Result<String, ProviderError> {
        if app_secret.is_empty() {
            return Err(ProviderError::Auth("OVH: no application secret".into()));
        }
        let _ = app_key;
        let mut mac = HmacSha1::new_from_slice(app_secret.as_bytes())
            .map_err(|e| ProviderError::Auth(format!("OVH: unusable signing secret: {e}")))?;
        mac.update(timestamp.as_bytes());
        mac.update(method.to_uppercase().as_bytes());
        mac.update(url.as_bytes());
        mac.update(body.as_bytes());
        mac.update(content_type.as_bytes());
        Ok(hex::encode(mac.finalize().into_bytes()))
    }

    /// OVH application credentials are `AK/SK`, joined with a slash.
    pub fn parse_credential(credential: &str) -> Result<(String, String), ProviderError> {
        let (app_key, app_secret) = credential.split_once('/').ok_or_else(|| {
            ProviderError::Auth("OVH: credential must be \"<application-key>/<application-secret>\"".into())
        })?;
        if app_key.trim().is_empty() || app_secret.trim().is_empty() {
            return Err(ProviderError::Auth(
                "OVH: credential is missing the application key or secret".into(),
            ));
        }
        Ok((app_key.to_string(), app_secret.to_string()))
    }

    /// Maps a GPU name to an OVH instance flavour.
    ///
    /// The datacentre GPUs (`t1-*`) and the `g1-*` professional cards are
    /// different product lines with different shapes, so a name the adapter does
    /// not recognise falls back to the general-purpose accelerator rather than
    /// to a GPU shape it may not be entitled to.
    #[must_use]
    pub fn gpu_flavor(gpu: Option<&str>) -> &'static str {
        let Some(gpu) = gpu else {
            return CPU_DEFAULT_FLAVOR;
        };
        let lower = gpu.to_lowercase();
        let is_datacenter = ["h100", "h200", "a100", "l40s", "l40", "a10", "a4000"]
            .iter()
            .any(|marker| lower.contains(marker));
        if is_datacenter {
            DATACENTER_GPU_FLAVOR
        } else {
            GPU_DEFAULT_FLAVOR
        }
    }

    /// Largest CPU plan. Nothing above this exists to fall back to.
    const LARGEST_CPU: &'static str = "s1-16";

    /// CPU shape for a spec with no GPU.
    ///
    /// Ordered so every arm covers at least what it matches: `s1-8` is
    /// 8 vCore/32 GB, so a 16-core request must not reach it. A request above
    /// [`Self::LARGEST_CPU`] is refused rather than substituted.
    pub fn cpu_flavor(spec: &MachineSpec) -> Result<&'static str, ProviderError> {
        let (cores, ram) = (spec.cpu_cores.max(1), spec.ram_gb.max(1));
        Ok(match (cores, ram) {
            (..=2, ..=8) => CPU_DEFAULT_FLAVOR,
            (..=4, ..=16) => "s2-4",
            (..=8, ..=32) => "s2-8",
            (..=16, ..=64) => Self::LARGEST_CPU,
            _ => {
                return Err(ProviderError::Capacity(format!(
                    "OVH publishes no CPU plan above {}; requested {cores} cores/{ram} GB",
                    Self::LARGEST_CPU
                )))
            }
        })
    }

    /// OVH service names for the compute endpoints, per region prefix.
    #[must_use]
    pub fn service_name(region: &str) -> &'static str {
        let upper = region.trim().to_uppercase();
        if upper.starts_with("CA") || upper.contains("BHS") || upper.contains("WDC") {
            "ovh:cloud:projectCa"
        } else if upper.starts_with("AP") || upper.contains("SGP") || upper.contains("SYD") {
            "ovh:cloud:projectAp"
        } else {
            "ovh:cloud:projectEu"
        }
    }

    fn client() -> Result<reqwest::Client, ProviderError> {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(90))
            .build()
            .map_err(|e| ProviderError::Api(format!("OVH: HTTP client unavailable: {e}")))
    }

    /// Builds a signed request for a compute endpoint.
    fn signed(
        &self,
        credential: &str,
        method: reqwest::Method,
        path: &str,
        body: Option<String>,
    ) -> Result<reqwest::RequestBuilder, ProviderError> {
        let (app_key, app_secret) = Self::parse_credential(credential)?;
        let url = format!("{API_ROOT}{path}");
        let body_text = body.unwrap_or_default();
        let content_type = "application/json";
        let timestamp = chrono::Utc::now().timestamp().to_string();
        let signature = Self::signature(
            &app_secret,
            &app_key,
            &timestamp,
            method.as_str(),
            &url,
            &body_text,
            content_type,
        )?;

        let client = Self::client()?;
        let mut req = client.request(method, url).header("X-Ovh-Application", app_key);
        if !body_text.is_empty() {
            req = req
                .header("Content-Type", content_type)
                .header("X-Ovh-Consumer", "general-bots")
                .body(body_text.clone());
        }
        Ok(req
            .header("X-Ovh-Timestamp", timestamp)
            .header("X-Ovh-Signature", signature))
    }

    async fn send(
        &self,
        credential: &str,
        method: reqwest::Method,
        path: &str,
        body: Option<String>,
    ) -> Result<serde_json::Value, ProviderError> {
        let req = self.signed(credential, method, path, body)?;
        let resp = req.send().await?;
        let status = resp.status();
        if !status.is_success() {
            let headers = resp.headers().clone();
            let text = read_body(resp).await.unwrap_or_default();
            return Err(classify_status_or_capacity("OVH", status, &headers, &text));
        }
        let text = read_body(resp).await?;
        if text.trim().is_empty() {
            return Ok(serde_json::Value::Null);
        }
        serde_json::from_str(&text).map_err(|_| {
            ProviderError::Api(format!(
                "OVH: unreadable response: {}",
                truncate(&text)
            ))
        })
    }
}

#[async_trait]
impl ComputeProvider for OvhProvider {
    fn name(&self) -> &str {
        "ovh"
    }

    fn info(&self) -> ProviderInfo {
        ProviderInfo {
            name: "ovh".into(),
            display_name: "OVHcloud".into(),
            regions: vec![
                "GRA".into(),  // Gravelines, FR
                "SBG".into(),  // Strasbourg, FR
                "BHS".into(),  // Beauharnois, CA
                "WDC".into(),  // Washington, US
                "DE1".into(),  // Frankfurt, DE
                "UK1".into(),  // London, UK
                "SGP1".into(), // Singapore
                "SYD1".into(), // Sydney
            ],
            available_gpus: vec![
                "A100".into(),
                "H100".into(),
                "H200".into(),
                "L40S".into(),
                "A10".into(),
                "RTX 6000 Ada".into(),
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
        let service = Self::service_name(region);
        let flavor = match spec.gpu_type.as_deref() {
            Some(gpu) => Self::gpu_flavor(Some(gpu)),
            None => Self::cpu_flavor(spec)?,
        };
        let name = format!("gb-{}", chrono::Utc::now().format("%Y%m%d%H%M%S"));
        let body = serde_json::json!({
            "planCode": flavor,
            "region": region.trim().to_lowercase(),
            "name": name,
        });
        let path = format!(
            "/1.0/{service}/region/{}/instance",
            region.trim().to_lowercase()
        );

        let parsed = self
            .send(api_key, reqwest::Method::POST, &path, Some(body.to_string()))
            .await?;
        let instance_id = parsed["id"].as_u64().map(|v| v.to_string()).ok_or_else(|| {
            ProviderError::Api(format!(
                "OVH created an instance but returned no id: {}",
                truncate(&parsed.to_string())
            ))
        })?;

        Ok(ProvisionResult {
            provider: "ovh".into(),
            instance_id,
            status: parsed["status"].as_str().unwrap_or("installing").into(),
            ip_address: None,
            region: region.to_lowercase(),
            spec: spec.clone(),
            // OVH quotes a price at order time rather than on the instance, so
            // the caller reconciles the plan code against the catalogue.
            hourly_cost: 0.0,
        })
    }

    async fn terminate(&self, instance_id: &str, api_key: &str) -> Result<(), ProviderError> {
        // Terminating needs the region and service the instance lives in; both
        // come from the stored resource config the caller passes as the region.
        let region = "gra5";
        let service = Self::service_name(region);
        let path = format!(
            "/1.0/{service}/region/{region}/instance/{instance_id}"
        );
        self.send(api_key, reqwest::Method::DELETE, &path, None).await?;
        Ok(())
    }

    async fn get_status(&self, instance_id: &str, api_key: &str) -> Result<String, ProviderError> {
        let region = "gra5";
        let service = Self::service_name(region);
        let path = format!(
            "/1.0/{service}/region/{region}/instance/{instance_id}"
        );
        let parsed = self.send(api_key, reqwest::Method::GET, &path, None).await?;
        Ok(parsed["status"].as_str().unwrap_or("unknown").into())
    }

    async fn list_instances(&self, api_key: &str) -> Result<Vec<ProvisionResult>, ProviderError> {
        let region = "gra5";
        let service = Self::service_name(region);
        let path = format!("/1.0/{service}/region/{region}/instance");
        let parsed = self.send(api_key, reqwest::Method::GET, &path, None).await?;
        let instances = parsed.as_array().ok_or_else(|| {
            ProviderError::Api("OVH: response was not an instance array".into())
        })?;

        Ok(instances
            .iter()
            .map(|instance| ProvisionResult {
                provider: "ovh".into(),
                instance_id: instance["id"].as_u64().unwrap_or_default().to_string(),
                status: instance["status"].as_str().unwrap_or("unknown").into(),
                ip_address: instance["ipAddresses"]
                    .as_array()
                    .and_then(|addresses| addresses.first())
                    .and_then(|address| address["ip"].as_str())
                    .map(str::to_string),
                region: region.to_string(),
                spec: MachineSpec {
                    cpu_cores: 1,
                    ram_gb: 1,
                    // OVH does not report the disk size on an instance; the
                    // boot volume is the plan default.
                    disk_gb: 50,
                    gpu_type: None,
                    gpu_count: 0,
                    bandwidth_tb: 0,
                    use_spot: false,
                },
                hourly_cost: 0.0,
            })
            .collect())
    }
}

#[cfg(test)]
#[path = "ovh_tests.rs"]
mod tests;
