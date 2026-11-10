//! Start-of-conversation suggestion chips (#2095).
//!
//! `start.bas` publishes chips through Redis; when the cache is flushed (a
//! restart) the static `ADD SUGGESTION` lines are replayed from the script and
//! their keyed labels are resolved for the user's locale.

use axum::extract::ws::Message;
use botcore::shared::state::AppState;
use futures_util::SinkExt;
use log::info;
use std::sync::Arc;
use uuid::Uuid;

/// Replays the static `ADD_SUGGESTION "…"` lines of a bot's start.bas.
/// Redis is the live store, but a restart flushes it; a reopened
/// conversation then had no chips at all because start.bas is skipped for
/// sessions that already have history.
fn suggestions_from_start_bas(
    source: &str,
    locale: &botcore::i18n::Locale,
) -> Vec<botlib::models::Suggestion> {
    let mut suggestions = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('\'') {
            continue;
        }
        // Shipped templates use both spellings ("ADD SUGGESTION" in 167 lines,
        // "ADD_SUGGESTION" in 6); matching only the underscore form left a
        // reopened conversation with no chips after Redis was flushed.
        let upper = trimmed.to_ascii_uppercase();
        if !(upper.starts_with("ADD_SUGGESTION") || upper.starts_with("ADD SUGGESTION")) {
            continue;
        }
        // Skip ADD_SUGGESTION_TOOL/_TEXT: those are context/tool actions and
        // their first quoted value is a tool name, not a display label.
        if !upper[14..].trim_start().starts_with('"') {
            continue;
        }
        let Some(open) = trimmed.find('"') else {
            continue;
        };
        let rest = &trimmed[open + 1..];
        let Some(close) = rest.find('"') else {
            continue;
        };
        // `ADD SUGGESTION "scan" AS "Scan a QR Code"` labels the chip with the
        // AS string; the first quoted value is only the command.
        let after = &rest[close + 1..];
        let after_upper = after.to_ascii_uppercase();
        let label = after_upper.find("AS \"").and_then(|position| {
            let label_rest = &after[position + 4..];
            let end = label_rest.find('"')?;
            Some(label_rest[..end].trim().to_string())
        });
        let text = label
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| rest[..close].trim().to_string());
        let text = resolve_suggestion_label(&text, locale);
        if !text.is_empty() && suggestions.len() < 12 {
            suggestions.push(botlib::models::Suggestion::new(text));
        }
    }
    suggestions
}

/// #2095 — a template authors its chips as catalogue keys
/// (`ADD SUGGESTION "scan" AS "bot-suggest-scan-qr"`); replaying the raw
/// `start.bas` after a cache flush must resolve them, otherwise a reopened
/// conversation would show identifiers instead of labels.
fn resolve_suggestion_label(raw: &str, locale: &botcore::i18n::Locale) -> String {
    let key = raw.trim();
    if !key.is_empty() && botcore::i18n::has_message(key) {
        return botcore::i18n::translate(locale, key, &[]);
    }
    raw.to_string()
}

async fn fallback_suggestions(
    state: &Arc<AppState>,
    bot_name: &str,
    user_id: Uuid,
) -> Vec<botlib::models::Suggestion> {
    let work_path = botcore::shared::utils::get_work_path();
    let rel = format!("{bot_name}.gbdialog/start.bas");
    match botcore::shared::utils::read_bot_file_across_layouts(&work_path, bot_name, &rel).await {
        Some(source) => suggestions_from_start_bas(
            &source,
            &botcore::i18n::locale_for_user_pool(&state.conn, user_id),
        ),
        None => {
            log::debug!("start.bas suggestions unavailable for {bot_name}");
            Vec::new()
        }
    }
}

pub async fn send_start_suggestions(
    state: &Arc<AppState>,
    ws_sender: &mut futures_util::stream::SplitSink<axum::extract::ws::WebSocket, Message>,
    bot_uuid: Uuid,
    session_id: Uuid,
    user_id: Uuid,
    bot_name: &str,
) -> usize {
    // #1369 — both ids are needed as strings several times below; format them
    // once per call instead of once per use. This runs on every session
    // connect, reconnect and after start.bas.
    let bot_id = bot_uuid.to_string();
    let session = session_id.to_string();

    let mut suggestions = {
        #[cfg(feature = "chat")]
        {
            crate::basic::keywords::add_suggestion::get_suggestions(
                state.cache.as_ref(),
                &bot_id,
                &session,
            )
        }
        #[cfg(not(feature = "chat"))]
        Vec::new()
    };
    if suggestions.is_empty() {
        suggestions = fallback_suggestions(state, bot_name, user_id).await;
    }
    let switchers = {
        #[cfg(feature = "chat")]
        {
            crate::basic::keywords::switcher::get_switchers(state.cache.as_ref(), &bot_id, &session)
        }
        #[cfg(not(feature = "chat"))]
        Vec::new()
    };
    let sent = suggestions.len();
    if !suggestions.is_empty() || !switchers.is_empty() {
        info!(
            "ws_handler: sending {} suggestions and {} switchers on reconnect",
            suggestions.len(),
            switchers.len()
        );
        let _ = ws_sender
            .send(Message::Text(
                serde_json::json!({
                    "bot_id": bot_id,
                    "user_id": user_id.to_string(),
                    "session_id": session,
                    "channel": "web",
                    "content": "",
                    "message_type": 2,
                    "is_complete": true,
                    "suggestions": suggestions,
                    "switchers": switchers,
                    "context_length": 0,
                    "context_max_length": 0,
                })
                .to_string(),
            ))
            .await;
    }
    sent
}

#[cfg(test)]
mod suggestion_label_tests {
    use super::{resolve_suggestion_label, suggestions_from_start_bas};

    fn pt_br() -> botcore::i18n::Locale {
        botcore::i18n::Locale::new("pt-BR").unwrap_or_else(botcore::i18n::Locale::default)
    }

    /// Loads the shipped catalogue once; a second call is a no-op.
    fn load_catalogue() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../botlib/locales");
        let _ = botcore::i18n::init_i18n(path);
    }

    /// #2095 — a template authors its chips as catalogue keys; the replay path
    /// (Redis flushed, conversation reopened) must resolve them too.
    #[test]
    fn keyed_labels_resolve_through_the_catalogue() {
        load_catalogue();
        assert!(
            botcore::i18n::has_message("bot-suggest-check-my-balance"),
            "the shipped catalogue must define the template keys"
        );
        assert_eq!(
            resolve_suggestion_label("bot-suggest-check-my-balance", &pt_br()),
            "Consultar saldo"
        );
    }

    /// A custom bot's own wording is never replaced by an identifier.
    #[test]
    fn authored_labels_are_kept() {
        assert_eq!(
            resolve_suggestion_label("Marcar consulta", &pt_br()),
            "Marcar consulta"
        );
    }

    #[test]
    fn replay_prefers_the_as_label_and_resolves_it() {
        load_catalogue();
        let source = "ADD SUGGESTION \"scan\" AS \"bot-suggest-scan-a-qr-code\"\n";
        let suggestions = suggestions_from_start_bas(source, &pt_br());
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].text, "Escanear um QR Code");
    }

    #[test]
    fn replay_keeps_a_literal_label() {
        load_catalogue();
        let source = "ADD SUGGESTION \"help\" AS \"How to use\"\n";
        let suggestions = suggestions_from_start_bas(source, &pt_br());
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].text, "How to use");
    }
}
