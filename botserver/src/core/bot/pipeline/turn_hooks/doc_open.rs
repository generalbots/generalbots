//! Deterministic fast path for "open my last document about X" (#2155 DOC-05).
//!
//! When the user clearly asks to open/find/show a recent document by topic,
//! search docs in Drive and return an actionable deep link instead of letting
//! the LLM wander into vm_exec/approval loops.

use std::sync::Arc;

use botcore::shared::state::AppState;
use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

use super::super::types::SessionIdentity;

pub(in crate::core::bot::pipeline) async fn maybe_open_document(identity: &SessionIdentity<'_>, user_text: &str) -> bool {
    let Some(query) = extract_doc_query(user_text) else {
        return false;
    };
    let SessionIdentity {
        sink,
        state,
        bot_uuid,
        session_id,
        user_id,
        ..
    } = *identity;

    let docs = search_docs(state, user_id, &query).await;
    let message = if docs.is_empty() {
        format!(
            "I couldn't find a document about \"{query}\" in your Drive. You can tell me the exact file name or upload it here."
        )
    } else {
        let best = &docs[0];
        let link = format!("app://docs?id={}", best.id);
        let mut lines = vec![format!(
            "Found **{}**. [Open in app]({link})",
            escape_md(&best.title),
        )];
        if docs.len() > 1 {
            lines.push(String::new());
            lines.push("Other matches:".to_string());
            for d in docs.iter().skip(1).take(4) {
                lines.push(format!(
                    "- [{}](app://docs?id={})",
                    escape_md(&d.title),
                    d.id
                ));
            }
        }
        lines.join("\n")
    };

    let resp = botlib::models::BotResponse::new(
        bot_uuid.to_string(),
        session_id.to_string(),
        user_id.to_string(),
        &message,
        sink.channel_type(),
    );
    if let Err(e) = sink.send_bot_response(&resp).await {
        log::warn!("doc open: response frame not delivered: {e}");
    }
    super::super::native_tools::persist(state, session_id, user_id, &message).await;
    true
}

fn extract_doc_query(text: &str) -> Option<String> {
    let lower = text.to_lowercase();
    let has_open = ["open", "abrir", "show", "mostrar", "find", "encontrar", "localizar", "busque", "buscar"]
        .iter()
        .any(|k| lower.contains(k));
    let has_doc = ["document", "documento", "doc", "file", "arquivo"]
        .iter()
        .any(|k| lower.contains(k));
    if !has_open || !has_doc {
        return None;
    }
    // Strip common request phrasing to get the topic.
    let mut q = lower.clone();
    for prefix in [
        "open my last document about the ",
        "open my last document about ",
        "open my recent document about the ",
        "open my recent document about ",
        "open the last document about the ",
        "open the last document about ",
        "open my last document on ",
        "open my last document ",
        "abra meu último documento sobre o ",
        "abra meu último documento sobre ",
        "abra meu ultimo documento sobre o ",
        "abra meu ultimo documento sobre ",
        "abra o último documento sobre o ",
        "abra o último documento sobre ",
        "abrir meu último documento sobre o ",
        "abrir meu último documento sobre ",
        "mostre meu último documento sobre ",
        "mostre o último documento sobre ",
        "encontre meu último documento sobre ",
    ] {
        if let Some(pos) = q.find(prefix) {
            q = q[pos + prefix.len()..].to_string();
            break;
        }
    }
    // Fallback: take words after "about/sobre"
    if q == lower {
        if let Some(pos) = lower.find(" about ") {
            q = lower[pos + 7..].to_string();
        } else if let Some(pos) = lower.find(" sobre ") {
            q = lower[pos + 7..].to_string();
        } else {
            return None;
        }
    }
    let query = q
        .trim()
        .trim_matches(|c: char| c == '"' || c == '\'' || c == '.' || c == '?' || c == '!')
        .trim()
        .to_string();
    if query.is_empty() { None } else { Some(query) }
}

struct DocHit {
    id: String,
    title: String,
    updated_at: DateTime<Utc>,
}

async fn search_docs(state: &Arc<AppState>, user_id: Uuid, query: &str) -> Vec<DocHit> {
    let Some(drive) = state.drive.as_ref() else {
        return Vec::new();
    };
    let bucket = state.bucket_name.as_str();
    let owner = user_id.to_string();
    let sanitized: String = owner
        .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
        .to_lowercase();
    let prefix = format!("users/{sanitized}/docs/");
    let objects = match drive.list_objects(bucket, Some(&prefix)).await {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };
    let q_lower = query.to_lowercase();
    let mut hits = Vec::new();
    for key in &objects {
        if !key.ends_with(".meta.json") {
            continue;
        }
        let Ok(bytes) = drive.get_object(bucket, key).await else {
            continue;
        };
        let Ok(meta) = serde_json::from_slice::<Value>(&bytes) else {
            continue;
        };
        let title = meta.get("title").and_then(|v| v.as_str()).unwrap_or("Untitled");
        let title_lower = title.to_lowercase();
        if !title_lower.contains(&q_lower) {
            // also try matching individual query words
            let words: Vec<&str> = q_lower.split_whitespace().filter(|w| w.len() > 2).collect();
            if words.is_empty() || !words.iter().all(|w| title_lower.contains(w)) {
                continue;
            }
        }
        let id = meta.get("id").and_then(|v| v.as_str()).map_or_else(Default::default, |v| v).to_string();
        if id.is_empty() {
            continue;
        }
        let updated_at = meta
            .get("updated_at")
            .and_then(|v| v.as_str())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|d| d.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);
        hits.push(DocHit { id, title: title.to_string(), updated_at });
    }
    hits.sort_by_key(|h| std::cmp::Reverse(h.updated_at));
    hits
}

fn escape_md(s: &str) -> String {
    s.replace('[', "\\[").replace(']', "\\]")
}
