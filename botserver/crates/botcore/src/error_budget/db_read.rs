//! Typed outcome for DB reads under the issue #2543 error-budget ratchet.
//!
//! The old pattern `.load(..).unwrap_or_default()` silently turned a failed
//! query into an empty `Vec`, so a broken DB was indistinguishable from an
//! empty table. The replacement is a `match` that logs the error and returns
//! a typed empty result. This module is the pure, DB-independent core of that
//! replacement: it maps a `Result<Vec<T>, E>` into three variants a caller can
//! match on without ever conflating `Failed` with `Empty`.
//!
//! No I/O, no DB client, no global state: the mapping is fully deterministic,
//! so the failure-vs-empty distinction is provable in a unit test. Logging and
//! the typed empty fallback belong at the real call site, which matches on the
//! outcome (e.g. `log::error!(..); DbRead::Failed` in the `Err` arm).

/// Outcome of a typed DB read that preserves the difference between a failed
/// query and a legitimately empty result set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DbRead<T> {
    /// The query succeeded and returned at least one row.
    Rows(Vec<T>),
    /// The query succeeded but returned no rows (empty table / no match).
    Empty,
    /// The query failed. Never collapse this into [`DbRead::Empty`].
    Failed,
}

impl<T> DbRead<T> {
    /// Map a fallible DB read into a typed outcome.
    ///
    /// - `Err(_)`   -> [`DbRead::Failed`]
    /// - `Ok([])`   -> [`DbRead::Empty`]
    /// - `Ok(rows)` -> [`DbRead::Rows`]
    ///
    /// Pure by design: no DB dependency and no logging here, so callers can
    /// attach `log::error!` at their own call site and tests stay deterministic.
    pub fn from_result<E>(result: Result<Vec<T>, E>) -> Self {
        match result {
            Ok(rows) if rows.is_empty() => DbRead::Empty,
            Ok(rows) => DbRead::Rows(rows),
            Err(_) => DbRead::Failed,
        }
    }

    /// Whether the read failed (as opposed to being merely empty).
    pub fn is_failed(&self) -> bool {
        matches!(self, DbRead::Failed)
    }
}

#[cfg(test)]
mod tests {
    use super::DbRead;

    #[test]
    fn error_is_failed_and_distinct_from_empty() {
        let failed = DbRead::<u8>::from_result(Err("connection refused"));
        assert_eq!(failed, DbRead::Failed);
        let empty = DbRead::<u8>::from_result::<&str>(Ok(Vec::new()));
        assert_eq!(empty, DbRead::Empty);
        assert!(
            failed != empty,
            "a DB failure must be distinguishable from an empty table"
        );
        assert!(failed.is_failed());
        assert!(!empty.is_failed());
    }

    #[test]
    fn ok_empty_is_empty() {
        let outcome = DbRead::<String>::from_result::<&str>(Ok(Vec::new()));
        assert_eq!(outcome, DbRead::Empty);
        assert!(!outcome.is_failed());
    }

    #[test]
    fn ok_rows_are_returned() {
        let outcome = DbRead::<u32>::from_result::<&str>(Ok(vec![7, 11]));
        assert_eq!(outcome, DbRead::Rows(vec![7, 11]));
        assert!(!outcome.is_failed());
    }
}
