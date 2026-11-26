//! Persistence for agent-batch undo records.
//!
//! [`PgUndoStore`] reuses `consent_audit` — the table behind the consent gate —
//! storing a typed envelope in its JSON `request` column and a distinct
//! outcome marker. Consumption is an atomic conditional UPDATE, so two
//! concurrent "undo" commands cannot both revert one batch. [`InMemoryUndoStore`]
//! mirrors the same contract for deterministic, server-free unit tests.

use std::collections::HashMap;
use std::sync::Mutex;

use botschema::consent_audit;
use chrono::Utc;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{BatchKey, BatchStatus, UndoBatch, UndoError, UndoOperation, UndoScope};

/// `consent_audit.outcome` is `VARCHAR(24)`; both markers fit.
pub const OUTCOME_ACTIVE: &str = "undo_batch_active";
pub const OUTCOME_UNDONE: &str = "undo_batch_undone";
const ENVELOPE_KIND: &str = "agent_undo_batch";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct UndoRecord {
    kind: String,
    batch: UndoBatch,
}

impl UndoRecord {
    fn new(batch: UndoBatch) -> Self {
        Self {
            kind: ENVELOPE_KIND.to_string(),
            batch,
        }
    }

    fn decode(value: &serde_json::Value) -> Option<Self> {
        serde_json::from_value(value.clone()).ok()
    }
}

/// Ownership-safe, single-use contract shared by both implementations.
pub trait UndoStore {
    fn record(&self, key: BatchKey, operations: Vec<UndoOperation>)
        -> Result<UndoBatch, UndoError>;

    fn find(&self, key: &BatchKey) -> Result<Option<UndoBatch>, UndoError>;

    /// Newest active batch owned by this exact scope, or `None`.
    fn latest(&self, scope: &UndoScope) -> Result<Option<UndoBatch>, UndoError>;

    /// Marks the batch consumed and returns its before states once.
    fn consume(&self, key: &BatchKey) -> Result<UndoBatch, UndoError>;
}

fn newest(left: &UndoBatch, right: &UndoBatch) -> std::cmp::Ordering {
    left.created_at
        .cmp(&right.created_at)
        .then_with(|| left.key.batch_id.cmp(&right.key.batch_id))
}

/// Test/local implementation. Not durable across restarts; the production
/// implementation is [`PgUndoStore`].
#[derive(Default)]
pub struct InMemoryUndoStore {
    rows: Mutex<HashMap<String, UndoBatch>>,
}

impl InMemoryUndoStore {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn rows(&self) -> Result<std::sync::MutexGuard<'_, HashMap<String, UndoBatch>>, UndoError> {
        self.rows.lock().map_err(|_| UndoError::Storage)
    }

    pub(crate) fn record_at(
        &self,
        key: BatchKey,
        operations: Vec<UndoOperation>,
        created_at: chrono::DateTime<Utc>,
    ) -> Result<UndoBatch, UndoError> {
        let batch = UndoBatch::new(key.clone(), operations, created_at)?;
        let mut rows = self.rows()?;
        if rows.contains_key(&key.storage_key()) {
            return Err(UndoError::AlreadyRecorded);
        }
        rows.insert(key.storage_key(), batch.clone());
        Ok(batch)
    }

    fn foreign_batch(&self, key: &BatchKey) -> Result<bool, UndoError> {
        Ok(self
            .rows()?
            .values()
            .any(|batch| batch.key.batch_id == key.batch_id && !batch.matches(key)))
    }
}

impl UndoStore for InMemoryUndoStore {
    fn record(
        &self,
        key: BatchKey,
        operations: Vec<UndoOperation>,
    ) -> Result<UndoBatch, UndoError> {
        self.record_at(key, operations, Utc::now())
    }

    fn find(&self, key: &BatchKey) -> Result<Option<UndoBatch>, UndoError> {
        let rows = self.rows()?;
        if let Some(batch) = rows.get(&key.storage_key()) {
            return Ok(Some(batch.clone()));
        }
        drop(rows);
        if self.foreign_batch(key)? {
            return Err(UndoError::ScopeMismatch);
        }
        Ok(None)
    }

    fn latest(&self, scope: &UndoScope) -> Result<Option<UndoBatch>, UndoError> {
        scope.validate()?;
        let rows = self.rows()?;
        Ok(rows
            .values()
            .filter(|batch| batch.key.scope.matches(scope) && batch.status == BatchStatus::Active)
            .max_by(|left, right| newest(left, right))
            .cloned())
    }

    fn consume(&self, key: &BatchKey) -> Result<UndoBatch, UndoError> {
        let mut rows = self.rows()?;
        match rows.get_mut(&key.storage_key()) {
            Some(batch) => {
                if batch.status == BatchStatus::Undone {
                    return Err(UndoError::AlreadyUndone);
                }
                batch.status = BatchStatus::Undone;
                Ok(batch.clone())
            }
            None => {
                drop(rows);
                if self.foreign_batch(key)? {
                    return Err(UndoError::ScopeMismatch);
                }
                Err(UndoError::NotFound)
            }
        }
    }
}

type PgConn = diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>;

/// Production store bound to the consent gate's process-wide pool, so the
/// undo routes and the consent gate always see the same database.
#[must_use]
pub fn shared_pool_store() -> Option<PgUndoStore> {
    let service = crate::core::bot::consent_gate::shared_service()?;
    Some(PgUndoStore::new(service.pool.clone()))
}

/// Production store: the consent gate's PostgreSQL pool and audit table.
pub struct PgUndoStore {
    pool: botconsent::DbPool,
}

impl PgUndoStore {
    #[must_use]
    pub fn new(pool: botconsent::DbPool) -> Self {
        Self { pool }
    }

    fn conn(&self) -> Result<PgConn, UndoError> {
        match self.pool.get() {
            Ok(conn) => Ok(conn),
            Err(error) => {
                log::error!("undo store connection failed: {error}");
                Err(UndoError::Storage)
            }
        }
    }

    /// Returns the audit row id together with its decoded batch. Only active
    /// rows are returned; a same-id batch in another scope is a typed mismatch.
    fn find_row(&self, key: &BatchKey) -> Result<Option<(Uuid, UndoBatch)>, UndoError> {
        use consent_audit::dsl;
        let mut conn = self.conn()?;
        let rows = dsl::consent_audit
            .filter(dsl::user_id.eq(key.scope.user_id))
            .filter(dsl::outcome.eq(OUTCOME_ACTIVE))
            .order(dsl::created_at.desc())
            .load::<botconsent::models::ConsentAuditRow>(&mut conn)
            .map_err(|error| {
                log::error!("undo batch lookup failed: {error}");
                UndoError::Storage
            })?;
        let mut foreign = false;
        for row in rows {
            let Some(record) = UndoRecord::decode(&row.request) else {
                continue;
            };
            if record.kind != ENVELOPE_KIND || record.batch.key.batch_id != key.batch_id {
                continue;
            }
            if record.batch.key.scope.matches(&key.scope) {
                return Ok(Some((row.id, record.batch)));
            }
            foreign = true;
        }
        if foreign {
            return Err(UndoError::ScopeMismatch);
        }
        Ok(None)
    }
}

impl UndoStore for PgUndoStore {
    fn record(
        &self,
        key: BatchKey,
        operations: Vec<UndoOperation>,
    ) -> Result<UndoBatch, UndoError> {
        if self.find(&key)?.is_some() {
            return Err(UndoError::AlreadyRecorded);
        }
        let batch = UndoBatch::new(key.clone(), operations, Utc::now())?;
        let request = serde_json::to_value(UndoRecord::new(batch.clone())).map_err(|error| {
            log::error!("undo batch serialization failed: {error}");
            UndoError::Storage
        })?;
        let row = botconsent::models::ConsentAuditRow {
            id: Uuid::new_v4(),
            permission_id: None,
            user_id: Some(key.scope.user_id),
            request,
            outcome: OUTCOME_ACTIVE.to_string(),
            decided_by: None,
            created_at: batch.created_at,
        };
        let mut conn = self.conn()?;
        diesel::insert_into(consent_audit::table)
            .values(&row)
            .execute(&mut conn)
            .map_err(|error| {
                log::error!("undo batch insert failed: {error}");
                UndoError::Storage
            })?;
        Ok(batch)
    }

    fn find(&self, key: &BatchKey) -> Result<Option<UndoBatch>, UndoError> {
        Ok(self.find_row(key)?.map(|(_, batch)| batch))
    }

    fn latest(&self, scope: &UndoScope) -> Result<Option<UndoBatch>, UndoError> {
        use consent_audit::dsl;
        scope.validate()?;
        let mut conn = self.conn()?;
        let rows = dsl::consent_audit
            .filter(dsl::user_id.eq(scope.user_id))
            .filter(dsl::outcome.eq(OUTCOME_ACTIVE))
            .order(dsl::created_at.desc())
            .load::<botconsent::models::ConsentAuditRow>(&mut conn)
            .map_err(|error| {
                log::error!("undo batch latest lookup failed: {error}");
                UndoError::Storage
            })?;
        Ok(rows
            .iter()
            .filter_map(|row| UndoRecord::decode(&row.request))
            .filter(|record| record.kind == ENVELOPE_KIND && record.batch.key.scope.matches(scope))
            .map(|record| record.batch)
            .max_by(|left, right| newest(left, right)))
    }

    fn consume(&self, key: &BatchKey) -> Result<UndoBatch, UndoError> {
        use consent_audit::dsl;
        let (row_id, mut batch) = match self.find_row(key)? {
            Some(found) => found,
            None => return Err(UndoError::NotFound),
        };
        let mut conn = self.conn()?;
        // Conditional UPDATE is the single-use guard: only one caller can flip
        // an active row to undone, even if both looked it up concurrently.
        let changed = diesel::update(
            dsl::consent_audit
                .filter(dsl::id.eq(row_id))
                .filter(dsl::outcome.eq(OUTCOME_ACTIVE)),
        )
        .set(dsl::outcome.eq(OUTCOME_UNDONE))
        .execute(&mut conn)
        .map_err(|error| {
            log::error!("undo batch consume failed: {error}");
            UndoError::Storage
        })?;
        if changed == 0 {
            return Err(UndoError::AlreadyUndone);
        }
        batch.status = BatchStatus::Undone;
        Ok(batch)
    }
}
