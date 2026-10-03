//! Sync engine: one change-feed pass per connection, plus the background
//! scheduler that keeps them fresh.
//!
//! A pass never re-lists a whole account. OneDrive replays its `@odata.deltaLink`
//! and Google replays `changes.list` from the stored page token, so a steady-state
//! sync costs a handful of API calls regardless of corpus size. Only the very
//! first pass of a Google connection walks `files.list`.
//!
//! Failure policy is deliberately uniform: whatever goes wrong, the pass stops,
//! the error is classified by `SyncError` (retry vs. reconnect) and the backoff
//! is written to the row so the next scheduler tick skips it.

use crate::external::{config, crypto, db, files, gdrive, onedrive, types::*};
use botcore::shared::utils::DbPool;
use log::{error, info, warn};
use uuid::Uuid;

/// Upper bound on pages in a single pass. Reaching it means the provider is
/// enormous or stuck mid-traversal; the cursor is simply not advanced, so the
/// next pass restarts from a known-good point instead of losing the rest.
const MAX_PAGES: usize = 400;

/// Connections examined per scheduler tick.
const DUE_BATCH: i64 = 20;

/// Shared HTTP client for the provider APIs.
pub fn http_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .user_agent(concat!("GeneralBots/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| format!("external drive http client: {e}"))
}

/// Run one sync pass for a connection.
///
/// Never fails: every outcome, including a missing connection or an unreachable
/// database, is reported inside the `SyncReport` so callers (HTTP handler and
/// scheduler) have a single shape to handle.
pub async fn sync_connection(pool: &DbPool, branch_id: Uuid, provider: Provider) -> SyncReport {
    let mut report = SyncReport::default();

    if !claim(branch_id, provider) {
        // Another pass (scheduler tick or a second click) owns this connection.
        report.error = Some("a sync is already running for this account".into());
        report.error_kind = Some("busy".into());
        return report;
    }
    let outcome = sync_connection_inner(pool, branch_id, provider, report).await;
    release(branch_id, provider);
    outcome
}

async fn sync_connection_inner(
    pool: &DbPool,
    branch_id: Uuid,
    provider: Provider,
    mut report: SyncReport,
) -> SyncReport {

    let loaded = {
        match db::conn(pool) {
            Ok(mut conn) => db::get_connection(&mut conn, branch_id, provider),
            Err(e) => return report.failed(&SyncError::Transient(e)),
        }
    };
    let connection = match loaded {
        Ok(Some(c)) => c,
        Ok(None) => {
            return report.failed(&SyncError::Permanent(format!(
                "{} is not connected",
                provider.display_name()
            )))
        }
        Err(e) => return report.failed(&SyncError::Transient(e)),
    };

    let http = match http_client() {
        Ok(c) => c,
        Err(e) => return report.failed(&SyncError::Transient(e)),
    };

    let token = match connection_access_token(&http, pool, branch_id, &connection).await {
        Ok(t) => t,
        Err(e) => return record_failure(pool, branch_id, provider, &connection, report, &e),
    };

    let outcome = match provider {
        Provider::OneDrive => {
            run_onedrive(&http, pool, branch_id, &token, connection.cursor.as_deref(), &mut report).await
        }
        Provider::GoogleDrive => {
            run_gdrive(&http, pool, branch_id, &token, connection.cursor.as_deref(), &mut report).await
        }
    };

    match outcome {
        // `None` means the traversal did not close (page cap); the stored
        // cursor is left untouched so nothing is silently skipped.
        Ok(Some(cursor)) => {
            let stored = db::conn(pool)
                .and_then(|mut conn| db::store_cursor(&mut conn, branch_id, provider, Some(&cursor)));
            match stored {
                Ok(()) => info!(
                    "external drive {}: {} files, {} removed ({} pages) for branch {}",
                    provider.as_str(), report.upserted, report.removed, report.pages, branch_id
                ),
                Err(e) => error!(
                    "external drive {}: pass succeeded but the cursor was not stored: {e}",
                    provider.as_str()
                ),
            }
            report
        }
        Ok(None) => {
            warn!(
                "external drive {}: pass hit the {MAX_PAGES}-page cap, cursor unchanged",
                provider.as_str()
            );
            report
        }
        Err(e) => record_failure(pool, branch_id, provider, &connection, report, &e),
    }
}

/// Replays the Graph delta feed: pages until `@odata.nextLink` stops appearing,
/// then the closing `@odata.deltaLink` becomes the stored cursor.
async fn run_onedrive(
    http: &reqwest::Client,
    pool: &DbPool,
    branch_id: Uuid,
    token: &str,
    cursor: Option<&str>,
    report: &mut SyncReport,
) -> Result<Option<String>, SyncError> {
    let mut next = cursor.map(str::to_string);
    let mut delta = None;
    for _ in 0..MAX_PAGES {
        let page = onedrive::fetch_page(http, token, next.as_deref()).await?;
        report.pages += 1;
        persist(pool, branch_id, Provider::OneDrive, &page, report)?;
        delta = page.delta.clone().or(delta);
        match page.next_cursor {
            Some(cursor) => next = Some(cursor),
            None => break,
        }
    }
    Ok(delta)
}

/// Google needs an explicit two-phase flow: the first pass walks `files.list`
/// and then captures a start page token; later passes replay `changes.list`.
async fn run_gdrive(
    http: &reqwest::Client,
    pool: &DbPool,
    branch_id: Uuid,
    token: &str,
    cursor: Option<&str>,
    report: &mut SyncReport,
) -> Result<Option<String>, SyncError> {
    match cursor {
        None => {
            let mut page_token: Option<String> = None;
            for _ in 0..MAX_PAGES {
                let page = gdrive::fetch_list_page(http, token, page_token.as_deref()).await?;
                report.pages += 1;
                persist(pool, branch_id, Provider::GoogleDrive, &page, report)?;
                match page.next_cursor {
                    Some(next) => page_token = Some(next),
                    None => break,
                }
            }
            // Taken after the listing so the token covers everything mirrored
            // above — a token taken first would re-deliver the whole account.
            Ok(Some(gdrive::start_page_token(http, token).await?))
        }
        Some(start) => {
            let mut page_token = Some(start.to_string());
            let mut new_start = None;
            for _ in 0..MAX_PAGES {
                let page = gdrive::fetch_change_page(http, token, page_token.as_deref()).await?;
                report.pages += 1;
                persist(pool, branch_id, Provider::GoogleDrive, &page, report)?;
                new_start = page.delta.clone().or(new_start);
                match page.next_cursor {
                    Some(next) => page_token = Some(next),
                    None => break,
                }
            }
            Ok(new_start)
        }
    }
}

/// Write one page's changes. Items first, then removals: a delta that both
/// renames and deletes must not resurrect the deleted rows.
fn persist(
    pool: &DbPool,
    branch_id: Uuid,
    provider: Provider,
    page: &DeltaPage,
    report: &mut SyncReport,
) -> Result<(), SyncError> {
    if page.items.is_empty() && page.removed.is_empty() {
        return Ok(());
    }
    let mut conn = db::conn(pool).map_err(SyncError::Transient)?;
    for file in &page.items {
        files::upsert_file(&mut conn, branch_id, provider, file).map_err(SyncError::Transient)?;
    }
    report.upserted += page.items.len();
    let removed = files::remove_files(&mut conn, branch_id, provider, &page.removed)
        .map_err(SyncError::Transient)?;
    report.removed += removed as usize;
    Ok(())
}

/// Return a usable access token, refreshing and persisting one when needed.
///
/// Public because the download route needs the same treatment: serving a file
/// with a stale token produces a 401 the user cannot act on.
pub async fn connection_access_token(
    http: &reqwest::Client,
    pool: &DbPool,
    branch_id: Uuid,
    connection: &Connection,
) -> Result<String, SyncError> {
    if !connection.access_token.is_empty() && !connection.token_expired(chrono::Utc::now()) {
        return Ok(connection.access_token.clone());
    }
    let refresh = connection
        .refresh_token
        .as_deref()
        .filter(|t| !t.is_empty())
        .ok_or_else(|| {
            SyncError::Auth(format!(
                "{} token expired and no refresh token is stored; reconnect the account",
                connection.provider.display_name()
            ))
        })?;
    let client = config::client_for(connection.provider).ok_or_else(|| {
        SyncError::Permanent(format!(
            "no OAuth client configured for {}",
            connection.provider.display_name()
        ))
    })?;
    // Each provider has its own token response type; the arms are flattened into
    // the three fields that actually get stored.
    let (access, refreshed, expires_at) = match connection.provider {
        Provider::OneDrive => {
            let t = onedrive::refresh_token(
                http,
                &client.tenant_id,
                &client.client_id,
                &client.client_secret,
                refresh,
            )
            .await?;
            (t.access_token, t.refresh_token, t.expires_at)
        }
        Provider::GoogleDrive => {
            let t = gdrive::refresh_token(http, &client.client_id, &client.client_secret, refresh)
                .await?;
            (t.access_token, t.refresh_token, t.expires_at)
        }
    };

    let access_enc = crypto::encrypt_token(&access).map_err(SyncError::Transient)?;
    let refresh_enc = refreshed
        .as_deref()
        .map(crypto::encrypt_token)
        .transpose()
        .map_err(SyncError::Transient)?;
    let mut conn = db::conn(pool).map_err(SyncError::Transient)?;
    db::store_refreshed_token(
        &mut conn,
        branch_id,
        connection.provider,
        &access_enc,
        refresh_enc.as_deref(),
        expires_at,
    )
    .map_err(SyncError::Transient)?;
    info!(
        "external drive {}: access token refreshed for branch {}",
        connection.provider.as_str(),
        branch_id
    );
    Ok(access)
}

/// Persist the failure and return the report, so callers never see a bare error.
fn record_failure(
    pool: &DbPool,
    branch_id: Uuid,
    provider: Provider,
    connection: &Connection,
    report: SyncReport,
    err: &SyncError,
) -> SyncReport {
    let backoff = err.backoff_secs(connection.attempt_count.clamp(0, 8) as u32);
    match db::conn(pool)
        .and_then(|mut conn| {
            db::store_failure(
                &mut conn,
                branch_id,
                provider,
                &err.to_string(),
                err.needs_reauth(),
                backoff,
            )
        }) {
        Ok(()) => warn!(
            "external drive {}: sync failed for branch {} ({}), retrying in {}s",
            provider.as_str(),
            branch_id,
            err,
            backoff
        ),
        Err(db_err) => error!(
            "external drive {}: sync failed ({err}) and the failure could not be stored: {db_err}",
            provider.as_str()
        ),
    }
    report.failed(err)
}

/// Connections with a pass in flight. One shared set (not one per function —
/// two statics would never see each other), guarded so a manual sync and a
/// scheduler tick cannot double the provider calls or interleave cursor writes.
static ACTIVE_PASSES: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

/// Try to take the pass for a connection; `false` means one is already running.
fn claim(branch_id: Uuid, provider: Provider) -> bool {
    let key = format!("{}:{}", branch_id, provider.as_str());
    let mut active = match ACTIVE_PASSES.lock() {
        Ok(guard) => guard,
        // A poisoned lock must not stop syncs; take the inner value.
        Err(poisoned) => poisoned.into_inner(),
    };
    if active.contains(&key) {
        return false;
    }
    active.push(key);
    true
}

fn release(branch_id: Uuid, provider: Provider) {
    let key = format!("{}:{}", branch_id, provider.as_str());
    if let Ok(mut active) = ACTIVE_PASSES.lock() {
        active.retain(|k| k != &key);
    }
}

/// Background scheduler: wakes up, syncs whatever is due, sleeps again.
///
/// Spawned once at boot. Failures inside `sync_connection` are already
/// recorded on the connection row, so a provider outage degrades to "the
/// External tab shows the last error" instead of noisy logs every tick.
pub fn spawn_scheduler(pool: DbPool) {
    let interval_secs = std::env::var("EXTERNAL_DRIVE_SYNC_INTERVAL")
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
        .filter(|v| (30..=3600).contains(v))
        .unwrap_or(300);
    let start_delay_secs = std::env::var("EXTERNAL_DRIVE_SYNC_STARTUP_DELAY")
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
        .unwrap_or(30);

    tokio::spawn(async move {
        info!("external drive scheduler started (every {interval_secs}s)");
        tokio::time::sleep(std::time::Duration::from_secs(start_delay_secs)).await;
        loop {
            let due = match db::conn(&pool)
                .and_then(|mut conn| db::list_due_connections(&mut conn, chrono::Utc::now(), DUE_BATCH))
            {
                Ok(rows) => rows,
                Err(e) => {
                    warn!("external drive scheduler: could not read due connections: {e}");
                    Vec::new()
                }
            };
            for (branch_id, provider) in due {
                let report = sync_connection(&pool, branch_id, provider).await;
                if !report.ok() {
                    debug_pass(&report, branch_id, provider);
                }
            }
            tokio::time::sleep(std::time::Duration::from_secs(interval_secs)).await;
        }
    });
}

fn debug_pass(report: &SyncReport, branch_id: Uuid, provider: Provider) {
    if let Some(err) = report.error.as_deref() {
        warn!(
            "external drive {}: scheduler pass for branch {} ended with {} ({})",
            provider.as_str(),
            branch_id,
            err,
            report.error_kind.as_deref().unwrap_or("unknown")
        );
    }
}