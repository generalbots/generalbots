//! #1545 — Drive as the default knowledge base.
//!
//! The chat client used to send a list of file *names*, so the model could only
//! say "you have a file called budget.xlsx". This indexes the bot's own
//! `.gbdrive` tree (plus the caller's user folder) on the server, picks the
//! documents that match the question and injects their text, so an answer can
//! quote a spreadsheet export or a note without any manual upload to `.gbkb`.
//!
//! The index is deliberately cheap: one listing per bucket+prefix cached for a
//! minute, content read only for the few files that score highest, and a hard
//! timeout so a slow object store never delays a chat turn.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use diesel::{OptionalExtension, RunQueryDsl};
use serde_json::json;
use uuid::Uuid;

use botcore::shared::state::AppState;

const LIST_TTL: Duration = Duration::from_secs(60);
const MAX_DOCS: usize = 3;
const MAX_EXCERPT_CHARS: usize = 3_500;
const MAX_FILE_BYTES: u64 = 2_000_000;
const SEARCH_BUDGET: Duration = Duration::from_secs(8);

/// Extensions whose bytes are worth sending to a language model as text.
const TEXT_EXTENSIONS: &[&str] = &[
    "txt", "md", "csv", "tsv", "json", "jsonl", "html", "htm", "xml", "yaml", "yml",
    "log", "ini", "toml", "sql", "py", "js", "ts", "rs", "sh", "bas", "rst",
];

#[derive(Clone, Debug)]
struct IndexedDoc {
    key: String,
    relative: String,
    size: u64,
}

type ListCache = Mutex<HashMap<String, (Instant, Vec<IndexedDoc>)>>;

fn cache() -> &'static ListCache {
    static CACHE: OnceLock<ListCache> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn is_text_path(path: &str) -> bool {
    let lower = path.to_lowercase();
    lower
        .rsplit_once('.')
        .map(|(_, ext)| TEXT_EXTENSIONS.contains(&ext))
        .unwrap_or(false)
}

/// Tokens long enough to be meaningful when matched against file names and
/// contents (skips articles and prepositions in both languages).
fn keywords(question: &str) -> Vec<String> {
    question
        .split(|c: char| !c.is_alphanumeric())
        .map(str::to_lowercase)
        .filter(|word| word.chars().count() >= 3)
        .collect()
}

/// The caller's Drive folder (`users/{email}/…`) when the account exists.
fn user_prefix(state: &Arc<AppState>, user_id: Uuid) -> Option<String> {
    use diesel::sql_types::{Text, Uuid as SqlUuid};
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
    Some(format!("users/{}/", email.trim()))
}

async fn indexed_docs(
    state: &Arc<AppState>,
    bucket: &str,
    prefix: &str,
) -> Vec<IndexedDoc> {
    let cache_key = format!("{bucket}/{prefix}");
    if let Ok(guard) = cache().lock() {
        if let Some((at, docs)) = guard.get(&cache_key) {
            if at.elapsed() < LIST_TTL {
                return docs.clone();
            }
        }
    }
    let drive = match state.drive.clone() {
        Some(drive) => drive,
        None => return Vec::new(),
    };
    let listed = match drive
        .list_objects_with_metadata(bucket, Some(prefix))
        .await
    {
        Ok(objects) => objects,
        Err(e) => {
            log::info!("drive context: list {prefix} failed: {e}");
            return Vec::new();
        }
    };
    let docs: Vec<IndexedDoc> = listed
        .into_iter()
        .filter(|object| is_text_path(&object.key) && object.size <= MAX_FILE_BYTES)
        .map(|object| IndexedDoc {
            relative: object
                .key
                .strip_prefix(prefix)
                .unwrap_or(&object.key)
                .to_string(),
            key: object.key,
            size: object.size,
        })
        .collect();
    if let Ok(mut guard) = cache().lock() {
        guard.insert(cache_key, (Instant::now(), docs.clone()));
    }
    docs
}

/// Picks the documents a question is most likely about.
fn rank(docs: &[IndexedDoc], words: &[String]) -> Vec<IndexedDoc> {
    let mut scored: Vec<(usize, &IndexedDoc)> = docs
        .iter()
        .filter_map(|doc| {
            let name = doc.relative.to_lowercase();
            let score: usize = words
                .iter()
                .map(|word| {
                    if name.contains(word.as_str()) {
                        4
                    } else {
                        0
                    }
                })
                .sum();
            (score > 0).then_some((score, doc))
        })
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.relative.cmp(&b.1.relative)));
    scored
        .into_iter()
        .take(MAX_DOCS)
        .map(|(_, doc)| doc.clone())
        .collect()
}

/// A readable window around the first keyword hit, or the head of the file.
fn excerpt(content: &str, words: &[String]) -> String {
    let lower = content.to_lowercase();
    let mut byte_hit = words
        .iter()
        .filter_map(|word| lower.find(word.as_str()))
        .min()
        .unwrap_or(0);
    while byte_hit > 0 && !content.is_char_boundary(byte_hit) {
        byte_hit -= 1;
    }
    let hit_char = content[..byte_hit].chars().count();
    let start_char = hit_char.saturating_sub(400);
    content
        .chars()
        .skip(start_char)
        .take(MAX_EXCERPT_CHARS)
        .collect()
}

async fn read_excerpt(
    state: &Arc<AppState>,
    bucket: &str,
    doc: &IndexedDoc,
    words: &[String],
) -> Option<String> {
    let drive = state.drive.clone()?;
    let bytes = drive.get_object(bucket, &doc.key).await.ok()?;
    let text = String::from_utf8_lossy(&bytes).to_string();
    if text.trim().is_empty() {
        return None;
    }
    Some(excerpt(&text, words))
}

/// Injects matching Drive documents as a system message. Never fails a turn:
/// a Drive outage just means the model answers without local context.
pub async fn inject(
    state: &Arc<AppState>,
    bot_uuid: &Uuid,
    user_id: Uuid,
    question: &str,
    messages: &mut Vec<serde_json::Value>,
) {
    let words = keywords(question);
    if words.is_empty() {
        return;
    }
    let location = {
        let mut conn = match state.conn.get() {
            Ok(conn) => conn,
            Err(e) => {
                log::info!("drive context: no db connection: {e}");
                return;
            }
        };
        botbasic_core::utils::bot_drive_location_for(&mut conn, *bot_uuid)
    };
    let mut prefixes = vec![location.key_prefix.clone()];
    if let Some(prefix) = user_prefix(state, user_id) {
        prefixes.push(prefix);
    }
    let work = async {
        let mut docs = Vec::new();
        for prefix in &prefixes {
            if prefix.is_empty() {
                continue;
            }
            docs.extend(indexed_docs(state, &location.bucket, prefix).await);
        }
        let ranked = rank(&docs, &words);
        let mut blocks = Vec::new();
        for doc in ranked {
            if let Some(text) = read_excerpt(state, &location.bucket, &doc, &words).await {
                blocks.push((doc.relative, text));
            }
        }
        blocks
    };
    let blocks = match tokio::time::timeout(SEARCH_BUDGET, work).await {
        Ok(blocks) => blocks,
        Err(_) => {
            log::info!("drive context: lookup timed out");
            return;
        }
    };
    log::info!("drive context: {} matching documents", blocks.len());
    if blocks.is_empty() {
        return;
    }
    let mut block = String::from(
        "The user's Drive contains the documents below. When they answer the \
         question, use their content and cite the file name. Do not invent data \
         that is not in the documents.\n\n",
    );
    for (name, text) in &blocks {
        block.push_str(&format!("--- {name} ---\n{text}\n\n"));
    }
    messages.push(json!({ "role": "system", "content": block }));
    log::info!("drive context: injected {} documents", blocks.len());
}
