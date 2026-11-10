use super::*;

/// True when an IP literal must never be reached from a user-supplied URL.
///
/// #2066 — an IPv4-mapped IPv6 literal (`[::ffff:127.0.0.1]`) parsed as a
/// plain IPv6 address and passed every check, and the unique-local
/// (`fc00::/7`) / link-local (`fe80::/10`) ranges were only matched by two
/// hard-coded substrings. Every non-global range is refused here instead.
///
/// #2131 — multicast, CGNAT (`100.64.0.0/10`), the reserved `240.0.0.0/4`
/// block and the NAT64 prefix are refused as well; the fetch path layers
/// `ssrf_guard::is_blocked_address` on top for the ranges `std` does not model.
pub(crate) fn is_blocked_ip(addr: IpAddr) -> bool {
    match addr {
        IpAddr::V4(v4) => {
            v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_unspecified()
                || v4.is_broadcast()
                || v4.is_documentation()
                || v4.is_multicast()
        }
        IpAddr::V6(v6) => {
            if let Some(mapped) = v6.to_ipv4_mapped() {
                return is_blocked_ip(IpAddr::V4(mapped));
            }
            let first = v6.segments()[0];
            v6.is_loopback()
                || v6.is_unspecified()
                || v6.is_multicast()
                || (first & 0xfe00) == 0xfc00
                || (first & 0xffc0) == 0xfe80
        }
    }
}

/// Validates a URL for form/field input and, since #2131, applies the same
/// outbound-request policy the fetch path uses.
///
/// The reason string is intentionally generic: the host is not echoed back,
/// because this error can reach an API caller.
pub fn validate_url_ssrf(url: &str) -> Result<(), ValidationError> {
    validate_url(url)?;

    let url_lower = url.to_lowercase();

    for blocked in SSRF_BLOCKED_HOSTS.iter() {
        if url_lower.contains(blocked) {
            return Err(ValidationError::Forbidden {
                field: "url".to_string(),
                reason: format!("URL contains blocked host or pattern: {}", blocked),
            });
        }
    }

    match super::ssrf_guard::validate_url_ssrf_strict(url) {
        Ok(_) => Ok(()),
        Err(err) => Err(ValidationError::Forbidden {
            field: "url".to_string(),
            reason: format!("URL rejected by the outbound request policy ({})", err.kind().as_str()),
        }),
    }
}
