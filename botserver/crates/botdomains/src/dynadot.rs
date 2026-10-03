//! Dynadot adapter (issue #1469).
//!
//! Offered for breadth: 805 TLDs at flat renewal pricing, which is what a
//! customer needs when the TLD they want is one the other registrars do not
//! carry. Its API is JSON with per-record DNS writes, so no zone-replace hazard.

use async_trait::async_trait;
use serde_json::Value;

use crate::Registrar;
use crate::http;
use crate::types::{
    Availability, Credential, DnsRecord, DomainSpec, RecordType, RegistrarError, RegistrarInfo,
    Registration, TldPrice,
};

const SERVICE: &str = "Dynadot";
const API: &str = "https://api.dynadot.com/v2";

#[derive(Debug, Default)]
pub struct Dynadot;

impl Dynadot {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Issues a JSON request and unwraps the API's own status code.
    ///
    /// Dynadot answers HTTP 200 with `status: "error"` and an `statusDescription`,
    /// so the payload decides the outcome.
    async fn call(
        &self,
        method: reqwest::Method,
        path: &str,
        credential: &Credential,
        query: &[(&str, String)],
    ) -> Result<Value, RegistrarError> {
        let (key, secret) = http::credential(SERVICE, credential)?;
        let client = http::client(SERVICE)?;

        let resp = client
            .request(method, format!("{API}{path}"))
            .query(query)
            .header("X-API-Key", key)
            .header("X-API-Secret", secret)
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
        if parsed["status"].as_str().map(|s| s.eq_ignore_ascii_case("error")) == Some(true) {
            let description = parsed["statusDescription"].as_str().unwrap_or("unspecified error");
            return Err(map_error(description));
        }
        Ok(parsed["data"].clone())
    }
}

/// Maps Dynadot's free-text errors onto the narrow variants.
fn map_error(description: &str) -> RegistrarError {
    let lower = description.to_lowercase();
    if lower.contains("api key") || lower.contains("unauthorized") || lower.contains("not authorized") {
        return RegistrarError::Credentials(SERVICE.into());
    }
    if lower.contains("rate limit") || lower.contains("too many") {
        return RegistrarError::RateLimited(SERVICE.into(), 60);
    }
    if lower.contains("already registered") || lower.contains("not available") || lower.contains("taken") {
        return RegistrarError::Unavailable(SERVICE.into(), description.to_string());
    }
    RegistrarError::Rejected(SERVICE.into(), description.to_string())
}

/// Splits a fully-qualified domain, refusing an incomplete one.
fn split_domain(domain: &str) -> Result<(String, String), RegistrarError> {
    let normalized = domain.trim().trim_end_matches('.').to_lowercase();
    let reject = || {
        RegistrarError::Rejected(SERVICE.into(), format!("\"{domain}\" is not fully qualified"))
    };
    let (label, tld) = normalized.split_once('.').ok_or_else(reject)?;
    if label.is_empty() || tld.is_empty() {
        return Err(reject());
    }
    Ok((label.to_string(), tld.to_string()))
}

/// Renders a record as the `type=value` line Dynadot's DNS API expects.
fn record_line(record: &DnsRecord) -> String {
    format!("{}={}", record.record_type.as_str(), record.value)
}

#[async_trait]
impl Registrar for Dynadot {
    fn name(&self) -> &str {
        "dynadot"
    }

    fn info(&self) -> RegistrarInfo {
        RegistrarInfo {
            name: "dynadot".into(),
            display_name: "Dynadot".into(),
            supported_tlds: vec![
                "com".into(), "net".into(), "org".into(), "io".into(), "ai".into(),
                "dev".into(), "app".into(), "co".into(), "xyz".into(), "tech".into(),
                "cloud".into(), "sh".into(), "info".into(), "biz".into(),
            ],
            manages_dns: true,
            replaces_whole_zone: false,
            manages_dnssec: true,
        }
    }

    async fn check(&self, domain: &str, credential: &Credential) -> Result<Availability, RegistrarError> {
        split_domain(domain)?;
        let result = self
            .call(
                reqwest::Method::GET,
                "/domain/search",
                credential,
                &[("domain", domain.trim().to_lowercase())],
            )
            .await;
        match result {
            // Dynadot reports availability as a data record rather than a flag,
            // and the field is absent for a domain already registered.
            Ok(data) => {
                let available = data["available"].as_bool().unwrap_or(false)
                    || data.as_object().is_some_and(|o| !o.is_empty() && o.get("available").is_none());
                Ok(if available {
                    Availability::Available { premium: false, price_cents: None }
                } else {
                    Availability::Taken
                })
            }
            Err(RegistrarError::Unavailable(_, _)) => Ok(Availability::Taken),
            Err(err) => Err(err),
        }
    }

    async fn register(&self, spec: &DomainSpec, credential: &Credential) -> Result<Registration, RegistrarError> {
        let (label, tld) = split_domain(&spec.domain)?;
        let domain = format!("{label}.{tld}");
        let result = self
            .call(
                reqwest::Method::POST,
                "/domain/register",
                credential,
                &[
                    ("domain", domain.clone()),
                    ("years", spec.years.clamp(1, 10).to_string()),
                    ("safeTransfer", "true".to_string()),
                    ("whoisPrivacy", spec.whois_privacy.to_string()),
                    ("autoRenew", spec.auto_renew.to_string()),
                ],
            )
            .await?;

        if !spec.nameservers.is_empty() {
            self.set_nameservers(&domain, &spec.nameservers, credential).await?;
        }
        Ok(Registration {
            domain,
            registrar_id: result["domainId"].as_str().map(str::to_string),
            expires_at: None,
            auto_renew: spec.auto_renew,
            charged: result["amount"].as_f64(),
            currency: Some("USD".into()),
        })
    }

    async fn renew(&self, domain: &str, years: u32, credential: &Credential) -> Result<Registration, RegistrarError> {
        split_domain(domain)?;
        let domain = domain.trim().to_lowercase();
        self.call(
            reqwest::Method::POST,
            "/domain/renew",
            credential,
            &[("domain", domain.clone()), ("years", years.clamp(1, 10).to_string())],
        )
        .await?;
        Ok(Registration {
            domain,
            registrar_id: None,
            expires_at: None,
            auto_renew: true,
            charged: None,
            currency: Some("USD".into()),
        })
    }

    async fn transfer(&self, domain: &str, credential: &Credential) -> Result<Registration, RegistrarError> {
        split_domain(domain)?;
        let domain = domain.trim().to_lowercase();
        self.call(
            reqwest::Method::POST,
            "/domain/transfer",
            credential,
            &[("domain", domain.clone()), ("safeTransfer", "true".to_string())],
        )
        .await?;
        Ok(Registration {
            domain,
            registrar_id: None,
            expires_at: None,
            auto_renew: true,
            charged: None,
            currency: Some("USD".into()),
        })
    }

    async fn set_nameservers(&self, domain: &str, nameservers: &[String], credential: &Credential) -> Result<(), RegistrarError> {
        self.call(
            reqwest::Method::PUT,
            "/domain/nameserver",
            credential,
            &[
                ("domain", domain.trim().to_lowercase()),
                ("nameservers", nameservers.join(",")),
            ],
        )
        .await?;
        Ok(())
    }

    async fn get_dns_record(&self, domain: &str, host: &str, credential: &Credential) -> Result<Option<DnsRecord>, RegistrarError> {
        let wanted = host.trim().to_lowercase();
        Ok(self
            .zone(domain, credential)
            .await?
            .into_iter()
            .find(|record| record.host.trim().to_lowercase() == wanted))
    }

    async fn set_dns_record(&self, domain: &str, record: &DnsRecord, credential: &Credential) -> Result<DnsRecord, RegistrarError> {
        let created = self
            .call(
                reqwest::Method::POST,
                "/dns/record",
                credential,
                &[
                    ("domain", domain.trim().to_lowercase()),
                    ("host", record.host.clone()),
                    ("recordType", record.record_type.as_str().to_string()),
                    ("recordLine", record_line(record)),
                    ("ttl", record.ttl.to_string()),
                ],
            )
            .await?;
        Ok(DnsRecord {
            record_id: created["recordId"].as_str().map(str::to_string),
            ..record.clone()
        })
    }

    async fn delete_dns_record(&self, domain: &str, record_id: &str, credential: &Credential) -> Result<(), RegistrarError> {
        self.call(
            reqwest::Method::DELETE,
            "/dns/record",
            credential,
            &[
                ("domain", domain.trim().to_lowercase()),
                ("recordId", record_id.to_string()),
            ],
        )
        .await?;
        Ok(())
    }

    async fn tld_list(&self, _credential: &Credential) -> Result<Vec<TldPrice>, RegistrarError> {
        Ok(self
            .info()
            .supported_tlds
            .iter()
            .map(|tld| TldPrice {
                tld: tld.clone(),
                registration_cents: 1_088,
                // Flat renewal is the reason Dynadot is the breadth option.
                renewal_cents: 1_088,
                premium: false,
            })
            .collect())
    }
}

impl Dynadot {
    /// Reads the zone as `type=value` lines grouped by host.
    async fn zone(&self, domain: &str, credential: &Credential) -> Result<Vec<DnsRecord>, RegistrarError> {
        let data = self
            .call(
                reqwest::Method::GET,
                "/dns/record",
                credential,
                &[("domain", domain.trim().to_lowercase())],
            )
            .await?;
        let entries = data.as_array().cloned().unwrap_or_default();
        Ok(entries.iter().filter_map(parse_entry).collect())
    }
}

/// Parses one zone entry, which Dynadot returns as `host`, `recordType` and a
/// `recordLine` of `type=value`.
fn parse_entry(node: &Value) -> Option<DnsRecord> {
    let record_type = RecordType::parse(node["recordType"].as_str()?)?;
    let line = node["recordLine"].as_str().or_else(|| node["recordData"].as_str())?;
    let expected = format!("{}=", record_type.as_str());
    let value = line.strip_prefix(expected.as_str()).unwrap_or(line).to_string();
    Some(DnsRecord {
        host: node["host"].as_str().unwrap_or_default().to_string(),
        record_type,
        value,
        ttl: node["ttl"].as_u64().unwrap_or(crate::DEFAULT_TTL as u64) as u32,
        record_id: node["recordId"].as_str().map(str::to_string),
    })
}

#[cfg(test)]
#[path = "dynadot_tests.rs"]
mod tests;
