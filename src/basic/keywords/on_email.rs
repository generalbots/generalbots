use diesel::prelude::*;
use log::{error, info, trace};
use rhai::{Dynamic, Engine};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::shared::models::TriggerKind;
use crate::shared::models::UserSession;
use crate::shared::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailMonitor {
    pub id: Uuid,
    pub bot_id: Uuid,
    pub email_address: String,
    pub script_path: String,
    pub is_active: bool,
    pub filter_from: Option<String>,
    pub filter_subject: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailReceivedEvent {
    pub id: Uuid,
    pub monitor_id: Uuid,
    pub message_uid: i64,
    pub message_id: Option<String>,
    pub from_address: String,
    pub to_addresses: Vec<String>,
    pub subject: Option<String>,
    pub has_attachments: bool,
    pub attachments: Vec<EmailAttachment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAttachment {
    pub filename: String,
    pub mime_type: String,
    pub size: i64,
}

pub fn on_email_keyword(state: &AppState, user: UserSession, engine: &mut Engine) {
    register_on_email(state, user.clone(), engine);
    register_on_email_from(state, user.clone(), engine);
    register_on_email_subject(state, user, engine);
}

fn register_on_email(state: &AppState, user: UserSession, engine: &mut Engine) {
    let state_clone = state.clone();
    let bot_id = user.bot_id;

    engine
        .register_custom_syntax(["ON", "EMAIL", "$string$"], true, move |context, inputs| {
            let email_address = context
                .eval_expression_tree(&inputs[0])?
                .to_string()
                .trim_matches('"')
                .to_string();

            trace!("ON EMAIL '{}' for bot: {}", email_address, bot_id);

            let script_name = format!(
                "on_email_{}.rhai",
                email_address.replace('@', "_at_").replace('.', "_")
            );

            let mut conn = state_clone
                .conn
                .get()
                .map_err(|e| format!("DB error: {}", e))?;

            let result =
                execute_on_email(&mut conn, bot_id, &email_address, &script_name, None, None)
                    .map_err(|e| format!("DB error: {}", e))?;

            if let Some(rows_affected) = result.get("rows_affected") {
                info!(
                    "Email monitor registered for '{}' on bot {}",
                    email_address, bot_id
                );
                Ok(Dynamic::from(rows_affected.as_i64().unwrap_or(0)))
            } else {
                Err("Failed to register email monitor".into())
            }
        })
        .expect("valid syntax registration");
}

fn register_on_email_from(state: &AppState, user: UserSession, engine: &mut Engine) {
    let state_clone = state.clone();
    let bot_id = user.bot_id;

    engine
        .register_custom_syntax(
            ["ON", "EMAIL", "$string$", "FROM", "$string$"],
            true,
            move |context, inputs| {
                let email_address = context
                    .eval_expression_tree(&inputs[0])?
                    .to_string()
                    .trim_matches('"')
                    .to_string();

                let filter_from = context
                    .eval_expression_tree(&inputs[1])?
                    .to_string()
                    .trim_matches('"')
                    .to_string();

                trace!(
                    "ON EMAIL '{}' FROM '{}' for bot: {}",
                    email_address,
                    filter_from,
                    bot_id
                );

                let script_name = format!(
                    "on_email_{}_from_{}.rhai",
                    email_address.replace('@', "_at_").replace('.', "_"),
                    filter_from.replace('@', "_at_").replace('.', "_")
                );

                let mut conn = state_clone
                    .conn
                    .get()
                    .map_err(|e| format!("DB error: {}", e))?;

                let result = execute_on_email(
                    &mut conn,
                    bot_id,
                    &email_address,
                    &script_name,
                    Some(&filter_from),
                    None,
                )
                .map_err(|e| format!("DB error: {}", e))?;

                if let Some(rows_affected) = result.get("rows_affected") {
                    info!(
                        "Email monitor registered for '{}' from '{}' on bot {}",
                        email_address, filter_from, bot_id
                    );
                    Ok(Dynamic::from(rows_affected.as_i64().unwrap_or(0)))
                } else {
                    Err("Failed to register email monitor".into())
                }
            },
        )
        .expect("valid syntax registration");
}

fn register_on_email_subject(state: &AppState, user: UserSession, engine: &mut Engine) {
    let state_clone = state.clone();
    let bot_id = user.bot_id;

    engine
        .register_custom_syntax(
            ["ON", "EMAIL", "$string$", "SUBJECT", "$string$"],
            true,
            move |context, inputs| {
                let email_address = context
                    .eval_expression_tree(&inputs[0])?
                    .to_string()
                    .trim_matches('"')
                    .to_string();

                let filter_subject = context
                    .eval_expression_tree(&inputs[1])?
                    .to_string()
                    .trim_matches('"')
                    .to_string();

                trace!(
                    "ON EMAIL '{}' SUBJECT '{}' for bot: {}",
                    email_address,
                    filter_subject,
                    bot_id
                );

                let script_name = format!(
                    "on_email_{}_subject.rhai",
                    email_address.replace('@', "_at_").replace('.', "_")
                );

                let mut conn = state_clone
                    .conn
                    .get()
                    .map_err(|e| format!("DB error: {}", e))?;

                let result = execute_on_email(
                    &mut conn,
                    bot_id,
                    &email_address,
                    &script_name,
                    None,
                    Some(&filter_subject),
                )
                .map_err(|e| format!("DB error: {}", e))?;

                if let Some(rows_affected) = result.get("rows_affected") {
                    info!(
                        "Email monitor registered for '{}' with subject filter '{}' on bot {}",
                        email_address, filter_subject, bot_id
                    );
                    Ok(Dynamic::from(rows_affected.as_i64().unwrap_or(0)))
                } else {
                    Err("Failed to register email monitor".into())
                }
            },
        )
        .expect("valid syntax registration");
}

pub fn execute_on_email(
    conn: &mut diesel::PgConnection,
    bot_id: Uuid,
    email_address: &str,
    script_path: &str,
    filter_from: Option<&str>,
    filter_subject: Option<&str>,
) -> Result<Value, String> {
    use crate::shared::models::system_automations;

    let new_automation = (
        system_automations::kind.eq(TriggerKind::EmailReceived as i32),
        system_automations::target.eq(email_address),
        system_automations::param.eq(script_path),
        system_automations::bot_id.eq(bot_id),
    );

    let result = diesel::insert_into(system_automations::table)
        .values(&new_automation)
        .execute(conn)
        .map_err(|e| {
            error!("SQL execution error: {}", e);
            e.to_string()
        })?;

    let monitor_id = Uuid::new_v4();
    let insert_sql = format!(
        "INSERT INTO email_monitors (id, bot_id, email_address, script_path, filter_from, filter_subject, is_active) \
         VALUES ('{}', '{}', '{}', '{}', {}, {}, true) \
         ON CONFLICT (bot_id, email_address) DO UPDATE SET \
         script_path = EXCLUDED.script_path, \
         filter_from = EXCLUDED.filter_from, \
         filter_subject = EXCLUDED.filter_subject, \
         is_active = true, \
         updated_at = NOW()",
        monitor_id,
        bot_id,
        email_address.replace('\'', "''"),
        script_path.replace('\'', "''"),
        filter_from.map(|f| format!("'{}'", f.replace('\'', "''"))).unwrap_or_else(|| "NULL".to_string()),
        filter_subject.map(|s| format!("'{}'", s.replace('\'', "''"))).unwrap_or_else(|| "NULL".to_string())
    );

    diesel::sql_query(&insert_sql).execute(conn).map_err(|e| {
        error!("Failed to insert email monitor: {}", e);
        e.to_string()
    })?;

    Ok(json!({
        "command": "on_email",
        "email_address": email_address,
        "script_path": script_path,
        "filter_from": filter_from,
        "filter_subject": filter_subject,
        "rows_affected": result
    }))
}

pub fn check_email_monitors(
    state: &AppState,
    bot_id: Uuid,
) -> Result<Vec<(EmailReceivedEvent, String)>, String> {
    let mut conn = state.conn.get().map_err(|e| e.to_string())?;

    let monitors_sql = format!(
        "SELECT id, bot_id, email_address, script_path, filter_from, filter_subject, last_uid \
         FROM email_monitors WHERE bot_id = '{}' AND is_active = true",
        bot_id
    );

    #[derive(QueryableByName)]
    struct MonitorRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        bot_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        email_address: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        script_path: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        filter_from: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        filter_subject: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::BigInt>)]
        last_uid: Option<i64>,
    }

    let monitors: Vec<MonitorRow> = diesel::sql_query(&monitors_sql)
        .load(&mut *conn)
        .map_err(|e| e.to_string())?;

    let mut events = Vec::new();

    for monitor in monitors {
        trace!(
            "Checking email monitor for {} on bot {} (last_uid: {:?})",
            monitor.email_address,
            monitor.bot_id,
            monitor.last_uid
        );

        let new_events = fetch_new_emails(
            state,
            monitor.id,
            &monitor.email_address,
            monitor.last_uid.unwrap_or(0),
            monitor.filter_from.as_deref(),
            monitor.filter_subject.as_deref(),
        )?;

        for event in new_events {
            events.push((event, monitor.script_path.clone()));
        }
    }

    Ok(events)
}

fn fetch_new_emails(
    _state: &AppState,
    monitor_id: Uuid,
    email_address: &str,
    last_uid: i64,
    filter_from: Option<&str>,
    filter_subject: Option<&str>,
) -> Result<Vec<EmailReceivedEvent>, String> {
    trace!("Fetching new emails for monitor {} address {}", monitor_id, email_address);

    // In production, this would connect to IMAP/Exchange/Gmail API
    // For now, return mock data to demonstrate the interface works

    // Only return mock data if this looks like a fresh request (last_uid == 0)
    if last_uid > 0 {
        // Already processed emails, return empty
        return Ok(Vec::new());
    }

    // Generate mock emails for testing
    let now = chrono::Utc::now();
    let mut events = Vec::new();

    // Mock email 1
    let mut should_include = true;
    if let Some(from_filter) = filter_from {
        should_include = "notifications@example.com".contains(from_filter);
    }
    if let Some(subject_filter) = filter_subject {
        should_include = should_include && "Welcome to the platform".to_lowercase().contains(&subject_filter.to_lowercase());
    }

    if should_include {
        events.push(EmailReceivedEvent {
            id: Uuid::new_v4(),
            monitor_id,
            from_address: "notifications@example.com".to_string(),
            from_name: Some("Platform Notifications".to_string()),
            to_address: email_address.to_string(),
            subject: "Welcome to the platform".to_string(),
            body_preview: "Thank you for signing up! Here's how to get started...".to_string(),
            body_html: Some("<html><body><h1>Welcome!</h1><p>Thank you for signing up!</p></body></html>".to_string()),
            body_plain: Some("Welcome! Thank you for signing up!".to_string()),
            received_at: now - chrono::Duration::minutes(5),
            message_id: format!("<{}@example.com>", Uuid::new_v4()),
            uid: 1,
            has_attachments: false,
            attachment_names: Vec::new(),
            is_read: false,
            is_important: false,
            labels: vec!["inbox".to_string()],
            processed: false,
            processed_at: None,
        });
    }

    Ok(events)
}

pub fn process_email_event(
    state: &AppState,
    event: &EmailReceivedEvent,
    script_path: &str,
) -> Result<(), String> {
    info!(
        "Processing email event {} from {} with script {}",
        event.id, event.from_address, script_path
    );

    let mut conn = state.conn.get().map_err(|e| e.to_string())?;

    let update_sql = format!(
        "UPDATE email_received_events SET processed = true, processed_at = NOW() WHERE id = '{}'",
        event.id
    );

    diesel::sql_query(&update_sql)
        .execute(&mut *conn)
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn parse_email_path(path: &str) -> Option<(String, Option<String>)> {
    if let Some(rest) = path.strip_prefix("email://") {
        if let Some(slash_pos) = rest.find('/') {
            let email = &rest[..slash_pos];
            let folder = &rest[slash_pos + 1..];
            return Some((email.to_string(), Some(folder.to_string())));
        }
        return Some((rest.to_string(), None));
    }
    None
}

pub fn is_email_path(path: &str) -> bool {
    path.starts_with("email://")
}

pub fn sanitize_email_for_filename(email: &str) -> String {
    email
        .replace('@', "_at_")
        .replace('.', "_")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect::<String>()
        .to_lowercase()
}
