//! Pages the person named, read before the model call for an engine that
//! cannot call `web_fetch` itself (the subscription agents answer from the
//! prompt alone, so without this they said they could not reach the web).
//!
//! Same rules as the tool: only addresses written in the message, at most
//! [`PAGES_PER_TURN`] pages, every fetch through the SSRF guard with its size
//! and time limits, and a visible `web_fetch` step naming the site. Each
//! page's text becomes a system note of this turn starting with
//! "[Página lida: <site>]".

use std::sync::Arc;

use botcore::shared::state::AppState;
use uuid::Uuid;

use super::super::types::SessionIdentity;
use super::super::workspace_progress::WorkspaceProgress;
use super::web_fetch::{read_page, PAGES_PER_TURN, PHASE, TOOL};
use super::web_text::{addresses_in, site_of};

/// Whether this turn's engine can call the native tools (`web_fetch`).
/// A subscription agent, chosen in the menu or as the bot's provider,
/// cannot; an API engine can.
fn engine_calls_tools(
    state: &Arc<AppState>,
    bot_uuid: Uuid,
    model_preference: &str,
) -> bool {
    #[cfg(feature = "agent-vm")]
    if let Some((agent, _)) = crate::core::ai_accounts::models::parse_choice(model_preference) {
        return !crate::core::ai_accounts::is_subscription_provider(agent);
    }
    if botsettings::model_config::accounts::parse_preference(model_preference).is_some() {
        return true;
    }
    let provider = botcore::config::ConfigManager::new(state.conn.clone())
        .get_config(&bot_uuid, "llm-provider", Some(""))
        .unwrap_or_default();
    !crate::core::ai_accounts::is_subscription_provider(&provider)
}

/// The addresses this turn reads ahead: those written in the message, in
/// order, within the per-answer page limit.
fn named_pages(user_text: &str) -> Vec<String> {
    addresses_in(user_text)
        .into_iter()
        .take(PAGES_PER_TURN)
        .collect()
}

/// The system note one read leaves for the model.
fn page_note(site: &str, read: bool, text: &str) -> String {
    let head = if read {
        format!("[Página lida: {site}]")
    } else {
        format!("[Página não lida: {site}]")
    };
    format!("{head}\n{text}")
}

/// This turn's web reading: `web_fetch` for an engine that calls tools, the
/// named pages read ahead for one that does not.
pub(crate) async fn apply(
    identity: &SessionIdentity<'_>,
    parsed: &serde_json::Value,
    model_preference: &str,
    user_text: &str,
    messages: &mut Vec<serde_json::Value>,
) {
    if engine_calls_tools(identity.state, identity.bot_uuid, model_preference) {
        super::web_fetch::apply(parsed, identity.session_id, user_text, messages);
    } else {
        read_named_pages(identity, user_text, messages).await;
    }
}

/// Reads the pages named in `user_text` and adds one system note per page.
async fn read_named_pages(
    identity: &SessionIdentity<'_>,
    user_text: &str,
    messages: &mut Vec<serde_json::Value>,
) {
    let addresses = named_pages(user_text);
    if addresses.is_empty() {
        return;
    }
    let progress = WorkspaceProgress::new(identity);
    for address in addresses {
        let site = site_of(&address).unwrap_or_default();
        if let Some(progress) = progress.as_ref() {
            progress.start(PHASE, TOOL, "Reading a web page", &site).await;
        }
        let (status, title, text) = read_page(&address, &site).await;
        if let Some(progress) = progress.as_ref() {
            progress.finish(PHASE, TOOL, status, &site).await;
        }
        let read = status == "ok";
        super::super::turn_record::note_page(identity.session_id, &site, Some(&title), read);
        messages.push(serde_json::json!({
            "role": "system",
            "content": page_note(&site, read, &text),
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_written_addresses_are_read_within_the_limit() {
        let text = "compare a.example.org, b.example.org, c.example.org, d.example.org, \
                    e.example.org e f.example.org";
        assert_eq!(named_pages(text).len(), PAGES_PER_TURN);
        assert!(named_pages("resuma o texto-colado.txt").is_empty());
    }

    #[test]
    fn a_note_starts_with_the_site() {
        assert!(page_note("example.com", true, "Title: Example").starts_with("[Página lida: example.com]\n"));
        assert!(page_note("example.com", false, "erro").starts_with("[Página não lida: example.com]"));
    }
}
