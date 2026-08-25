//! Import pipeline for `user_memories`: shared dry-run projection and the
//! live write path, both driven by the dedupe contract in [`crate::store`].

use chrono::Utc;
use diesel::PgConnection;
use uuid::Uuid;

use crate::models::{
    ensure_content, ensure_kind, ensure_scope, ensure_source, ImportReport, MemoryItem,
    SCOPE_BRANCH, SCOPE_PRIVATE,
};
use crate::store::{
    candidate_of_row, decide_dedupe, do_insert, fetch_candidates, link_supersede,
    normalize_content, DedupeCandidate, DedupeDecision, NewMemory,
};

const SOURCE_MANUAL: &str = "manual";
const MAX_IMPORT_ITEMS: usize = 500;

fn static_scope(value: &str) -> &'static str {
    match value {
        SCOPE_BRANCH => SCOPE_BRANCH,
        _ => SCOPE_PRIVATE,
    }
}

/// Validated, owned form of one import item; borrowed into [`NewMemory`] for
/// the write phase.
struct CleanedItem {
    scope: &'static str,
    kind: String,
    content: String,
    source: String,
    confidence: f32,
    pinned: bool,
}

/// Projects import counters without touching the database. Dry-run engine
/// shared by [`import`]; later items observe earlier simulated inserts.
pub fn plan_import(mut candidates: Vec<DedupeCandidate>, items: &[MemoryItem]) -> ImportReport {
    let mut report = ImportReport::default();
    for item in items {
        let raw_lowercase = item.content.trim().to_lowercase();
        let normalized = normalize_content(&item.content);
        match decide_dedupe(&candidates, &normalized, &raw_lowercase, item.confidence) {
            DedupeDecision::Fresh => {
                report.would_create += 1;
                candidates.push(DedupeCandidate::build(Uuid::nil(), &item.content, item.confidence, Utc::now()));
            }
            DedupeDecision::Supersede { candidate } => {
                report.superseded += 1;
                candidates.retain(|existing| existing.id != candidate);
                candidates.push(DedupeCandidate::build(Uuid::nil(), &item.content, item.confidence, Utc::now()));
            }
            _ => report.skipped += 1,
        }
    }
    report
}

/// Imports items under the dedupe contract. With `dry_run` no row is written
/// and the report carries projections only; otherwise performed work is
/// counted in `created`/`superseded`/`skipped`.
pub fn import(
    conn: &mut PgConnection,
    owner: Uuid,
    org_id: Option<Uuid>,
    branch: Option<Uuid>,
    items: &[MemoryItem],
    dry_run: bool,
) -> Result<ImportReport, String> {
    if items.len() > MAX_IMPORT_ITEMS {
        return Err(format!("import limited to {MAX_IMPORT_ITEMS} items per call"));
    }

    let mut cleaned: Vec<CleanedItem> = Vec::with_capacity(items.len());
    for item in items {
        cleaned.push(CleanedItem {
            scope: static_scope(&ensure_scope(&item.scope)?),
            kind: ensure_kind(&item.kind)?,
            content: ensure_content(&item.content)?,
            source: match item.source.as_deref() {
                Some(raw) => ensure_source(raw)?,
                None => SOURCE_MANUAL.to_string(),
            },
            confidence: item.confidence,
            pinned: item.pinned,
        });
    }

    let mut candidates = fetch_candidates(conn, owner, branch)?;
    if dry_run {
        return Ok(plan_import(candidates, items));
    }

    let mut report = ImportReport::default();
    for entry in cleaned {
        let spec = NewMemory {
            org_id,
            branch_id: branch,
            owner_user_id: owner,
            scope: entry.scope,
            kind: &entry.kind,
            content: &entry.content,
            source: &entry.source,
            confidence: entry.confidence,
            pinned: entry.pinned,
        };
        let raw_lowercase = spec.content.trim().to_lowercase();
        let normalized = normalize_content(spec.content);
        match decide_dedupe(&candidates, &normalized, &raw_lowercase, spec.confidence) {
            DedupeDecision::Fresh => {
                let memory = do_insert(conn, &spec)?;
                candidates.push(candidate_of_row(&memory));
                report.created += 1;
            }
            DedupeDecision::Supersede { candidate } => {
                let memory = do_insert(conn, &spec)?;
                link_supersede(conn, candidate, memory.id)?;
                candidates.retain(|existing| existing.id != candidate);
                candidates.push(candidate_of_row(&memory));
                report.superseded += 1;
            }
            _ => report.skipped += 1,
        }
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(content: &str, confidence: f32) -> DedupeCandidate {
        DedupeCandidate::build(Uuid::new_v4(), content, confidence, Utc::now())
    }

    fn item(content: &str, confidence: f32) -> MemoryItem {
        MemoryItem {
            kind: "fact".to_string(),
            content: content.to_string(),
            scope: SCOPE_PRIVATE.to_string(),
            pinned: false,
            confidence,
            source: None,
        }
    }

    #[test]
    fn dry_run_report_counts() {
        let existing = vec![candidate("Allergic to peanuts", 0.9)];
        let items = vec![
            item("Speaks Portuguese natively", 0.7),
            item("ALLERGIC TO PEANUTS", 0.4),
            item("allergic to peanuts", 0.95),
        ];
        let report = plan_import(existing, &items);
        assert_eq!(report.would_create, 1);
        assert_eq!(report.superseded, 1);
        assert_eq!(report.skipped, 1);
        assert_eq!(report.created, 0);
    }
}
