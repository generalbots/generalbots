//! #2059 / #2060 / #2061 — compact connector catalogue injected before the
//! model answers.
//!
//! Integrations were only reachable when the user `@`-mentioned one: the prompt
//! carried the app catalogue and the UI hints but nothing about connectors, so
//! "post this on Instagram" matched nothing unless the model volunteered a
//! search. This block names what is connected, what matches the sentence (with
//! its capability and least privilege) and what is catalogue-only, without
//! listing 153 providers and without any credential value.
//!
//! It also carries the connect contract (#2060): credential fields come from
//! `integrations.connect.start`, OAuth providers use the existing
//! `/api/bots/<bot_id>/integrations/oauth/<provider>/start` route, and the
//! connection is proven with a read action before the original request resumes.
//!
//! Declines (#2061) are remembered per user and provider so a setup is offered
//! once; reopening the subject re-arms the offer.

use std::collections::HashSet;
use std::sync::{Arc, Mutex, OnceLock};

use botcore::shared::state::AppState;
use uuid::Uuid;

use crate::apps::integration_catalog::LlmProviderSummary;
#[cfg(feature = "integrations")]
use diesel::prelude::*;

use super::connector_match::{
    split_matches, ConnectorMatch, MAX_AVAILABLE, MAX_CATALOGUE_ONLY, MIN_TOKEN_LEN,
};

/// Message words are matched one by one, so the work stays bounded no matter
/// how long the user turn is.
const MAX_INTENT_TOKENS: usize = 16;
/// Cap on the tenant's own connections; the block never grows past this.
const MAX_CONNECTED: usize = 12;

/// Words that carry no capability signal; matching them would only add noise.
const STOPWORDS: &[&str] = &[
    "the", "this", "that", "these", "those", "with", "from", "into", "onto", "your", "yours",
    "and", "for", "you", "please", "can", "could", "would", "should", "have", "has", "was", "are",
    "how", "what", "when", "where", "which", "who", "why", "some", "any", "all", "not", "but",
    "then", "than", "them", "they", "here", "there", "today", "tomorrow", "agora", "quero", "pode",
    "para", "com", "uma", "esse", "essa", "isso", "esta", "este", "aqui", "amanha", "hoje", "voce",
    "meu", "minha",
];

/// Connected providers (slug, chat-executable action count).
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
                let count =
                    botintegrations::providers::implemented_action_names(&row.provider_slug).len();
                (row.provider_slug, count)
            })
            .collect()
    }
}

/// #2061 — providers this user already declined. A decline is remembered for
/// the process (session lifetime) so the assistant proposes a setup once
/// instead of nagging on every turn; it never hides a provider the user
/// connects explicitly or reopens the subject about.
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
        .map(|declined| declined.contains(&(user_id, provider.trim().to_ascii_lowercase())))
        .unwrap_or(false)
}

/// Words of the user turn that can name a provider or a capability.
pub(super) fn intent_tokens(query: &str) -> Vec<String> {
    let mut tokens: Vec<String> = Vec::new();
    for raw in query.split(|character: char| !character.is_alphanumeric()) {
        let token = raw.trim().to_lowercase();
        if token.chars().count() < MIN_TOKEN_LEN || STOPWORDS.contains(&token.as_str()) {
            continue;
        }
        if !tokens.iter().any(|seen| seen == &token) {
            tokens.push(token);
        }
        if tokens.len() == MAX_INTENT_TOKENS {
            break;
        }
    }
    tokens
}

/// Fixed instructions — the only part of the block that is always present.
/// They carry the connect contract (#2060) and the offer rules (#2061).
const INSTRUCTIONS: &str = "\
To act: integrations.actions.list {provider} then integrations.invoke {provider, action, params}; \
only call actions whose \"implemented\" flag is true.
To connect: integrations.connect.start {provider} returns that provider's own credential fields \
(secret fields are typed into the form, never into chat), its setup steps, least_privilege and the \
connect path; for an oauth2 provider open GET /api/bots/<bot_id>/integrations/oauth/<provider>/start \
and send the link. Ask for the fields one at a time in the provider's order, show the matching setup \
step (step 2 of 4), say secrets are stored in the vault, and offer app://integrations?provider=<provider> \
for the full form. After connecting, prove it with a harmless read action through integrations.invoke \
before the real one, then finish the original request without asking the user to repeat it.
Offer a setup at most once per turn and per provider, only for a provider matched above, and state \
what it will and will not do. If the user declines, call integrations.decline {provider} and do not \
offer it again unless the user reopens that provider's subject.
Catalogue-only providers are listed but not executable: say so instead of offering them, and never \
promise an action or a credential field the provider does not implement. The full catalogue stays on \
demand with integrations.catalog.search.
Never ask for, repeat or store a credential, token or scope value in the reply.";

/// The block itself. Kept pure so its size can be pinned by a test: it must not
/// grow with the number of providers, and it never carries a credential.
pub(super) fn render_block(
    connected: &[(String, usize)],
    available: &[ConnectorMatch],
    catalogue_only: &[LlmProviderSummary],
) -> Option<String> {
    if connected.is_empty() && available.is_empty() && catalogue_only.is_empty() {
        return None;
    }
    let mut text = String::from("Connector catalogue (compact):\n");
    if !connected.is_empty() {
        let listed: Vec<String> = connected
            .iter()
            .take(MAX_CONNECTED)
            .map(|(slug, count)| format!("{slug} ({count} actions)"))
            .collect();
        text.push_str(&format!("Connected: {}\n", listed.join(", ")));
    }
    if !available.is_empty() {
        let listed: Vec<String> = available
            .iter()
            .take(MAX_AVAILABLE)
            .map(|matched| {
                let mut detail = vec![
                    matched.summary.category.as_str().to_string(),
                    matched.auth_method.clone(),
                    format!("{} actions", matched.summary.action_count),
                ];
                if !matched.capability.is_empty() {
                    detail.push(matched.capability.clone());
                }
                let mut line = format!("{} ({})", matched.summary.id, detail.join(", "));
                if !matched.least_privilege.is_empty() {
                    line.push_str(&format!("; least privilege: {}", matched.least_privilege));
                }
                line
            })
            .collect();
        text.push_str(&format!(
            "Matched to this request: {}\n",
            listed.join(" | ")
        ));
    }
    if !catalogue_only.is_empty() {
        let listed: Vec<String> = catalogue_only
            .iter()
            .take(MAX_CATALOGUE_ONLY)
            .map(|provider| format!("{} ({})", provider.id, provider.name))
            .collect();
        text.push_str(&format!(
            "Catalogue-only, not executable yet (say so, do not offer): {}\n",
            listed.join(", ")
        ));
    }
    text.push_str(INSTRUCTIONS);
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
    let tokens = intent_tokens(query);
    let (available, catalogue_only) = split_matches(user_id, query, &tokens, &connected);
    let Some(block) = render_block(&connected, &available, &catalogue_only) else {
        return;
    };
    if let Some(array) = messages.as_array_mut() {
        array.push(serde_json::json!({ "role": "system", "content": block }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "integrations")]
    use crate::apps::integration_catalog;

    #[test]
    fn an_empty_catalogue_injects_nothing() {
        assert!(render_block(&[], &[], &[]).is_none());
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
    fn intent_tokens_drop_stopwords_and_stay_bounded() {
        let tokens = intent_tokens("Please could you post this on Instagram tomorrow");
        assert!(tokens.contains(&"instagram".to_string()));
        assert!(tokens.contains(&"post".to_string()));
        assert!(!tokens.contains(&"this".to_string()));
        assert!(!tokens.contains(&"please".to_string()));
        let long = "alpha bravo charlie delta echo foxtrot golf hotel india juliet kilo lima mike \
                    november oscar papa quebec romeo sierra tango uniform victor whiskey";
        assert!(intent_tokens(long).len() <= MAX_INTENT_TOKENS);
    }

    #[cfg(feature = "integrations")]
    fn match_of(id: &str, token: &str) -> ConnectorMatch {
        let summary = integration_catalog::llm_search(Some(id), None, None)
            .into_iter()
            .find(|provider| provider.id == id)
            .expect("provider in catalogue");
        super::super::connector_match::describe(summary, &[token.to_string()])
    }

    #[cfg(feature = "integrations")]
    #[test]
    fn the_block_lists_connected_and_offers_the_matched_provider() {
        let connected = vec![("slack".to_string(), 4usize)];
        let available = vec![match_of("instagram", "post")];
        let block = render_block(&connected, &available, &[]).expect("block");
        assert!(block.contains("slack (4 actions)"));
        assert!(block.contains("instagram (social_messaging, oauth2"));
        assert!(block.contains("least privilege:"));
        assert!(block.contains("integrations.actions.list"));
        assert!(block.contains("integrations.connect.start"));
        assert!(block.contains("integrations.invoke"));
        assert!(block.contains("oauth/<provider>/start"));
        assert!(
            block.len() < 3000,
            "prompt block grew to {} bytes",
            block.len()
        );
        assert!(!block.to_lowercase().contains("access_token"));
        assert!(!block.to_lowercase().contains("client_secret"));
    }

    #[cfg(feature = "integrations")]
    #[test]
    fn the_block_asks_for_a_single_offer_and_records_a_decline() {
        let available = vec![match_of("instagram", "post")];
        let rendered = render_block(&[], &available, &[]).expect("block");
        assert!(rendered.contains("at most once"));
        assert!(rendered.contains("integrations.decline"));
    }

    #[cfg(feature = "integrations")]
    #[test]
    fn a_catalogue_only_provider_is_reported_honestly() {
        let catalogue_only: Vec<LlmProviderSummary> =
            integration_catalog::llm_search(None, None, None)
                .into_iter()
                .filter(|provider| !provider.llm_available)
                .take(1)
                .collect();
        let Some(provider) = catalogue_only.first() else {
            return;
        };
        let rendered = render_block(&[], &[], &catalogue_only).expect("block");
        assert!(rendered.contains(provider.id));
        assert!(rendered.contains("not executable yet"));
        assert!(rendered.contains("do not offer"));
    }

    #[cfg(feature = "integrations")]
    #[test]
    fn the_block_size_is_independent_of_the_catalogue() {
        let all = integration_catalog::llm_search(None, None, None);
        assert!(
            all.len() > 100,
            "catalogue shrank to {} providers",
            all.len()
        );
        let tokens = vec!["instagram".to_string()];
        let describe_all = |count: usize| -> Vec<ConnectorMatch> {
            all.iter()
                .take(count)
                .cloned()
                .map(|summary| super::super::connector_match::describe(summary, &tokens))
                .collect()
        };
        let small = render_block(&[], &describe_all(3), &[]).expect("block");
        let large = render_block(&[], &describe_all(all.len()), &[]).expect("block");
        assert_eq!(small.len(), large.len(), "block grew with the catalogue");
        // The same holds for the tenant's own connections.
        let connected: Vec<(String, usize)> = (0..40)
            .map(|index| (format!("provider_{index}"), index as usize))
            .collect();
        let wide = render_block(&connected, &describe_all(3), &[]).expect("block");
        assert!(wide.len() < 3000, "block grew to {} bytes", wide.len());
    }
}
