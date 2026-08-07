//! Demo email account + messages so the Mail app shows real content.
//!
//! The demo inbox is attached only to accounts of the dedicated sample tenant
//! (`user@sample.com` etc.), never to real or guest users.

use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{BigInt, Bool, Text, Uuid as SqlUuid};
use uuid::Uuid;

use crate::sample::SAMPLE_ORG_ID;

const DEMO_EMAIL: &str = "demo@generalbots.local";

#[derive(diesel::QueryableByName)]
struct CountRow {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    n: i64,
}

#[derive(diesel::QueryableByName)]
struct UuidRowNamed {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
}

#[derive(diesel::QueryableByName)]
struct UserIdRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
}

/// Seeds the demo mailbox for every user bound to the sample organization only.
pub fn seed(conn: &mut diesel::PgConnection) -> Result<(), String> {
    // Sample-org users: bound via user_organizations to the sample org.
    let users: Vec<UserIdRow> = sql_query(
        "SELECT u.id FROM users u \
         JOIN user_organizations uo ON uo.user_id = u.id AND uo.org_id = $1 \
         WHERE u.is_active = true ORDER BY u.created_at",
    )
    .bind::<SqlUuid, _>(SAMPLE_ORG_ID)
    .load(conn)
    .map_err(|e| e.to_string())?;

    if users.is_empty() {
        log::warn!("botsampledata/email: no sample-org users found to attach demo inbox");
        return Ok(());
    }

    let mut targets: Vec<Uuid> = users.iter().map(|u| u.id).collect();
    targets.sort();
    targets.dedup();

    let messages: &[(&str, &str, &str, &str, &str)] = &[
        ("Welcome to General Bots", "alice@sample.com", "Hello! Glad you joined the platform.", "INBOX", "false"),
        ("Your invoice INV-2026-0001", "billing@acme.example.com", "Your invoice is ready.", "INBOX", "false"),
        ("Sprint planning invite", "calendar@generalbots.local", "Join us for sprint planning.", "INBOX", "true"),
        ("Quarterly report ready", "reports@generalbots.local", "Q3 summary is available in Drive.", "INBOX", "false"),
    ];

    for user_id in &targets {
        let account_id = ensure_account(conn, *user_id)?;
        for (idx, (subject, sender, body, folder, is_read)) in messages.iter().enumerate() {
            let n: i64 = sql_query(
                "SELECT count(*) AS n FROM email_messages WHERE account_id = $1 AND subject = $2",
            )
            .bind::<SqlUuid, _>(account_id)
            .bind::<Text, _>(subject)
            .get_result::<CountRow>(conn)
            .map_err(|e| e.to_string())?
            .n;
            if n == 0 {
                let uid: i64 = 1000 + idx as i64;
                sql_query(
                    "INSERT INTO email_messages
                     (id, account_id, message_id_header, subject, normalized_subject, from_address, to_addresses, body_text, body_html, has_attachments, folder, uid, flags, is_read, is_flagged, received_at, synced_at)
                     VALUES ($1, $2, $1::text, $3, lower($3), $4, $5, $6, '<p>' || $6 || '</p>', false, $7, $8, '[]', $9, false, NOW(), NOW())",
                )
                    .bind::<SqlUuid, _>(Uuid::new_v4())
                    .bind::<SqlUuid, _>(account_id)
                    .bind::<Text, _>(subject)
                    .bind::<Text, _>(sender)
                    .bind::<Text, _>(DEMO_EMAIL)
                    .bind::<Text, _>(body)
                    .bind::<Text, _>(folder)
                    .bind::<BigInt, _>(uid)
                    .bind::<Bool, _>(*is_read == "true")
                    .execute(conn)
                    .map_err(|e| e.to_string())?;
            }
        }
    }

    Ok(())
}

fn ensure_account(conn: &mut diesel::PgConnection, user_id: Uuid) -> Result<Uuid, String> {
    let n: i64 = sql_query(
        "SELECT count(*) AS n FROM user_email_accounts WHERE user_id = $1 AND email = $2",
    )
    .bind::<SqlUuid, _>(user_id)
    .bind::<Text, _>(DEMO_EMAIL)
    .get_result::<CountRow>(conn)
    .map_err(|e| e.to_string())?
    .n;

    if n == 0 {
        sql_query(
            "INSERT INTO user_email_accounts
             (id, user_id, email, display_name, imap_server, imap_port, smtp_server, smtp_port, username, password_encrypted, is_primary, is_active, created_at)
             VALUES ($1, $2, $3, 'Demo Inbox', 'localhost', 993, 'localhost', 587, 'demo', 'ZGVtbw==', true, true, NOW())
             RETURNING id",
        )
        .bind::<SqlUuid, _>(Uuid::new_v4())
        .bind::<SqlUuid, _>(user_id)
        .bind::<Text, _>(DEMO_EMAIL)
        .get_result::<UuidRowNamed>(conn)
        .map(|r| r.id)
        .map_err(|e| e.to_string())
    } else {
        sql_query("SELECT id FROM user_email_accounts WHERE user_id = $1 AND email = $2 LIMIT 1")
            .bind::<SqlUuid, _>(user_id)
            .bind::<Text, _>(DEMO_EMAIL)
            .get_result::<UuidRowNamed>(conn)
            .map(|r| r.id)
            .map_err(|e| e.to_string())
    }
}