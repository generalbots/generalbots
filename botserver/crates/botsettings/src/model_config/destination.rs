//! Host and scope of a model endpoint URL — the same logic as
//! `chat_destination.rs` in the main binary, duplicated here so the
//! settings crate does not depend on the binary.

use std::net::IpAddr;

/// Scope of a model endpoint, from the point of view of the person's data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    Machine,
    Network,
    External,
    Unknown,
}

impl Scope {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Machine => "machine",
            Self::Network => "network",
            Self::External => "external",
            Self::Unknown => "unknown",
        }
    }
}

fn scope_of_ip(ip: IpAddr) -> Scope {
    match ip {
        IpAddr::V4(v4) if v4.is_loopback() || v4.is_unspecified() => Scope::Machine,
        IpAddr::V4(v4) if v4.is_private() || v4.is_link_local() => Scope::Network,
        IpAddr::V6(v6) if v6.is_loopback() || v6.is_unspecified() => Scope::Machine,
        IpAddr::V6(v6)
            if (v6.segments()[0] & 0xfe00) == 0xfc00
                || (v6.segments()[0] & 0xffc0) == 0xfe80 =>
        {
            Scope::Network
        }
        IpAddr::V4(_) | IpAddr::V6(_) => Scope::External,
    }
}

fn scope_of_host(host: &str) -> Scope {
    let bare = host.trim_start_matches('[').trim_end_matches(']');
    if let Ok(ip) = bare.parse::<IpAddr>() {
        return scope_of_ip(ip);
    }
    let name = bare.trim_end_matches('.').to_ascii_lowercase();
    if name == "localhost" || name.ends_with(".localhost") {
        return Scope::Machine;
    }
    let private_suffix = [".local", ".lan", ".internal", ".intranet", ".home.arpa"]
        .iter()
        .any(|suffix| name.ends_with(suffix));
    if !name.contains('.') || private_suffix {
        return Scope::Network;
    }
    Scope::External
}

/// Host and scope of an endpoint URL.
pub fn describe(endpoint: &str) -> (Option<String>, Scope) {
    let trimmed = endpoint.trim();
    if trimmed.is_empty() {
        return (None, Scope::Unknown);
    }
    match reqwest::Url::parse(trimmed) {
        Ok(url) => match url.host_str() {
            Some(host) => (Some(host.to_string()), scope_of_host(host)),
            None => (None, Scope::Unknown),
        },
        Err(_) => (None, Scope::Unknown),
    }
}

#[cfg(test)]
mod tests {
    use super::{describe, Scope};

    #[test]
    fn scope_matches_chat_destination() {
        assert_eq!(describe("http://localhost:8081/v1").1, Scope::Machine);
        assert_eq!(describe("http://127.0.0.1:8081/v1").1, Scope::Machine);
        assert_eq!(describe("http://10.0.0.5:8000/v1").1, Scope::Network);
        assert_eq!(describe("https://api.example.com/v1").1, Scope::External);
        assert_eq!(describe("").1, Scope::Unknown);
    }
}
