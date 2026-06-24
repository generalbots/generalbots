//! Zitadel client interface and identity types.
//!
//! The trait abstracts OIDC calls so unit tests can run without a real
//! Zitadel instance. Production wires the implementation in
//! `botcoredirectory`.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Identity record returned by Zitadel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZitadelIdentity {
    /// Zitadel user ID (opaque string).
    pub id: String,
    /// Human display name.
    pub display_name: String,
    /// Primary email (login).
    pub email: String,
    /// Whether MFA is enrolled.
    pub mfa_enrolled: bool,
    /// Account creation time.
    pub created_at: DateTime<Utc>,
}

/// Errors that may surface from the Zitadel client.
#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum ZitadelError {
    /// User already exists in the tenant.
    #[error("user with email {0} already exists")]
    UserExists(String),
    /// Network/HTTP failure.
    #[error("transport: {0}")]
    Transport(String),
    /// OIDC token issue.
    #[error("token rejected: {0}")]
    Unauthorized(String),
}

/// Async client interface for Zitadel.
#[async_trait]
pub trait ZitadelClient: Send + Sync {
    /// Provision a new identity from a KYC-approved case.
    async fn provision(&self, email: &str, display_name: &str) -> Result<ZitadelIdentity, ZitadelError>;

    /// Fetch an existing identity by ID.
    async fn get(&self, id: &str) -> Result<ZitadelIdentity, ZitadelError>;

    /// Disable an identity (e.g. after KYC revocation).
    async fn disable(&self, id: &str) -> Result<(), ZitadelError>;
}
