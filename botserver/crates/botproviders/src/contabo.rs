use crate::{
    ComputeProvider, MachineSpec, ProviderError, ProviderInfo, ProvisionResult, classify_status,
};
use async_trait::async_trait;
use hmac::Mac;

/// Contabo signs every request with HMAC-SHA1, keyed by the API token secret.
/// A bearer token alone is rejected by the gateway, so this adapter owns the
/// signing helper rather than reusing another provider's.
const SIGN_HEADER: &str = "X-Contabo";
const CONTENT_TYPE_JSON: &str = "application/json";

/// Contabo's documented digest for an absent body, which every `GET` and
/// `DELETE` sends.
pub const EMPTY_BODY_MD5: &str = "d41d8cd98f00b204e9800998ecf8427e";

/// Computes the `X-Contabo` header value.
///
/// Layout: `X-Contabo <timestamp>,<consumer>,<hmac>`, where the digest covers
/// `timestamp|consumer|method|uri|body-md5`. Contabo uses a pipe separator, not
/// a newline, and `GET` is uppercased before signing — both are easy to get
/// wrong and produce a 401 that reads like a bad credential.
pub fn sign(
    secret: &str,
    timestamp: &str,
    consumer: &str,
    method: &str,
    uri: &str,
    body_md5: &str,
) -> String {
    let payload = format!("{timestamp}|{consumer}|{}|{uri}|{body_md5}", method.to_uppercase());
    let digest =
        hmac::Hmac::<sha1::Sha1>::new_from_slice(secret.as_bytes())
            .map(|mut mac| {
                mac.update(payload.as_bytes());
                hex::encode(mac.finalize().into_bytes())
            })
            // HMAC accepts a key of any length, so this arm is unreachable in
            // practice; returning an empty digest surfaces the problem as a 401
            // instead of aborting the process.
            .unwrap_or_default();
    format!("{SIGN_HEADER} {timestamp},{consumer},{digest}")
}

#[derive(Debug, Default)]
pub struct ContaboProvider;

impl ContaboProvider {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Contabo credentials are `<token-id>:<token-secret>`; the token id is the
    /// consumer half of the signature.
    pub fn parse_credential(credential: &str) -> Result<(String, String), ProviderError> {
        let (consumer, secret) = credential.split_once(':').ok_or_else(|| {
            ProviderError::Auth("Contabo: credential must be \"<token-id>:<token-secret>\"".into())
        })?;
        if consumer.trim().is_empty() || secret.trim().is_empty() {
            return Err(ProviderError::Auth(
                "Contabo: credential is missing the token id or secret".into(),
            ));
        }
        Ok((consumer.to_string(), secret.to_string()))
    }

    /// Attaches Contabo's signature headers to a request.
    fn signed(
        &self,
        credential: &str,
        method: reqwest::Method,
        url: &str,
        body: Option<String>,
    ) -> Result<reqwest::RequestBuilder, ProviderError> {
        let (consumer, secret) = Self::parse_credential(credential)?;
        let timestamp = chrono::Utc::now().timestamp().to_string();
        let uri = url.strip_prefix("https://api.contabo.com").unwrap_or(url);
        let signature = sign(&secret, &timestamp, &consumer, method.as_str(), uri, EMPTY_BODY_MD5);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(90))
            .build()
            .map_err(|e| ProviderError::Api(format!("Contabo: HTTP client unavailable: {e}")))?;
        let mut req = client
            .request(method, url)
            .header("Authorization", format!("Bearer {consumer}"))
            .header(SIGN_HEADER, signature);
        if let Some(body_text) = body {
            req = req.header("Content-Type", CONTENT_TYPE_JSON).body(body_text);
        }
        Ok(req)
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
        let region_str = Self::map_region(region);
        let label = format!("gb-{}", chrono::Utc::now().format("%Y%m%d%H%M%S"));
        let body = serde_json::json!({
            "displayName": label,
            "region": region_str,
            "productId": Self::map_gpu(spec),
            "imageId": "ubuntu-22.04",
            "sshKeys": [],
            "period": 1,
            "extraStorageGb": spec.disk_gb.max(50).saturating_sub(50),
            "ramMb": (spec.ram_gb * 1024).max(16384),
            "cpuCores": spec.cpu_cores.max(4),
        });

        let path = "/v1/compute/instances";
        let req = self.signed(
            api_key,
            reqwest::Method::POST,
            &format!("https://api.contabo.com{path}"),
            Some(body.to_string()),
        )?;
        let resp = req.send().await?;
        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(classify_status("Contabo", status, &reqwest::header::HeaderMap::new(), &text));
        }

        let text = resp.text().await.unwrap_or_default();
        let parsed: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| ProviderError::Api(format!("Contabo: unreadable create response: {e}")))?;

        let instance_id = parsed["data"][0]["instanceId"]
            .as_str()
            .or_else(|| parsed["data"][0]["id"].as_str())
            .map(str::to_string)
            .ok_or_else(|| {
                ProviderError::Api(format!(
                    "Contabo created an instance but returned no id: {}",
                    crate::truncate(&text)
                ))
            })?;

        Ok(ProvisionResult {
            provider: "contabo".into(),
            instance_id,
            status: "provisioning".into(),
            ip_address: None,
            region: region_str.into(),
            spec: spec.clone(),
            // Contabo prices per product, not per instance; the caller
            // reconciles the product id against the catalogue.
            hourly_cost: 0.0,
        })
    }

    async fn terminate(&self, instance_id: &str, api_key: &str) -> Result<(), ProviderError> {
        let path = format!("/v1/compute/instances/{instance_id}");
        let req = self.signed(
            api_key,
            reqwest::Method::DELETE,
            &format!("https://api.contabo.com{path}"),
            None,
        )?;
        let resp = req.send().await?;
        let status = resp.status();
        if status.as_u16() == 404 {
            return Err(ProviderError::NotFound(format!(
                "Contabo: instance {instance_id} not found"
            )));
        }
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(classify_status("Contabo", status, &reqwest::header::HeaderMap::new(), &text));
        }
        Ok(())
    }

    async fn get_status(&self, instance_id: &str, api_key: &str) -> Result<String, ProviderError> {
        let parsed = self.fetch_instance(instance_id, api_key).await?;
        Ok(parsed["data"][0]["status"].as_str().unwrap_or("unknown").to_string())
    }

    async fn list_instances(&self, api_key: &str) -> Result<Vec<ProvisionResult>, ProviderError> {
        let parsed = self
            .send_signed(api_key, reqwest::Method::GET, "/v1/compute/instances", None)
            .await?;
        let instances = parsed["data"]
            .as_array()
            .ok_or_else(|| ProviderError::Api("Contabo: response carried no data array".into()))?;

        Ok(instances
            .iter()
            .map(|inst| ProvisionResult {
                provider: "contabo".into(),
                instance_id: inst["instanceId"]
                    .as_str()
                    .or_else(|| inst["id"].as_str())
                    .unwrap_or_default()
                    .to_string(),
                status: inst["status"].as_str().unwrap_or("unknown").into(),
                ip_address: inst["ipAddress"]
                    .as_str()
                    .or_else(|| inst["ipConfig"]["v4"]["ip"].as_str())
                    .map(str::to_string),
                region: inst["region"].as_str().unwrap_or("EU-WEST").into(),
                spec: MachineSpec {
                    cpu_cores: inst["cpuCores"].as_u64().unwrap_or(4) as u32,
                    ram_gb: (inst["ramMb"].as_u64().unwrap_or(16384) / 1024) as u32,
                    // Contabo's 50 GB base disk is included in the plan, so only
                    // the extra is reported here.
                    disk_gb: inst["extraStorageGb"].as_u64().unwrap_or(0) as u32 + 50,
                    gpu_type: inst["productId"].as_str().map(str::to_string),
                    gpu_count: 1,
                    bandwidth_tb: inst["bandwidthLimitTb"].as_u64().unwrap_or(0) as u32,
                    use_spot: false,
                },
                hourly_cost: inst["hourlyCost"].as_f64().unwrap_or(0.0),
            })
            .collect())
    }
}

impl ContaboProvider {
    async fn send_signed(
        &self,
        credential: &str,
        method: reqwest::Method,
        path: &str,
        body: Option<String>,
    ) -> Result<serde_json::Value, ProviderError> {
        let req = self.signed(credential, method, &format!("https://api.contabo.com{path}"), body)?;
        let resp = req.send().await?;
        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(classify_status("Contabo", status, &reqwest::header::HeaderMap::new(), &text));
        }
        let text = resp.text().await.unwrap_or_default();
        serde_json::from_str(&text)
            .map_err(|e| ProviderError::Api(format!("Contabo: unreadable response: {e}")))
    }

    async fn fetch_instance(
        &self,
        instance_id: &str,
        credential: &str,
    ) -> Result<serde_json::Value, ProviderError> {
        self.send_signed(
            credential,
            reqwest::Method::GET,
            &format!("/v1/compute/instances/{instance_id}"),
            None,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sign_at(ts: &str, method: &str, uri: &str) -> String {
        sign("secret", ts, "42", method, uri, EMPTY_BODY_MD5)
    }

    #[test]
    fn the_signature_header_carries_timestamp_consumer_and_digest() {
        let header = sign_at("1700000000", "GET", "/v1/compute/instances");
        assert!(header.starts_with("X-Contabo 1700000000,42,"));
        let digest = header.rsplit(',').next().expect("digest segment");
        assert_eq!(digest.len(), 40, "HMAC-SHA1 is 20 bytes, hex-encoded");
    }

    #[test]
    fn the_signature_is_stable_for_fixed_inputs() {
        assert_eq!(
            sign_at("1700000000", "GET", "/v1/compute/instances"),
            sign_at("1700000000", "GET", "/v1/compute/instances")
        );
    }

    #[test]
    fn every_signed_field_changes_the_digest() {
        let reference = sign_at("1700000000", "GET", "/v1/compute/instances");
        assert_ne!(reference, sign_at("1700000001", "GET", "/v1/compute/instances"));
        assert_ne!(reference, sign_at("1700000000", "DELETE", "/v1/compute/instances"));
        assert_ne!(reference, sign_at("1700000000", "GET", "/v1/compute/instances/7"));
        let other_secret = sign("different", "1700000000", "42", "GET", "/v1/compute/instances", EMPTY_BODY_MD5);
        assert_ne!(reference, other_secret);
    }

    #[test]
    fn the_method_is_uppercased_before_signing() {
        assert_eq!(
            sign_at("1700000000", "get", "/v1/compute/instances"),
            sign_at("1700000000", "GET", "/v1/compute/instances")
        );
    }

    #[test]
    fn the_body_digest_is_part_of_the_signed_string() {
        let with_empty = sign("s", "1", "42", "POST", "/v1/x", EMPTY_BODY_MD5);
        let with_other = sign("s", "1", "42", "POST", "/v1/x", "098f6bcd4621d373cade4e832627b4f6");
        assert_ne!(with_empty, with_other);
    }

    #[test]
    fn credentials_must_be_id_colon_secret() {
        assert!(ContaboProvider::parse_credential("42:secret").is_ok());
        assert!(matches!(
            ContaboProvider::parse_credential("no-separator"),
            Err(ProviderError::Auth(_))
        ));
        assert!(matches!(
            ContaboProvider::parse_credential(":secret"),
            Err(ProviderError::Auth(_))
        ));
    }

    #[test]
    fn regions_and_gpus_map_to_catalog_values() {
        assert_eq!(ContaboProvider::map_region("us"), "US-EAST");
        assert_eq!(ContaboProvider::map_region("unknown"), "EU-WEST");
        let spec = MachineSpec {
            gpu_type: Some("A100".into()),
            ..MachineSpec::default()
        };
        assert_eq!(ContaboProvider::map_gpu(&spec), "A100-80GB");
        assert_eq!(ContaboProvider::map_gpu(&MachineSpec::default()), "RTX-4090");
    }
}
