//! #2059 / #2061 — which connectors the user's sentence is asking for.
//!
//! The catalogue answers one substring at a time, so a whole sentence ("post
//! this on Instagram") never matched anything. This module turns the turn into
//! keywords, scores the providers each keyword hits, and keeps the ones that can
//! be connected — while reporting catalogue-only providers honestly (#2061).

use uuid::Uuid;

#[cfg(feature = "integrations")]
use crate::apps::integration_catalog;
use crate::apps::integration_catalog::LlmProviderSummary;

#[cfg(feature = "integrations")]
use super::connector_hints::declined;

/// Shortest word that can name a provider or a capability ("aws", "s3", "post").
pub(super) const MIN_TOKEN_LEN: usize = 3;
/// Hard caps: the offer stays a handful of lines, never a catalogue dump.
pub(super) const MAX_AVAILABLE: usize = 3;
pub(super) const MAX_CATALOGUE_ONLY: usize = 2;
const MAX_CAPABILITY_CHARS: usize = 60;
const MAX_SCOPE_CHARS: usize = 90;

/// Words that mean the user reopened the connector subject on purpose, which
/// re-arms an offer they declined earlier in the session (#2061).
const REARM_CUES: &[&str] = &[
    "connect",
    "connecting",
    "integration",
    "integrate",
    "setup",
    "set up",
    "configure",
    "configurar",
    "conectar",
    "ativar",
    "enable",
    "how do",
    "como",
    "what can",
    "o que",
];

/// One provider that can be connected and matches the user's sentence.
#[derive(Debug, Clone)]
pub(super) struct ConnectorMatch {
    pub summary: LlmProviderSummary,
    /// `oauth2`, `api_key`, `token`, `basic`, … — from the provider's profile.
    pub auth_method: String,
    /// The provider's own least-privilege text, bounded and never a value.
    pub least_privilege: String,
    /// The matched action summary, e.g. "Publish a social post.".
    pub capability: String,
}

/// Whether the user named this provider ("instagram", "google drive").
#[cfg(feature = "integrations")]
fn names_provider(tokens: &[String], provider_id: &str) -> bool {
    let id = provider_id.trim().to_ascii_lowercase();
    if id.is_empty() {
        return false;
    }
    if tokens.iter().any(|token| token == &id) {
        return true;
    }
    let words: Vec<&str> = id
        .split('_')
        .filter(|word| word.chars().count() >= MIN_TOKEN_LEN)
        .collect();
    words.len() > 1
        && words
            .iter()
            .all(|word| tokens.iter().any(|token| token == word))
}

/// A decline silences the offer; asking *about* the connector (naming it while
/// asking how to connect it) re-arms it, repeating the same request does not.
#[cfg(feature = "integrations")]
fn rearms(query: &str, tokens: &[String], provider_id: &str) -> bool {
    if !names_provider(tokens, provider_id) {
        return false;
    }
    let normalized = query.to_lowercase();
    REARM_CUES.iter().any(|cue| normalized.contains(cue))
}

#[cfg(feature = "integrations")]
fn truncate(text: &str, max_chars: usize) -> String {
    let trimmed = text.trim();
    if trimmed.chars().count() <= max_chars {
        return trimmed.to_string();
    }
    let mut shortened: String = trimmed.chars().take(max_chars.saturating_sub(1)).collect();
    shortened.push('…');
    shortened
}

/// Provider matches, ordered by how many message words hit them (stable by id).
/// When the sentence names providers explicitly ("post this on Instagram"),
/// only those are returned — a capability word alone never dilutes the answer.
#[cfg(feature = "integrations")]
fn matched_providers(tokens: &[String]) -> Vec<LlmProviderSummary> {
    let mut scored: Vec<(usize, LlmProviderSummary)> = Vec::new();
    for token in tokens {
        for provider in integration_catalog::llm_search(Some(token), None, None) {
            match scored.iter_mut().find(|(_, seen)| seen.id == provider.id) {
                Some((score, _)) => *score += 1,
                None => scored.push((1, provider)),
            }
        }
    }
    scored.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.id.cmp(right.1.id)));
    let mut ordered: Vec<LlmProviderSummary> =
        scored.into_iter().map(|(_, provider)| provider).collect();
    if ordered
        .iter()
        .any(|provider| names_provider(tokens, provider.id))
    {
        ordered.retain(|provider| names_provider(tokens, provider.id));
    }
    ordered
}

/// Action verbs that publish or change something, preferred when the sentence
/// asks for an outcome ("post this") rather than a read.
const WRITE_VERBS: &[&str] = &[
    "create", "publish", "post", "send", "upload", "schedule", "add", "reply", "update", "delete",
];

/// The action that best explains why this provider matched the sentence. A
/// write action wins over a read one, so "post this" explains publishing rather
/// than listing (#2061 "why: publish a post").
#[cfg(feature = "integrations")]
fn capability_for(provider_id: &str, tokens: &[String]) -> String {
    let Some(actions) = integration_catalog::llm_actions(provider_id) else {
        return String::new();
    };
    let mut best: Option<(u32, &'static str)> = None;
    for action in &actions {
        let key = action.name.to_lowercase();
        let summary = action.summary.to_lowercase();
        let verb = key.rsplit('.').next().unwrap_or(key.as_str());
        let mut score = 0u32;
        for token in tokens {
            // `LlmAction` carries the key and the human summary, not a label.
            if key.contains(token.as_str()) {
                score += 3;
            }
            if summary.contains(token.as_str()) {
                score += 1;
            }
        }
        if score > 0 && WRITE_VERBS.contains(&verb) {
            score += 5;
        }
        if score > best.map_or(0, |(best_score, _)| best_score) {
            best = Some((score, action.summary));
        }
    }
    best.map(|(_, summary)| truncate(summary, MAX_CAPABILITY_CHARS))
        .unwrap_or_default()
}

/// Adds the auth method and least privilege the offer must state (#2061).
#[cfg(feature = "integrations")]
pub(super) fn describe(summary: LlmProviderSummary, tokens: &[String]) -> ConnectorMatch {
    let item = integration_catalog::provider_by_id(summary.id);
    let auth_method = item
        .as_ref()
        .and_then(|item| serde_json::to_value(item.auth.method).ok())
        .and_then(|value| value.as_str().map(str::to_string))
        .unwrap_or_default();
    let least_privilege = item
        .as_ref()
        .map(|item| truncate(item.auth.least_privilege, MAX_SCOPE_CHARS))
        .unwrap_or_default();
    let capability = capability_for(summary.id, tokens);
    ConnectorMatch {
        summary,
        auth_method,
        least_privilege,
        capability,
    }
}

/// Splits the matches into "can be connected now" and "catalogue only".
///
/// The truthful signal is the adapter registry (`llm_available`), not the seed
/// status: Instagram ships as `partial` yet executes today, so gating on
/// `built` alone would drop the very provider the user asked for.
pub(super) fn split_matches(
    user_id: Uuid,
    query: &str,
    tokens: &[String],
    connected: &[(String, usize)],
) -> (Vec<ConnectorMatch>, Vec<LlmProviderSummary>) {
    #[cfg(not(feature = "integrations"))]
    {
        let _ = (user_id, query, tokens, connected);
        (Vec::new(), Vec::new())
    }
    #[cfg(feature = "integrations")]
    {
        let mut available: Vec<ConnectorMatch> = Vec::new();
        let mut catalogue_only: Vec<LlmProviderSummary> = Vec::new();
        for provider in matched_providers(tokens) {
            if connected
                .iter()
                .any(|(slug, _)| slug.eq_ignore_ascii_case(provider.id))
            {
                continue;
            }
            if declined(user_id, provider.id) && !rearms(query, tokens, provider.id) {
                continue;
            }
            if provider.llm_available {
                if available.len() < MAX_AVAILABLE {
                    available.push(describe(provider, tokens));
                }
            } else if catalogue_only.len() < MAX_CATALOGUE_ONLY {
                catalogue_only.push(provider);
            }
        }
        (available, catalogue_only)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "integrations")]
    use crate::core::bot::pipeline::connector_hints::{intent_tokens, remember_decline};

    #[test]
    fn a_sentence_matches_its_provider_and_its_capability() {
        #[cfg(feature = "integrations")]
        {
            let query = "post this on Instagram";
            let tokens = intent_tokens(query);
            let (available, _) = split_matches(Uuid::new_v4(), query, &tokens, &[]);
            let instagram = available
                .iter()
                .find(|matched| matched.summary.id == "instagram")
                .expect("instagram matched without an @mention");
            assert_eq!(instagram.auth_method, "oauth2");
            assert!(!instagram.least_privilege.is_empty());
            assert!(instagram.capability.to_lowercase().contains("post"));
            assert!(available.len() <= MAX_AVAILABLE);
        }
    }

    #[test]
    fn a_connected_provider_is_never_offered_as_available() {
        #[cfg(feature = "integrations")]
        {
            let query = "post this on Instagram";
            let tokens = intent_tokens(query);
            let connected = vec![("instagram".to_string(), 4usize)];
            let (available, _) = split_matches(Uuid::new_v4(), query, &tokens, &connected);
            assert!(!available
                .iter()
                .any(|matched| matched.summary.id == "instagram"));
        }
    }

    #[test]
    fn a_decline_silences_the_offer_until_the_subject_is_reopened() {
        #[cfg(feature = "integrations")]
        {
            let user = Uuid::new_v4();
            let query = "quero postar essa foto no Instagram";
            let tokens = intent_tokens(query);
            let (before, _) = split_matches(user, query, &tokens, &[]);
            assert!(before
                .iter()
                .any(|matched| matched.summary.id == "instagram"));

            remember_decline(user, "instagram");
            // The same request after "não" must not offer again.
            let (after, _) = split_matches(user, query, &tokens, &[]);
            assert!(!after
                .iter()
                .any(|matched| matched.summary.id == "instagram"));

            // Asking about the connector re-arms it.
            let reopened = "como conecto o Instagram?";
            let (again, _) = split_matches(user, reopened, &intent_tokens(reopened), &[]);
            assert!(again
                .iter()
                .any(|matched| matched.summary.id == "instagram"));
        }
    }

    #[test]
    fn a_catalogue_only_provider_is_never_offered_as_available() {
        #[cfg(feature = "integrations")]
        {
            let catalogue_only: Vec<LlmProviderSummary> =
                crate::apps::integration_catalog::llm_search(None, None, None)
                    .into_iter()
                    .filter(|provider| {
                        !provider.llm_available && provider.id.chars().count() >= MIN_TOKEN_LEN
                    })
                    .take(1)
                    .collect();
            let Some(provider) = catalogue_only.first() else {
                return;
            };
            let query = provider.id.replace('_', " ");
            let (available, reported) =
                split_matches(Uuid::new_v4(), &query, &intent_tokens(&query), &[]);
            assert!(!available
                .iter()
                .any(|matched| matched.summary.id == provider.id));
            assert!(reported.iter().any(|seen| seen.id == provider.id));
        }
    }
}
