//! #2765 — tamper-evident audit records for security decisions.
//!
//! The hash-chained `AuditStore` had no production callers: DLP redactions,
//! prompt-injection blocks, CSRF rejects and RBAC denies were only logged at
//! `debug!`/`warn!` level. This module is the single bridge from those
//! decisions to the existing `AuditLogger`. It is deliberately narrow: a
//! caller supplies a server-minted actor, a static action/reason code and the
//! raw payload, of which only a SHA-256 fingerprint is retained. The payload
//! itself is never copied into the event.

use super::*;
use std::sync::OnceLock;

/// A security decision to be written to the audit chain.
///
/// `action` and `reason_code` are static strings chosen at the call site, so a
/// caller cannot smuggle user content into the record through them. `payload`
/// is the raw input that triggered the decision; only its digest is stored.
pub struct SecurityDecision<'a> {
    pub event_type: AuditEventType,
    pub actor: AuditActor,
    pub outcome: AuditOutcome,
    pub action: &'static str,
    pub reason_code: &'static str,
    pub payload: &'a str,
}

/// Pure predicate: whether a security decision must reach the audit chain.
///
/// Enforcement outcomes (`Failure`, `Partial`) are always recorded, as are
/// explicitly auditable successes (authentication, role/group and admin
/// changes). Other successes and unknown outcomes are skipped so a busy server
/// does not flood the chain with non-events.
pub fn should_emit_security_decision(decision: &SecurityDecision<'_>) -> bool {
    if decision.action.is_empty() || decision.reason_code.is_empty() {
        return false;
    }
    match &decision.outcome {
        AuditOutcome::Failure | AuditOutcome::Partial => true,
        AuditOutcome::Success => matches!(
            decision.event_type,
            AuditEventType::LoginSuccess
                | AuditEventType::PermissionGranted
                | AuditEventType::MfaEnabled
                | AuditEventType::MfaDisabled
                | AuditEventType::RoleAssigned
                | AuditEventType::RoleRemoved
                | AuditEventType::GroupJoined
                | AuditEventType::GroupLeft
                | AuditEventType::UserCreate
                | AuditEventType::UserDelete
                | AuditEventType::UserModify
                | AuditEventType::ConfigChange
                | AuditEventType::SettingChange
                | AuditEventType::BotCreate
                | AuditEventType::BotDelete
                | AuditEventType::BotModify
        ),
        AuditOutcome::Unknown => false,
    }
}

/// Pure predicate: an event produced by [`security_decision_event`] carries the
/// actor/action/outcome/timestamp shape of a security-decision record and no
/// free-form payload. Kept separate from the builder so a guard can assert the
/// invariant without writing to the store.
pub fn is_security_decision_event(event: &AuditEvent) -> bool {
    !event.action.is_empty()
        && !event.description.is_empty()
        && event.metadata.contains_key("reason_code")
        && event.metadata.contains_key("payload_sha256")
}

/// Pure builder: turns a decision into an `AuditEvent`. The raw payload is
/// hashed here and never placed in `description`, `resource` or `metadata`.
pub fn security_decision_event(decision: &SecurityDecision<'_>) -> AuditEvent {
    let mut hasher = Sha256::new();
    hasher.update(decision.payload.as_bytes());
    let fingerprint = hex::encode(hasher.finalize());

    AuditEvent::new(decision.event_type, decision.actor.clone())
        .with_outcome(decision.outcome.clone())
        .with_description(decision.action)
        .with_metadata("reason_code", serde_json::json!(decision.reason_code))
        .with_metadata("payload_sha256", serde_json::json!(fingerprint))
}

/// Synchronous persistence: each decision is chained and written immediately,
/// so `verify_chain` covers it even if the process restarts right afterwards.
pub fn security_decision_config() -> AuditConfig {
    AuditConfig {
        async_logging: false,
        ..AuditConfig::default()
    }
}

static SECURITY_AUDIT: OnceLock<AuditLogger<InMemoryAuditStore>> = OnceLock::new();
// The logger hands out the previous hash before awaiting the store write.
// Serialize process-wide emissions so chain order and store order cannot
// diverge when two denials land in the same instant.
static SECURITY_AUDIT_WRITE: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

/// Process-wide audit sink for security decisions. Installed lazily so a
/// library-only caller (or a test) still gets a chain without server setup.
pub fn security_audit_logger() -> &'static AuditLogger<InMemoryAuditStore> {
    SECURITY_AUDIT.get_or_init(|| {
        AuditLogger::new(security_decision_config(), InMemoryAuditStore::default())
    })
}

/// Let the server install a configured sink once at boot. Replacements are
/// refused: swapping the logger mid-flight would break the hash chain.
pub fn install_security_audit_logger(logger: AuditLogger<InMemoryAuditStore>) {
    if SECURITY_AUDIT.set(logger).is_err() {
        tracing::warn!("security audit logger already installed; existing chain kept");
    }
}

/// Emit a decision through an explicit logger (used by callers that own one and
/// by the inline chain test).
pub async fn audit_security_decision<S: AuditStore>(
    logger: &AuditLogger<S>,
    decision: &SecurityDecision<'_>,
) -> Result<()> {
    if !should_emit_security_decision(decision) {
        return Ok(());
    }
    logger.log(security_decision_event(decision)).await
}

/// Emit a decision through the process-wide sink. `actor` is supplied by the
/// server (never by the request), so a client cannot attribute an event to a
/// different principal.
pub async fn record_security_decision(decision: &SecurityDecision<'_>) -> Result<()> {
    let guard = SECURITY_AUDIT_WRITE
        .get_or_init(|| tokio::sync::Mutex::new(()))
        .lock()
        .await;
    let result = audit_security_decision(security_audit_logger(), decision).await;
    drop(guard);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decision<'a>(
        event_type: AuditEventType,
        outcome: AuditOutcome,
        payload: &'a str,
    ) -> SecurityDecision<'a> {
        SecurityDecision {
            event_type,
            actor: AuditActor::user(Uuid::nil()),
            outcome,
            action: "rbac.deny",
            reason_code: "rbac.denied",
            payload,
        }
    }

    #[test]
    fn enforcement_outcomes_are_emitted_and_benign_success_is_not() {
        assert!(should_emit_security_decision(&decision(
            AuditEventType::PermissionDenied,
            AuditOutcome::Failure,
            "x"
        )));
        assert!(should_emit_security_decision(&decision(
            AuditEventType::PolicyViolation,
            AuditOutcome::Partial,
            "x"
        )));
        assert!(should_emit_security_decision(&decision(
            AuditEventType::LoginSuccess,
            AuditOutcome::Success,
            "x"
        )));
        assert!(!should_emit_security_decision(&decision(
            AuditEventType::DataRead,
            AuditOutcome::Success,
            "x"
        )));
        assert!(!should_emit_security_decision(&decision(
            AuditEventType::DataRead,
            AuditOutcome::Unknown,
            "x"
        )));
    }

    #[test]
    fn empty_action_or_reason_code_is_never_emitted() {
        let mut candidate = decision(
            AuditEventType::PermissionDenied,
            AuditOutcome::Failure,
            "x",
        );
        candidate.action = "";
        assert!(!should_emit_security_decision(&candidate));
        candidate.action = "rbac.deny";
        candidate.reason_code = "";
        assert!(!should_emit_security_decision(&candidate));
    }

    #[test]
    fn event_predicate_accepts_only_payload_free_decision_records() {
        let event = security_decision_event(&decision(
            AuditEventType::PermissionDenied,
            AuditOutcome::Failure,
            "secret",
        ));
        assert!(is_security_decision_event(&event));
        let mut tampered = event.clone();
        tampered.metadata.remove("payload_sha256");
        assert!(!is_security_decision_event(&tampered));
    }

    #[test]
    fn raw_payload_is_never_placed_in_the_event() {
        let secret = "4111111111111111";
        let event = security_decision_event(&decision(
            AuditEventType::PolicyViolation,
            AuditOutcome::Partial,
            secret,
        ));
        let serialized = serde_json::to_string(&event).expect("serialize event");
        assert!(!serialized.contains(secret));
        assert_eq!(event.actor.actor_type, ActorType::User);
        assert_eq!(event.outcome, AuditOutcome::Partial);
        assert!(event.timestamp <= Utc::now());
    }

    #[tokio::test]
    async fn denied_rbac_decision_produces_a_hash_chained_record() {
        let logger = AuditLogger::new(security_decision_config(), InMemoryAuditStore::new(16));
        audit_security_decision(
            &logger,
            &decision(AuditEventType::PermissionDenied, AuditOutcome::Failure, "first payload"),
        )
        .await
        .expect("first audit write");
        audit_security_decision(
            &logger,
            &decision(AuditEventType::PermissionDenied, AuditOutcome::Failure, "second payload"),
        )
        .await
        .expect("second audit write");

        let result = logger
            .query(AuditQuery::new().with_limit(10))
            .await
            .expect("audit query");
        assert_eq!(result.total_count, 2);
        assert_eq!(result.events.len(), 2);
        // query() returns newest first.
        let newest = &result.events[0];
        let oldest = &result.events[1];
        assert_eq!(newest.previous_hash.as_deref(), Some(oldest.hash.as_str()));
        assert!(newest.verify_hash());
        assert!(oldest.verify_hash());
        assert!(is_security_decision_event(newest));
    }
}
