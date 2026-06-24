//! Append-only audit log for biometric actions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Audit action recorded for each biometric event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditAction {
    /// KYC case created.
    KycCreated,
    /// Document uploaded.
    KycDocumentUploaded,
    /// KYC state transition.
    KycStateChanged,
    /// Liveness check issued / answered.
    LivenessPerformed,
    /// Biometric signature captured.
    SignatureCaptured,
    /// Digital certificate issued / revoked.
    CertificateAction,
    /// Zitadel identity provisioned / disabled.
    ZitadelAction,
}

/// A single audit log entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BiometricAuditEvent {
    /// Server-assigned event ID.
    pub id: Uuid,
    /// Tenant (Zitadel org).
    pub tenant_id: String,
    /// Acting user (or system for `system` automated actions).
    pub actor_id: String,
    /// What happened.
    pub action: AuditAction,
    /// Free-form JSON payload with action-specific context.
    pub payload_json: String,
    /// When the event was recorded.
    pub at: DateTime<Utc>,
}

/// In-memory append-only audit log.
#[derive(Debug, Default, Clone)]
pub struct AuditLog {
    events: Vec<BiometricAuditEvent>,
}

impl AuditLog {
    /// Append an event to the log.
    pub fn append(&mut self, event: BiometricAuditEvent) {
        self.events.push(event);
    }

    /// Iterate over all events.
    pub fn iter(&self) -> impl Iterator<Item = &BiometricAuditEvent> {
        self.events.iter()
    }

    /// Total number of recorded events.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Whether the log is empty.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_and_iter() {
        let mut log = AuditLog::default();
        assert!(log.is_empty());
        log.append(BiometricAuditEvent {
            id: Uuid::new_v4(),
            tenant_id: "t1".to_string(),
            actor_id: "user-1".to_string(),
            action: AuditAction::KycCreated,
            payload_json: "{}".to_string(),
            at: Utc::now(),
        });
        assert_eq!(log.len(), 1);
        assert_eq!(log.iter().count(), 1);
    }
}
