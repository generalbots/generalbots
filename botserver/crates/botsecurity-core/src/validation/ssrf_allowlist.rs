//! Internal-service allowlist for the SSRF guard (#2131).
//!
//! The guard exists to stop a *caller-supplied* URL from reaching the local
//! node, the private network or a cloud metadata endpoint. Platform code also
//! talks to its own services (Vault, MinIO, Valkey, the LLM, BotModels) with
//! URLs the **server** built from configuration — never from a request body,
//! a bot script or an LLM tool call. Those calls are allowed through
//! [`super::fetch_internal_service_url`], and only when the host matches one of
//! the entries below.
//!
//! Rules for this list:
//! * it is a *host* allowlist, not a bypass switch: scheme, credentials and
//!   redirects are still validated on every hop;
//! * entries are added only for a service this deployment runs itself;
//! * a caller-supplied URL must never be fetched with the internal helper.
//!   `scripts/check_ssrf.py` enforces the pairing at review time.

/// One allowlisted internal service.
#[derive(Debug, Clone, Copy)]
pub struct InternalService {
    /// Host as it appears in the URL (container name or loopback alias).
    pub host: &'static str,
    /// Default port used for documentation and log messages.
    pub port: u16,
    /// Why the server is allowed to reach it.
    pub reason: &'static str,
}

/// Documented internal services reachable through the allowlisted path.
pub const INTERNAL_SERVICES: &[InternalService] = &[
    InternalService {
        host: "localhost",
        port: 8080,
        reason: "botserver API on the same node (health, self-calls)",
    },
    InternalService {
        host: "127.0.0.1",
        port: 8080,
        reason: "botserver API on the same node (health, self-calls)",
    },
    InternalService {
        host: "vault",
        port: 8200,
        reason: "Vault secret reads performed at boot",
    },
    InternalService {
        host: "tables",
        port: 5432,
        reason: "PostgreSQL container (health probes)",
    },
    InternalService {
        host: "cache",
        port: 6379,
        reason: "Valkey container (health probes)",
    },
    InternalService {
        host: "drive",
        port: 9000,
        reason: "MinIO object storage",
    },
    InternalService {
        host: "directory",
        port: 9000,
        reason: "Zitadel identity provider",
    },
    InternalService {
        host: "llm",
        port: 8081,
        reason: "local llama.cpp inference server",
    },
    InternalService {
        host: "meet",
        port: 7880,
        reason: "LiveKit media server",
    },
    InternalService {
        host: "vectordb",
        port: 6333,
        reason: "Qdrant vector database",
    },
    InternalService {
        host: "alm",
        port: 4747,
        reason: "Forgejo ALM (git + CI API)",
    },
    InternalService {
        host: "botmodels",
        port: 8085,
        reason: "BotModels pythonbox (speech, vision, anomaly)",
    },
];

/// True when `host` is a documented internal service host.
///
/// Matching is exact and case-insensitive: no suffix or wildcard entry, so a
/// DNS name that merely *ends* with an internal label is not allowed.
pub fn is_internal_service_host(host: &str) -> bool {
    let host = host.trim_end_matches('.').to_ascii_lowercase();
    INTERNAL_SERVICES
        .iter()
        .any(|service| service.host.eq_ignore_ascii_case(&host))
}

/// Reason string for logs when `host` is allowlisted.
pub fn internal_service_reason(host: &str) -> Option<&'static str> {
    let host = host.trim_end_matches('.').to_ascii_lowercase();
    INTERNAL_SERVICES
        .iter()
        .find(|service| service.host.eq_ignore_ascii_case(&host))
        .map(|service| service.reason)
}
