use crate::core::shared::sanitize_path_for_filename;
use diesel::prelude::*;
use log::{error, info, trace};
use rhai::{Dynamic, Engine};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::shared::models::TriggerKind;
use crate::shared::models::UserSession;
use crate::shared::state::AppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FolderProvider {
    GDrive,
    OneDrive,
    Dropbox,
    Local,
}

impl std::str::FromStr for FolderProvider {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "gdrive" | "googledrive" | "google" => Ok(Self::GDrive),
            "onedrive" | "microsoft" | "ms" => Ok(Self::OneDrive),
            "dropbox" | "dbx" => Ok(Self::Dropbox),
            "local" | "filesystem" | "fs" => Ok(Self::Local),
            _ => Err(()),
        }
    }
}

impl FolderProvider {
    pub fn parse(s: &str) -> Option<Self> {
        s.parse().ok()
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GDrive => "gdrive",
            Self::OneDrive => "onedrive",
            Self::Dropbox => "dropbox",
            Self::Local => "local",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeEventType {
    Create,
    Modify,
    Delete,
    Rename,
    Move,
}

impl ChangeEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Modify => "modify",
            Self::Delete => "delete",
            Self::Rename => "rename",
            Self::Move => "move",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "create" | "created" | "new" => Some(Self::Create),
            "modify" | "modified" | "change" | "changed" => Some(Self::Modify),
            "delete" | "deleted" | "remove" | "removed" => Some(Self::Delete),
            "rename" | "renamed" => Some(Self::Rename),
            "move" | "moved" => Some(Self::Move),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderMonitor {
    pub id: Uuid,
    pub bot_id: Uuid,
    pub provider: String,
    pub account_email: Option<String>,
    pub folder_path: String,
    pub folder_id: Option<String>,
    pub script_path: String,
    pub is_active: bool,
    pub watch_subfolders: bool,
    pub event_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderChangeEvent {
    pub id: Uuid,
    pub monitor_id: Uuid,
    pub event_type: String,
    pub file_path: String,
    pub file_id: Option<String>,
    pub file_name: Option<String>,
    pub file_size: Option<i64>,
    pub mime_type: Option<String>,
    pub old_path: Option<String>,
}

pub fn parse_folder_path(path: &str) -> (FolderProvider, Option<String>, String) {
    if let Some(rest) = path.strip_prefix("account://") {
        if let Some(slash_pos) = rest.find('/') {
            let email = &rest[..slash_pos];
            let folder_path = &rest[slash_pos..];
            let provider = detect_provider_from_email(email);
            return (provider, Some(email.to_string()), folder_path.to_string());
        }
    }

    if let Some(folder_path) = path.strip_prefix("gdrive://") {
        return (FolderProvider::GDrive, None, folder_path.to_string());
    }

    if let Some(folder_path) = path.strip_prefix("onedrive://") {
        return (FolderProvider::OneDrive, None, folder_path.to_string());
    }

    if let Some(folder_path) = path.strip_prefix("dropbox://") {
        return (FolderProvider::Dropbox, None, folder_path.to_string());
    }

    (FolderProvider::Local, None, path.to_string())
}

pub fn detect_provider_from_email(email: &str) -> FolderProvider {
    let lower = email.to_lowercase();
    if lower.ends_with("@gmail.com") || lower.contains("google") {
        FolderProvider::GDrive
    } else if lower.ends_with("@outlook.com")
        || lower.ends_with("@hotmail.com")
        || lower.contains("microsoft")
    {
        FolderProvider::OneDrive
    } else {
        FolderProvider::GDrive
    }
}

pub fn is_cloud_path(path: &str) -> bool {
    path.starts_with("account://")
        || path.starts_with("gdrive://")
        || path.starts_with("onedrive://")
        || path.starts_with("dropbox://")
}

pub fn on_change_keyword(state: &AppState, user: UserSession, engine: &mut Engine) {
    register_on_change_basic(state, user.clone(), engine);
    register_on_change_with_events(state, user, engine);
}

fn register_on_change_basic(state: &AppState, user: UserSession, engine: &mut Engine) {
    let state_clone = state.clone();
    let bot_id = user.bot_id;

    engine
        .register_custom_syntax(
            ["ON", "CHANGE", "$string$"],
            true,
            move |context, inputs| {
                let path = context
                    .eval_expression_tree(&inputs[0])?
                    .to_string()
                    .trim_matches('"')
                    .to_string();

                let (provider, account_email, folder_path) = parse_folder_path(&path);

                trace!(
                    "ON CHANGE '{}' (provider: {}, account: {:?}) for bot: {}",
                    folder_path,
                    provider.as_str(),
                    account_email,
                    bot_id
                );

                let script_name = format!(
                    "on_change_{}.rhai",
                    sanitize_path_for_filename(&folder_path)
                );

                let mut conn = state_clone
                    .conn
                    .get()
                    .map_err(|e| format!("DB error: {}", e))?;

                let result = execute_on_change(
                    &mut conn,
                    bot_id,
                    provider,
                    account_email.as_deref(),
                    &folder_path,
                    &script_name,
                    true,
                    vec!["create", "modify", "delete"],
                )
                .map_err(|e| format!("DB error: {}", e))?;

                if let Some(rows_affected) = result.get("rows_affected") {
                    info!(
                        "Folder monitor registered for '{}' ({}) on bot {}",
                        folder_path,
                        provider.as_str(),
                        bot_id
                    );
                    Ok(Dynamic::from(rows_affected.as_i64().unwrap_or(0)))
                } else {
                    Err("Failed to register folder monitor".into())
                }
            },
        )
        .expect("valid syntax registration");
}

fn register_on_change_with_events(state: &AppState, user: UserSession, engine: &mut Engine) {
    let state_clone = state.clone();
    let bot_id = user.bot_id;

    engine
        .register_custom_syntax(
            ["ON", "CHANGE", "$string$", "EVENTS", "$expr$"],
            true,
            move |context, inputs| {
                let path = context
                    .eval_expression_tree(&inputs[0])?
                    .to_string()
                    .trim_matches('"')
                    .to_string();

                let events_value = context.eval_expression_tree(&inputs[1])?;
                let events_str = events_value.to_string();
                let events: Vec<&str> = events_str
                    .trim_matches('"')
                    .split(',')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .collect();

                let (provider, account_email, folder_path) = parse_folder_path(&path);

                trace!(
                    "ON CHANGE '{}' EVENTS {:?} (provider: {}) for bot: {}",
                    folder_path,
                    events,
                    provider.as_str(),
                    bot_id
                );

                let script_name = format!(
                    "on_change_{}.rhai",
                    sanitize_path_for_filename(&folder_path)
                );

                let mut conn = state_clone
                    .conn
                    .get()
                    .map_err(|e| format!("DB error: {}", e))?;

                let result = execute_on_change(
                    &mut conn,
                    bot_id,
                    provider,
                    account_email.as_deref(),
                    &folder_path,
                    &script_name,
                    true,
                    events,
                )
                .map_err(|e| format!("DB error: {}", e))?;

                if let Some(rows_affected) = result.get("rows_affected") {
                    info!(
                        "Folder monitor registered for '{}' with events {:?} on bot {}",
                        folder_path, events_str, bot_id
                    );
                    Ok(Dynamic::from(rows_affected.as_i64().unwrap_or(0)))
                } else {
                    Err("Failed to register folder monitor".into())
                }
            },
        )
        .expect("valid syntax registration");
}



pub fn execute_on_change(
    conn: &mut diesel::PgConnection,
    bot_id: Uuid,
    provider: FolderProvider,
    account_email: Option<&str>,
    folder_path: &str,
    script_path: &str,
    watch_subfolders: bool,
    event_types: Vec<&str>,
) -> Result<Value, String> {
    use crate::shared::models::system_automations;

    let target = match account_email {
        Some(email) => format!("account://{}{}", email, folder_path),
        None => format!("{}://{}", provider.as_str(), folder_path),
    };

    let new_automation = (
        system_automations::kind.eq(TriggerKind::FolderChange as i32),
        system_automations::target.eq(&target),
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
    let events_json = serde_json::to_string(&event_types).unwrap_or_else(|_| "[]".to_string());
    let account_sql = account_email
        .map(|e| format!("'{}'", e.replace('\'', "''")))
        .unwrap_or_else(|| "NULL".to_string());

    let insert_sql = format!(
        "INSERT INTO folder_monitors (id, bot_id, provider, folder_path, script_path, is_active, watch_subfolders, event_types_json) \
         VALUES ('{}', '{}', '{}', '{}', '{}', true, {}, '{}') \
         ON CONFLICT (bot_id, provider, folder_path) DO UPDATE SET \
         script_path = EXCLUDED.script_path, \
         watch_subfolders = EXCLUDED.watch_subfolders, \
         event_types_json = EXCLUDED.event_types_json, \
         is_active = true, \
         updated_at = NOW()",
        monitor_id,
        bot_id,
        provider.as_str(),
        folder_path.replace('\'', "''"),
        script_path.replace('\'', "''"),
        watch_subfolders,
        events_json.replace('\'', "''")
    );

    diesel::sql_query(&insert_sql).execute(conn).map_err(|e| {
        error!("Failed to insert folder monitor: {}", e);
        e.to_string()
    })?;

    Ok(json!({
        "command": "on_change",
        "provider": provider.as_str(),
        "account_email": account_sql,
        "folder_path": folder_path,
        "script_path": script_path,
        "watch_subfolders": watch_subfolders,
        "event_types": event_types,
        "rows_affected": result
    }))
}

pub fn check_folder_monitors(
    state: &AppState,
    bot_id: Uuid,
) -> Result<Vec<(FolderChangeEvent, String)>, String> {
    let mut conn = state.conn.get().map_err(|e| e.to_string())?;

    let monitors_sql = format!(
        "SELECT id, bot_id, provider, folder_path, folder_id, script_path, \
         watch_subfolders, last_change_token, event_types_json \
         FROM folder_monitors WHERE bot_id = '{}' AND is_active = true",
        bot_id
    );

    #[derive(QueryableByName)]
    struct MonitorRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        bot_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        provider: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        folder_path: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        folder_id: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Text)]
        script_path: String,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        watch_subfolders: bool,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        last_change_token: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        event_types_json: Option<String>,
    }

    let monitors: Vec<MonitorRow> = diesel::sql_query(&monitors_sql)
        .load(&mut *conn)
        .map_err(|e| e.to_string())?;

    let mut events = Vec::new();

    for monitor in monitors {
        let event_types: Vec<String> = monitor
            .event_types_json
            .as_ref()
            .and_then(|j| serde_json::from_str(j.as_str()).ok())
            .unwrap_or_else(|| {
                vec![
                    "create".to_string(),
                    "modify".to_string(),
                    "delete".to_string(),
                ]
            });

        trace!(
            "Checking folder monitor {} for {} on bot {} (provider: {}, events: {:?}, subfolders: {})",
            monitor.id,
            monitor.folder_path,
            monitor.bot_id,
            monitor.provider,
            event_types,
            monitor.watch_subfolders
        );

        let provider = monitor.provider.parse().unwrap_or(FolderProvider::Local);

        let new_events = fetch_folder_changes(
            state,
            monitor.id,
            provider,
            &monitor.folder_path,
            monitor.folder_id.as_deref(),
            monitor.last_change_token.as_deref(),
            monitor.watch_subfolders,
            &event_types,
        )?;

        for event in new_events {
            events.push((event, monitor.script_path.clone()));
        }
    }

    Ok(events)
}

fn fetch_folder_changes(
    _state: &AppState,
    monitor_id: Uuid,
    provider: FolderProvider,
    folder_path: &str,
    _folder_id: Option<&str>,
    _last_token: Option<&str>,
    _watch_subfolders: bool,
    _event_types: &[String],
) -> Result<Vec<FolderChangeEvent>, String> {
    trace!(
        "Fetching {} changes for monitor {} path {}",
        provider.as_str(),
        monitor_id,
        folder_path
    );
    Ok(Vec::new())
}

pub fn process_folder_event(
    state: &AppState,
    event: &FolderChangeEvent,
    script_path: &str,
) -> Result<(), String> {
    info!(
        "Processing folder event {} ({}) for {} with script {}",
        event.id, event.event_type, event.file_path, script_path
    );

    let mut conn = state.conn.get().map_err(|e| e.to_string())?;

    let update_sql = format!(
        "UPDATE folder_change_events SET processed = true, processed_at = NOW() WHERE id = '{}'",
        event.id
    );

    diesel::sql_query(&update_sql)
        .execute(&mut *conn)
        .map_err(|e| e.to_string())?;

    Ok(())
}
