//! In-memory persistence for typed approval records.
//!
//! A workflow slice intentionally needs no migration: records live for the
//! process lifetime, which is enough to exercise routing, SLA, delegation,
//! signed decisions and publishing end-to-end. A future persistent backend
//! can replace this store without changing the workflow engine or API.

use std::collections::HashMap;
use std::sync::RwLock;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::engine;
use super::signing::{verify_decision, version_hash};
use super::types::{
    ApprovalRecord, ApprovalStatus, ApprovalStep, DecisionKind, Delegation, PublishedArtifact,
    SlaEvent,
};

pub struct ApprovalStore {
    records: RwLock<HashMap<Uuid, ApprovalRecord>>,
    secret: Vec<u8>,
}

/// Signing key comes from the environment when configured; otherwise a fresh
/// per-process random key is generated. No credential is hardcoded.
fn signing_secret() -> Vec<u8> {
    match std::env::var("GB_APPROVALS_SIGNING_KEY") {
        Ok(value) if !value.is_empty() => value.into_bytes(),
        _ => Uuid::new_v4().as_bytes().to_vec(),
    }
}

impl Default for ApprovalStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ApprovalStore {
    #[must_use]
    pub fn new() -> Self {
        Self {
            records: RwLock::new(HashMap::new()),
            secret: signing_secret(),
        }
    }

    fn read(&self) -> Result<std::sync::RwLockReadGuard<'_, HashMap<Uuid, ApprovalRecord>>, String> {
        self.records.read().map_err(|_| "approval store lock poisoned".to_string())
    }

    fn write(&self) -> Result<std::sync::RwLockWriteGuard<'_, HashMap<Uuid, ApprovalRecord>>, String> {
        self.records.write().map_err(|_| "approval store lock poisoned".to_string())
    }

    pub fn create(
        &self,
        tenant: &str,
        doc_id: &str,
        doc_class: &str,
        title: &str,
        owner: &str,
        content: &str,
    ) -> Result<ApprovalRecord, String> {
        let hash = version_hash(content.as_bytes());
        let mut record = engine::blank_record(tenant, doc_id, doc_class, owner, &hash);
        record.title = title.to_string();
        let mut records = self.write()?;
        records.insert(record.id, record.clone());
        Ok(record)
    }

    pub fn get(&self, id: Uuid) -> Result<Option<ApprovalRecord>, String> {
        Ok(self.read()?.get(&id).cloned())
    }

    pub fn list(&self) -> Result<Vec<ApprovalRecord>, String> {
        let mut rows: Vec<ApprovalRecord> = self.read()?.values().cloned().collect();
        rows.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(rows)
    }

    /// Pending approvals for a person/team/tenant identifier, after applying
    /// delegations. Used by the dashboard and by chat status queries.
    pub fn pending_for(&self, actor: &str) -> Result<Vec<ApprovalRecord>, String> {
        let now = Utc::now();
        let mut rows: Vec<ApprovalRecord> = self
            .read()?
            .values()
            .filter(|record| {
                matches!(
                    record.status,
                    ApprovalStatus::InReview | ApprovalStatus::AwaitingSignature
                ) && record.effective_assignees(now).iter().any(|name| name == actor)
            })
            .cloned()
            .collect();
        rows.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(rows)
    }

    pub fn set_route(
        &self,
        id: Uuid,
        route: Vec<ApprovalStep>,
        message: &str,
        due_at: Option<DateTime<Utc>>,
    ) -> Result<ApprovalRecord, String> {
        let now = Utc::now();
        let mut records = self.write()?;
        let record = records.get_mut(&id).ok_or("approval not found".to_string())?;
        engine::start_route(record, route, message, due_at, now)?;
        Ok(record.clone())
    }

    pub fn delegate(&self, id: Uuid, delegation: Delegation) -> Result<ApprovalRecord, String> {
        let mut records = self.write()?;
        let record = records.get_mut(&id).ok_or("approval not found".to_string())?;
        if record.status.is_immutable() {
            return Err("published approval is immutable".to_string());
        }
        record.delegations.push(delegation);
        record.updated_at = Utc::now();
        Ok(record.clone())
    }

    pub fn decide(
        &self,
        id: Uuid,
        actor: &str,
        kind: DecisionKind,
        reason: &str,
        comment: &str,
        expected_hash: Option<&str>,
    ) -> Result<ApprovalRecord, String> {
        let now = Utc::now();
        let mut records = self.write()?;
        let record = records.get_mut(&id).ok_or("approval not found".to_string())?;
        engine::apply_decision(
            record,
            &self.secret,
            actor,
            kind,
            reason,
            comment,
            expected_hash,
            now,
        )?;
        Ok(record.clone())
    }

    /// Explicit e-signature step: only legal on a signature step, produces the
    /// same signed decision event and sets the signed status.
    pub fn sign(&self, id: Uuid, actor: &str, comment: &str) -> Result<ApprovalRecord, String> {
        let mut records = self.write()?;
        let record = records.get_mut(&id).ok_or("approval not found".to_string())?;
        if record
            .current_step()
            .map(|step| step.kind != super::types::StepKind::Signature)
            == Some(true)
        {
            return Err("no signature step is active".to_string());
        }
        let hash = record.version_hash.clone();
        engine::apply_decision(
            record,
            &self.secret,
            actor,
            DecisionKind::Approve,
            "",
            comment,
            Some(&hash),
            Utc::now(),
        )?;
        Ok(record.clone())
    }

    /// A substantive edit after routing invalidates the approval visibly.
    pub fn update_content(&self, id: Uuid, content: &str) -> Result<ApprovalRecord, String> {
        let hash = version_hash(content.as_bytes());
        let mut records = self.write()?;
        let record = records.get_mut(&id).ok_or("approval not found".to_string())?;
        if record.status.is_immutable() {
            return Err("published approval is immutable".to_string());
        }
        engine::invalidate_if_changed(record, &hash);
        record.updated_at = Utc::now();
        Ok(record.clone())
    }

    /// SLA sweep. `now` is injectable so a scheduler or UAT can simulate a
    /// breach without waiting. Reminder and escalation are marked once.
    pub fn check_sla_at(&self, now: DateTime<Utc>) -> Result<Vec<SlaEvent>, String> {
        let mut records = self.write()?;
        let mut events = Vec::new();
        for record in records.values_mut() {
            let found = engine::check_sla(record, now);
            if found
                .iter()
                .any(|event| event.kind == super::types::SlaEventKind::Reminder)
            {
                record.reminder_sent = true;
            }
            if found
                .iter()
                .any(|event| event.kind == super::types::SlaEventKind::Escalation)
            {
                record.escalation_sent = true;
            }
            if !found.is_empty() {
                record.updated_at = now;
            }
            events.extend(found);
        }
        Ok(events)
    }

    pub fn publish(&self, id: Uuid) -> Result<PublishedArtifact, String> {
        let mut records = self.write()?;
        let record = records.get_mut(&id).ok_or("approval not found".to_string())?;
        engine::publish(record, Utc::now())
    }

    /// Verifies every decision signature in the trail. A tampered decision
    /// (or a key change across restarts) makes this return `false`.
    pub fn verify_trail(&self, id: Uuid) -> Result<bool, String> {
        let record = match self.get(id)? {
            Some(record) => record,
            None => return Ok(false),
        };
        for decision in record.decisions.iter() {
            let body = engine::decision_body(
                record.id,
                &decision.step_id,
                &decision.actor,
                decision.decision,
                &decision.reason,
                &decision.comment,
                &decision.version_hash,
                &decision.prev_hash,
                decision.decided_at.timestamp(),
            );
            if !verify_decision(&self.secret, &decision.signature, &body) {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests;
