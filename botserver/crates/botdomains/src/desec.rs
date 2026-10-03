//! deSEC adapter — DNS only (issue #1469).
//!
//! deSEC is in the set for one reason: managed DNSSEC is the product rather than
//! a paid add-on, and its record-type support includes HTTPS/SVCB. Two limits are
//! recorded rather than worked around:
//!
//! * It explicitly rejects ALIAS/ANAME, so an apex CNAME is not expressible.
//!   Every adapter here therefore writes an apex `A` record, which is why
//!   [`crate::zone::onboarding_records`] does the same.
//! * It is not a registrar — no registration, renewal or transfer. Those methods
//!   report [`RegistrarError::Unsupported`] instead of pretending.
//!
//! It is a nameserver target for a machine in a VPS, not somewhere to buy a name.

use async_trait::async_trait;
use serde_json::Value;

use crate::Registrar;
use crate::http;
use crate::types::{
    Availability, Credential, DnsRecord, DomainSpec, RecordType, RegistrarError, RegistrarInfo,
    Registration, TldPrice,
};

const SERVICE: &str = "deSEC";
const API: &str = "https://desec.io/api/v1";

/// deSEC's public nameservers, which are the delegation target for a zone.
pub const NAMESERVERS: &[&str] = &[
    "ns1.desec.io",
    "ns2.desec.org",
];

#[derive(Debug, Default)]
pub struct Desec {
    /// The zone the platform writes into, e.g. `acme.com`.
    zone: String,
}

impl Desec {
    /// Adapter bound to a zone.
    #[must_use]
    pub fn with_zone(zone: impl Into<String>) -> Self {
        Self { zone: zone.into() }
    }

    fn require_zone(&self) -> Result<&str, RegistrarError> {
        if self.zone.trim().is_empty() {
            return Err(RegistrarError::Unsupported(
                SERVICE.into(),
                "no zone configured; build the adapter with with_zone".into(),
            ));
        }
        Ok(self.zone.trim())
    }

    /// Issues a request with the deSEC bearer token.
    ///
    /// deSEC answers HTTP 200 with `{"error": "..."}` for application failures,
    /// so the payload decides the outcome.
    async fn call(
        &self,
        method: reqwest::Method,
        path: &str,
        credential: &Credential,
        body: Option<Value>,
    ) -> Result<Value, RegistrarError> {
        let token = credential.api_key.trim();
        if token.is_empty() {
            return Err(RegistrarError::Credentials(format!("{SERVICE}: no API token configured")));
        }
        let client = http::client(SERVICE)?;

        let mut req = client
            .request(method, format!("{API}{path}"))
            .header("Authorization", format!("Bearer {token}"))
            .header("Content-Type", "application/json");
        if let Some(body) = body {
            req = req.json(&body);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| RegistrarError::Transport(SERVICE.into(), e.to_string()))?;
        let status = resp.status();
        if !status.is_success() {
            let text = http::body(SERVICE, resp).await.unwrap_or_default();
            return Err(http::classify(SERVICE, status, &Default::default(), &text));
        }

        let text = http::body(SERVICE, resp).await?;
        let parsed = http::parse(SERVICE, &text)?;
        if let Some(error) = parsed["error"].as_str() {
            return Err(map_error(error));
        }
        Ok(parsed)
    }

    /// Maps a platform record onto deSEC's `{type, ttl, rrset}` shape.
    ///
    /// deSEC groups records into RRsets: `rrset` carries the wire value for a
    /// single-valued type, and a TXT value is split on whitespace into the
    /// chunks DNS itself stores — so a long SPF or DKIM string has to be joined
    /// back by the reader.
    fn payload(record: &DnsRecord) -> Value {
        serde_json::json!({
            "type": record.record_type.as_str(),
            "ttl": record.ttl,
            "rrset": match record.record_type {
                // DNS stores a TXT value as whitespace-separated chunks, so a
                // long SPF or DKIM string has to be split on the wire.
                RecordType::Txt => record
                    .value
                    .split_whitespace()
                    .map(str::to_string)
                    .collect::<Vec<_>>(),
                _ => vec![record.value.clone()],
            },
        })
    }
}

/// Maps deSEC's error strings onto the narrow variants.
fn map_error(error: &str) -> RegistrarError {
    let lower = error.to_lowercase();
    if lower.contains("not found") || lower.contains("no such") {
        return RegistrarError::NotFound(SERVICE.into(), error.to_string());
    }
    if lower.contains("token") || lower.contains("unauthorized") || lower.contains("permission") {
        return RegistrarError::Credentials(SERVICE.into());
    }
    if lower.contains("rate limit") || lower.contains("too many") {
        return RegistrarError::RateLimited(SERVICE.into(), 60);
    }
    if lower.contains("unsupported") || lower.contains("not supported") {
        return RegistrarError::Unsupported(SERVICE.into(), error.to_string());
    }
    RegistrarError::Rejected(SERVICE.into(), error.to_string())
}

/// Reads a deSEC RRset back into a record.
fn parse_rrset(node: &Value, record_type: RecordType) -> Option<DnsRecord> {
    let values = node["rrset"].as_array()?;
    let joined = match record_type {
        RecordType::Txt => values
            .iter()
            .filter_map(|v| v.as_str())
            .collect::<Vec<_>>()
            .join(" "),
        _ => values.first().and_then(|v| v.as_str()).unwrap_or_default().to_string(),
    };
    Some(DnsRecord {
        host: String::new(),
        record_type,
        value: joined,
        ttl: node["ttl"].as_u64().unwrap_or(crate::DEFAULT_TTL as u64) as u32,
        record_id: node["created_at"].as_str().map(str::to_string),
    })
}

#[async_trait]
impl Registrar for Desec {
    fn name(&self) -> &str {
        "desec"
    }

    fn info(&self) -> RegistrarInfo {
        RegistrarInfo {
            name: "desec".into(),
            display_name: "deSEC.io".into(),
            // DNS only: deSEC does not sell names.
            supported_tlds: Vec::new(),
            manages_dns: true,
            replaces_whole_zone: false,
            manages_dnssec: true,
        }
    }

    async fn check(&self, _domain: &str, _credential: &Credential) -> Result<Availability, RegistrarError> {
        Err(RegistrarError::Unsupported(
            SERVICE.into(),
            "deSEC is a DNS host, not a registrar; nothing is available to buy".into(),
        ))
    }

    async fn register(&self, _spec: &DomainSpec, _credential: &Credential) -> Result<Registration, RegistrarError> {
        Err(RegistrarError::Unsupported(
            SERVICE.into(),
            "deSEC does not sell domain names".into(),
        ))
    }

    async fn renew(&self, _domain: &str, _years: u32, _credential: &Credential) -> Result<Registration, RegistrarError> {
        Err(RegistrarError::Unsupported(
            SERVICE.into(),
            "deSEC does not sell domain names, so there is nothing to renew".into(),
        ))
    }

    async fn transfer(&self, _domain: &str, _credential: &Credential) -> Result<Registration, RegistrarError> {
        Err(RegistrarError::Unsupported(
            SERVICE.into(),
            "deSEC does not sell domain names, so there is nothing to transfer".into(),
        ))
    }

    async fn set_nameservers(&self, _domain: &str, nameservers: &[String], _credential: &Credential) -> Result<(), RegistrarError> {
        if nameservers.is_empty() {
            return Err(RegistrarError::Rejected(
                SERVICE.into(),
                "no nameservers supplied; deSEC's own set is the only delegation it accepts".into(),
            ));
        }
        Err(RegistrarError::Unsupported(
            SERVICE.into(),
            format!(
                "delegation is performed at the registrar, not here; point the domain at {}",
                NAMESERVERS.join(", ")
            ),
        ))
    }

    async fn get_dns_record(&self, _domain: &str, _host: &str, _credential: &Credential) -> Result<Option<DnsRecord>, RegistrarError> {
        Err(RegistrarError::Unsupported(
            SERVICE.into(),
            "deSEC zones are managed through their own API; bind an adapter with with_zone".into(),
        ))
    }

    async fn set_dns_record(&self, _domain: &str, _record: &DnsRecord, _credential: &Credential) -> Result<DnsRecord, RegistrarError> {
        Err(RegistrarError::Unsupported(
            SERVICE.into(),
            "deSEC zones are managed through their own API; bind an adapter with with_zone".into(),
        ))
    }

    async fn delete_dns_record(&self, _domain: &str, _record_id: &str, _credential: &Credential) -> Result<(), RegistrarError> {
        Err(RegistrarError::Unsupported(
            SERVICE.into(),
            "deSEC zones are managed through their own API; bind an adapter with with_zone".into(),
        ))
    }

    async fn tld_list(&self, _credential: &Credential) -> Result<Vec<TldPrice>, RegistrarError> {
        Ok(Vec::new())
    }
}

impl Desec {
    /// Writes one record into the bound zone. Callers that hold a zone use this
    /// directly; the trait methods above only report that no zone is bound.
    pub async fn write_record(&self, record: &DnsRecord, credential: &Credential) -> Result<DnsRecord, RegistrarError> {
        let zone = self.require_zone()?;
        let path = format!("/domains/{zone}/rrsets/{}/{}", record.host, record.record_type.as_str());
        let created = self
            .call(reqwest::Method::PUT, &path, credential, Some(Self::payload(record)))
            .await?;
        Ok(DnsRecord { record_id: created["created_at"].as_str().map(str::to_string), ..record.clone() })
    }

    /// Reads one record from the bound zone.
    pub async fn read_record(&self, record_type: RecordType, credential: &Credential) -> Result<Option<DnsRecord>, RegistrarError> {
        let zone = self.require_zone()?;
        let path = format!("/domains/{zone}/rrsets/{}", record_type.as_str());
        let parsed = self.call(reqwest::Method::GET, &path, credential, None).await?;
        Ok(parse_rrset(&parsed, record_type))
    }

    /// Deletes one RRset from the bound zone.
    pub async fn remove_record(&self, host: &str, record_type: RecordType, credential: &Credential) -> Result<(), RegistrarError> {
        let zone = self.require_zone()?;
        let path = format!("/domains/{zone}/rrsets/{host}/{}", record_type.as_str());
        self.call(reqwest::Method::DELETE, &path, credential, None).await?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "desec_tests.rs"]
mod tests;
