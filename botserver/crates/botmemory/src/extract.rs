//! Conversation-side memory extraction. A sampled (or explicitly requested)
//! turn is sent through the injected LLM, which must answer with a strict
//! JSON array of durable facts and preferences; results are stored with the
//! dedupe contract. Every failure path is logged and swallowed — extraction
//! must never disrupt the chat flow.

use uuid::Uuid;

use crate::models::SCOPE_PRIVATE;
use crate::state::MemoryService;
use crate::store;

const EXTRACTION_SYSTEM_PROMPT: &str = "You extract durable long-term memories about a user \
from one conversation turn. Answer with ONLY a strict JSON array, no prose. Each element is an \
object: {\"kind\": string (one of fact|preference|goal|person|skill), \"content\": string \
(concise third-person statement), \"confidence\": number between 0.0 and 1.0}. Extract only \
durable facts and stable preferences that remain true over time. Exclude transient states, \
moods, temporary plans, session context, questions and anything time-bound. When nothing \
durable exists answer with [].";

const EXPLICIT_PHRASES: &[&str] = &["remember that", "lembre que"];
const MAX_EXTRACTED_ITEMS: usize = 8;
const CONTENT_MAX_CHARS: usize = 512;
const KIND_MAX_CHARS: usize = 32;
const KIND_DEFAULT: &str = "fact";
const CONFIDENCE_DEFAULT: f32 = 0.8;

/// One memory proposed by the LLM.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ExtractedMemory {
    pub kind: String,
    pub content: String,
    #[serde(default = "default_confidence")]
    pub confidence: f32,
}

fn default_confidence() -> f32 {
    CONFIDENCE_DEFAULT
}

/// True when the user explicitly asked for something to be remembered.
pub fn is_explicit_remember(user_text: &str) -> bool {
    let lowered = user_text.to_lowercase();
    EXPLICIT_PHRASES.iter().any(|phrase| lowered.contains(phrase))
}

/// Deterministic sampling probe derived from a UUID v4 last byte:
/// `(last_byte % 100) < sample_pct`.
pub fn sample_hit(sample_pct: u8) -> bool {
    let probe = Uuid::new_v4();
    let last_byte = probe.as_bytes()[15];
    (last_byte % 100) < sample_pct
}

fn strip_code_fences(raw: &str) -> &str {
    let trimmed = raw.trim();
    let without_open = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```JSON"))
        .or_else(|| trimmed.strip_prefix("```"))
        .unwrap_or(trimmed);
    without_open.trim().strip_suffix("```").unwrap_or(without_open).trim()
}

fn coerce_item(entry: &serde_json::Value) -> Option<ExtractedMemory> {
    let content = entry.get("content")?.as_str()?.trim();
    if content.is_empty() {
        return None;
    }
    let raw_kind = entry.get("kind").and_then(|v| v.as_str()).unwrap_or(KIND_DEFAULT);
    let lowered_kind: String = raw_kind.trim().to_lowercase().chars().take(KIND_MAX_CHARS).collect();
    let kind = if lowered_kind.is_empty() {
        KIND_DEFAULT.to_string()
    } else {
        lowered_kind
    };
    let confidence = entry
        .get("confidence")
        .and_then(|v| v.as_f64())
        .map(|value| value as f32)
        .unwrap_or(CONFIDENCE_DEFAULT)
        .clamp(0.0, 1.0);
    let clipped: String = content.chars().take(CONTENT_MAX_CHARS).collect();
    Some(ExtractedMemory { kind, content: clipped, confidence })
}

/// Parses the LLM output defensively: accepts fenced or bare JSON arrays,
/// an object wrapper `{\"memories\": [...]}`, and ignores malformed entries.
pub fn parse_extractions(raw: &str) -> Vec<ExtractedMemory> {
    let cleaned = strip_code_fences(raw);

    if let Ok(items) = serde_json::from_str::<Vec<serde_json::Value>>(cleaned) {
        return finalize(items);
    }

    let start = cleaned.find('[');
    let end = cleaned.rfind(']');
    if let (Some(start), Some(end)) = (start, end) {
        if start < end {
            if let Ok(items) = serde_json::from_str::<Vec<serde_json::Value>>(&cleaned[start..=end]) {
                return finalize(items);
            }
        }
    }

    match serde_json::from_str::<serde_json::Value>(cleaned) {
        Ok(wrapper) => wrapper
            .get("memories")
            .and_then(|v| v.as_array())
            .map(|items| finalize(items.clone()))
            .unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

fn finalize(entries: Vec<serde_json::Value>) -> Vec<ExtractedMemory> {
    entries
        .iter()
        .filter_map(coerce_item)
        .take(MAX_EXTRACTED_ITEMS)
        .collect()
}

/// Samples the turn, extracts durable memories via the injected LLM and
/// stores them. Errors are logged and never propagated to the chat path.
pub async fn maybe_extract(
    state: &MemoryService,
    user_id: Uuid,
    branch_id: Option<Uuid>,
    session_id: &str,
    user_text: &str,
    assistant_text: &str,
) {
    if !state.enabled {
        return;
    }
    if !is_explicit_remember(user_text) && !sample_hit(state.sample_pct) {
        return;
    }
    if user_text.trim().is_empty() && assistant_text.trim().is_empty() {
        return;
    }

    let payload = format!(
        "User message:\n{}\n\nAssistant reply:\n{}",
        user_text.trim(),
        assistant_text.trim()
    );
    let raw = match (state.llm_generate)(EXTRACTION_SYSTEM_PROMPT, &payload, "{}") {
        Ok(text) => text,
        Err(e) => {
            tracing::error!("memory extraction LLM call failed for session {session_id}: {e}");
            return;
        }
    };

    let extractions = parse_extractions(&raw);
    if extractions.is_empty() {
        return;
    }

    let conn_result = state.pool.get();
    let mut conn = match conn_result {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("memory extraction could not acquire a connection: {e}");
            return;
        }
    };

    let source = format!("conversation:{session_id}");
    for item in extractions {
        let spec = store::NewMemory {
            org_id: None,
            branch_id,
            owner_user_id: user_id,
            scope: SCOPE_PRIVATE,
            kind: &item.kind,
            content: &item.content,
            source: &source,
            confidence: item.confidence,
            pinned: false,
        };
        if let Err(e) = store::insert(&mut conn, spec) {
            tracing::error!("memory extraction insert failed for session {session_id}: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_fenced_array_with_clamping() {
        let raw = "Sure.\n```json\n[{\"kind\":\"FACT\",\"content\":\"Works at Acme\",\"confidence\":4.2},\
                   {\"content\":\"\"},{\"kind\":\"preference\",\"content\":\"Prefers email contact\"}]\n```";
        let parsed = parse_extractions(raw);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].kind, "fact");
        assert_eq!(parsed[0].confidence, 1.0);
        assert_eq!(parsed[1].confidence, CONFIDENCE_DEFAULT);
    }

    #[test]
    fn parses_object_wrapper_and_bare_array() {
        let wrapped = "{\"memories\":[{\"kind\":\"goal\",\"content\":\"Learns Rust\",\"confidence\":0.5}]}";
        assert_eq!(parse_extractions(wrapped).len(), 1);
        assert_eq!(
            parse_extractions("[{\"kind\":\"fact\",\"content\":\"Has two cats\",\"confidence\":0.7}]").len(),
            1
        );
    }

    #[test]
    fn garbage_yields_no_memories() {
        assert!(parse_extractions("I am just prose").is_empty());
        assert!(parse_extractions("{\"broken\": true").is_empty());
        assert!(parse_extractions("").is_empty());
    }

    #[test]
    fn explicit_phrases_and_sample_bounds() {
        assert!(is_explicit_remember("Please REMEMBER THAT I hate crowds"));
        assert!(is_explicit_remember("Lembre que eu chego tarde"));
        assert!(!is_explicit_remember("What is the weather?"));

        assert!(!sample_hit(0));
        assert!(sample_hit(100));
    }
}
