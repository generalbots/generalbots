//! Background IMAP sync worker for the unified inbox.
//!
//! The worker polls every active `user_email_accounts` row on a fixed
//! interval, connects to each mailbox over IMAPS, fetches INBOX messages
//! newer than the last synced UID and writes them into `email_messages`.
//! Inserts are UID-deduped per account so a repeated pass is idempotent even
//! if the mailbox changes between passes.

use base64::{engine::general_purpose, Engine as _};
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
    #[diesel(sql_type = Text)]
    password_encrypted: String,
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
    has_attachments: bool,
    is_read: bool,
    is_flagged: bool,
    flags: serde_json::Value,
    received_at: chrono::DateTime<chrono::Utc>,
}

async fn sync_all_accounts(pool: &DbPool) -> Result<(), String> {
    let pool = pool.clone();
    let accounts = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| format!("DB pool error: {e}"))?;
        diesel::sql_query(
            "SELECT id, imap_server, imap_port, username, password_encrypted \
             FROM user_email_accounts WHERE is_active = true",
        )
        .load::<SyncAccount>(&mut conn)
        .map_err(|e| format!("Failed to load email accounts: {e}"))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    for account in &accounts {
        if let Err(e) = sync_account(pool.clone(), account).await {
            warn!("IMAP sync for account {} failed: {e}", account.id);
        }
    }
    Ok(())
}

async fn sync_account(pool: DbPool, account: &SyncAccount) -> Result<(), String> {
    let owned = SyncAccount {
        id: account.id,
        imap_server: account.imap_server.clone(),
        imap_port: account.imap_port,
        username: account.username.clone(),
        password_encrypted: account.password_encrypted.clone(),
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
    let password = decrypt_password(&account.password_encrypted)?;

    let client = imap::ClientBuilder::new(account.imap_server.as_str(), account.imap_port as u16)
        .connect()
        .map_err(|e| format!("IMAP connect failed: {e:?}"))?;
    let mut session = client
        .login(&account.username, &password)
        .map_err(|(e, _)| format!("IMAP login failed: {e:?}"))?;
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
    .bind::<Nullable<Text>, _>(Option::<&str>::None)
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

fn decrypt_password(encrypted: &str) -> Result<String, String> {
    general_purpose::STANDARD
        .decode(encrypted)
        .map_err(|e| format!("Password decryption failed: {e}"))
        .and_then(|bytes| {
            String::from_utf8(bytes).map_err(|e| format!("Password is not UTF-8: {e}"))
        })
}

fn non_empty(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
