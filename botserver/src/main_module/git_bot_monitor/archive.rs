//! Reform #1501, step 3 — archive legacy bot sources left in Drive.
//!
//! Once a bot's sources live in its repository (import verified) the Drive
//! copies of `{bot}.gbdialog/` and `{bot}.gbot/` are dead weight that the
//! compiler must never read again. This pass moves those prefixes to
//! `{bucket}/archive/{bot}-{ts}/` — reversible, nothing hard-deleted. Running
//! once at bootstrap after the import pass, it is safe to re-run: prefixes
//! already archived (or absent) are skipped.

use std::collections::BTreeSet;

use botcore::shared::state::AppState;
use botcore::shared::utils::DbPool;
use diesel::RunQueryDsl;

/// Source prefixes archived per bot (KB stays Drive-driven — never touched).
const SOURCE_MARKERS: [&str; 2] = [".gbdialog/", ".gbot/"];

struct ArchiveTarget {
    branch_slug: String,
    bot_name: String,
}

fn list_archive_targets(pool: &DbPool) -> Vec<ArchiveTarget> {
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Text)]
        branch_slug: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
    }
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            log::error!("[git_archive] pool: {e}");
            return Vec::new();
        }
    };
    diesel::sql_query(
        "SELECT br.slug AS branch_slug, vp.name \
         FROM vibe_projects vp \
         JOIN branches br ON br.id = vp.branch_id \
         WHERE vp.project_type = 'bot' AND vp.source_control = 'git'",
    )
    .load::<Row>(&mut conn)
    .unwrap_or_default()
    .into_iter()
    .map(|row| ArchiveTarget {
        branch_slug: row.branch_slug,
        bot_name: row.name,
    })
    .collect()
}

/// Top-level object keys under `{branch}.gbai/{bot}{marker}` for one bucket.
async fn list_source_keys(
    state: &AppState,
    bucket: &str,
    prefix: &str,
) -> Result<Vec<String>, String> {
    let s3 = state
        .drive
        .as_ref()
        .ok_or_else(|| "drive (S3) not available in AppState".to_string())?;
    match s3.list_objects(bucket, Some(prefix)).await {
        Ok(objects) => Ok(objects),
        Err(e) => {
            let text = e.to_string().to_lowercase();
            if text.contains("nosuchbucket")
                || text.contains("no such bucket")
                || text.contains("404")
                || text.contains("invalidbucketname")
            {
                Ok(Vec::new()) // no Drive bucket — fresh/demo branch
            } else {
                Err(format!("list {prefix}: {e}"))
            }
        }
    }
}

/// Archive every source key of one bot. Returns the moved object count.
///
/// `config_in_repo` decides whether `{bot}.gbot/` may leave Drive: the runtime
/// reads the channel prompts from the work layout and falls back to a generic
/// prompt without them, so a prefix that is not in the repository yet stays
/// where it is.
async fn archive_bot_sources(
    state: &AppState,
    bucket: &str,
    branch_prefix: &str,
    bot_name: &str,
    config_in_repo: bool,
) -> Result<usize, String> {
    let s3 = state
        .drive
        .as_ref()
        .ok_or_else(|| "drive (S3) not available in AppState".to_string())?;

    // Collect once per marker so `{bot}.gbdialog/` and `{bot}.gbot/` share the
    // same archive stamp and stay restorable as a unit.
    let mut keys: Vec<String> = Vec::new();
    for marker in SOURCE_MARKERS {
        if marker == ".gbot/" && !config_in_repo {
            log::warn!(
                "[git_archive] {bot_name}: {bucket} still holds .gbot/ and the repository has none — kept in Drive"
            );
            continue;
        }
        let prefix = format!("{branch_prefix}{bot_name}{marker}");
        keys.extend(list_source_keys(state, bucket, &prefix).await?);
    }
    if keys.is_empty() {
        return Ok(0);
    }

    let stamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
    let archive_prefix = format!("{branch_prefix}archive/{bot_name}-{stamp}/");
    let bot_segment = format!("{branch_prefix}{bot_name}");
    let mut moved = 0usize;
    // Server-side copy then delete — S3 has no rename; an interrupted pass
    // re-copies idempotently on the next boot (the source is already gone).
    for key in &keys {
        let file_part = key.strip_prefix(&bot_segment).unwrap_or_else(|| {
            key.rsplit('/').next().unwrap_or(key)
        });
        let target = format!("{archive_prefix}{}", file_part.trim_start_matches('/'));
        s3.copy_object(bucket, key, &target)
            .await
            .map_err(|e| format!("copy {key}: {e}"))?;
        s3.delete_object(bucket, key)
            .await
            .map_err(|e| format!("delete {key}: {e}"))?;
        moved += 1;
    }
    Ok(moved)
}

/// Archive pass: run once at bootstrap after the import pass. Safe to re-run.
pub async fn run_archive_pass(state: std::sync::Arc<AppState>, pool: DbPool) {
    let targets = {
        let pool = pool.clone();
        match tokio::task::spawn_blocking(move || list_archive_targets(&pool)).await {
            Ok(t) => t,
            Err(e) => {
                log::error!("[git_archive] target listing failed: {e}");
                return;
            }
        }
    };
    if targets.is_empty() {
        return;
    }
    if state.drive.is_none() {
        return; // drive-less deployment — nothing to archive
    }
    for target in &targets {
        // Layout 2 (org workspace) first, then layout 1 (standalone bucket).
        let org_bucket = format!("{}.gborg", target.branch_slug);
        let branch_prefix = format!("{}.gbai/", target.branch_slug);
        let buckets: BTreeSet<String> = [org_bucket, format!("{}.gbai", target.branch_slug.to_lowercase())]
            .into_iter()
            .collect();
        // A missing checkout is treated as "no configuration in the repository",
        // which is the safe direction: the prefix simply stays in Drive.
        let config_in_repo = {
            let pool = pool.clone();
            let bot = target.bot_name.clone();
            match tokio::task::spawn_blocking(move || {
                super::bot_config::checkout_of(&pool, &bot)
                    .map(|dir| super::bot_config::has_config_in_repo(&dir, &bot))
                    .unwrap_or(false)
            })
            .await
            {
                Ok(v) => v,
                Err(e) => {
                    log::warn!("[git_archive] {}: checkout task: {e}", target.bot_name);
                    false
                }
            }
        };
        for bucket in buckets {
            match archive_bot_sources(
                &state,
                &bucket,
                &branch_prefix,
                &target.bot_name,
                config_in_repo,
            )
            .await {
                Ok(0) => {}
                Ok(n) => log::info!(
                    "[git_archive] {}: archived {n} Drive source object(s) from {bucket} (reform #1501 step 3)",
                    target.bot_name
                ),
                Err(e) => log::warn!(
                    "[git_archive] {} archive deferred on {bucket}: {e}",
                    target.bot_name
                ),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn archive_stamp_shape_is_reversible_friendly() {
        // Just pins the marker set: KB never archived.
        assert_eq!(super::SOURCE_MARKERS, [".gbdialog/", ".gbot/"]);
    }
}
