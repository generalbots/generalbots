//! SigV4 request executor for the AWS provider adapter (#950).
//!
//! Credentials arrive strictly from the Vault envelope loaded by
//! [`crate::providers::invoke_registered`]; they are never logged and never
//! included in error strings. Signing uses the pinned `aws-sigv4` crate with
//! a fixed-offset system clock; responses are size-capped before parsing.

use std::sync::OnceLock;
use std::time::{Duration, SystemTime};

use aws_credential_types::Credentials;
use aws_sigv4::http_request::{
    sign, PayloadChecksumKind, SignableBody, SignableRequest, SigningSettings,
};
use aws_sigv4::sign::v4::SigningParams;
use reqwest::Method;
use serde_json::Value;

const CREDENTIAL_PROVIDER_NAME: &str = "botintegrations-aws";
const REQUEST_TIMEOUT_SECS: u64 = 15;
/// Hard cap applied to every provider response body (1 MiB).
pub(crate) const MAX_RESPONSE_BYTES: usize = 1024 * 1024;

pub(crate) const ERR_SIGNING: &str = "signing_failed";
pub(crate) const ERR_NETWORK: &str = "provider_unreachable";
pub(crate) const ERR_RESPONSE_CAP: &str = "response_too_large";

/// Parsed AWS credential envelope from Vault.
#[derive(Debug, Clone)]
pub(crate) struct AwsCreds {
    pub(crate) access_key_id: String,
    pub(crate) secret_access_key: String,
    pub(crate) session_token: Option<String>,
}

impl AwsCreds {
    /// Validates required `access_key_id`/`secret_access_key`; optional
    /// `session_token` enables temporary (STS) credentials. Missing required
    /// keys are validation failures reported before any network activity.
    pub(crate) fn parse(credentials: &Value) -> Result<Self, String> {
        let object = match credentials {
            Value::Object(map) => map,
            _ => {
                return Err(
                    "invalid_request: stored credential envelope must be an object".to_string(),
                )
            }
        };
        let required = |key: &str| -> Result<String, String> {
            let value = object
                .get(key)
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|text| !text.is_empty())
                .ok_or_else(|| {
                    format!("invalid_request: credential key {key} is missing or empty")
                })?;
            Ok(value.to_string())
        };
        let session_token = match object.get("session_token") {
            None | Some(Value::Null) => None,
            Some(Value::String(text)) if text.trim().is_empty() => None,
            Some(Value::String(text)) => Some(text.trim().to_string()),
            Some(_) => {
                return Err(
                    "invalid_request: credential key session_token must be a string".to_string(),
                )
            }
        };
        Ok(Self {
            access_key_id: required("access_key_id")?,
            secret_access_key: required("secret_access_key")?,
            session_token,
        })
    }
}

fn shared_client() -> Result<&'static reqwest::Client, String> {
    static CLIENT: OnceLock<Option<reqwest::Client>> = OnceLock::new();
    CLIENT
        .get_or_init(|| {
            reqwest::Client::builder()
                .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
                .connect_timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
                .build()
                .map_err(|error| {
                    log::error!("shared AWS http client init failed: {error}");
                })
                .ok()
        })
        .as_ref()
        .ok_or_else(|| ERR_NETWORK.to_string())
}

/// Fully specified outbound AWS request. Grouping the parameters keeps the
/// executor signatures small and makes every call site self-describing.
pub(crate) struct RequestPlan<'a> {
    pub(crate) service: &'a str,
    pub(crate) region: &'a str,
    pub(crate) method: Method,
    pub(crate) host: String,
    pub(crate) path_and_query: String,
    pub(crate) body: Option<Vec<u8>>,
    pub(crate) content_type: Option<&'a str>,
    pub(crate) extra_headers: Vec<(&'static str, String)>,
    /// Hard cap applied to the response body of this request.
    pub(crate) response_cap: usize,
}

/// Computes the full signed header set for one request, including
/// `authorization`, `x-amz-date`, `x-amz-content-sha256` and, when the
/// envelope carries a session token, `x-amz-security-token`. Only headers
/// newly computed by the signer are returned; supplied ones are already on
/// the request.
///
/// Pure function over its inputs so tests can pin the clock deterministically.
pub(crate) fn signed_headers(
    plan: &RequestPlan<'_>,
    payload: &[u8],
    creds: &AwsCreds,
    time: SystemTime,
) -> Result<Vec<(String, String)>, String> {
    let uri = format!("https://{}{}", plan.host, plan.path_and_query);
    let mut supplied: Vec<(String, String)> = vec![("host".to_string(), plan.host.clone())];
    if let Some(content_type) = plan.content_type {
        supplied.push(("content-type".to_string(), content_type.to_string()));
    }
    for (name, value) in &plan.extra_headers {
        supplied.push(((*name).to_string(), value.clone()));
    }
    let borrowed: Vec<(&str, &str)> = supplied
        .iter()
        .map(|(name, value)| (name.as_str(), value.as_str()))
        .collect();

    let identity = Credentials::new(
        creds.access_key_id.clone(),
        creds.secret_access_key.clone(),
        creds.session_token.clone(),
        None,
        CREDENTIAL_PROVIDER_NAME,
    )
    .into();
    let mut settings = SigningSettings::default();
    // Required for S3 and harmless for query/JSON services.
    settings.payload_checksum_kind = PayloadChecksumKind::XAmzSha256;
    let params: aws_sigv4::http_request::SigningParams = SigningParams::builder()
        .identity(&identity)
        .region(plan.region)
        .name(plan.service)
        .time(time)
        .settings(settings)
        .build()
        .map_err(|error| {
            log::warn!("sigv4 params build failed: {error}");
            ERR_SIGNING.to_string()
        })?
        .into();
    let signable = SignableRequest::new(
        plan.method.as_str(),
        uri,
        borrowed.into_iter(),
        SignableBody::Bytes(payload),
    )
    .map_err(|error| {
        log::warn!("sigv4 signable request build failed: {error}");
        ERR_SIGNING.to_string()
    })?;
    let output = sign(signable, &params).map_err(|error| {
        log::warn!("sigv4 signing failed: {error}");
        ERR_SIGNING.to_string()
    })?;
    let (instructions, _signature) = output.into_parts();
    Ok(instructions
        .headers()
        .map(|(name, value)| (name.to_string(), value.to_string()))
        .collect())
}

/// Sends one SigV4-signed request and returns status, capped body and the
/// response headers (used for `etag`/`x-amz-version-id` style metadata).
///
/// The response body is read incrementally and rejected once it exceeds
/// `plan.response_cap`; oversized payloads never reach parsers or callers.
pub(crate) async fn signed_request(
    mut plan: RequestPlan<'_>,
    creds: &AwsCreds,
) -> Result<AwsResponse, String> {
    let host = plan.host.clone();
    let payload = plan.body.take().unwrap_or_default();
    let header_set = signed_headers(&plan, &payload, creds, SystemTime::now())?;
    let url = format!("https://{}{}", plan.host, plan.path_and_query);
    let mut request = shared_client()?.request(plan.method.clone(), &url);
    for (name, value) in header_set {
        request = request.header(name, value);
    }
    request = request.body(payload);
    let response = request.send().await.map_err(|error| {
        log::warn!("AWS request to {host} failed: {error}");
        ERR_NETWORK.to_string()
    })?;
    let status = response.status().as_u16();
    let headers: Vec<(String, String)> = response
        .headers()
        .iter()
        .map(|(name, value)| {
            (
                name.to_string(),
                String::from_utf8_lossy(value.as_bytes()).into_owned(),
            )
        })
        .collect();
    let body_bytes = read_capped(response, plan.response_cap).await?;
    Ok(AwsResponse {
        status,
        headers,
        body: body_bytes,
    })
}

/// Status line, selected metadata headers and the size-capped response body.
pub(crate) struct AwsResponse {
    pub(crate) status: u16,
    pub(crate) headers: Vec<(String, String)>,
    pub(crate) body: bytes::Bytes,
}

impl AwsResponse {
    pub(crate) fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }

    /// Fails non-2xx responses. The provider error code is logged with the
    /// safe action label; only a static sentinel reaches the caller.
    pub(crate) fn require_success(&self, what: &str) -> Result<(), String> {
        if (200..300).contains(&self.status) {
            return Ok(());
        }
        let document = String::from_utf8_lossy(&self.body);
        let code = crate::providers::aws::xml::first_text_by_path(&document, &["Error", "Code"])
            .unwrap_or_default();
        log::warn!(
            "AWS {what} failed with status {}: provider code {code}",
            self.status
        );
        Err(format!(
            "provider_request_failed: {what} returned status {}",
            self.status
        ))
    }

    pub(crate) fn text(&self) -> std::borrow::Cow<'_, str> {
        String::from_utf8_lossy(&self.body)
    }
}

async fn read_capped(mut response: reqwest::Response, cap: usize) -> Result<bytes::Bytes, String> {
    let mut buffer: Vec<u8> = Vec::with_capacity(8192);
    loop {
        let chunk = response.chunk().await.map_err(|error| {
            log::warn!("AWS response read failed: {error}");
            ERR_NETWORK.to_string()
        })?;
        let Some(chunk) = chunk else {
            return Ok(bytes::Bytes::from(buffer));
        };
        if buffer.len().saturating_add(chunk.len()) > cap {
            log::warn!("AWS response exceeded the {cap} byte cap");
            return Err(format!(
                "{ERR_RESPONSE_CAP}: response exceeds {cap} byte cap"
            ));
        }
        buffer.extend_from_slice(&chunk);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::time::Duration;

    fn fixed_time() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_755_000_000)
    }

    fn sample_creds(with_token: bool) -> AwsCreds {
        AwsCreds {
            access_key_id: "AKIDEXAMPLE".to_string(),
            secret_access_key: "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY".to_string(),
            session_token: with_token.then(|| "theSessionToken".to_string()),
        }
    }

    #[test]
    fn parses_required_keys_and_rejects_missing_secret() {
        let ok = AwsCreds::parse(&json!({
            "access_key_id": " AKIA ",
            "secret_access_key": "shh",
            "session_token": "tok"
        }))
        .unwrap_or_else(|error| panic!("valid envelope rejected: {error}"));
        assert_eq!(ok.access_key_id, "AKIA");
        assert_eq!(ok.session_token.as_deref(), Some("tok"));

        assert!(AwsCreds::parse(&json!({ "access_key_id": "AKIA" })).is_err());
        assert!(AwsCreds::parse(&json!({
            "access_key_id": "AKIA",
            "secret_access_key": ""
        }))
        .is_err());
        assert!(AwsCreds::parse(&json!([1])).is_err());
    }

    fn sts_plan(region: &str) -> RequestPlan<'_> {
        RequestPlan {
            service: "sts",
            region,
            method: Method::POST,
            host: format!("sts.{region}.amazonaws.com"),
            path_and_query: "/".to_string(),
            body: Some(b"Action=GetCallerIdentity&Version=2011-06-15".to_vec()),
            content_type: Some("application/x-www-form-urlencoded"),
            extra_headers: Vec::new(),
            response_cap: MAX_RESPONSE_BYTES,
        }
    }

    #[test]
    fn signed_authorization_header_is_well_formed() {
        let plan = sts_plan("us-east-1");
        let payload = plan.body.clone().unwrap_or_default();
        let headers = signed_headers(&plan, &payload, &sample_creds(false), fixed_time())
            .unwrap_or_else(|error| panic!("signing failed: {error}"));

        let authorization = headers
            .iter()
            .find(|(name, _)| name == "authorization")
            .map(|(_, value)| value.clone())
            .unwrap_or_default();
        assert!(authorization.starts_with("AWS4-HMAC-SHA256 Credential=AKIDEXAMPLE/"));
        assert!(authorization.contains("/us-east-1/sts/aws4_request"));
        assert!(authorization
            .contains("SignedHeaders=content-type;host;x-amz-content-sha256;x-amz-date"));
        assert!(authorization.contains("Signature="));
        assert!(headers.iter().any(|(name, _)| name == "x-amz-date"));
        assert!(!headers
            .iter()
            .any(|(name, _)| name == "x-amz-security-token"));
    }

    #[test]
    fn session_token_is_signed_when_present() {
        let plan = sts_plan("eu-west-1");
        let payload = plan.body.clone().unwrap_or_default();
        let headers = signed_headers(&plan, &payload, &sample_creds(true), fixed_time())
            .unwrap_or_else(|error| panic!("signing failed: {error}"));

        let token = headers
            .iter()
            .find(|(name, _)| name == "x-amz-security-token")
            .map(|(_, value)| value.clone())
            .unwrap_or_default();
        assert_eq!(token, "theSessionToken");
        let authorization = headers
            .iter()
            .find(|(name, _)| name == "authorization")
            .map(|(_, value)| value.clone())
            .unwrap_or_default();
        assert!(authorization.contains(
            "SignedHeaders=content-type;host;x-amz-content-sha256;x-amz-date;x-amz-security-token"
        ));
    }

    #[test]
    fn extra_headers_join_the_signed_set() {
        let plan = RequestPlan {
            service: "execute-api",
            region: "us-east-1",
            method: Method::GET,
            host: "example.execute-api.us-east-1.amazonaws.com".to_string(),
            path_and_query: "/prod".to_string(),
            body: None,
            content_type: None,
            extra_headers: vec![("x-amz-target", "Something.List".to_string())],
            response_cap: MAX_RESPONSE_BYTES,
        };
        let headers = signed_headers(&plan, b"", &sample_creds(false), fixed_time())
            .unwrap_or_else(|error| panic!("signing failed: {error}"));

        // Supplied headers are not echoed back by the signing instructions -
        // only newly computed ones (authorization, x-amz-date, checksum) are.
        // The signed set inside the authorization header must still list the
        // supplied extra header; there is no content-type on a bodyless GET.
        let authorization = headers
            .iter()
            .find(|(name, _)| name == "authorization")
            .map(|(_, value)| value.clone())
            .unwrap_or_default();
        assert!(authorization
            .contains("SignedHeaders=host;x-amz-content-sha256;x-amz-date;x-amz-target"));
    }
}
