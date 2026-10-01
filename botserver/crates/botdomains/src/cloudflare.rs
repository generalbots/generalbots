//! Cloudflare Registrar + DNS adapter (issue #1469).
//!
//! Worth offering where Cloudflare DNS is already authoritative: at-cost pricing
//! with no markup, and one dashboard for domain, DNS, CDN and R2. The at-cost
//! model only covers the ~390 TLDs Cloudflare supports and no ccTLDs, so
//! [`RegistrarInfo::supported_tlds`] carries only what the account can actually
//! buy.
//!
//! Cloudflare splits the two responsibilities across two APIs on the same
//! credential: zone records come from the DNS API (`/zones/{id}/dns_records`)
//! and registration from the Registrar API (`/accounts/{id}/registrar/...`).
//! Zone lookup is by name, so the zone id is resolved before any record write.

use async_trait::async_trait;
use serde_json::Value;

use crate::http;
use crate::types::{
    Availability, Credential, DnsRecord, DomainSpec, RecordType, RegistrarError, RegistrarInfo,
    Registration, TldPrice,
};
use crate::Registrar;

const SERVICE: &str = "Cloudflare";
const API: &str = "https://api.cloudflare.com/client/v4";

#[derive(Debug, Default)]
pub struct Cloudflare {
    /// The account the domain is registered into.
    account_id: String,
}

impl Cloudflare {
    /// Adapter bound to a Cloudflare account.
    #[must_use]
    pub fn with_account(account_id: impl Into<String>) -> Self {
        Self { account_id: account_id.into() }
    }


    fn require_account(&self) -> Result<&str, RegistrarError> {
        if self.account_id.trim().is_empty() {
            return Err(RegistrarError::Credentials(format!(
                "{SERVICE}: no account id configured; build the adapter with with_account"
            )));
        }
        Ok(self.account_id.trim())
    }

    /// Issues an authenticated request and unwraps Cloudflare's envelope.
    ///
    /// Cloudflare answers HTTP 200 with `success: false` and an `errors` array,
    /// so the envelope decides the outcome, not the status code.
    async fn call(
        &self,
        method: reqwest::Method,
        path: &str,
        credential: &Credential,
        body: Option<Value>,
    ) -> Result<Value, RegistrarError> {
        let (token, _) = http::credential(SERVICE, credential)?;
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
        let text = http::body(SERVICE, resp).await?;
        if !status.is_success() {
            return Err(http::classify(SERVICE, status, &Default::default(), &text));
        }

        let parsed = http::parse(SERVICE, &text)?;
        if parsed["success"].as_bool() == Some(false) {
            return Err(envelope_error(&parsed));
        }
        Ok(parsed["result"].clone())
    }

    /// Resolves the zone id for a domain, which every DNS call needs.
    async fn zone_id(&self, domain: &str, credential: &Credential) -> Result<String, RegistrarError> {
        let result = self
            .call(
                reqwest::Method::GET,
                &format!("/zones?name={}", encode(domain)),
                credential,
                None,
            )
            .await?;
        result[0]["id"]
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| RegistrarError::NotFound(SERVICE.into(), format!("no zone named {domain}")))
    }
}

/// Renders Cloudflare's `success: false` envelope as a narrow error.
fn envelope_error(parsed: &Value) -> RegistrarError {
    let first = parsed["errors"].get(0);
    let code = first.and_then(|e| e["code"].as_u64()).unwrap_or(0);
    let message = first
        .and_then(|e| e["message"].as_str())
        .unwrap_or("unspecified");
    match code {
        1001 | 10000 | 9103 | 9109 => RegistrarError::Credentials(SERVICE.into()),
        1002 => RegistrarError::RateLimited(SERVICE.into(), 60),
        8103 | 81057 => RegistrarError::NotFound(SERVICE.into(), message.to_string()),
        _ => RegistrarError::Rejected(SERVICE.into(), format!("{code}: {message}")),
    }
}

/// Percent-encodes a hostname for a query string.
fn encode(value: &str) -> String {
    value
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' => {
                (byte as char).to_string()
            }
            other => format!("%{other:02X}"),
        })
        .collect()
}

/// Converts a Cloudflare DNS record into the platform's shape.
fn record_from(node: &Value) -> Option<DnsRecord> {
    let record_type = RecordType::parse(node["type"].as_str()?)?;
    Some(DnsRecord {
        host: node["name"].as_str().unwrap_or_default().to_string(),
        record_type,
        value: node["content"].as_str().unwrap_or_default().to_string(),
        ttl: node["ttl"].as_u64().unwrap_or(crate::DEFAULT_TTL as u64) as u32,
        record_id: node["id"].as_str().map(str::to_string),
    })
}

/// Converts a platform record into Cloudflare's payload.
///
/// Cloudflare wants the fully-qualified name and `proxied: false` for records it
/// must serve verbatim — a DMARC or SPF TXT record behind the proxy would never
/// be read by a mail server.
fn record_payload(record: &DnsRecord) -> Value {
    let host = if record.host.is_empty() {
        "@".to_string()
    } else {
        record.host.clone()
    };
    serde_json::json!({
        "type": record.record_type.as_str(),
        "name": host,
        "content": record.value,
        "ttl": record.ttl,
        "proxied": false,
    })
}

#[async_trait]
impl Registrar for Cloudflare {
    fn name(&self) -> &str {
        "cloudflare"
    }

    fn info(&self) -> RegistrarInfo {
        RegistrarInfo {
            name: "cloudflare".into(),
            display_name: "Cloudflare Registrar".into(),
            // Only TLDs Cloudflare resells at cost; no ccTLDs.
            supported_tlds: vec![
                "com".into(), "net".into(), "org".into(), "dev".into(), "app".into(),
                "io".into(), "ai".into(), "co".into(), "xyz".into(), "tech".into(),
                "cloud".into(), "sh".into(),
            ],
            manages_dns: true,
            replaces_whole_zone: false,
            manages_dnssec: true,
        }
    }

    async fn check(&self, domain: &str, credential: &Credential) -> Result<Availability, RegistrarError> {
        let account = self.require_account()?;
        let result = self
            .call(
                reqwest::Method::GET,
                &format!("/accounts/{account}/registrar/domains/{}", encode(domain)),
                credential,
                None,
            )
            .await;
        match result {
            // A domain already in the account is not available to buy.
            Ok(_) => Ok(Availability::Taken),
            Err(RegistrarError::NotFound(_, _)) => Ok(Availability::Available { premium: false, price_cents: None }),
            Err(RegistrarError::Rejected(_, message)) if message.contains("available") => {
                Ok(Availability::Available { premium: false, price_cents: None })
            }
            Err(RegistrarError::Unavailable(_, _)) => Ok(Availability::Taken),
            Err(err) => Err(err),
        }
    }

    async fn register(&self, spec: &DomainSpec, credential: &Credential) -> Result<Registration, RegistrarError> {
        let account = self.require_account()?;
        let domain = spec.domain.trim().to_lowercase();
        let result = self
            .call(
                reqwest::Method::POST,
                &format!("/accounts/{account}/registrar/domains"),
                credential,
                Some(serde_json::json!({
                    "name": domain,
                    "auto_renew": spec.auto_renew,
                    // Cloudflare has no paid whois privacy: privacy is included.
                    "private_whois": spec.whois_privacy,
                })),
            )
            .await?;

        if !spec.nameservers.is_empty() {
            self.set_nameservers(&domain, &spec.nameservers, credential).await?;
        }

        Ok(Registration {
            domain,
            registrar_id: result["id"].as_str().map(str::to_string),
            expires_at: None,
            auto_renew: spec.auto_renew,
            charged: result["price"].as_f64(),
            currency: Some("USD".into()),
        })
    }

    async fn renew(&self, domain: &str, _years: u32, credential: &Credential) -> Result<Registration, RegistrarError> {
        let account = self.require_account()?;
        let domain = domain.trim().to_lowercase();
        let result = self
            .call(
                reqwest::Method::PUT,
                &format!("/accounts/{account}/registrar/domains/{}/renew", encode(&domain)),
                credential,
                Some(serde_json::json!({ "auto_renew": true })),
            )
            .await?;
        Ok(Registration {
            domain,
            registrar_id: None,
            expires_at: None,
            auto_renew: true,
            charged: result["price"].as_f64(),
            currency: Some("USD".into()),
        })
    }

    async fn transfer(&self, domain: &str, credential: &Credential) -> Result<Registration, RegistrarError> {
        let account = self.require_account()?;
        let domain = domain.trim().to_lowercase();
        self.call(
            reqwest::Method::POST,
            &format!("/accounts/{account}/registrar/domains/{}/transfer", encode(&domain)),
            credential,
            Some(serde_json::json!({ "priority": "high" })),
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
        let zone = self.zone_id(domain, credential).await?;
        self.call(
            reqwest::Method::PUT,
            &format!("/zones/{zone}/dns_settings/nameservers"),
            credential,
            Some(serde_json::json!({
                "ns_nameservers": nameservers.iter().map(|ns| serde_json::json!({ "ns": ns })).collect::<Vec<_>>(),
            })),
        )
        .await?;
        Ok(())
    }

    async fn get_dns_record(&self, domain: &str, host: &str, credential: &Credential) -> Result<Option<DnsRecord>, RegistrarError> {
        let zone = self.zone_id(domain, credential).await?;
        let wanted = host.trim().to_lowercase();
        let result = self
            .call(
                reqwest::Method::GET,
                &format!("/zones/{zone}/dns_records?per_page=100"),
                credential,
                None,
            )
            .await?;
        Ok(result
            .as_array()
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(record_from)
            .find(|record| {
                let name = record.host.trim_start_matches('.').to_lowercase();
                name == wanted || wanted.is_empty()
            }))
    }

    async fn set_dns_record(&self, domain: &str, record: &DnsRecord, credential: &Credential) -> Result<DnsRecord, RegistrarError> {
        let zone = self.zone_id(domain, credential).await?;
        let fqdn = if record.host.is_empty() {
            domain.trim().to_lowercase()
        } else {
            format!("{}.{domain}", record.host.trim().to_lowercase())
        };

        // Update when the (name, type) pair already exists, so a re-run of the
        // provisioning task is idempotent instead of accumulating duplicates.
        let existing = self
            .call(
                reqwest::Method::GET,
                &format!("/zones/{zone}/dns_records?type={}&name={}", record.record_type.as_str(), encode(&fqdn)),
                credential,
                None,
            )
            .await?;
        let previous = existing.as_array().and_then(|entries| entries.first());

        let (method, path) = match previous.and_then(|entry| entry["id"].as_str()) {
            Some(id) => (reqwest::Method::PUT, format!("/zones/{zone}/dns_records/{id}")),
            None => (reqwest::Method::POST, format!("/zones/{zone}/dns_records")),
        };

        let mut payload = record_payload(record);
        payload["name"] = Value::String(fqdn);
        let created = self.call(method, &path, credential, Some(payload)).await?;

        Ok(DnsRecord {
            record_id: created["id"].as_str().map(str::to_string),
            ..record.clone()
        })
    }

    async fn delete_dns_record(&self, domain: &str, record_id: &str, credential: &Credential) -> Result<(), RegistrarError> {
        let zone = self.zone_id(domain, credential).await?;
        self.call(
            reqwest::Method::DELETE,
            &format!("/zones/{zone}/dns_records/{record_id}"),
            credential,
            None,
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
                // At cost, and the renewal price is the same number.
                registration_cents: 1_046,
                renewal_cents: 1_046,
                premium: false,
            })
            .collect())
    }
}

#[cfg(test)]
#[path = "cloudflare_tests.rs"]
mod tests;
