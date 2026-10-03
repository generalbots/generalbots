//! Namecheap adapter (issue #1469).
//!
//! Kept selectable for customers who already register there, with two real
//! constraints recorded rather than smoothed over:
//!
//! 1. **API access is gated.** Production calls need 20 domains in the account,
//!    $50 in balance, or $50 spent in the last two years, so this adapter cannot
//!    be exercised against production without that.
//! 2. **`domains.dns.setHosts` replaces the entire zone.** There is no
//!    per-record create, update or delete, so every single-record change is
//!    read-all → modify-in-memory → write-all, and a record the client fails to
//!    re-send is silently deleted from the customer's zone.
//!
//! The second constraint is the dangerous one, so [`Registrar::set_dns_record`]
//! reads the current zone, computes an explicit [`crate::zone::ZoneDiff`] and
//! refuses the write if it would drop anything.

use crate::http;
use crate::types::{Credential, DnsRecord, RecordType, RegistrarError};
use crate::xml;
use crate::zone;

const SERVICE: &str = "Namecheap";
const API: &str = "https://api.namecheap.com/xml.response";
const SANDBOX: &str = "https://api.sandbox.namecheap.com/xml.response";

#[derive(Debug, Default)]
pub struct Namecheap {
    /// Namecheap pins API access to a registered client IP, which the operator
    /// supplies through the constructor.
    client_ip: String,
    production: bool,
}

impl Namecheap {
    /// Adapter against the sandbox, which needs no account gate.
    #[must_use]
    pub fn sandbox() -> Self {
        Self { client_ip: String::new(), production: false }
    }

    /// Adapter against production. `client_ip` must be the account's registered
    /// egress address or every call is rejected.
    #[must_use]
    pub fn production(client_ip: impl Into<String>) -> Self {
        Self { client_ip: client_ip.into(), production: true }
    }

    fn base(&self) -> &'static str {
        if self.production { API } else { SANDBOX }
    }

    /// Issues a GET call and parses the XML response.
    ///
    /// Errors arrive with HTTP 200, so the parsed `<Errors>` element — not the
    /// status code — decides the outcome.
    async fn call(
        &self,
        command: &str,
        credential: &Credential,
        extra: &[(&str, String)],
    ) -> Result<xml::Response, RegistrarError> {
        let (key, secret) = http::credential(SERVICE, credential)?;
        if self.client_ip.trim().is_empty() {
            return Err(RegistrarError::Credentials(format!(
                "{SERVICE}: no client IP configured; the API pins calls to a registered address"
            )));
        }
        let client = http::client(SERVICE)?;

        let mut query = vec![
            ("ApiUser".to_string(), key.to_string()),
            ("ApiKey".to_string(), secret.to_string()),
            ("UserName".to_string(), key.to_string()),
            ("Command".to_string(), command.to_string()),
            ("ClientIp".to_string(), self.client_ip.clone()),
        ];
        query.extend(extra.iter().map(|(k, v)| ((*k).to_string(), v.clone())));

        let resp = client
            .get(self.base())
            .query(&query)
            .send()
            .await
            .map_err(|e| RegistrarError::Transport(SERVICE.into(), e.to_string()))?;
        let status = resp.status();
        let text = http::body(SERVICE, resp).await?;
        if !status.is_success() {
            return Err(http::classify(SERVICE, status, &Default::default(), &text));
        }

        let parsed = xml::parse(SERVICE, &text)?;
        if let Some(errors) = xml::error_summary(&parsed) {
            return Err(map_error(&errors));
        }
        Ok(parsed)
    }

    /// Reads the current zone, which every write needs.
    async fn zone(&self, domain: &str, credential: &Credential) -> Result<Vec<DnsRecord>, RegistrarError> {
        let parsed = self
            .call(
                "namecheap.domains.dns.getList",
                credential,
                &[("domain", domain.to_lowercase())],
            )
            .await?;
        Ok(records_from(&parsed))
    }

    /// Replaces the whole zone, refusing any diff that deletes records.
    async fn replace_zone(
        &self,
        domain: &str,
        desired: &[DnsRecord],
        credential: &Credential,
    ) -> Result<(), RegistrarError> {
        let current = self.zone(domain, credential).await?;
        let diff = zone::diff(&current, desired);
        diff.ensure_no_silent_deletions(SERVICE)
            .map_err(|message| RegistrarError::Rejected(SERVICE.into(), message))?;

        let mut extra: Vec<(&str, String)> = vec![("domain", domain.to_lowercase())];
        for record in desired {
            extra.push(("HostName", record.host.clone()));
            extra.push(("RecordType", record.record_type.as_str().to_string()));
            extra.push(("Address", record.value.clone()));
            extra.push(("TTL", record.ttl.to_string()));
        }
        self.call("namecheap.domains.dns.setHosts", credential, &extra).await?;
        Ok(())
    }
}

/// Maps a Namecheap error number onto the narrow variants.
///
/// The numbers are stable and documented, which is why this table exists rather
/// than matching on English message text.
fn map_error(errors: &str) -> RegistrarError {
    let lower = errors.to_lowercase();
    if lower.contains("20110") || lower.contains("invalid api key") || lower.contains("access denied") {
        return RegistrarError::Credentials(SERVICE.into());
    }
    if lower.contains("20118") || lower.contains("too many") {
        return RegistrarError::RateLimited(SERVICE.into(), 60);
    }
    if lower.contains("2017") || lower.contains("already registered") || lower.contains("not available") {
        return RegistrarError::Unavailable(SERVICE.into(), errors.to_string());
    }
    if lower.contains("2019") || lower.contains("not supported") {
        return RegistrarError::Unsupported(SERVICE.into(), errors.to_string());
    }
    RegistrarError::Rejected(SERVICE.into(), errors.to_string())
}

/// Converts a parsed zone listing into records.
///
/// `<hostattr>` blocks are skipped: they describe the target of an MX or SRV
/// record whose value already lives in the sibling `<host>` block, so reading
/// both would duplicate every mail record.
fn records_from(parsed: &xml::Response) -> Vec<DnsRecord> {
    xml::zone_blocks(parsed)
        .into_iter()
        .filter(|(tag, _)| tag == "host")
        .filter_map(|(_, block)| {
            let record_type = RecordType::parse(&xml::leaf_in(&block, "Type")?)?;
            Some(DnsRecord {
                host: xml::leaf_in(&block, "Host").unwrap_or_default(),
                record_type,
                value: xml::leaf_in(&block, "Address").unwrap_or_default(),
                ttl: xml::leaf_in(&block, "TTL")
                    .and_then(|ttl| ttl.parse().ok())
                    .unwrap_or(1800),
                record_id: None,
            })
        })
        .collect()
}

/// Splits a fully-qualified domain, refusing an incomplete one.
///
/// Returns owned strings: the normalised value is built here, so a borrowed
/// return would dangle.
pub(super) fn split_domain(domain: &str) -> Result<(String, String), RegistrarError> {
    let normalized = domain.trim().trim_end_matches('.').to_lowercase();
    let reject = || {
        RegistrarError::Rejected(
            SERVICE.into(),
            format!("\"{domain}\" is not fully qualified"),
        )
    };
    let (label, tld) = normalized.split_once('.').ok_or_else(reject)?;
    if label.is_empty() || tld.is_empty() {
        return Err(reject());
    }
    Ok((label.to_string(), tld.to_string()))
}

/// Positional query parameter name for a nameserver index.
pub(super) fn nameserver_param(index: usize) -> &'static str {
    match index {
        0 => "ns1",
        1 => "ns2",
        2 => "ns3",
        _ => "ns4",
    }
}

mod api;

#[cfg(test)]
#[path = "namecheap_tests.rs"]
mod tests;
