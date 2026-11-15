//! Where the model of a conversation runs.
//!
//! The chat interface shows, next to the model name, whether prompts stay on
//! this machine, go to a server on the organisation's network, or leave for an
//! external provider. Only the host is reported: never the path, the query or
//! a credential.

use std::net::IpAddr;

/// Scope of a model endpoint, from the point of view of the person's data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// The endpoint is on this computer.
    Machine,
    /// A private address or an unqualified host name: the local network.
    Network,
    /// Anything else: a host on the internet.
    External,
    /// No endpoint is configured, or it cannot be read.
    Unknown,
}

impl Scope {
    fn as_str(self) -> &'static str {
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
        // fc00::/7 unique local, fe80::/10 link-local.
        IpAddr::V6(v6)
            if (v6.segments()[0] & 0xfe00) == 0xfc00 || (v6.segments()[0] & 0xffc0) == 0xfe80 =>
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

/// Host and scope of a model endpoint URL.
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
        Err(error) => {
            log::debug!("model endpoint is not a URL: {error}");
            (None, Scope::Unknown)
        }
    }
}

/// JSON for the chat interface: `{ "host": …, "scope": … }`.
pub fn destination_json(endpoint: &str) -> serde_json::Value {
    let (host, scope) = describe(endpoint);
    serde_json::json!({ "host": host, "scope": scope.as_str() })
}

#[cfg(test)]
mod tests {
    use super::{describe, destination_json, Scope};

    #[test]
    fn this_machine_the_local_network_and_the_internet_are_told_apart() {
        let cases = [
            ("http://127.0.0.1:8081/v1/chat/completions", Scope::Machine),
            ("http://localhost:8081/v1", Scope::Machine),
            ("http://[::1]:8081/v1", Scope::Machine),
            ("http://10.20.30.40:8000/v1", Scope::Network),
            ("https://192.168.1.10/v1", Scope::Network),
            ("http://modelos:8000/v1", Scope::Network),
            ("https://ia.orgao.internal/v1", Scope::Network),
            ("https://api.example.com/v1/chat/completions", Scope::External),
            ("https://8.8.8.8/v1", Scope::External),
        ];
        for (endpoint, expected) in cases {
            assert_eq!(describe(endpoint).1, expected, "{endpoint}");
        }
    }

    #[test]
    fn only_the_host_is_reported() {
        let value = destination_json("https://user:secret@api.example.com:8443/v1/x?key=abc");
        assert_eq!(value["host"], "api.example.com");
        assert_eq!(value["scope"], "external");
        assert!(!value.to_string().contains("secret"));
        assert!(!value.to_string().contains("abc"));
    }

    #[test]
    fn a_missing_or_unreadable_endpoint_is_unknown() {
        assert_eq!(describe("").1, Scope::Unknown);
        assert_eq!(describe("not a url").1, Scope::Unknown);
        assert!(destination_json("").get("host").is_some_and(serde_json::Value::is_null));
    }
}
