//! Biometric identity verification, KYC, and digital signature.
//!
//! `botbiometry` exposes types and a state machine for:
//! - KYC (Know Your Customer) document collection and validation
//! - Facial recognition liveness and matching
//! - Handwritten biometric signature capture
//! - Digital certificate (e-signature) lifecycle
//!
//! Zitadel integration is exposed via the [`zitadel`] module as a trait
//! — production wiring plugs a real OIDC client; tests use a stub.

#![deny(missing_docs)]
#![deny(unsafe_code)]

pub mod kyc;
pub mod zitadel;
pub mod liveness;
pub mod signature;
pub mod certificate;
pub mod audit;

pub use kyc::{KycCase, KycState, KycDocument, DocumentKind, KycError};
pub use zitadel::{ZitadelClient, ZitadelIdentity, ZitadelError};
pub use liveness::{LivenessCheck, LivenessResult, LivenessChallenge};
pub use signature::{BiometricSignature, SignatureStroke, SignatureCapture};
pub use certificate::{DigitalCertificate, CertificateKind, CertificateStatus};
pub use audit::{BiometricAuditEvent, AuditAction, AuditLog};
