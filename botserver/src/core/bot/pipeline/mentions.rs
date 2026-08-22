//! @integration mention resolution for the chat prompt pipeline (#939 D).
//!
//! The frontend attaches the mentions selected in the composer to every user
//! message. For each mention whose `kind` is `integration`, this module
//! resolves the referenced connection against the tenant control plane and
//! appends one system block describing the implemented, chat-executable
//! actions of that provider - so the LLM can invoke them through the
//! existing `integrations.invoke` command.
//!
//! Security contract:
//! - connections resolve strictly by (id, bot_id, owner_user_id,
//!   status='active'); a mention can never reach another tenant's row;
//! - unresolvable or stale ids are counted and skipped silently - they are
//!   never errors surfaced to the chat;
//! - action metadata comes from `botintegrations::providers::llm_safe_actions`
//!   which excludes all credential material; no secret, token or Vault path
//!   is ever rendered into the block.

use botlib::models::MentionRef;

#[cfg(feature = "integrations")]
use std::sync::Arc;

#[cfg(feature = "integrations")]
use botcore::shared::state::AppState;

/// Leniently extracts mention references from the raw WS payload.
///
/// Entries missing `kind`, or a payload without a `mentions` array at all,
/// simply yield fewer (or zero) references - malformed input must never
/// break message delivery.
pub fn parse_lenient_mentions(parsed: &serde_json::Value) -> Vec<MentionRef> {
    parsed
        .get("mentions")
        .and_then(|v| v.as_array())
        .map(|entries| {
            entries
                .iter()
                .filter_map(|entry| {
                    let kind = entry.get("kind")?.as_str()?.to_string();
                    Some(botlib::models::MentionRef {
                        id: entry
                            .get("id")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string(),
                        label: entry
                            .get("label")
                            .and_then(|v| v.as_str())
                            .map(std::string::ToString::to_string),
                        kind,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(feature = "integrations")]
fn integration_mention_block(
    label: &str,
    provider: &str,
    actions: &[botintegrations::providers::LlmSafeAction],
) -> String {
    let list = actions
        .iter()
        .map(|action| {
            let approval = if action.requires_approval {
                "; requires approval"
            } else {
                ""
            };
            format!(
                "- {} ({}, risk: {}{approval})",
                action.name, action.summary, action.risk
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "Connected integration {label} ({provider}) is available. Implemented \
         actions you may execute via the integrations.invoke command:\n{list}"
    )
}

#[cfg(feature = "integrations")]
#[derive(diesel::QueryableByName)]
struct MentionConnectionRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    display_name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    provider_slug: String,
}

/// Appends one system block per resolvable @integration mention, positioned
/// before the user turn. Unresolvable ids are ignored but counted; the count
/// is logged at debug level for observability without error noise.
///
/// Why an inline query instead of the botintegrations repository: the chat
/// pipeline has no `IntegrationState` handle (it carries the Vault wrapper)
/// and no pre-resolved org/branch scope; issue #939 phase D explicitly
/// authorizes this minimal owner-scoped lookup (id + bot + owner + active)
/// as a single round trip.
#[cfg(feature = "integrations")]
pub async fn append_integration_mention_blocks(
    state: &Arc<AppState>,
    bot_uuid: uuid::Uuid,
    user_id: uuid::Uuid,
    mentions: &[MentionRef],
    messages: &mut serde_json::Value,
) {
    use diesel::prelude::*;

    let mut unresolved = 0usize;
    let mut resolved = 0usize;
    for mention in mentions.iter().filter(|m| m.kind == "integration") {
        let Ok(connection_id) = uuid::Uuid::parse_str(mention.id.trim()) else {
            unresolved += 1;
            continue;
        };
        if connection_id.is_nil() {
            unresolved += 1;
            continue;
        }
        let mut conn = match state.conn.get() {
            Ok(conn) => conn,
            Err(error) => {
                log::error!("integration mention pool unavailable: {error}");
                unresolved += 1;
                continue;
            }
        };
        let row = match diesel::sql_query(
            "SELECT display_name, provider_slug FROM integration_connections \
             WHERE id = $1 AND bot_id = $2 AND owner_user_id = $3 AND status = 'active' \
             LIMIT 1",
        )
        .bind::<diesel::sql_types::Uuid, _>(connection_id)
        .bind::<diesel::sql_types::Uuid, _>(bot_uuid)
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .get_result::<MentionConnectionRow>(&mut conn)
        .optional()
        {
            Ok(row) => row,
            Err(error) => {
                log::error!("integration mention lookup failed: {error}");
                unresolved += 1;
                continue;
            }
        };
        let Some(row) = row else {
            unresolved += 1;
            continue;
        };

        let label = mention
            .label
            .clone()
            .unwrap_or_else(|| row.display_name.clone());
        let actions = botintegrations::providers::llm_safe_actions(&row.provider_slug);
        if actions.is_empty() {
            unresolved += 1;
            continue;
        }

        let block = integration_mention_block(&label, &row.provider_slug, &actions);
        match messages.as_array_mut() {
            Some(array) => {
                array.push(serde_json::json!({ "role": "system", "content": block }));
                resolved += 1;
            }
            None => {
                log::warn!("prompt messages container is not an array; mention block dropped");
            }
        }
    }
    if unresolved > 0 {
        log::debug!(
            "{unresolved} @integration mention(s) did not resolve to an active owned connection"
        );
    }
    if resolved > 0 {
        log::info!("injected {resolved} integration mention block(s) into chat prompt");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_mentions_and_ignores_malformed_entries() {
        let payload = serde_json::json!({
            "text": "hello",
            "mentions": [
                { "kind": "integration", "id": "abc", "label": "AWS" },
                { "kind": "contact", "name": "no-id-entry" },
                "not-an-object",
                { "id": "missing-kind-entry" }
            ]
        });
        let mentions = parse_lenient_mentions(&payload);
        assert_eq!(mentions.len(), 2);
        assert_eq!(mentions[0].kind, "integration");
        assert_eq!(mentions[0].id, "abc");
        assert_eq!(mentions[0].label.as_deref(), Some("AWS"));
        // Other entity kinds stay parseable; only integrations ever carry an
        // id, so this one resolves to nothing downstream.
        assert_eq!(mentions[1].kind, "contact");
        assert_eq!(mentions[1].id, "");
    }

    #[test]
    fn missing_or_non_array_mentions_yield_empty_vec() {
        assert!(parse_lenient_mentions(&serde_json::json!({ "text": "x" })).is_empty());
        assert!(parse_lenient_mentions(&serde_json::json!({ "mentions": 7 })).is_empty());
        assert!(parse_lenient_mentions(&serde_json::json!({ "mentions": [] })).is_empty());
    }

    #[cfg(feature = "integrations")]
    #[test]
    fn mention_block_lists_action_names_without_secret_material() {
        let actions = botintegrations::providers::llm_safe_actions("aws");
        let block = integration_mention_block("AWS Prod", "aws", &actions);
        assert!(block.contains("Connected integration AWS Prod (aws) is available"));
        assert!(block.contains("integrations.invoke"));
        for key in botintegrations::providers::implemented_action_names("aws") {
            assert!(block.contains(key), "block missing action {key}");
        }
        let lowered = block.to_lowercase();
        for banned in ["secret", "token", "vault"] {
            assert!(!lowered.contains(banned), "block leaked {banned}");
        }
    }

    #[cfg(feature = "integrations")]
    #[test]
    fn unknown_provider_has_no_chat_safe_metadata_to_advertise() {
        assert!(botintegrations::providers::llm_safe_actions("nonexistent").is_empty());
    }
}
