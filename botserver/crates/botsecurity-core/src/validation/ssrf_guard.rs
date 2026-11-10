//! Shared SSRF guard for server-side fetches of caller-supplied URLs (#2131).
//!
//! Every outbound request whose target came from a bot script, an LLM tool
//! call or an API request must go through [`fetch_user_url`] (see
//! `ssrf_fetch.rs`). This module holds the policy primitives it is built from:
//!
//! * [`validate_url_ssrf_strict`] — synchronous, no DNS: scheme, credentials,
//!   host shape, obfuscated IPv4 encodings, control-plane ports;
//! * [`resolve_checked`] — resolves the host and refuses the request when
//!   *any* resolved address is non-global (blocks DNS rebinding by a
//!   resolver that returns a mixed public/private answer set);
//! * [`validate_redirect_target`] — the per-hop re-validation used by the
//!   fetch loop, so a 302 to `127.0.0.1` is refused before it is followed.
//!
//! Errors are deliberately generic: [`SsrfError`]'s `Display` never contains
//! the rejected host or address, because it is returned to the caller.
//! [`SsrfError::detail`] is for server logs only.

use std::fmt;
use std::net::{IpAddr, Ipv4Addr};
use url::Url;

use super::validate_url_ssrf::is_blocked_ip;

/// Ports that only expose control-plane or datastore services. A public web
/// request has no business reaching them, and on an internal host they are
/// exactly what SSRF is used to hit. The list is intentionally short and
/// documented; everything else (including 80/443/8080/8443) stays reachable.
pub const BLOCKED_PORTS: &[u16] = &[
    22,     // ssh
    2375,   // docker daemon (plain)
    2376,   // docker daemon (tls)
    2377,   // docker swarm
    4243,   // docker legacy
    5432,   // postgres
    6379,   // redis / valkey
    6443,   // kubernetes api
    8200,   // vault
    8500,   // consul
    9000,   // minio api (also php-fpm)
    9001,   // minio console
    9100,   // node exporter
    10250,  // kubelet
    10255,  // kubelet (read-only)
    11211,  // memcached
    27017,  // mongodb
];

/// Hostnames that resolve to the local node or to a cloud metadata service.
/// Kept as suffixes where a wildcard is meaningful (`*.localhost`, `*.local`).
const BLOCKED_HOST_SUFFIXES: &[&str] = &[
    ".localhost",
    ".local",
    ".internal",
    ".localdomain",
    ".home.arpa",
];

/// Exact hostnames that are internal by convention.
const BLOCKED_HOSTS: &[&str] = &[
    "localhost",
    "metadata",
    "metadata.google.internal",
    "instance-data",
    "linklocal.amazonaws.com",
    "kubernetes.default.svc",
    "kubernetes.default",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SsrfErrorKind {
    /// `file://`, `gopher://`, `ftp://`, `javascript:` …
    UnsupportedScheme,
    /// `http://user:pass@host/`
    CredentialsInUrl,
    MissingHost,
    /// Hostname or host suffix known to be local/metadata.
    BlockedHost,
    /// Literal or resolved address in a non-global range.
    BlockedAddress,
    /// Port in [`BLOCKED_PORTS`].
    BlockedPort,
    /// Decimal / octal / hex / short-form IPv4 (`2130706433`, `0x7f000001`).
    ObfuscatedAddress,
    ResolutionFailed,
    /// Resolver returned no address at all.
    NoAddresses,
    /// The connected peer is not the address the policy approved.
    DnsRebinding,
    RedirectLimit,
    /// `Location:` header that is not an absolute or same-origin http(s) URL.
    InvalidRedirect,
    /// Transport error (connect, TLS, body read).
    Transport,
    /// Body larger than the policy cap.
    TooLarge,
}

impl SsrfErrorKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::UnsupportedScheme => "unsupported-scheme",
            Self::CredentialsInUrl => "credentials-in-url",
            Self::MissingHost => "missing-host",
            Self::BlockedHost => "blocked-host",
            Self::BlockedAddress => "blocked-address",
            Self::BlockedPort => "blocked-port",
            Self::ObfuscatedAddress => "obfuscated-address",
            Self::ResolutionFailed => "resolution-failed",
            Self::NoAddresses => "no-addresses",
            Self::DnsRebinding => "dns-rebinding",
            Self::RedirectLimit => "redirect-limit",
            Self::InvalidRedirect => "invalid-redirect",
            Self::Transport => "transport",
            Self::TooLarge => "too-large",
        }
    }
}

/// Rejection from the outbound request policy.
///
/// `Display` is what callers may return to the user: it is a constant, generic
/// message. `detail()` carries the host/address for `log::warn!` only.
#[derive(Debug, Clone)]
pub struct SsrfError {
    kind: SsrfErrorKind,
    detail: String,
}

impl SsrfError {
    pub fn new(kind: SsrfErrorKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }

    pub fn kind(&self) -> SsrfErrorKind {
        self.kind
    }

    /// Log-only explanation. Never render this into an HTTP response.
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl fmt::Display for SsrfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "the requested URL cannot be fetched ({})",
            self.kind.as_str()
        )
    }
}

impl std::error::Error for SsrfError {}

/// True for the IPv4 ranges reserved for carrier-grade NAT (`100.64.0.0/10`).
fn is_cgnat(v4: Ipv4Addr) -> bool {
    let octets = v4.octets();
    octets[0] == 100 && (64..=127).contains(&octets[1])
}

/// Non-global IPv4 blocks that `std`'s helpers do not cover.
fn is_reserved_v4(v4: Ipv4Addr) -> bool {
    let o = v4.octets();
    is_cgnat(v4)
        // 192.0.0.0/24 — IETF protocol assignments
        || (o[0] == 192 && o[1] == 0 && o[2] == 0)
        // 198.18.0.0/15 — benchmarking
        || (o[0] == 198 && (o[1] == 18 || o[1] == 19))
        // 240.0.0.0/4 — reserved, includes the 255.255.255.255 broadcast
        || o[0] >= 240
}

/// Address classifier used by every SSRF decision.
pub fn is_blocked_address(addr: IpAddr) -> bool {
    if is_blocked_ip(addr) {
        return true;
    }
    match addr {
        IpAddr::V4(v4) => is_reserved_v4(v4),
        IpAddr::V6(v6) => {
            let segments = v6.segments();
            let first = segments[0];
            v6.is_multicast()
                // 2001:db8::/32 — documentation
                || (first == 0x2001 && segments[1] == 0x0db8)
                // 64:ff9b::/96 — NAT64, can embed an internal IPv4
                || (first == 0x0064 && segments[1] == 0xff9b)
        }
    }
}

/// Normalises a hostname for policy checks: lower-case, no trailing dot.
fn normalise_host(host: &str) -> String {
    host.trim_start_matches('[')
        .trim_end_matches(']')
        .trim_end_matches('.')
        .to_ascii_lowercase()
}

/// Rejects hosts written as a number the URL parser might normalise to a
/// loopback address (`2130706433`, `0x7f000001`, `127.1`, `017700000001`).
///
/// Only bare numeric and `0x`-prefixed hosts are treated as encoded addresses;
/// a normal domain that happens to use only hex letters and digits
/// (`face2face.com`) is left to DNS.
fn looks_like_obfuscated_ipv4(host: &str) -> bool {
    let host = host.trim_start_matches('[').trim_end_matches(']');
    if host.is_empty() {
        return false;
    }
    if let Some(rest) = host.strip_prefix("0x").or_else(|| host.strip_prefix("0X")) {
        return !rest.is_empty()
            && rest
                .chars()
                .all(|c| c.is_ascii_hexdigit() || c == '.');
    }
    host.chars().all(|c| c.is_ascii_digit() || c == '.')
        && host.chars().any(|c| c.is_ascii_digit())
        && host.parse::<IpAddr>().is_err()
}

/// Full synchronous policy check. Returns the parsed URL so callers do not
/// re-parse (and re-interpret) the input.
pub fn validate_url_ssrf_strict(raw: &str) -> Result<Url, SsrfError> {
    let url = Url::parse(raw).map_err(|e| {
        SsrfError::new(SsrfErrorKind::MissingHost, format!("unparsable URL: {e}"))
    })?;

    match url.scheme() {
        "http" | "https" => {}
        other => {
            return Err(SsrfError::new(
                SsrfErrorKind::UnsupportedScheme,
                format!("scheme `{other}` is not allowed"),
            ))
        }
    }

    if !url.username().is_empty() || url.password().is_some() {
        return Err(SsrfError::new(
            SsrfErrorKind::CredentialsInUrl,
            "URL carries userinfo",
        ));
    }

    let host = url.host_str().ok_or_else(|| {
        SsrfError::new(SsrfErrorKind::MissingHost, "URL has no host component")
    })?;
    let host = normalise_host(host);

    if host.is_empty() {
        return Err(SsrfError::new(
            SsrfErrorKind::MissingHost,
            "URL has an empty host",
        ));
    }

    if BLOCKED_HOSTS.contains(&host.as_str())
        || BLOCKED_HOST_SUFFIXES
            .iter()
            .any(|suffix| host.ends_with(suffix))
    {
        return Err(SsrfError::new(
            SsrfErrorKind::BlockedHost,
            format!("host `{host}` is local by policy"),
        ));
    }

    // IPv6 literals keep their brackets out of `host_str()`; `IpAddr` accepts
    // the bare form. `url` already normalises decimal/hex/short IPv4 into
    // dotted-decimal, so this catches `http://2130706433/` too.
    if let Ok(addr) = host.parse::<IpAddr>() {
        if is_blocked_address(addr) {
            return Err(SsrfError::new(
                SsrfErrorKind::BlockedAddress,
                format!("host `{host}` is a non-global address"),
            ));
        }
    } else if looks_like_obfuscated_ipv4(&host) {
        return Err(SsrfError::new(
            SsrfErrorKind::ObfuscatedAddress,
            format!("host `{host}` is an encoded address"),
        ));
    } else if !host.contains('.') && !host.contains(':') {
        // Single-label names only resolve through a local search domain.
        return Err(SsrfError::new(
            SsrfErrorKind::BlockedHost,
            format!("host `{host}` is a single-label name"),
        ));
    }

    if let Some(port) = url.port_or_known_default() {
        if BLOCKED_PORTS.contains(&port) {
            return Err(SsrfError::new(
                SsrfErrorKind::BlockedPort,
                format!("port {port} is a control-plane port"),
            ));
        }
    }

    Ok(url)
}

/// Resolves the host and refuses the request unless every answer is global.
///
/// Checking *all* answers (not just the first) is what stops a resolver that
/// alternates a public and a private address between lookups.
pub async fn resolve_checked(url: &Url) -> Result<Vec<IpAddr>, SsrfError> {
    let host = url
        .host_str()
        .ok_or_else(|| SsrfError::new(SsrfErrorKind::MissingHost, "URL has no host"))?;
    let port = url.port_or_known_default().unwrap_or(443);

    let addrs: Vec<IpAddr> = match host.parse::<IpAddr>() {
        Ok(addr) => vec![addr],
        Err(_) => {
            let lookup = tokio::net::lookup_host((host, port)).await.map_err(|e| {
                SsrfError::new(
                    SsrfErrorKind::ResolutionFailed,
                    format!("DNS lookup for `{host}` failed: {e}"),
                )
            })?;
            lookup.map(|sa| sa.ip()).collect()
        }
    };

    if addrs.is_empty() {
        return Err(SsrfError::new(
            SsrfErrorKind::NoAddresses,
            format!("`{host}` resolved to no address"),
        ));
    }

    for addr in &addrs {
        if is_blocked_address(*addr) {
            return Err(SsrfError::new(
                SsrfErrorKind::BlockedAddress,
                format!("`{host}` resolves to non-global address {addr}"),
            ));
        }
    }

    Ok(addrs)
}

/// Re-validates one redirect hop. Relative `Location` values are resolved
/// against `base`; the result goes through the same strict policy as the
/// original URL, so scheme changes and private targets are refused.
pub fn validate_redirect_target(base: &Url, location: &str) -> Result<Url, SsrfError> {
    let joined = base.join(location).map_err(|e| {
        SsrfError::new(
            SsrfErrorKind::InvalidRedirect,
            format!("unusable Location header: {e}"),
        )
    })?;
    validate_url_ssrf_strict(joined.as_str())
}
