//! Inter-service mTLS configuration and status (#2590).
//!
//! Containerized (Incus) installs place backing services in separate containers
//! on a private bridge. On that hop, reachability plus a password/token is the
//! only authentication, so a compromised container can impersonate another
//! service. Mutual TLS closes that gap: both ends present a certificate issued
//! by the install's internal CA and verify the peer against that CA.
//!
//! This module resolves *configuration* only. It never embeds certificate or
//! key material: paths are read from the environment or from Vault
//! (`secret/gbo/system/security`) and only their presence is checked. The
//! [`runtime`] helpers turn that configuration into secure client URLs and
//! path descriptors; the staged rollout and its current limits are documented
//! in `botbook/src/09-security/README.md`.
//!
//! Single-host/desktop-local installs keep their loopback threat model and are
//! explicitly out of scope: the opt-in is off unless an operator sets it.

pub mod config;
pub mod runtime;
pub mod status;

#[cfg(test)]
mod tests;

pub use config::{
    MtlsConfig, RolloutStage, TransportMode, ENV_CA_CERT, ENV_CLIENT_CERT, ENV_CLIENT_KEY,
    ENV_MODE, ENV_SERVER_CERT, ENV_SERVER_KEY, ENV_STAGE,
};
pub use runtime::{secure_cache_url, secure_database_url, secure_http_url, ClientTlsPaths};
pub use status::{current_status, init, is_enabled, is_supported, MtlsStatus};
