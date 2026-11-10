use super::*;

/// True when an IP literal must never be reached from a user-supplied URL.
///
/// #2066 — an IPv4-mapped IPv6 literal (`[::ffff:127.0.0.1]`) parsed as a
/// plain IPv6 address and passed every check, and the unique-local
/// (`fc00::/7`) / link-local (`fe80::/10`) ranges were only matched by two
/// hard-coded substrings. Every non-global range is refused here instead.
pub(crate) fn is_blocked_ip(addr: IpAddr) -> bool {
    match addr {
        IpAddr::V4(v4) => {
            v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_unspecified()
                || v4.is_broadcast()
                || v4.is_documentation()
        }
        IpAddr::V6(v6) => {
            if let Some(mapped) = v6.to_ipv4_mapped() {
                return is_blocked_ip(IpAddr::V4(mapped));
            }
            let first = v6.segments()[0];
            v6.is_loopback()
                || v6.is_unspecified()
                || (first & 0xfe00) == 0xfc00
                || (first & 0xffc0) == 0xfe80
        }
    }
}

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

    if let Ok(parsed) = url::Url::parse(url) {
        let host_str: &str = match parsed.host_str() {
            Some(h) => h,
            None => {
                return Err(ValidationError::InvalidUrl(url.to_string()));
            }
        };

        if let Ok(addr) = host_str.parse::<IpAddr>() {
            if is_blocked_ip(addr) {
                return Err(ValidationError::Forbidden {
                    field: "url".to_string(),
                    reason: format!("URL resolves to private/internal address: {}", addr),
                });
            }
        }
    }

    Ok(())
}
