//! #2686 — bounded retention for exported Vibe backup tarballs.
//!
//! Without a retention window every `export_snapshot` leaves another tarball
//! (and DB row) behind until the disk fills. On each successful export we now
//! prune records that are past the count cap or older than the age cap. Both
//! limits come from the environment with sane defaults.

use std::path::Path;

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use super::{backup_root, sanitize_for_path, Backups};

/// Defaults when no `VIBE_BACKUP_*` override is set.
pub(crate) const DEFAULT_KEEP_COUNT: usize = 5;
pub(crate) const DEFAULT_MAX_AGE_DAYS: i64 = 30;

fn env_positive_usize(name: &str, default: usize) -> usize {
    match std::env::var(name) {
        Ok(raw) => match raw.trim().parse::<usize>() {
            Ok(n) if n > 0 => n,
            _ => default,
        },
        Err(_) => default,
    }
}

fn env_days(name: &str, default: i64) -> i64 {
    match std::env::var(name) {
        Ok(raw) => match raw.trim().parse::<i64>() {
            Ok(n) if n >= 0 => n,
            _ => default,
        },
        Err(_) => default,
    }
}

pub(crate) fn keep_count() -> usize {
    env_positive_usize("VIBE_BACKUP_KEEP", DEFAULT_KEEP_COUNT)
}

pub(crate) fn max_age_days() -> i64 {
    env_days("VIBE_BACKUP_MAX_AGE_DAYS", DEFAULT_MAX_AGE_DAYS)
}

/// Pure retention selector. `candidates` must be newest-first. A record is
/// pruned when its rank is at/beyond `max_count` OR it is older than
/// `max_age_days` (0 disables the age rule). Returning ids keeps the policy
/// unit-testable without a DB or filesystem.
pub(crate) fn select_prunable(
    candidates: &[(Uuid, DateTime<Utc>)],
    now: DateTime<Utc>,
    max_count: usize,
    max_age_days: i64,
) -> Vec<Uuid> {
    candidates
        .iter()
        .enumerate()
        .filter(|(rank, (_, created))| {
            let beyond_count = *rank >= max_count;
            let too_old = max_age_days > 0 && (now - *created).num_days() > max_age_days;
            beyond_count || too_old
        })
        .map(|(_, (id, _))| *id)
        .collect()
}

impl Backups {
    /// Delete tarballs + rows that fall outside the retention window. Returns
    /// the number of backups removed. Errors are returned (not swallowed) so
    /// the caller can log them without failing the whole export.
    pub fn apply_retention(&self, project_id: Uuid) -> Result<usize, String> {
        let records = self.list(project_id)?;
        let candidates: Vec<(Uuid, DateTime<Utc>)> = records
            .iter()
            .map(|record| (record.id, record.created_at))
            .collect();
        let prunable = select_prunable(&candidates, Utc::now(), keep_count(), max_age_days());
        if prunable.is_empty() {
            return Ok(0);
        }
        let root = Path::new(&backup_root()).join(project_id.to_string());
        let mut conn = self.conn()?;
        let mut removed = 0usize;
        for id in &prunable {
            if let Some(record) = records.iter().find(|record| record.id == *id) {
                let tarball = root.join(format!(
                    "{}-{}.tar.gz",
                    sanitize_for_path(&record.tag),
                    sanitize_for_path(&record.env)
                ));
                match std::fs::remove_file(&tarball) {
                    Ok(()) => {}
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                    Err(e) => {
                        return Err(format!("remove backup tarball {}: {e}", tarball.display()))
                    }
                }
            }
            diesel::sql_query("DELETE FROM vm_backups WHERE id = $1")
                .bind::<diesel::sql_types::Uuid, _>(*id)
                .execute(&mut conn)
                .map_err(|e| format!("delete backup row: {e}"))?;
            removed += 1;
        }
        Ok(removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> DateTime<Utc> {
        "2026-11-23T12:00:00Z".parse().unwrap()
    }

    fn id(n: u8) -> Uuid {
        Uuid::from_u128(n as u128)
    }

    #[test]
    fn keeps_only_latest_n_when_age_rule_disabled() {
        // Four records, 0/1/2/3 days old, newest first; age rule off (0).
        let candidates = vec![
            (id(1), now()),
            (id(2), now() - chrono::Duration::days(1)),
            (id(3), now() - chrono::Duration::days(2)),
            (id(4), now() - chrono::Duration::days(3)),
        ];
        let pruned = select_prunable(&candidates, now(), 2, 0);
        assert_eq!(pruned, vec![id(3), id(4)]);
    }

    #[test]
    fn prunes_records_older_than_age_cap() {
        // Count cap is huge; only the 40-day-old record should go.
        let candidates = vec![
            (id(1), now() - chrono::Duration::days(10)),
            (id(2), now() - chrono::Duration::days(40)),
        ];
        let pruned = select_prunable(&candidates, now(), 100, 30);
        assert_eq!(pruned, vec![id(2)]);
    }

    #[test]
    fn age_rule_prunes_within_count_records() {
        // Rank 1 is still inside the count cap (2) but older than 1 day, so
        // the age rule alone removes it; ranks 2/3 are past the count cap too.
        let candidates = vec![
            (id(1), now()),
            (id(2), now() - chrono::Duration::days(2)),
            (id(3), now() - chrono::Duration::days(3)),
            (id(4), now() - chrono::Duration::days(4)),
        ];
        let pruned = select_prunable(&candidates, now(), 2, 1);
        assert_eq!(pruned, vec![id(2), id(3), id(4)]);
    }

    #[test]
    fn zero_count_prunes_all_and_empty_is_noop() {
        let candidates = vec![(id(1), now()), (id(2), now())];
        assert_eq!(select_prunable(&candidates, now(), 0, 0), vec![id(1), id(2)]);
        assert!(select_prunable(&[], now(), 5, 30).is_empty());
    }

    #[test]
    fn defaults_are_bounded() {
        assert_eq!(DEFAULT_KEEP_COUNT, 5);
        assert_eq!(DEFAULT_MAX_AGE_DAYS, 30);
    }
}
