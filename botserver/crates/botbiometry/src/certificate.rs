//! Digital certificate (e-signature) lifecycle.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Kind of digital certificate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CertificateKind {
    /// ICP-Brasil A1/A3 (Brazilian PKI).
    IcpBrasilA1,
    /// ICP-Brasil A3 (smart card / token).
    IcpBrasilA3,
    /// Generic X.509 (e.g. issued by internal CA).
    X509,
    /// eIDAS qualified certificate (EU).
    Eidas,
}

/// Status of a digital certificate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CertificateStatus {
    /// Active and not yet close to expiry.
    Active,
    /// Within 30 days of expiry.
    ExpiringSoon,
    /// Expired (timestamp past `not_after`).
    Expired,
    /// Revoked by the issuing CA.
    Revoked,
}

/// A digital certificate bound to a user.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DigitalCertificate {
    /// Server-assigned cert ID.
    pub id: Uuid,
    /// Owner user ID (Zitadel ID).
    pub owner_id: String,
    /// Kind.
    pub kind: CertificateKind,
    /// Subject DN (e.g. "CN=Maria Silva, O=ACME").
    pub subject_dn: String,
    /// Issuer DN.
    pub issuer_dn: String,
    /// Serial number (hex).
    pub serial: String,
    /// Not-before timestamp.
    pub not_before: DateTime<Utc>,
    /// Not-after timestamp.
    pub not_after: DateTime<Utc>,
    /// Current status.
    pub status: CertificateStatus,
    /// Drive path to the PEM/PFX bytes.
    pub drive_path: String,
}

impl DigitalCertificate {
    /// Returns true if the cert is currently usable for signing.
    pub fn is_usable(&self, now: DateTime<Utc>) -> bool {
        matches!(self.status, CertificateStatus::Active | CertificateStatus::ExpiringSoon)
            && now >= self.not_before
            && now < self.not_after
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn cert(not_after: DateTime<Utc>, status: CertificateStatus) -> DigitalCertificate {
        DigitalCertificate {
            id: Uuid::new_v4(),
            owner_id: "user-1".to_string(),
            kind: CertificateKind::IcpBrasilA1,
            subject_dn: "CN=Test".to_string(),
            issuer_dn: "CN=ACME CA".to_string(),
            serial: "ab".to_string(),
            not_before: Utc::now() - Duration::days(30),
            not_after,
            status,
            drive_path: "certs/user-1/ab.pem".to_string(),
        }
    }

    #[test]
    fn usable_when_active_and_in_window() {
        let c = cert(Utc::now() + Duration::days(60), CertificateStatus::Active);
        assert!(c.is_usable(Utc::now()));
    }

    #[test]
    fn not_usable_when_expired() {
        let c = cert(Utc::now() - Duration::days(1), CertificateStatus::Expired);
        assert!(!c.is_usable(Utc::now()));
    }
}
