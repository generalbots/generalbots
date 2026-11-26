//! Deterministic, database-free core for idempotent agent-batch restore (#2448).
//!
//! A batch can hold several operations and restore is append-only: every
//! successful restore creates a new version. If a later operation fails (e.g.
//! a missing collab grant) the batch stays active for a retry; without
//! idempotency the retry would restore earlier operations a second time and
//! leave duplicate restored versions. [`apply_restore_plan`] asks the sink
//! whether a recorded `before.version_id` was already restored by this actor
//! *before* restoring it, so a retry only performs the operations still
//! outstanding.

use std::collections::HashSet;
use std::future::Future;

use serde_json::Value;
use uuid::Uuid;

use super::RevertOperation;

/// A restore target is read only from the recorded before-state. The
/// after-state is diagnostic and can never select a version to restore.
#[must_use]
pub fn before_version_id(before: &Value) -> Option<Uuid> {
    before
        .get("version_id")
        .and_then(Value::as_str)
        .and_then(|value| Uuid::parse_str(value).ok())
}

/// Version-history probe for idempotent restore (#2448). `restore_version`
/// appends a row with the source row's `content_hash` under the verified actor;
/// a newer such row means this before-state is already current, so a retry must
/// not append a duplicate. Uses the existing `collab_versions` table — no new
/// schema and no caller-supplied scope.
pub fn history_has_newer_restore(
    conn: &mut diesel::PgConnection,
    actor_id: &str,
    version_id: Uuid,
) -> Result<bool, diesel::result::Error> {
    use diesel::sql_types::{BigInt, Text, Uuid as SqlUuid};
    use diesel::{QueryableByName, RunQueryDsl};

    #[derive(QueryableByName)]
    struct CountRow {
        #[diesel(sql_type = BigInt)]
        count: i64,
    }
    diesel::sql_query(
        "SELECT COUNT(*)::bigint AS count FROM collab_versions AS newer \
         JOIN collab_versions AS src ON src.id = $1 \
         WHERE newer.resource_type = src.resource_type \
           AND newer.resource_id = src.resource_id \
           AND newer.actor_id = $2 \
           AND newer.content_hash = src.content_hash \
           AND newer.created_at > src.created_at",
    )
    .bind::<SqlUuid, _>(version_id)
    .bind::<Text, _>(actor_id)
    .get_result::<CountRow>(conn)
    .map(|row| row.count > 0)
}

/// The two capabilities the restore loop needs: a version-history probe and
/// the (async) restore itself. Keeping them behind a trait makes the whole loop
/// testable without a server, database or network.
pub trait RestoreSink<E> {
    /// Whether this actor already created a newer restored version for the
    /// given source version. Implemented by a version-history query in the
    /// production route and by an in-memory set in deterministic tests.
    fn was_restored(&self, version_id: Uuid) -> bool;

    /// Restores the source version and returns the id of the new version.
    fn restore(&self, version_id: Uuid) -> impl Future<Output = Result<String, E>> + '_;
}

/// Applies a revert plan at most once per recorded before version.
///
/// Operations without a parseable `before.version_id` are ignored. The first
/// error is returned unchanged so the caller keeps the batch active for a
/// retry; already restored operations are skipped, never re-applied.
pub async fn apply_restore_plan<E, S: RestoreSink<E>>(
    sink: &S,
    plan: &[RevertOperation],
) -> Result<Vec<String>, E> {
    let mut restored = Vec::new();
    let mut seen = HashSet::new();
    for operation in plan {
        let Some(version_id) = before_version_id(&operation.before) else {
            continue;
        };
        if !seen.insert(version_id) || sink.was_restored(version_id) {
            continue;
        }
        restored.push(sink.restore(version_id).await?);
    }
    Ok(restored)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::cell::RefCell;

    fn operation(version_id: &str) -> RevertOperation {
        RevertOperation {
            op_id: version_id.to_string(),
            app_id: "docs".to_string(),
            target: "doc-42".to_string(),
            before: json!({ "version_id": version_id }),
        }
    }

    /// Models the append-only version history and the collab grant for one
    /// blocked operation. Every successful restore records its source version
    /// in `history`, exactly as a newer restored row in `collab_versions` does.
    struct MockRestore {
        blocked: Uuid,
        history: RefCell<HashSet<Uuid>>,
        granted: RefCell<bool>,
        calls: RefCell<Vec<Uuid>>,
    }

    impl MockRestore {
        fn new(blocked: Uuid) -> Self {
            Self {
                blocked,
                history: RefCell::new(HashSet::new()),
                granted: RefCell::new(false),
                calls: RefCell::new(Vec::new()),
            }
        }

        async fn run(&self, plan: &[RevertOperation]) -> Result<Vec<String>, String> {
            apply_restore_plan(self, plan).await
        }
    }

    impl RestoreSink<String> for MockRestore {
        fn was_restored(&self, version_id: Uuid) -> bool {
            self.history.borrow().contains(&version_id)
        }

        fn restore(&self, version_id: Uuid) -> impl Future<Output = Result<String, String>> + '_ {
            async move {
                if version_id == self.blocked && !*self.granted.borrow() {
                    return Err("no collab grant".to_string());
                }
                self.calls.borrow_mut().push(version_id);
                self.history.borrow_mut().insert(version_id);
                Ok(format!("restored-{version_id}"))
            }
        }
    }

    #[tokio::test]
    async fn retry_skips_an_already_restored_operation_and_adds_only_the_new_one() {
        let first = Uuid::from_u128(1);
        let second = Uuid::from_u128(2);
        let plan = vec![
            operation(&first.to_string()),
            operation(&second.to_string()),
        ];
        let mock = MockRestore::new(second);

        // Attempt 1: the first operation restores, the second has no grant.
        assert!(mock.run(&plan).await.is_err());
        assert_eq!(mock.calls.borrow().as_slice(), &[first]);
        assert_eq!(*mock.history.borrow(), HashSet::from([first]));

        // Retry with the grant still missing: the first must not be restored
        // again; the second still fails, so the history is unchanged.
        assert!(mock.run(&plan).await.is_err());
        assert_eq!(
            mock.calls.borrow().as_slice(),
            &[first],
            "a retry must not duplicate the first restored version"
        );
        assert_eq!(*mock.history.borrow(), HashSet::from([first]));

        // Grant fixed: the first stays skipped, only the second is newly
        // restored in this attempt. Across attempts each version was restored
        // exactly once, and the two restored versions are distinct.
        mock.granted.replace(true);
        assert_eq!(
            mock.run(&plan).await,
            Ok(vec![format!("restored-{second}")])
        );
        assert_eq!(mock.calls.borrow().as_slice(), &[first, second]);
        assert_eq!(*mock.history.borrow(), HashSet::from([first, second]));
        assert_ne!(first, second);

        // A further invocation (double undo) finds nothing left to restore.
        assert_eq!(mock.run(&plan).await, Ok(Vec::new()));
        assert_eq!(mock.calls.borrow().as_slice(), &[first, second]);
    }

    #[tokio::test]
    async fn operations_without_a_version_id_are_ignored() {
        let version = Uuid::from_u128(7);
        let no_version = RevertOperation {
            op_id: "diagnostic".to_string(),
            app_id: "docs".to_string(),
            target: "doc-42".to_string(),
            before: json!({ "title": "before" }),
        };
        let plan = vec![no_version, operation(&version.to_string())];
        let mock = MockRestore::new(version);
        mock.granted.replace(true);

        let restored = match mock.run(&plan).await {
            Ok(ids) => ids,
            Err(_) => Vec::new(),
        };
        assert_eq!(restored, vec![format!("restored-{version}")]);
        assert_eq!(mock.calls.borrow().as_slice(), &[version]);
    }

    #[test]
    fn before_version_id_reads_only_the_before_state() {
        let version = Uuid::from_u128(9);
        assert_eq!(
            before_version_id(&json!({ "version_id": version.to_string() })),
            Some(version)
        );
        assert_eq!(before_version_id(&json!({ "version_id": 1 })), None);
        assert_eq!(before_version_id(&json!({ "name": "x" })), None);
    }
}
