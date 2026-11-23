//! Pure validation and authorization helpers for the DNS endpoints (#2681).
//!
//! Every untrusted form field accepted by `/api/dns/register` is funnelled
//! through [`validate_dns_record`] before it can reach SQL. The pure helpers in
//! this module are deliberately free of I/O so they can be unit-tested without
//! a running server; [`hostname_in_zone`] only does string work, while
//! [`caller_owns_hostname`] is the DB-backed authorization gate shared by the
//! register and remove handlers.

use crate::security::auth_api::types::AuthenticatedUser;
use std::net::{Ipv4Addr, Ipv6Addr};

use super::dns_scope::caller_owns_hostname_in;

/// Upper-cased allow-list of record types this endpoint is able to validate.
/// Anything outside this list is rejected rather than stored verbatim.
pub(crate) const ALLOWED_RECORD_TYPES: &[&str] =
    &["A", "AAAA", "CNAME", "MX", "NS", "PTR", "TXT"];

const MAX_NAME_LEN: usize = 253;
const MAX_LABEL_LEN: usize = 63;
const MAX_TARGET_LEN: usize = 255;
const MAX_TXT_LEN: usize = 255;
const MIN_TTL: i64 = 1;
const MAX_TTL: i64 = 604_800; // one week

/// Validated, normalized record ready to be persisted.
pub(crate) struct ValidatedDnsRecord {
    pub(crate) hostname: String,
    pub(crate) record_type: String,
    pub(crate) target: String,
    pub(crate) ttl: i32,
}

/// Validates one DNS name (used for both the record name and host-style
/// targets). Returns the normalized lower-case name without the optional
/// trailing dot, or a human-readable error.
pub(crate) fn validate_dns_name(input: &str, kind: &str) -> Result<String, String> {
    let trimmed = input.trim();
    let bare = trimmed.strip_suffix('.').unwrap_or(trimmed);
    if bare.is_empty() || bare.len() > MAX_NAME_LEN {
        return Err(format!("{kind} has an invalid length"));
    }

    let labels: Vec<&str> = bare.split('.').collect();
    if labels.len() < 2 {
        return Err(format!("{kind} must include a domain"));
    }
    for label in labels {
        if label.is_empty() || label.len() > MAX_LABEL_LEN {
            return Err(format!("{kind} contains an invalid label"));
        }
        let bytes = label.as_bytes();
        // RFC 1123 label: starts/ends alphanumeric, interior alphanumeric or
        // hyphen. This rejects traversal (`../`), injection (whitespace,
        // quotes, `;`, newlines, NUL), wildcards and IDN/unicode outright.
        if !bytes[0].is_ascii_alphanumeric()
            || !bytes[bytes.len() - 1].is_ascii_alphanumeric()
            || bytes
                .iter()
                .any(|b| !(b.is_ascii_alphanumeric() || *b == b'-'))
        {
            return Err(format!("{kind} contains invalid characters"));
        }
    }
    Ok(bare.to_ascii_lowercase())
}

/// Normalizes and allow-lists the record type.
pub(crate) fn validate_record_type(raw: Option<&str>) -> Result<String, String> {
    let normalized = raw.unwrap_or("A").trim().to_ascii_uppercase();
    if ALLOWED_RECORD_TYPES.contains(&normalized.as_str()) {
        Ok(normalized)
    } else {
        Err("Unsupported DNS record type".to_string())
    }
}

/// Explicit public-address check for IPv4 (avoids `IpAddr` method MSRV churn).
fn is_public_v4(ip: Ipv4Addr) -> bool {
    let o = ip.octets();
    if o[0] == 0 || o[0] == 10 || o[0] == 127 || o[0] >= 224 {
        return false; // unspecified, private, loopback, multicast/reserved
    }
    if o[0] == 169 && o[1] == 254 {
        return false; // link-local
    }
    if o[0] == 172 && (16..=31).contains(&o[1]) {
        return false; // private
    }
    if o[0] == 192 && o[1] == 168 {
        return false; // private
    }
    if o[0] == 100 && (64..=127).contains(&o[1]) {
        return false; // CGNAT shared space
    }
    if o[0] == 192 && o[1] == 0 && o[2] == 0 {
        return false; // IETF protocol assignments
    }
    if o[0] == 198 && (o[1] == 18 || o[1] == 19) {
        return false; // benchmarking
    }
    true
}

/// Explicit public-address check for IPv6.
fn is_public_v6(ip: Ipv6Addr) -> bool {
    if ip.is_unspecified() || ip.is_loopback() || ip.is_multicast() {
        return false;
    }
    if ip.to_ipv4_mapped().is_some() {
        return false; // mapped IPv4 must be validated as IPv4 instead
    }
    let s = ip.segments();
    if s[0] & 0xfe00 == 0xfc00 {
        return false; // unique-local fc00::/7
    }
    if s[0] & 0xffc0 == 0xfe80 {
        return false; // link-local fe80::/10
    }
    true
}

/// Validates a target according to its (already normalized) record type.
pub(crate) fn validate_target(record_type: &str, raw: &str) -> Result<String, String> {
    let target = raw.trim();
    if target.is_empty() || target.len() > MAX_TARGET_LEN {
        return Err("Target has an invalid length".to_string());
    }
    match record_type {
        "A" => {
            let ip: Ipv4Addr = target
                .parse()
                .map_err(|_| "A record target must be an IPv4 address".to_string())?;
            if !is_public_v4(ip) {
                return Err("A record target must be a public IPv4 address".to_string());
            }
            Ok(ip.to_string())
        }
        "AAAA" => {
            let ip: Ipv6Addr = target
                .parse()
                .map_err(|_| "AAAA record target must be an IPv6 address".to_string())?;
            if !is_public_v6(ip) {
                return Err("AAAA record target must be a public IPv6 address".to_string());
            }
            Ok(ip.to_string())
        }
        "CNAME" | "MX" | "NS" | "PTR" => validate_dns_name(target, "Target"),
        "TXT" => {
            if target.len() > MAX_TXT_LEN || target.chars().any(char::is_control) {
                return Err("TXT target contains invalid characters".to_string());
            }
            Ok(target.to_string())
        }
        _ => Err("Unsupported DNS record type".to_string()),
    }
}

/// Parses and range-checks the TTL, defaulting to 300 when absent.
pub(crate) fn validate_ttl(raw: Option<&str>) -> Result<i32, String> {
    let Some(value) = raw else {
        return Ok(300);
    };
    if value.trim().is_empty() {
        return Ok(300);
    }
    let parsed: i64 = value.trim().parse().map_err(|_| "Invalid TTL".to_string())?;
    if parsed < MIN_TTL || parsed > MAX_TTL {
        return Err("TTL out of range".to_string());
    }
    Ok(parsed as i32)
}

/// Validates all form fields in one shot.
pub(crate) fn validate_dns_record(
    hostname: Option<&str>,
    record_type: Option<&str>,
    target: Option<&str>,
    ttl: Option<&str>,
) -> Result<ValidatedDnsRecord, String> {
    let hostname = validate_dns_name(hostname.unwrap_or(""), "Hostname")?;
    let record_type = validate_record_type(record_type)?;
    let target = validate_target(&record_type, target.unwrap_or(""))?;
    let ttl = validate_ttl(ttl)?;
    Ok(ValidatedDnsRecord {
        hostname,
        record_type,
        target,
        ttl,
    })
}

/// True when `hostname` is the zone apex or a sub-domain of `zone`.
pub(crate) fn hostname_in_zone(hostname: &str, zone: &str) -> bool {
    let host = hostname.trim().trim_end_matches('.').to_ascii_lowercase();
    let zone = zone.trim().trim_end_matches('.').to_ascii_lowercase();
    if zone.is_empty() || host.is_empty() {
        return false;
    }
    host == zone || host.ends_with(&format!(".{zone}"))
}

/// Authorization gate: the caller must control a `bot_domains` zone that
/// covers `hostname`. Admins are trusted for every zone; everyone else must
/// match the zone's organization or hold a management role on its bot.
pub(crate) fn caller_owns_hostname(
    conn: &mut diesel::PgConnection,
    user: &AuthenticatedUser,
    hostname: &str,
) -> Result<bool, String> {
    caller_owns_hostname_in(conn, user, None, hostname)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_names() {
        for name in [
            "example.com",
            "WWW.Example.COM.",
            "my-host.example.com",
            "a.b.c.example.co.uk",
        ] {
            assert!(validate_dns_name(name, "Hostname").is_ok(), "{name}");
        }
        assert_eq!(
            validate_dns_name("WWW.Example.COM.", "Hostname").unwrap(),
            "www.example.com"
        );
    }

    #[test]
    fn rejects_malformed_and_injection_names() {
        for name in [
            "",
            "localhost",
            "../etc/passwd",
            "a..example.com",
            "-bad.example.com",
            "bad-.example.com",
            "exa_mple.com",
            "*.example.com",
            "foo bar.example.com",
            "foo\0bar.example.com",
            "evil.com\nX:1",
            "user@example.com",
            &format!("{}.com", "a".repeat(64)),
        ] {
            assert!(validate_dns_name(name, "Hostname").is_err(), "{name}");
        }
    }

    #[test]
    fn validates_record_types() {
        assert_eq!(validate_record_type(None).unwrap(), "A");
        assert_eq!(validate_record_type(Some(" aaaa ")).unwrap(), "AAAA");
        assert_eq!(validate_record_type(Some("Txt")).unwrap(), "TXT");
        for bad in ["", "ANY", "A; DROP TABLE", "AAAAA", "NS\r\nX"] {
            assert!(validate_record_type(Some(bad)).is_err(), "{bad}");
        }
    }

    #[test]
    fn validates_a_and_aaaa_targets() {
        assert!(validate_target("A", "8.8.8.8").is_ok());
        for private in ["10.0.0.1", "127.0.0.1", "192.168.1.1", "169.254.1.1"] {
            assert!(validate_target("A", private).is_err(), "{private}");
        }
        assert!(validate_target("A", "not-an-ip").is_err());
        assert!(validate_target("AAAA", "2606:4700:4700::1111").is_ok());
        for private in ["::1", "fc00::1", "fe80::1", "::ffff:10.0.0.1"] {
            assert!(validate_target("AAAA", private).is_err(), "{private}");
        }
    }

    #[test]
    fn validates_host_targets_and_txt() {
        assert!(validate_target("CNAME", "target.example.com").is_ok());
        assert!(validate_target("MX", "mail.example.com").is_ok());
        assert!(validate_target("CNAME", "target").is_err());
        assert!(validate_target("NS", "evil.com\nX").is_err());
        assert!(validate_target("TXT", "hello world").is_ok());
        assert!(validate_target("TXT", "line1\nline2").is_err());
    }

    #[test]
    fn validates_ttl() {
        assert_eq!(validate_ttl(None).unwrap(), 300);
        assert_eq!(validate_ttl(Some("600")).unwrap(), 600);
        for bad in ["0", "-1", "700000", "abc", "1.5"] {
            assert!(validate_ttl(Some(bad)).is_err(), "{bad}");
        }
    }

    #[test]
    fn zone_matching_respects_label_boundaries() {
        assert!(hostname_in_zone("www.example.com", "example.com"));
        assert!(hostname_in_zone("a.b.example.com", "example.com."));
        assert!(hostname_in_zone("example.com", "example.com"));
        assert!(!hostname_in_zone("notexample.com", "example.com"));
        assert!(!hostname_in_zone("example.com.evil.org", "example.com"));
    }

    #[test]
    fn full_record_rejects_injection() {
        assert!(validate_dns_record(
            Some("www.example.com"),
            Some("A"),
            Some("8.8.8.8"),
            Some("300")
        )
        .is_ok());
        assert!(validate_dns_record(
            Some("../etc/passwd"),
            Some("A"),
            Some("8.8.8.8"),
            None
        )
        .is_err());
        assert!(validate_dns_record(
            Some("www.example.com"),
            Some("A; DROP TABLE dns_records"),
            Some("8.8.8.8"),
            None
        )
        .is_err());
    }
}
