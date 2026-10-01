//! Data types shared by every registrar adapter.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A registration request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainSpec {
    /// Fully-qualified name, e.g. `acme.com`.
    pub domain: String,
    /// Term in years, 1..=10.
    pub years: u32,
    /// Nameservers to delegate to. Empty means "keep the registrar defaults".
    pub nameservers: Vec<String>,
    /// Renew automatically at expiry.
    pub auto_renew: bool,
    /// Whois privacy where the registrar offers it free.
    pub whois_privacy: bool,
}

impl DomainSpec {
    /// Builds a one-year registration with auto-renew and privacy enabled, which
    /// is what the catalogue SKUs sell.
    #[must_use]
    pub fn one_year(domain: impl Into<String>) -> Self {
        Self {
            domain: domain.into(),
            years: 1,
            nameservers: Vec::new(),
            auto_renew: true,
            whois_privacy: true,
        }
    }

    /// With delegation pointed at a specific nameserver set.
    #[must_use]
    pub fn with_nameservers(mut self, nameservers: &[&str]) -> Self {
        self.nameservers = nameservers.iter().map(|ns| (*ns).to_string()).collect();
        self
    }

    /// The registrable label, i.e. everything before the first dot.
    #[must_use]
    pub fn label(&self) -> &str {
        self.domain.split('.').next().unwrap_or(&self.domain)
    }

    /// The TLD without its leading dot, lowercased.
    #[must_use]
    pub fn tld(&self) -> String {
        match self.domain.split_once('.') {
            Some((_, rest)) => rest.to_lowercase(),
            None => String::new(),
        }
    }
}

/// Outcome of an availability probe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Availability {
    Available { premium: bool, price_cents: Option<i64> },
    Taken,
    /// The registrar answered, but does not sell this TLD at all.
    Unsupported,
    Reserved,
}

/// A registration or its current state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Registration {
    pub domain: String,
    /// Registrar-side identifier, when the registrar exposes one.
    pub registrar_id: Option<String>,
    pub expires_at: Option<String>,
    pub auto_renew: bool,
    /// What the customer was charged, in the registrar's currency units.
    pub charged: Option<f64>,
    pub currency: Option<String>,
}

/// One DNS record in a zone.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DnsRecord {
    /// Relative host label: `""` for the apex, `"www"` for `www.example.com`.
    pub host: String,
    pub record_type: RecordType,
    pub value: String,
    pub ttl: u32,
    /// Registrar-assigned id, needed to delete or update an existing record.
    pub record_id: Option<String>,
}

impl DnsRecord {
    /// A record with the platform's default TTL.
    #[must_use]
    pub fn new(host: impl Into<String>, record_type: RecordType, value: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            record_type,
            value: value.into(),
            ttl: DEFAULT_TTL,
            record_id: None,
        }
    }

    /// With an explicit TTL, which the A2P records need to be short-lived.
    #[must_use]
    pub fn with_ttl(mut self, ttl: u32) -> Self {
        self.ttl = ttl;
        self
    }
}

/// Platform default TTL in seconds — low enough that a DNS change converges
/// before a TLS issuance retry.
pub const DEFAULT_TTL: u32 = 300;

/// Record types the platform writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum RecordType {
    A,
    Aaaa,
    Cname,
    Txt,
    Caa,
    Mx,
    Ns,
    Srv,
}

impl RecordType {
    /// Wire representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::A => "A",
            Self::Aaaa => "AAAA",
            Self::Cname => "CNAME",
            Self::Txt => "TXT",
            Self::Caa => "CAA",
            Self::Mx => "MX",
            Self::Ns => "NS",
            Self::Srv => "SRV",
        }
    }

    /// Parses a wire value, returning `None` for an unsupported type.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_uppercase().as_str() {
            "A" => Some(Self::A),
            "AAAA" => Some(Self::Aaaa),
            "CNAME" => Some(Self::Cname),
            "TXT" => Some(Self::Txt),
            "CAA" => Some(Self::Caa),
            "MX" => Some(Self::Mx),
            "NS" => Some(Self::Ns),
            "SRV" => Some(Self::Srv),
            _ => None,
        }
    }
}

/// A published price for one TLD.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TldPrice {
    pub tld: String,
    /// Registration price for the first year.
    pub registration_cents: i64,
    /// Renewal price, which is what a customer actually pays long term.
    pub renewal_cents: i64,
    pub premium: bool,
}

/// Static description of an adapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistrarInfo {
    pub name: String,
    pub display_name: String,
    /// TLDs the adapter knows how to register.
    pub supported_tlds: Vec<String>,
    /// Whether the adapter can edit DNS records, or only delegate nameservers.
    pub manages_dns: bool,
    /// True when a write replaces the whole zone instead of one record.
    pub replaces_whole_zone: bool,
    /// True when the registrar signs DS records for DNSSEC.
    pub manages_dnssec: bool,
}

/// Everything an adapter needs to authenticate.
///
/// The field names are the vendor's own; keeping them flat means a stored
/// credential from Vault maps onto this struct without translation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Credential {
    /// Primary key: an API key, an access key ID, or an application key.
    pub api_key: String,
    /// Shared secret or token, where the vendor uses one.
    pub api_secret: String,
    /// A third secret for vendors that need one (Namecheap's client IP).
    pub extra: String,
}

impl Credential {
    #[must_use]
    pub fn new(api_key: impl Into<String>, api_secret: impl Into<String>) -> Self {
        Self { api_key: api_key.into(), api_secret: api_secret.into(), extra: String::new() }
    }

    #[must_use]
    pub fn with_extra(mut self, extra: impl Into<String>) -> Self {
        self.extra = extra.into();
        self
    }

    /// Fails when the adapter cannot possibly authenticate, so the failure is a
    /// clear credential error instead of an opaque 401 from the vendor.
    pub fn require(&self, service: &str) -> Result<(), RegistrarError> {
        if self.api_key.trim().is_empty() {
            return Err(RegistrarError::Credentials(format!(
                "{service}: no API key configured"
            )));
        }
        if self.api_secret.trim().is_empty() {
            return Err(RegistrarError::Credentials(format!(
                "{service}: no API secret configured"
            )));
        }
        Ok(())
    }
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum RegistrarError {
    /// The stored credential is unusable — missing, malformed, or revoked.
    #[error("{0} rejected the credential")]
    Credentials(String),
    #[error("{0} rate limited the request; retry after {1}s")]
    RateLimited(String, u64),
    #[error("{0} has no capacity or the TLD is unavailable: {1}")]
    Unavailable(String, String),
    #[error("{0} does not support this operation: {1}")]
    Unsupported(String, String),
    #[error("{0}: {1}")]
    NotFound(String, String),
    #[error("{0} rejected the request: {1}")]
    Rejected(String, String),
    #[error("{0} transport failure: {1}")]
    Transport(String, String),
    #[error("{0} returned an unreadable response: {1}")]
    Decode(String, String),
}

impl RegistrarError {
    /// True when retrying elsewhere is worthwhile.
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::RateLimited(..) | Self::Transport(..))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_spec_splits_into_label_and_tld() {
        let spec = DomainSpec::one_year("Acme.COM");
        assert_eq!(spec.label(), "Acme");
        assert_eq!(spec.tld(), "com");
    }

    #[test]
    fn a_bare_label_has_no_tld() {
        let spec = DomainSpec::one_year("localhost");
        assert_eq!(spec.label(), "localhost");
        assert!(spec.tld().is_empty());
    }

    #[test]
    fn nameservers_can_be_attached() {
        let spec = DomainSpec::one_year("acme.com").with_nameservers(&["ns1.example.net"]);
        assert_eq!(spec.nameservers, vec!["ns1.example.net".to_string()]);
    }

    #[test]
    fn record_types_round_trip_through_their_wire_form() {
        for record_type in [
            RecordType::A, RecordType::Aaaa, RecordType::Cname, RecordType::Txt,
            RecordType::Caa, RecordType::Mx, RecordType::Ns, RecordType::Srv,
        ] {
            assert_eq!(RecordType::parse(record_type.as_str()), Some(record_type));
        }
    }

    #[test]
    fn an_unsupported_record_type_parses_to_nothing() {
        assert_eq!(RecordType::parse("HTTPS"), None);
        assert_eq!(RecordType::parse(""), None);
    }

    #[test]
    fn record_types_parse_case_insensitively() {
        assert_eq!(RecordType::parse("txt"), Some(RecordType::Txt));
        assert_eq!(RecordType::parse(" Aaaa "), Some(RecordType::Aaaa));
    }

    #[test]
    fn a_new_record_carries_the_platform_default_ttl() {
        let record = DnsRecord::new("www", RecordType::Cname, "target.example.net");
        assert_eq!(record.ttl, DEFAULT_TTL);
        assert!(record.record_id.is_none());
        assert_eq!(record.with_ttl(60).ttl, 60);
    }

    #[test]
    fn an_empty_credential_is_refused_before_any_request() {
        assert!(matches!(
            Credential::default().require("Porkbun"),
            Err(RegistrarError::Credentials(_))
        ));
        assert!(matches!(
            Credential::new("key", "  ").require("Porkbun"),
            Err(RegistrarError::Credentials(_))
        ));
        assert!(Credential::new("key", "secret").require("Porkbun").is_ok());
    }

    #[test]
    fn only_transient_failures_are_worth_retrying() {
        assert!(RegistrarError::RateLimited("Porkbun".into(), 30).is_retryable());
        assert!(RegistrarError::Transport("Porkbun".into(), "timeout".into()).is_retryable());
        assert!(!RegistrarError::Rejected("Porkbun".into(), "bad tld".into()).is_retryable());
        assert!(!RegistrarError::Credentials("Porkbun".into()).is_retryable());
    }

    #[test]
    fn error_messages_do_not_echo_the_credential() {
        let err = Credential::new("key", "super-secret-value").require("Porkbun");
        assert!(err.is_ok());
        let missing = Credential::new("", "super-secret-value").require("Porkbun").unwrap_err();
        assert!(!missing.to_string().contains("super-secret-value"));
    }
}
