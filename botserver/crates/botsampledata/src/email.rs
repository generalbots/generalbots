//! Demo email account + messages so the Mail app shows real content.
//!
//! The account is created for every real (non-guest) user so whichever
//! account logs in can see the demo inbox.

use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{BigInt, Bool, Text, Uuid as SqlUuid};
use uuid::Uuid;

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

pub fn seed(conn: &mut diesel::PgConnection) -> Result<(), String> {
    // Seed for every real user (skip anonymous guests).
    let users: Vec<UserIdRow> = sql_query(
        "SELECT id FROM users WHERE is_active = true AND username NOT LIKE 'guest%' ORDER BY created_at",
    )
    .load(conn)
    .map_err(|e| e.to_string())?;

    if users.is_empty() {
        log::warn!("botsampledata/email: no non-guest users found to attach demo inbox");
    }

    // The mail app resolves the session to Uuid::nil() in suite mode
    // (extract_user_from_session returns nil), so always seed the nil
    // user too — that is the account the Mail UI actually reads.
    sql_query(
        "INSERT INTO users (id, username, email, password_hash, is_active, created_at, updated_at)
         VALUES ($1, 'demo', 'demo@generalbots.local', 'x', true, NOW(), NOW())
         ON CONFLICT (id) DO NOTHING",
    )
    .bind::<SqlUuid, _>(Uuid::nil())
    .execute(conn)
    .map_err(|e| e.to_string())?;

    let mut targets: Vec<Uuid> = users.iter().map(|u| u.id).collect();
    targets.push(Uuid::nil());
    targets.sort();
    targets.dedup();

    let messages: &[(&str, &str, &str, &str, &str)] = &[
        ("Welcome to General Bots", "alice.sample@example.com", "Hello! Glad you joined the platform.", "INBOX", "false"),
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
