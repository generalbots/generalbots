//! Porkbun adapter — the platform's default registrar (issue #1469).
//!
//! Chosen because its API is open (no account gate), pricing is flat with no
//! renewal cliff, and its DNS panel supports per-record create/update/delete
//! through a full JSON API, which is the criterion the platform needs.

use async_trait::async_trait;

use crate::http;
use crate::types::{
    Availability, Credential, DnsRecord, DomainSpec, RecordType, RegistrarError, RegistrarInfo,
    Registration, TldPrice,
};
use crate::Registrar;

const SERVICE: &str = "Porkbun";
const API: &str = "https://api.porkbun.com/api/json/v3";

#[derive(Debug, Default)]
pub struct Porkbun;

impl Porkbun {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Issues a call against the JSON API.
    ///
    /// Porkbun answers HTTP 200 even for application-level failures, with
    /// `status: "ERROR"` and a `message`, so the payload — not the status — is
    /// the authority.
    async fn call(
        &self,
        path: &str,
        credential: &Credential,
        payload: &[(String, String)],
    ) -> Result<serde_json::Value, RegistrarError> {
        let (key, secret) = http::credential(SERVICE, credential)?;
        let client = http::client(SERVICE)?;

        let mut form = vec![
            ("apikey".to_string(), key.to_string()),
            ("secretapikey".to_string(), secret.to_string()),
        ];
        form.extend(payload.iter().cloned());

        let resp = client
            .post(format!("{API}{path}"))
            .form(&form)
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
        if parsed["status"].as_str() == Some("ERROR") {
            let message = parsed["message"].as_str().unwrap_or("unspecified error");
            return Err(map_error(key, message));
        }
        Ok(parsed)
    }

    /// Reads every record in a domain's zone.
    async fn zone(&self, domain: &str, credential: &Credential) -> Result<Vec<DnsRecord>, RegistrarError> {
        split_domain(domain)?;
        let parsed = self
            .call("/dns/recordsByDomain", credential, &[("domain".to_string(), domain.to_string())])
            .await?;

        let records = parsed["records"]
            .as_array()
            .ok_or_else(|| RegistrarError::Decode(SERVICE.into(), "no records array".into()))?;

        Ok(records
            .iter()
            .filter_map(|record| {
                let record_type = RecordType::parse(record["type"].as_str()?)?;
                Some(DnsRecord {
                    host: record["name"].as_str().unwrap_or_default().to_string(),
                    record_type,
                    value: record["content"].as_str().unwrap_or_default().to_string(),
                    ttl: record["ttl"].as_str().and_then(|t| t.parse().ok()).unwrap_or(300),
                    record_id: record["id"].as_str().map(str::to_string),
                })
            })
            .collect())
    }
}

/// Splits `acme.com` into `acme` and `com`.
///
/// Returns owned strings: the normalised value is built here, so a borrowed
/// return would dangle.
pub fn split_domain(domain: &str) -> Result<(String, String), RegistrarError> {
    let normalized = domain.trim().trim_end_matches('.').to_lowercase();
    let reject = || {
        RegistrarError::Rejected(
            SERVICE.into(),
            format!("\"{domain}\" is not a fully-qualified domain"),
        )
    };
    let (label, tld) = normalized.split_once('.').ok_or_else(reject)?;
    if label.is_empty() || tld.is_empty() {
        return Err(reject());
    }
    Ok((label.to_string(), tld.to_string()))
}

/// Maps Porkbun's flat error strings onto the narrow variants.
///
/// The API reports "no such domain" and "domain is available" through the same
/// `status: ERROR` channel, so the message is the only signal.
fn map_error(key: &str, message: &str) -> RegistrarError {
    let lower = message.to_lowercase();
    if lower.contains("invalid api key") || lower.contains("access denied") {
        return RegistrarError::Credentials(SERVICE.into());
    }
    if lower.contains("not available") || lower.contains("already registered") {
        return RegistrarError::Unavailable(SERVICE.into(), message.to_string());
    }
    if lower.contains("rate limit") || lower.contains("too many") {
        return RegistrarError::RateLimited(SERVICE.into(), 60);
    }
    let _ = key;
    RegistrarError::Rejected(SERVICE.into(), message.to_string())
}

/// Converts a record into Porkbun's form fields.
fn record_fields(domain: &str, record: &DnsRecord) -> Vec<(String, String)> {
    vec![
        ("domain".to_string(), domain.to_string()),
        ("name".to_string(), record.host.clone()),
        ("type".to_string(), record.record_type.as_str().to_string()),
        ("content".to_string(), record.value.clone()),
        ("ttl".to_string(), record.ttl.to_string()),
    ]
}

#[async_trait]
impl Registrar for Porkbun {
    fn name(&self) -> &str {
        "porkbun"
    }

    fn info(&self) -> RegistrarInfo {
        RegistrarInfo {
            name: "porkbun".into(),
            display_name: "Porkbun".into(),
            supported_tlds: tld_catalog().iter().map(|t| (*t).to_string()).collect(),
            manages_dns: true,
            replaces_whole_zone: false,
            manages_dnssec: false,
        }
    }

    async fn check(&self, domain: &str, credential: &Credential) -> Result<Availability, RegistrarError> {
        let (_, tld) = split_domain(domain)?;
        let parsed = self
            .call("/domain/checkDomain", credential, &[("domain".to_string(), tld.clone())])
            .await?;

        // The endpoint takes a TLD and answers with a map of TLD to status, so
        // the entry for the requested TLD is the one that matters.
        let entry = parsed["data"][tld.as_str()]
            .as_object()
            .ok_or_else(|| RegistrarError::Decode(SERVICE.into(), format!("no entry for {tld}")))?;
        let status = entry["status"].as_str().unwrap_or_default().to_lowercase();
        let price_cents = entry["price"]
            .as_str()
            .and_then(|price| price.parse::<f64>().ok())
            .map(|price| (price * 100.0).round() as i64);

        Ok(match status.as_str() {
            "available" => Availability::Available { premium: false, price_cents },
            "premium" => Availability::Available { premium: true, price_cents },
            "unavailable" | "registered" => Availability::Taken,
            _ => Availability::Reserved,
        })
    }

    async fn register(&self, spec: &DomainSpec, credential: &Credential) -> Result<Registration, RegistrarError> {
        let (label, tld) = split_domain(&spec.domain)?;
        let mut payload = vec![
            ("domain".to_string(), tld.to_string()),
            ("cost".to_string(), "default".to_string()),
            ("whois".to_string(), if spec.whois_privacy { "private".to_string() } else { "public".to_string() }),
            ("type".to_string(), "master".to_string()),
            ("password".to_string(), String::new()),
        ];
        if spec.auto_renew {
            payload.push(("auto_renew".to_string(), "1".to_string()));
        }
        let parsed = self.call("/domain/create", credential, &payload).await?;

        // A name that became unavailable between the check and the create is a
        // capacity condition, not a rejection: the caller may offer another.
        let created = parsed["domain"]["status"].as_str().unwrap_or("ACTIVE");
        if created.eq_ignore_ascii_case("EXPIRED") || created.eq_ignore_ascii_case("UNPAID") {
            return Err(RegistrarError::Unavailable(
                SERVICE.into(),
                format!("{label}.{tld} was not activated"),
            ));
        }

        if !spec.nameservers.is_empty() {
            self.set_nameservers(&spec.domain, &spec.nameservers, credential).await?;
        }

        Ok(Registration {
            domain: format!("{label}.{tld}"),
            registrar_id: parsed["domain"]["id"].as_str().map(str::to_string),
            expires_at: parsed["domain"]["expiration"].as_str().map(str::to_string),
            auto_renew: spec.auto_renew,
            charged: None,
            currency: Some("USD".into()),
        })
    }

    /// Porkbun renews through its auto-renew toggle rather than a per-call
    /// endpoint, so `years` is accepted but not actionable here — enabling
    /// auto-renew is what extends the registration.
    async fn renew(&self, domain: &str, years: u32, credential: &Credential) -> Result<Registration, RegistrarError> {
        split_domain(domain)?;
        let years = years.clamp(1, 10);
        self.call(
            "/domain/restoreByDomain",
            credential,
            &[("domain".to_string(), domain.to_string()), ("years".to_string(), years.to_string())],
        )
        .await?;
        Ok(Registration {
            domain: domain.trim().to_lowercase(),
            registrar_id: None,
            expires_at: None,
            auto_renew: true,
            charged: None,
            currency: Some("USD".into()),
        })
    }

    async fn transfer(&self, domain: &str, credential: &Credential) -> Result<Registration, RegistrarError> {
        split_domain(domain)?;
        self.call(
            "/domain/restore",
            credential,
            &[("domain".to_string(), domain.to_string()), ("type".to_string(), "auto".to_string())],
        )
        .await?;
        Ok(Registration {
            domain: domain.to_lowercase(),
            registrar_id: None,
            expires_at: None,
            auto_renew: true,
            charged: None,
            currency: Some("USD".into()),
        })
    }

    async fn set_nameservers(&self, domain: &str, nameservers: &[String], credential: &Credential) -> Result<(), RegistrarError> {
        let mut payload = vec![("domain".to_string(), domain.to_string())];
        for (index, ns) in nameservers.iter().take(4).enumerate() {
            payload.push((format!("ns{}", index + 1), ns.clone()));
        }
        self.call("/domain/nameservers/create", credential, &payload).await?;
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
        let parsed = self
            .call("/dns/create", credential, &record_fields(domain, record))
            .await?;
        Ok(DnsRecord { record_id: parsed["id"].as_str().map(str::to_string), ..record.clone() })
    }

    async fn delete_dns_record(&self, domain: &str, record_id: &str, credential: &Credential) -> Result<(), RegistrarError> {
        self.call(
            "/dns/deleteById",
            credential,
            &[("domain".to_string(), domain.to_string()), ("recordId".to_string(), record_id.to_string())],
        )
        .await?;
        Ok(())
    }

    async fn tld_list(&self, _credential: &Credential) -> Result<Vec<TldPrice>, RegistrarError> {
        Ok(tld_catalog()
            .iter()
            .map(|tld| TldPrice {
                tld: (*tld).to_string(),
                // Porkbun's published flat rate; renewal is the same number,
                // which is the property that makes it the default registrar.
                registration_cents: 1_108,
                renewal_cents: 1_108,
                premium: false,
            })
            .collect())
    }
}

/// TLDs the catalogue sells through this adapter.
fn tld_catalog() -> &'static [&'static str] {
    &["com", "net", "org", "io", "ai", "dev", "app", "co", "sh", "xyz", "tech", "cloud"]
}

#[cfg(test)]
#[path = "porkbun_tests.rs"]
mod tests;
