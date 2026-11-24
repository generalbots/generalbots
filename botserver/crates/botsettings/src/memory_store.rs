//! Per-person memory rows (`chat_preferences`), shared by the chat pipeline
//! and the Settings → Memory API.
//!
//! Values are sealed with AES-GCM under a key derived from the master key and
//! the person's id, the same scheme the message history uses per bot, so one
//! person's rows can never be opened with another person's key. Keys of rows
//! the assistant writes are opaque hashes: the fact itself never appears in
//! clear text. Rows written before sealing existed are read as they are and
//! sealed again on the next write.

use botsecurity::encryption::{
    decrypt_field, derive_scope_key, encrypt_field, hash_for_search, load_master_encryption_key,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::{Nullable, Text, Uuid as SqlUuid};
use diesel::PgConnection;
use uuid::Uuid;

use botschema::chat_preferences;

use crate::settings_api::{read_pref, try_write_pref};

const KEY_DOMAIN: &str = "user-memory";
/// `user_preferences` key of the on/off switch; absent means on.
pub const ENABLED_PREF: &str = "memory_enabled";
const ENTRY_KEY_PREFIX: &str = "m-";
const ENTRY_KEY_HASH_LEN: usize = 24;
/// Longest fact or value accepted, in characters.
pub const MAX_VALUE_CHARS: usize = 2000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryEntry {
    pub key: String,
    pub value: String,
    pub bot_id: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
}

fn person_key(master: &[u8], user_id: Uuid) -> Vec<u8> {
    derive_scope_key(master, KEY_DOMAIN, &user_id)
}

/// Seals `plain` for `user_id` under `master`.
pub fn seal_with(master: &[u8], user_id: Uuid, plain: &str) -> Result<String, String> {
    encrypt_field(plain, &person_key(master, user_id)).map_err(|error| error.to_string())
}

fn looks_sealed(stored: &str) -> bool {
    let mut parts = stored.split(':');
    let shaped = |part: Option<&str>| {
        part.is_some_and(|text| {
            !text.is_empty()
                && text
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '/' | '=' | '-' | '_'))
        })
    };
    parts.next() == Some("1") && shaped(parts.next()) && shaped(parts.next()) && parts.next().is_none()
}

/// The value of a stored row: sealed rows are opened with this person's key
/// (None when that fails, e.g. another person's row or a rotated master key);
/// a row written before sealing existed is returned as it is.
pub fn open_with(master: &[u8], user_id: Uuid, stored: &str) -> Option<String> {
    if looks_sealed(stored) {
        decrypt_field(stored, &person_key(master, user_id)).ok()
    } else {
        Some(stored.to_string())
    }
}

/// Opaque, stable key for a fact: repeating the same fact updates its row.
pub fn entry_key_with(master: &[u8], user_id: Uuid, normalized_fact: &str) -> String {
    let digest = hash_for_search(normalized_fact, &person_key(master, user_id));
    format!("{ENTRY_KEY_PREFIX}{}", &digest[..ENTRY_KEY_HASH_LEN.min(digest.len())])
}

/// True for keys made by [`entry_key_with`] (they carry no meaning to show).
pub fn is_entry_key(key: &str) -> bool {
    key.strip_prefix(ENTRY_KEY_PREFIX).is_some_and(|rest| {
        rest.len() == ENTRY_KEY_HASH_LEN && rest.chars().all(|c| c.is_ascii_hexdigit())
    })
}

/// Case, spacing and punctuation do not make a fact new.
pub fn normalize_fact(fact: &str) -> String {
    fact.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// The key under which `fact` is stored for this person.
pub fn fact_key(user_id: Uuid, fact: &str) -> String {
    entry_key_with(&load_master_encryption_key(), user_id, &normalize_fact(fact))
}

/// Whether memory is on for this person (on unless switched off).
pub fn is_enabled(conn: &mut PgConnection, user_id: Uuid) -> bool {
    read_pref(conn, user_id, ENABLED_PREF) != serde_json::Value::Bool(false)
}

pub fn set_enabled(conn: &mut PgConnection, user_id: Uuid, enabled: bool) -> QueryResult<usize> {
    try_write_pref(conn, user_id, ENABLED_PREF, &serde_json::Value::Bool(enabled))
}

/// The person's memories, newest first. With `bot_id`, only that bot's rows
/// and the rows that apply to every bot. Rows that cannot be opened are left
/// out.
pub fn list(
    conn: &mut PgConnection,
    user_id: Uuid,
    bot_id: Option<Uuid>,
    limit: i64,
) -> QueryResult<Vec<MemoryEntry>> {
    let mut query = chat_preferences::table
        .filter(chat_preferences::user_id.eq(user_id))
        .into_boxed();
    if let Some(bot_id) = bot_id {
        query = query.filter(
            chat_preferences::bot_id
                .eq(bot_id)
                .or(chat_preferences::bot_id.is_null()),
        );
    }
    let rows = query
        .order(chat_preferences::updated_at.desc())
        .limit(limit)
        .select((
            chat_preferences::memory_key,
            chat_preferences::memory_value,
            chat_preferences::bot_id,
            chat_preferences::updated_at,
        ))
        .load::<(String, String, Option<Uuid>, DateTime<Utc>)>(conn)?;
    let master = load_master_encryption_key();
    Ok(rows
        .into_iter()
        .filter_map(|(key, stored, bot_id, updated_at)| {
            open_with(&master, user_id, &stored).map(|value| MemoryEntry {
                key,
                value,
                bot_id,
                updated_at,
            })
        })
        .collect())
}

/// Stores `value` under `key`, sealed. UPDATE-then-INSERT works with the
/// COALESCE expression index on (user, bot, key).
pub fn save(
    conn: &mut PgConnection,
    user_id: Uuid,
    bot_id: Option<Uuid>,
    key: &str,
    value: &str,
) -> Result<(), String> {
    let sealed = seal_with(&load_master_encryption_key(), user_id, value)?;
    let updated = diesel::sql_query(
        "UPDATE chat_preferences SET memory_value = $3, updated_at = NOW() \
         WHERE user_id = $1 AND memory_key = $4 \
           AND ((bot_id IS NULL AND $2::uuid IS NULL) OR bot_id = $2::uuid)",
    )
    .bind::<SqlUuid, _>(user_id)
    .bind::<Nullable<SqlUuid>, _>(bot_id)
    .bind::<Text, _>(&sealed)
    .bind::<Text, _>(key)
    .execute(conn)
    .map_err(|error| error.to_string())?;
    if updated > 0 {
        return Ok(());
    }
    diesel::sql_query(
        "INSERT INTO chat_preferences (user_id, bot_id, memory_key, memory_value) \
         VALUES ($1, $2, $3, $4)",
    )
    .bind::<SqlUuid, _>(user_id)
    .bind::<Nullable<SqlUuid>, _>(bot_id)
    .bind::<Text, _>(key)
    .bind::<Text, _>(&sealed)
    .execute(conn)
    .map(|_| ())
    .map_err(|error| error.to_string())
}

/// Deletes the person's rows with these keys. With `bot_id`, only that bot's
/// rows and the rows that apply to every bot.
pub fn delete_keys(
    conn: &mut PgConnection,
    user_id: Uuid,
    bot_id: Option<Uuid>,
    keys: &[String],
) -> QueryResult<usize> {
    let keys: Vec<&str> = keys.iter().map(String::as_str).collect();
    let base = chat_preferences::table
        .filter(chat_preferences::user_id.eq(user_id))
        .filter(chat_preferences::memory_key.eq_any(keys));
    match bot_id {
        Some(bot_id) => diesel::delete(base.filter(
            chat_preferences::bot_id
                .eq(bot_id)
                .or(chat_preferences::bot_id.is_null()),
        ))
        .execute(conn),
        None => diesel::delete(base).execute(conn),
    }
}

/// Deletes every memory of the person.
pub fn clear(conn: &mut PgConnection, user_id: Uuid) -> QueryResult<usize> {
    diesel::delete(chat_preferences::table.filter(chat_preferences::user_id.eq(user_id)))
        .execute(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MASTER: [u8; 32] = [7; 32];

    #[test]
    fn a_sealed_value_opens_for_its_owner_only() {
        let owner = Uuid::new_v4();
        let other = Uuid::new_v4();
        let sealed = seal_with(&MASTER, owner, "Secretaria de Saúde").unwrap();
        assert!(!sealed.contains("Saúde"));
        assert!(looks_sealed(&sealed));
        assert_eq!(open_with(&MASTER, owner, &sealed).as_deref(), Some("Secretaria de Saúde"));
        assert_eq!(open_with(&MASTER, other, &sealed), None);
    }

    #[test]
    fn a_row_written_before_sealing_is_read_as_it_is() {
        let owner = Uuid::new_v4();
        assert_eq!(
            open_with(&MASTER, owner, "prefers short answers: yes").as_deref(),
            Some("prefers short answers: yes")
        );
    }

    #[test]
    fn entry_keys_are_stable_opaque_and_per_person() {
        let first = Uuid::new_v4();
        let second = Uuid::new_v4();
        let key = entry_key_with(&MASTER, first, "meu-setor-e-a-secretaria-de-saude");
        assert_eq!(key, entry_key_with(&MASTER, first, "meu-setor-e-a-secretaria-de-saude"));
        assert_ne!(key, entry_key_with(&MASTER, second, "meu-setor-e-a-secretaria-de-saude"));
        assert!(is_entry_key(&key));
        assert!(!key.contains("saude"));
        assert!(!is_entry_key("language"));
    }

    #[test]
    fn spacing_case_and_punctuation_do_not_make_a_fact_new() {
        assert_eq!(
            normalize_fact("Meu setor é a  Secretaria de Saúde."),
            normalize_fact("meu setor é a secretaria de saúde")
        );
    }
}
