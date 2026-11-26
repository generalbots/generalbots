//! A spent subscription plan ends the turn at once with an
//! `agent_limit_reached` refusal.
//!
//! The subscription provider reports the spent plan through the same error
//! string as every other provider failure; [`AgentLimit::encode`] gives that
//! string a shape the attempt loop recognises before classifying it, so the
//! turn is neither retried (each retry ran the CLI again in the workspace) nor
//! shown as a short "wait a moment" rate limit.

use serde::{Deserialize, Serialize};

use super::refusal::{RefusalError, AGENT_LIMIT_REACHED};

mod display;

const PREFIX: &str = "agent_limit_reached:";

/// What the shell needs to say which plan ran out and when it comes back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct AgentLimit {
    /// Provider id (`codex-cli`, `claude-code`, `antigravity`).
    pub agent: String,
    /// Display name of the provider.
    pub label: String,
    /// RFC 3339 instant the plan comes back, when it is known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reset: Option<String>,
    /// The CLI's own sentence, as it printed it: logged whole, and shown on
    /// the card only after `display::message` cleaned it.
    #[serde(default)]
    pub detail: String,
}

impl AgentLimit {
    #[cfg(any(feature = "agent-vm", test))]
    pub(crate) fn encode(&self) -> String {
        format!("{PREFIX}{}", serde_json::to_string(self).unwrap_or_else(|e| { log::debug!("using default for agent limit encoding: {e}"); Default::default() }))
    }

    fn decode(error: &str) -> Option<Self> {
        let at = error.find(PREFIX)?;
        let mut limit: Self = serde_json::from_str(error[at + PREFIX.len()..].trim()).ok()?;
        limit.reset = limit
            .reset
            .filter(|r| chrono::DateTime::parse_from_rfc3339(r).is_ok());
        Some(limit)
    }
}

/// The refusal for an error that reports a spent plan; `None` for any other
/// error, which the attempt loop classifies as before (a short-lived 429
/// stays `ModelRateLimited` and is retried).
pub(super) fn refusal_for(error: &str) -> Option<RefusalError> {
    let limit = AgentLimit::decode(error)?;
    botlog::safe_warn!(
        "agent plan limit: agent={} reset={} detail={}",
        limit.agent,
        limit.reset.as_deref().unwrap_or("unknown"),
        limit.detail
    );
    let shown = display::message(&limit.detail);
    let fallback = format!(
        "You have reached the usage limit of your {} plan. Choose another model meanwhile.",
        limit.label
    );
    let content = if shown.is_empty() { fallback } else { shown.clone() };
    let mut refusal = RefusalError::new(AGENT_LIMIT_REACHED, content)
        .param("agent", limit.agent)
        .param("label", limit.label);
    if !shown.is_empty() {
        refusal = refusal.param("message", shown);
    }
    Some(match limit.reset {
        Some(reset) => refusal.param("reset", reset),
        None => refusal,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::bot::pipeline::llm::error_classify::{classify, LlmErrorCode};

    fn codex(reset: Option<&str>) -> AgentLimit {
        AgentLimit {
            agent: "codex-cli".into(),
            label: "Codex".into(),
            reset: reset.map(str::to_string),
            detail: "You've hit your usage limit. Upgrade to Pro (https://chatgpt.com/explore/pro), visit https://chatgpt.com/settings/usage to purchase more credits or try again at 10:15 PM.".into(),
        }
    }

    #[test]
    fn a_spent_plan_becomes_a_refusal_with_its_reset() {
        let error = codex(Some("2026-11-22T22:15:11Z")).encode();
        let Some(refusal) = refusal_for(&error) else { panic!("not recognised") };
        assert_eq!(refusal.code, Some(AGENT_LIMIT_REACHED));
        assert_eq!(refusal.params["agent"], "codex-cli");
        assert_eq!(refusal.params["label"], "Codex");
        assert_eq!(refusal.params["reset"], "2026-11-22T22:15:11Z");
        assert!(!refusal.params.contains_key("detail"));
        assert_eq!(
            refusal.params["message"],
            "You've hit your usage limit. Upgrade to Pro (https://chatgpt.com/explore/pro), visit https://chatgpt.com/settings/usage to purchase more credits or try again at 10:15 PM."
        );
        assert_eq!(refusal.message, refusal.params["message"]);
    }

    #[test]
    fn an_unknown_reset_is_left_out() {
        let Some(refusal) = refusal_for(&codex(None).encode()) else { panic!("not recognised") };
        assert!(!refusal.params.contains_key("reset"));
        let Some(bad) = refusal_for(&codex(Some("10:15 PM")).encode()) else { panic!("not recognised") };
        assert!(!bad.params.contains_key("reset"));
    }

    #[test]
    fn a_spent_plan_is_never_retried_and_a_short_429_still_is() {
        // The attempt loop checks `refusal_for` before `classify`: a spent
        // plan ends the turn instead of reaching the retry branch.
        assert!(refusal_for(&codex(None).encode()).is_some());
        assert_eq!(classify(&codex(None).encode()), LlmErrorCode::ModelRateLimited);
        let short = "429 rate limit: Rate limit reached for gpt-5, try again in 20ms";
        assert!(refusal_for(short).is_none());
        assert_eq!(classify(short), LlmErrorCode::ModelRateLimited);
        assert!(LlmErrorCode::ModelRateLimited.is_retryable());
    }
}
