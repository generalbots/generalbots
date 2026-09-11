//! Background IMAP sync worker for the unified inbox.
//!
//! The worker polls every active `user_email_accounts` row on a fixed
//! interval, connects to each mailbox over IMAPS, fetches INBOX messages
//! newer than the last synced UID and writes them into `email_messages`.
//! Inserts are UID-deduped per account so a repeated pass is idempotent even
//! if the mailbox changes between passes.

use diesel::prelude::*;
use diesel::sql_types::{
    BigInt, Bool, Integer, Jsonb, Nullable, Text, Timestamptz, Uuid as SqlUuid,
};
use log::{info, warn};
use mailparse::{dateparse, parse_mail, MailHeaderMap};
use std::collections::HashSet;
use std::time::Duration;
use uuid::Uuid;

use crate::models::DbPool;

/// Delay between full sync passes over all active accounts.
const SYNC_INTERVAL_SECS: u64 = 300;

/// Registers the background poller with the tokio runtime. The spawned task
/// keeps running for the lifetime of the process and never blocks request
/// handling.
pub fn spawn_imap_sync_worker(pool: DbPool) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(SYNC_INTERVAL_SECS));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            interval.tick().await;
            if let Err(e) = sync_all_accounts(&pool).await {
                warn!("IMAP sync pass failed: {e}");
            }
        }
    });
    info!("IMAP sync worker started (interval {SYNC_INTERVAL_SECS}s)");
}

#[derive(Debug, QueryableByName)]
struct SyncAccount {
    #[diesel(sql_type = SqlUuid)]
    id: Uuid,
    #[diesel(sql_type = Text)]
    imap_server: String,
    #[diesel(sql_type = Integer)]
    imap_port: i32,
    #[diesel(sql_type = Text)]
    username: String,
}

#[derive(Debug, QueryableByName)]
struct UidRow {
    #[diesel(sql_type = BigInt)]
    uid: i64,
}

struct StoredMessage {
    account_id: Uuid,
    uid: i64,
    message_id: Option<String>,
    in_reply_to: Option<String>,
    subject: String,
    normalized_subject: String,
    from_address: String,
    to_addresses: Option<String>,
    body_text: Option<String>,
    body_html: Option<String>,
    has_attachments: bool,
    is_read: bool,
    is_flagged: bool,
    flags: serde_json::Value,
    received_at: chrono::DateTime<chrono::Utc>,
}

async fn sync_all_accounts(pool: &DbPool) -> Result<(), String> {
    let closure_pool = pool.clone();
    let accounts = tokio::task::spawn_blocking(move || {
        let mut conn = closure_pool.get().map_err(|e| format!("DB pool error: {e}"))?;
        diesel::sql_query(
            // The stored credential is not read here: the worker authenticates
            // through `imap_auth::resolve_imap_auth`, which resolves a password
            // or a bearer token and refreshes the latter when it has expired.
            "SELECT id, imap_server, imap_port, username \
             FROM user_email_accounts WHERE is_active = true",
        )
        .load::<SyncAccount>(&mut conn)
        .map_err(|e| format!("Failed to load email accounts: {e}"))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    for account in &accounts {
        match sync_account(pool.clone(), account).await {
            Ok(()) => record_sync_outcome(pool, account.id, None).await,
            Err(e) => {
                warn!("IMAP sync for account {} failed: {e}", account.id);
                record_sync_outcome(pool, account.id, Some(&e)).await;
            }
        }
    }
    Ok(())
}

/// Records the outcome of a sync pass on the account row.
///
/// Without this the Mail application cannot tell an unreachable mailbox from an
/// empty one: the failure existed only as a warning in the server log.
async fn record_sync_outcome(pool: &DbPool, account_id: Uuid, error: Option<&str>) {
    let pool = pool.clone();
    let error = error.map(str::to_string);

    let handle = tokio::task::spawn_blocking(move || -> Result<(), String> {
        let mut conn = pool.get().map_err(|e| format!("DB pool error: {e}"))?;
        let outcome = match error {
            Some(message) => diesel::sql_query(
                "UPDATE user_email_accounts SET last_error = $1, last_error_at = now() \
                 WHERE id = $2",
            )
            .bind::<Text, _>(message)
            .bind::<SqlUuid, _>(account_id)
            .execute(&mut conn),
            None => diesel::sql_query(
                "UPDATE user_email_accounts SET last_sync_at = now(), last_error = NULL, \
                 last_error_at = NULL WHERE id = $1",
            )
            .bind::<SqlUuid, _>(account_id)
            .execute(&mut conn),
        };
        outcome
            .map(|_| ())
            .map_err(|e| format!("Failed to record the sync outcome: {e}"))
    })
    .await;

    match handle {
        Ok(Ok(())) => {}
        Ok(Err(e)) => warn!("Sync outcome for account {account_id} was not recorded: {e}"),
        Err(e) => warn!("Sync outcome task for account {account_id} failed: {e}"),
    }
}

async fn sync_account(pool: DbPool, account: &SyncAccount) -> Result<(), String> {
    let owned = SyncAccount {
        id: account.id,
        imap_server: account.imap_server.clone(),
        imap_port: account.imap_port,
        username: account.username.clone(),
    };
    let inserted = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| format!("DB pool error: {e}"))?;
        fetch_and_store(&mut conn, &owned)
    })
    .await
    .map_err(|e| format!("IMAP task join error: {e}"))??;

    if inserted > 0 {
        info!("IMAP sync for account {} inserted {inserted} messages", account.id);
    }
    Ok(())
}

fn fetch_and_store(conn: &mut diesel::PgConnection, account: &SyncAccount) -> Result<usize, String> {
    // Password and OAuth2 accounts are resolved in one place so the sync worker
    // and the send path always authenticate the same way.
    let auth = crate::imap_auth::resolve_imap_auth(conn, account.id)?;

    let mut session = crate::imap_auth::open_session(
        account.imap_server.as_str(),
        account.imap_port as u16,
        &auth,
    )?;
    session
        .select("INBOX")
        .map_err(|e| format!("IMAP select INBOX failed: {e:?}"))?;

    let existing = existing_uids(conn, account.id)?;
    let next_uid = existing.iter().copied().max().unwrap_or(0) + 1;
    let range = if next_uid == 1 {
        "1:*".to_string()
    } else {
        format!("{next_uid}:*")
    };
    let fetches = session
        .uid_fetch(&range, "(RFC822 FLAGS)")
        .map_err(|e| format!("IMAP UID FETCH failed: {e:?}"))?;

    let now = chrono::Utc::now();
    let mut inserted = 0;
    for fetch in fetches.iter() {
        let uid = match fetch.uid {
            Some(uid) => uid as i64,
            None => continue,
        };
        if existing.contains(&uid) {
            continue;
        }
        let raw = match fetch.body() {
            Some(body) => body,
            None => continue,
        };
        let parsed = match parse_mail(raw) {
            Ok(mail) => mail,
            Err(_) => continue,
        };
        let headers = parsed.get_headers();
        let subject = headers.get_first_value("Subject").unwrap_or_default();
        let from_address = headers.get_first_value("From").unwrap_or_default();
        let to_addresses = non_empty(&headers.get_first_value("To").unwrap_or_default());
        let date_str = headers.get_first_value("Date").unwrap_or_default();
        let received_at = dateparse(&date_str)
            .ok()
            .and_then(|secs| chrono::DateTime::<chrono::Utc>::from_timestamp(secs, 0))
            .unwrap_or(now);
        let is_read = fetch.flags().iter().any(|f| matches!(f, imap::types::Flag::Seen));
        let is_flagged = fetch
            .flags()
            .iter()
            .any(|f| matches!(f, imap::types::Flag::Flagged));
        let flags: serde_json::Value = fetch
            .flags()
            .iter()
            .map(|f| serde_json::Value::String(format!("{f:?}")))
            .collect();
        let has_attachments = parsed.subparts.iter().any(|p| {
            p.get_content_disposition().disposition == mailparse::DispositionType::Attachment
        });

        // Extract the HTML alternative when present (multipart/alternative or
        // a standalone text/html body) so the mail app can render rich content.
        let body_html = parsed
            .subparts
            .iter()
            .find(|p| p.ctype.mimetype.eq_ignore_ascii_case("text/html"))
            .and_then(|p| p.get_body().ok())
            .filter(|b| !b.is_empty())
            .or_else(|| {
                if parsed.ctype.mimetype.eq_ignore_ascii_case("text/html") {
                    parsed.get_body().ok()
                } else {
                    None
                }
            });

        let message = StoredMessage {
            account_id: account.id,
            uid,
            message_id: non_empty(&headers.get_first_value("Message-ID").unwrap_or_default()),
            in_reply_to: non_empty(&headers.get_first_value("In-Reply-To").unwrap_or_default()),
            subject: subject.clone(),
            normalized_subject: crate::unified_inbox::normalize_subject(&subject),
            from_address,
            to_addresses,
            body_text: parsed.get_body().ok(),
            body_html,
            has_attachments,
            is_read,
            is_flagged,
            flags,
            received_at,
        };
        insert_message(conn, &message)?;
        inserted += 1;
    }

    let _ = session.logout();
    Ok(inserted)
}

fn existing_uids(conn: &mut diesel::PgConnection, account_id: Uuid) -> Result<HashSet<i64>, String> {
    let rows: Vec<UidRow> = diesel::sql_query(
        "SELECT uid FROM email_messages WHERE account_id = $1 AND folder = 'INBOX'",
    )
    .bind::<SqlUuid, _>(account_id)
    .load(conn)
    .map_err(|e| format!("Failed to read existing message uids: {e}"))?;
    Ok(rows.into_iter().map(|r| r.uid).collect())
}

fn insert_message(conn: &mut diesel::PgConnection, message: &StoredMessage) -> Result<(), String> {
    diesel::sql_query(
        "INSERT INTO email_messages \
         (id, account_id, message_id_header, in_reply_to, subject, normalized_subject, \
          from_address, to_addresses, body_text, body_html, has_attachments, folder, uid, \
          flags, is_read, is_flagged, received_at, synced_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)",
    )
    .bind::<SqlUuid, _>(Uuid::new_v4())
    .bind::<SqlUuid, _>(message.account_id)
    .bind::<Nullable<Text>, _>(message.message_id.as_deref())
    .bind::<Nullable<Text>, _>(message.in_reply_to.as_deref())
    .bind::<Text, _>(&message.subject)
    .bind::<Text, _>(&message.normalized_subject)
    .bind::<Text, _>(&message.from_address)
    .bind::<Nullable<Text>, _>(message.to_addresses.as_deref())
    .bind::<Nullable<Text>, _>(message.body_text.as_deref())
    .bind::<Nullable<Text>, _>(message.body_html.as_deref())
    .bind::<Bool, _>(message.has_attachments)
    .bind::<Text, _>("INBOX")
    .bind::<BigInt, _>(message.uid)
    .bind::<Jsonb, _>(&message.flags)
    .bind::<Bool, _>(message.is_read)
    .bind::<Bool, _>(message.is_flagged)
    .bind::<Timestamptz, _>(message.received_at)
    .bind::<Timestamptz, _>(chrono::Utc::now())
    .execute(conn)
    .map_err(|e| format!("Failed to insert email message: {e}"))?;
    Ok(())
}

fn non_empty(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
