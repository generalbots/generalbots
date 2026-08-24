//! Council mode: several distinct profiles answer concurrently and an arbiter
//! merges the contributions into a single attributed response (issue #1173).

use crate::router::{
    attempt, chat_with_fallback, init_once, route_candidates, ProfileRuntime,
};
use log::warn;
use serde_json::Value;
use std::sync::Arc;
use tokio::task::JoinSet;

/// System instructions given to the arbiter pass.
const ARBITER_SYSTEM: &str = "You are an impartial arbiter. You receive several \
candidate answers produced by independent model profiles. Merge them into one \
authoritative answer: preserve correct details from every candidate, resolve \
contradictions conservatively, and mention which profile contributed each key \
point using its identifier.";

/// Final result of a council invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CouncilOutcome {
    /// Merged answer produced by the arbiter.
    pub text: String,
    /// Identifiers of the profiles whose answers were merged, in order.
    pub contributors: Vec<String>,
}

/// Selects up to `n` distinct profiles from the routing table.
fn distinct_profiles(n: usize, caps: &[&str]) -> Vec<Arc<ProfileRuntime>> {
    let mut picked: Vec<Arc<ProfileRuntime>> = Vec::new();
    for rt in route_candidates(caps, false) {
        if picked.iter().any(|p| p.profile.id == rt.profile.id) {
            continue;
        }
        picked.push(rt);
        if picked.len() >= n {
            break;
        }
    }
    picked
}

/// Asks `n` distinct LLM profiles concurrently, then merges their answers.
///
/// Profiles that fail or have open circuits are skipped by the router; when
/// fewer than `n` distinct healthy profiles exist, all available ones are used.
/// The merge itself is performed through [`chat_with_fallback`], so the arbiter
/// pass also benefits from fallback routing.
pub async fn council_answer(
    system: &str,
    user: &str,
    n: usize,
    caps: &[&str],
) -> Result<CouncilOutcome, String> {
    init_once();
    let picks = distinct_profiles(n, caps);
    if picks.is_empty() {
        return Err("no available LLM profile matches the requested capabilities".to_string());
    }
    let expected = picks.len();
    let mut tasks: JoinSet<(String, Option<String>)> = JoinSet::new();
    for rt in picks {
        let owned_system = system.to_string();
        let owned_user = user.to_string();
        tasks.spawn(async move {
            let outcome = attempt(&rt, &owned_system, &owned_user, &Value::Null).await;
            match outcome {
                Ok((text, _, _)) => (rt.profile.id.clone(), Some(text)),
                Err(_) => (rt.profile.id.clone(), None),
            }
        });
    }
    let mut sections: Vec<String> = Vec::new();
    let mut contributors: Vec<String> = Vec::new();
    while let Some(joined) = tasks.join_next().await {
        match joined {
            Ok((id, Some(text))) => {
                sections.push(format!("[profile {id}]\n{text}"));
                contributors.push(id);
            }
            Ok((id, None)) => warn!("Council contributor '{id}' failed and was excluded"),
            Err(e) => warn!("Council task join failed: {e}"),
        }
    }
    if sections.is_empty() {
        return Err("all council contributors failed".to_string());
    }
    let merge_prompt = format!(
        "Merge the following answers from {expected} independent model profiles into a single authoritative answer.\n\n{}",
        sections.join("\n\n")
    );
    let (text, _) = chat_with_fallback(ARBITER_SYSTEM, &merge_prompt, &empty_params, caps).await?;
    Ok(CouncilOutcome {
        text,
        contributors,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distinct_selection_deduplicates_by_id() {
        // Without configured profiles nothing can be selected; the helper must
        // simply yield an empty set instead of duplicating entries.
        let picked = distinct_profiles(3, &[]);
        let ids: Vec<&str> = picked.iter().map(|p| p.profile.id.as_str()).collect();
        let unique = ids.iter().collect::<std::collections::HashSet<_>>();
        assert_eq!(ids.len(), unique.len());
    }
}
