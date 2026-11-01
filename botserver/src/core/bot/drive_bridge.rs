//! Drive ↔ agent workspace bridge (#1584).
//!
//! One place that resolves the bot's Drive location, finds an object by the
//! name a user typed, reads it for the VM (`data/`) and publishes artifacts
//! back under a conversation folder keyed by the conversation *title*, so the
//! copy survives deletion of the ephemeral session folder.
//!
//! Every call is scoped to the bot's own tree or the caller's `users/{email}/`
//! folder — the same buckets `/api/files/*` uses. No Drive credential ever
//! reaches the model or a generated program.

use std::sync::Arc;

use botcore::shared::state::AppState;
use uuid::Uuid;

pub const MAX_FETCH_BYTES: u64 = 100 * 1024 * 1024;
pub const MAX_FETCH_FILES: usize = 3;

#[derive(Clone, Debug)]
pub struct DriveScope {
    pub bucket: String,
    /// `{bot}.gbdrive/` inside the bot's workspace.
    pub bot_prefix: String,
    /// `users/{email}/`, when the account row is available.
    pub user_prefix: Option<String>,
}

impl DriveScope {
    fn prefixes(&self) -> Vec<&str> {
        let mut prefixes = vec![self.bot_prefix.as_str()];
        if let Some(user) = self.user_prefix.as_deref() {
            prefixes.push(user);
        }
        prefixes
    }
}

/// The caller's account email, used for the `users/{email}/` Drive scope.
fn user_email(state: &Arc<AppState>, user_id: Uuid) -> Option<String> {
    use diesel::sql_types::{Text, Uuid as SqlUuid};
    use diesel::{OptionalExtension, RunQueryDsl};
    #[derive(diesel::QueryableByName)]
    struct EmailRow {
        #[diesel(sql_type = Text)]
        email: String,
    }
    let mut conn = state.conn.get().ok()?;
    let row: Option<EmailRow> =
        diesel::sql_query("SELECT email FROM users WHERE id = $1 LIMIT 1")
            .bind::<SqlUuid, _>(user_id)
            .get_result(&mut conn)
            .optional()
            .ok()
            .flatten();
    let email = row?.email;
    if email.trim().is_empty() {
        return None;
    }
    Some(email.trim().to_string())
}

pub fn scope(state: &Arc<AppState>, bot_uuid: &Uuid, user_id: Uuid) -> Option<DriveScope> {
    let mut conn = state.conn.get().ok()?;
    let location = botbasic_core::utils::bot_drive_location_for(&mut conn, *bot_uuid);
    if location.key_prefix.is_empty() {
        return None;
    }
    drop(conn);
    Some(DriveScope {
        bucket: location.bucket,
        bot_prefix: location.key_prefix,
        user_prefix: user_email(state, user_id),
    })
}

/// Finds the object a user meant by `reference` (a file name, a relative path
/// such as `documents/report.csv`, or a full Drive key).
pub async fn find_object(
    state: &Arc<AppState>,
    scope: &DriveScope,
    reference: &str,
) -> Option<(String, String)> {
    let drive = state.drive.clone()?;
    let wanted = reference.trim().trim_start_matches('/').to_lowercase();
    if wanted.is_empty() {
        return None;
    }
    let wanted_name = wanted.rsplit('/').next().unwrap_or(&wanted);
    let mut fallback: Option<(String, String)> = None;
    for prefix in scope.prefixes() {
        let listed = drive
            .list_objects_with_metadata(&scope.bucket, Some(prefix))
            .await
            .ok()?;
        for object in listed {
            let key = object.key;
            let relative = key
                .strip_prefix(prefix)
                .unwrap_or(&key)
                .to_string();
            let lower = relative.to_lowercase();
            let name = lower.rsplit('/').next().unwrap_or(&lower);
            if lower == wanted {
                return Some((key, relative));
            }
            if name == wanted_name && fallback.is_none() {
                fallback = Some((key, relative));
            }
        }
    }
    fallback
}

pub async fn read_object(
    state: &Arc<AppState>,
    scope: &DriveScope,
    key: &str,
) -> Option<Vec<u8>> {
    let drive = state.drive.clone()?;
    match drive.get_object(&scope.bucket, key).await {
        Ok(bytes) => Some(bytes),
        Err(e) => {
            log::warn!("drive bridge: read {key} failed: {e}");
            None
        }
    }
}

fn content_type(name: &str) -> &'static str {
    match name.rsplit('.').next().unwrap_or("").to_lowercase().as_str() {
        "pdf" => "application/pdf",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "csv" => "text/csv; charset=utf-8",
        "json" => "application/json",
        "txt" | "md" => "text/plain; charset=utf-8",
        "html" => "text/html; charset=utf-8",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "zip" => "application/zip",
        _ => "application/octet-stream",
    }
}

/// Writes `bytes` under `{bot}.gbdrive/conversations/{folder}/{name}`.
/// Idempotent: the key is deterministic, so saving twice overwrites.
pub async fn save_object(
    state: &Arc<AppState>,
    bot_uuid: &Uuid,
    folder: &str,
    name: &str,
    bytes: Vec<u8>,
) -> Option<String> {
    let drive = state.drive.clone()?;
    let mut conn = state.conn.get().ok()?;
    let location = botbasic_core::utils::bot_drive_location_for(&mut conn, *bot_uuid);
    drop(conn);
    let key = location.key_for(&format!("conversations/{folder}/{name}"));
    let bucket = location.bucket;
    match drive
        .put_object(&bucket, &key, bytes, Some(content_type(name)))
        .await
    {
        Ok(()) => Some(format!("{bucket}/{key}")),
        Err(e) => {
            log::warn!("drive bridge: save {key} failed: {e}");
            None
        }
    }
}

/// Folder-safe, human-readable conversation name used as the Drive folder.
pub fn slug(text: &str) -> String {
    let mut out = String::new();
    for ch in text.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
        } else if matches!(ch, ' ' | '-' | '_' | '.' | ':' | '(' | ')') && !out.ends_with('-') {
            out.push('-');
        }
        if out.len() >= 60 {
            break;
        }
    }
    let out = out.trim_matches('-').to_string();
    if out.is_empty() {
        "conversation".to_string()
    } else {
        out
    }
}

/// Conversation title (first user turn) or a short session fallback.
pub async fn conversation_title(state: &Arc<AppState>, session_id: Uuid) -> String {
    let first = {
        let mut sm = state.session_manager.lock().await;
        sm.get_first_user_message(session_id).ok().flatten()
    };
    match first {
        Some(text) if !text.trim().is_empty() => slug(&text),
        _ => {
            let id = session_id.to_string();
            format!("conversation-{}", &id[..8.min(id.len())])
        }
    }
}
