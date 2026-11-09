//! #2059 — compact connector catalogue injected before the model answers.
//!
//! Integrations were only reachable when the user `@`-mentioned one: the prompt
//! carried the app catalogue and the UI hints but nothing about connectors, so
//! "post this on Instagram" matched nothing unless the model volunteered a
//! search. This block names what is connected (and what can be connected,
//! matched to the message) without listing 153 providers or any credential.
//!
//! Two steps stay intact: the catalogue is only fetched on demand through
//! `integrations.catalog.search`, and a single provider's action list through
//! `integrations.actions.list`.

use std::collections::HashSet;
use std::sync::{Arc, Mutex, OnceLock};

use botcore::shared::state::AppState;
use uuid::Uuid;

use crate::apps::integration_catalog::LlmProviderSummary;
#[cfg(feature = "integrations")]
use crate::apps::integration_catalog;
#[cfg(feature = "integrations")]
use diesel::prelude::*;

/// Connected providers (slug, chat-executable action count), newest first.
fn connected_providers(state: &Arc<AppState>, bot_uuid: Uuid) -> Vec<(String, usize)> {
    #[cfg(not(feature = "integrations"))]
    {
        let _ = (state, bot_uuid);
        return Vec::new();
    }
    #[cfg(feature = "integrations")]
    {
        #[derive(diesel::QueryableByName)]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::Text)]
            provider_slug: String,
        }
        let Ok(mut conn) = state.conn.get() else {
            return Vec::new();
        };
        let rows: Vec<Row> = diesel::sql_query(
            "SELECT DISTINCT provider_slug FROM integration_connections
              WHERE bot_id = $1 AND status <> 'revoked'
              ORDER BY provider_slug",
        )
        .bind::<diesel::sql_types::Uuid, _>(bot_uuid)
        .load(&mut conn)
        .unwrap_or_default();
        rows.into_iter()
            .map(|row| {
                let count = botintegrations::providers::implemented_action_names(
                    &row.provider_slug,
                )
                .len();
                (row.provider_slug, count)
            })
            .collect()
    }
}

/// #2061 — providers this user already declined. A decline is remembered for
/// the process so the assistant proposes a setup once instead of nagging on
/// every turn; it never hides a provider the user connects explicitly.
fn declines() -> &'static Mutex<HashSet<(Uuid, String)>> {
    static DECLINES: OnceLock<Mutex<HashSet<(Uuid, String)>>> = OnceLock::new();
    DECLINES.get_or_init(|| Mutex::new(HashSet::new()))
}

/// Records that the user declined a provider's setup offer.
pub(crate) fn remember_decline(user_id: Uuid, provider: &str) {
    if let Ok(mut declined) = declines().lock() {
        declined.insert((user_id, provider.trim().to_ascii_lowercase()));
    }
}

/// Whether the user already declined this provider.
pub(super) fn declined(user_id: Uuid, provider: &str) -> bool {
    declines()
        .lock()
        .map(|declined| {
            declined.contains(&(user_id, provider.trim().to_ascii_lowercase()))
        })
        .unwrap_or(false)
}

/// At most three providers that can be connected and match this message,
/// minus the ones this user already declined.
fn available_providers(
    user_id: Uuid,
    query: &str,
    connected: &[(String, usize)],
) -> Vec<LlmProviderSummary> {
    #[cfg(not(feature = "integrations"))]
    {
        let _ = (user_id, query, connected);
        return Vec::new();
    }
    #[cfg(feature = "integrations")]
    {
        if query.trim().is_empty() {
            return Vec::new();
        }
        integration_catalog::llm_search(Some(query), None, None)
            .into_iter()
            .filter(|provider| provider.llm_available)
            .filter(|provider| {
                !connected
                    .iter()
                    .any(|(slug, _)| slug.eq_ignore_ascii_case(&provider.id))
            })
            .filter(|provider| !declined(user_id, provider.id))
            .take(3)
            .collect()
    }
}

/// The block itself. Kept pure so its size can be pinned by a test: it must not
/// grow with the number of providers, and it never carries a credential.
pub(super) fn render_block(
    connected: &[(String, usize)],
    available: &[LlmProviderSummary],
) -> Option<String> {
    if connected.is_empty() && available.is_empty() {
        return None;
    }
    let mut text = String::from("Connector catalogue (compact):\n");
    if !connected.is_empty() {
        let listed: Vec<String> = connected
            .iter()
            .take(12)
            .map(|(slug, count)| format!("{slug} ({count} actions)"))
            .collect();
        text.push_str(&format!("Connected: {}\n", listed.join(", ")));
    }
    if !available.is_empty() {
        let listed: Vec<String> = available
            .iter()
            .map(|provider| {
                format!(
                    "{} ({}, {} actions)",
                    provider.id,
                    provider.category.as_str(),
                    provider.action_count
                )
            })
            .collect();
        text.push_str(&format!(
            "Available to connect (matched to this request): {}\n",
            listed.join(", ")
        ));
    }
    text.push_str(
        "To act: integrations.actions.list {provider} then \
         integrations.invoke {provider, action, params}.\n\
         To connect: integrations.connect.start {provider}.\n\
         Offer a setup at most once per provider. If the user declines, call \
         integrations.decline {provider} and do not offer it again.\n\
         The full catalogue is fetched only on demand with \
         integrations.catalog.search. Never invent an action: list them first. \
         Never ask for or repeat a credential, token or scope value in the reply.",
    );
    Some(text)
}

/// Appends the block as one system message right before the user turn.
pub(super) fn inject(
    state: &Arc<AppState>,
    bot_uuid: Uuid,
    user_id: Uuid,
    query: &str,
    messages: &mut serde_json::Value,
) {
    let connected = connected_providers(state, bot_uuid);
    let available = available_providers(user_id, query, &connected);
    let Some(block) = render_block(&connected, &available) else {
        return;
    };
    if let Some(array) = messages.as_array_mut() {
        array.push(serde_json::json!({ "role": "system", "content": block }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_block_lists_connected_and_at_most_three_available() {
        let connected = vec![("slack".to_string(), 4usize)];
        #[cfg(feature = "integrations")]
        let available = crate::apps::integration_catalog::llm_search(Some("instagram"), None, None);
        #[cfg(not(feature = "integrations"))]
        let available: Vec<LlmProviderSummary> = Vec::new();

        let block = render_block(&connected, &available).expect("block");
        assert!(block.contains("slack (4 actions)"));
        assert!(block.contains("integrations.actions.list"));
        assert!(block.contains("integrations.connect.start"));
        assert!(block.len() < 2000, "prompt block grew to {} bytes", block.len());
        let listed = available.iter().filter(|p| block.contains(&p.id)).count();
        assert!(listed <= 3);
        assert!(!block.to_lowercase().contains("access_token"));
        assert!(!block.to_lowercase().contains("client_secret"));
    }

    #[test]
    fn an_empty_catalogue_injects_nothing() {
        assert!(render_block(&[], &[]).is_none());
    }

    #[test]
    fn a_decline_is_remembered_per_user_and_provider() {
        let user = Uuid::new_v4();
        assert!(!declined(user, "instagram"));
        remember_decline(user, "Instagram");
        assert!(declined(user, "instagram"));
        // Another user still gets the offer.
        assert!(!declined(Uuid::new_v4(), "instagram"));
    }

    #[test]
    fn the_block_asks_for_a_single_offer_and_records_a_decline() {
        let block = render_block(&[], &[]).unwrap_or_default();
        assert!(block.is_empty());
        #[cfg(feature = "integrations")]
        {
            let available = crate::apps::integration_catalog::llm_search(
                Some("instagram"),
                None,
                None,
            );
            let rendered = render_block(&[], &available).expect("block");
            assert!(rendered.contains("at most once"));
            assert!(rendered.contains("integrations.decline"));
        }
    }
}
