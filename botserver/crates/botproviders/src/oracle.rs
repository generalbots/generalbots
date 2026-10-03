//! Oracle Cloud Infrastructure adapter (issue #1470).
//!
//! Fills the ARM gap: Ampere A1 shapes are the best price per core for CPU
//! inference, which the x86 hosts cannot match.
//!
//! OCI authenticates with a user API key OCID plus its secret. The trait hands
//! the adapter one credential string, so the two are joined with a colon —
//! `"<key-ocid>:<secret>"` — and [`parse_credential`] splits them. Basic auth is
//! used rather than ADB1 request signing: signing needs the RSA private key in
//! PKCS#8 form and is only required for raw REST callers, whereas Basic auth is
//! supported by the same endpoint and keeps the credential a single opaque
//! secret in Vault.

use async_trait::async_trait;
use base64::Engine;

use crate::{
    ComputeProvider, MachineSpec, ProviderError, ProviderInfo, ProvisionResult, classify_status,
    read_body,
};

/// Oracle signs nothing in the URL path; the region is a query parameter.
const REGION_PARAM: &str = "iaas";
const API_ROOT: &str = "https://oraclecloud.com";
const DEFAULT_REGION: &str = "us-ashburn-1";

/// OCI needs two identifiers the platform cannot invent: the tenancy the
/// instances land in, and the boot image. Both are deployment configuration, so
/// they are constructor arguments rather than constants — a fabricated OCID
/// would fail at the API with an error the operator cannot act on.
#[derive(Debug, Default, Clone)]
pub struct OracleProvider {
    tenancy_ocid: String,
    image_ocid: String,
}

impl OracleProvider {
    /// An unconfigured adapter. Every method that talks to OCI reports the
    /// missing configuration instead of guessing.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adapter bound to a tenancy and a boot image.
    #[must_use]
    pub fn with_config(tenancy_ocid: impl Into<String>, image_ocid: impl Into<String>) -> Self {
        Self { tenancy_ocid: tenancy_ocid.into(), image_ocid: image_ocid.into() }
    }

    /// Rejects an unconfigured adapter before a request is built.
    fn require_config(&self) -> Result<(&str, &str), ProviderError> {
        if self.tenancy_ocid.trim().is_empty() {
            return Err(ProviderError::Auth(
                "Oracle: no tenancy OCID configured; build the adapter with with_config".into(),
            ));
        }
        if self.image_ocid.trim().is_empty() {
            return Err(ProviderError::Auth(
                "Oracle: no boot image OCID configured; build the adapter with with_config".into(),
            ));
        }
        Ok((self.tenancy_ocid.as_str(), self.image_ocid.as_str()))
    }

    /// Splits the joined credential into the key OCID and the secret.
    ///
    /// The secret is a base64 blob that may itself contain `=` padding but never
    /// a colon, so the first colon is a safe separator.
    pub fn parse_credential(credential: &str) -> Result<(String, String), ProviderError> {
        let (key_ocid, secret) = credential.split_once(':').ok_or_else(|| {
            ProviderError::Auth("Oracle: credential must be \"<key-ocid>:<secret>\"".into())
        })?;
        if key_ocid.trim().is_empty() || secret.trim().is_empty() {
            return Err(ProviderError::Auth(
                "Oracle: credential is missing the key OCID or the secret".into(),
            ));
        }
        Ok((key_ocid.to_string(), secret.to_string()))
    }

    /// Maps a spec to a shape.
    ///
    /// Ampere A1 is the reason this adapter exists; the x86 `Flex` families are
    /// the fallback for a spec the bare-metal family does not cover.
    #[must_use]
    pub fn shape(spec: &MachineSpec) -> &'static str {
        let (cores, ram) = (spec.cpu_cores.max(1), spec.ram_gb.max(1));
        if cores <= 4 && ram <= 24 {
            "VM.Standard.A1.Flex"
        } else if cores <= 8 && ram <= 48 {
            "VM.Standard.E4.Flex"
        } else {
            "VM.Standard.E5.Flex"
        }
    }

    /// OCI region ids are full slugs. Anything unrecognised lands in the home
    /// region rather than failing the request.
    #[must_use]
    pub fn region_id(region: &str) -> String {
        match region.trim().to_uppercase().as_str() {
            "US" | "US-EAST" | "ASH" => "us-ashburn-1".to_string(),
            "US-WEST" | "PHX" | "PHOENIX" => "us-phoenix-1".to_string(),
            "EU" | "EU-FRA" | "FRANKFURT" => "eu-frankfurt-1".to_string(),
            "EU-AMS" | "AMSTERDAM" => "eu-amsterdam-1".to_string(),
            "UK" | "LONDON" => "uk-london-1".to_string(),
            "BR" | "SAO PAULO" | "SAOPAULO" => "sa-saopaulo-1".to_string(),
            "JP" | "OSAKA" => "jp-osaka-1".to_string(),
            "AU" | "SYDNEY" => "ap-sydney-1".to_string(),
            // A full OCI slug passes through unchanged; a shorthand like
            // `EU-ZURICH` needs its availability-domain index.
            other if other.starts_with("EU-") => {
                let slug = other.to_lowercase();
                if slug.rsplit('-').next().is_some_and(|tail| tail.chars().all(|c| c.is_ascii_digit())) {
                    slug
                } else {
                    format!("{slug}-1")
                }
            }
            _ => DEFAULT_REGION.to_string(),
        }
    }

    fn auth_header(credential: &str) -> Result<String, ProviderError> {
        let (key_ocid, secret) = Self::parse_credential(credential)?;
        let encoded = base64::engine::general_purpose::STANDARD.encode(format!("{key_ocid}:{secret}"));
        Ok(format!("Basic {encoded}"))
    }

    fn client() -> Result<reqwest::Client, ProviderError> {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(90))
            .build()
            .map_err(|e| ProviderError::Api(format!("Oracle: HTTP client unavailable: {e}")))
    }

    async fn get(&self, path: &str, credential: &str) -> Result<serde_json::Value, ProviderError> {
        let client = Self::client()?;
        let resp = client
            .get(format!("{API_ROOT}{path}"))
            .header("Authorization", Self::auth_header(credential)?)
            .header("accept", "application/json")
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            let headers = resp.headers().clone();
            let body = read_body(resp).await.unwrap_or_default();
            return Err(classify_status("Oracle", status, &headers, &body));
        }
        let text = read_body(resp).await?;
        serde_json::from_str(&text)
            .map_err(|e| ProviderError::Api(format!("Oracle: unreadable response: {e}")))
    }
}

#[async_trait]
impl ComputeProvider for OracleProvider {
    fn name(&self) -> &str {
        "oracle"
    }

    fn info(&self) -> ProviderInfo {
        ProviderInfo {
            name: "oracle".into(),
            display_name: "Oracle Cloud".into(),
            regions: vec![
                "us-ashburn-1".into(),
                "us-phoenix-1".into(),
                "eu-frankfurt-1".into(),
                "eu-amsterdam-1".into(),
                "uk-london-1".into(),
                "sa-saopaulo-1".into(),
                "jp-osaka-1".into(),
                "ap-sydney-1".into(),
            ],
            available_gpus: vec![],
            supports_spot: false,
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
                "Oracle adapter covers CPU shapes only; use a GPU provider for this SKU".into(),
            ));
        }
        let (tenancy_ocid, image_ocid) = self.require_config()?;
        let region_id = Self::region_id(region);
        let shape = Self::shape(spec);
        let display_name = format!("gb-{}", chrono::Utc::now().format("%Y%m%d%H%M%S"));
        let body = serde_json::json!({
            "displayName": display_name,
            "shape": shape,
            "shapeConfig": {
                "ocpus": spec.cpu_cores.max(1),
                "memoryInGBs": spec.ram_gb.max(1),
            },
            "sourceDetails": {
                "sourceType": "image",
                "sourceId": image_ocid,
            },
            "compartmentId": tenancy_ocid,
        });

        let client = Self::client()?;
        let resp = client
            .post(format!(
                "{API_ROOT}/20160918/instances?{REGION_PARAM}={region_id}"
            ))
            .header("Authorization", Self::auth_header(api_key)?)
            .header("content-type", "application/json")
            .header("accept", "application/json")
            .json(&body)
            .send()
            .await?;
        let status = resp.status();
        let headers = resp.headers().clone();
        let text = read_body(resp).await?;
        if !status.is_success() {
            return Err(classify_status("Oracle", status, &headers, &text));
        }
        let parsed: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| ProviderError::Api(format!("Oracle: unreadable create response: {e}")))?;

        let instance_id = parsed["id"].as_str().map(str::to_string).ok_or_else(|| {
            ProviderError::Api("Oracle launched an instance but returned no id".into())
        })?;

        Ok(ProvisionResult {
            provider: "oracle".into(),
            instance_id,
            status: parsed["lifecycle_state"].as_str().unwrap_or("PROVISIONING").into(),
            // A launched compute instance has no address until it reaches RUNNING,
            // so the caller polls `get_status` rather than reading it here.
            ip_address: None,
            region: region_id,
            spec: spec.clone(),
            // OCI returns no price on launch; the caller reconciles the shape
            // against the catalogue.
            hourly_cost: 0.0,
        })
    }

    async fn terminate(&self, instance_id: &str, api_key: &str) -> Result<(), ProviderError> {
        let client = Self::client()?;
        let resp = client
            .delete(format!(
                "{API_ROOT}/20160918/instances/{instance_id}?{REGION_PARAM}={DEFAULT_REGION}"
            ))
            .header("Authorization", Self::auth_header(api_key)?)
            .header("accept", "application/json")
            .send()
            .await?;
        let status = resp.status();
        if status.as_u16() == 404 {
            return Err(ProviderError::NotFound(format!(
                "Oracle: instance {instance_id} not found"
            )));
        }
        if !status.is_success() {
            let headers = resp.headers().clone();
            let body = read_body(resp).await.unwrap_or_default();
            return Err(classify_status("Oracle", status, &headers, &body));
        }
        Ok(())
    }

    async fn get_status(&self, instance_id: &str, api_key: &str) -> Result<String, ProviderError> {
        let path = format!("/20160918/instances/{instance_id}?{REGION_PARAM}={DEFAULT_REGION}");
        let parsed = self.get(&path, api_key).await?;
        Ok(parsed["lifecycle_state"].as_str().unwrap_or("UNKNOWN").into())
    }

    async fn list_instances(&self, api_key: &str) -> Result<Vec<ProvisionResult>, ProviderError> {
        let path = format!("/20160918/instances?{REGION_PARAM}={DEFAULT_REGION}");
        let parsed = self.get(&path, api_key).await?;
        let instances = parsed["data"]
            .as_array()
            .ok_or_else(|| ProviderError::Api("Oracle: response carried no data array".into()))?;

        Ok(instances
            .iter()
            .map(|instance| ProvisionResult {
                provider: "oracle".into(),
                instance_id: instance["id"].as_str().unwrap_or_default().to_string(),
                status: instance["lifecycle_state"].as_str().unwrap_or("UNKNOWN").into(),
                ip_address: None,
                region: DEFAULT_REGION.to_string(),
                spec: MachineSpec {
                    cpu_cores: instance["shape_config"]["ocpus"].as_u64().unwrap_or(1) as u32,
                    ram_gb: instance["shape_config"]["memory_in_gbs"].as_u64().unwrap_or(1) as u32,
                    disk_gb: instance["source_details"]["boot_volume_size_in_gbs"]
                        .as_u64()
                        .unwrap_or(50) as u32,
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
#[path = "oracle_tests.rs"]
mod tests;
