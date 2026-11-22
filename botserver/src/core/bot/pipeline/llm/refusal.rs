//! Structured refusal errors for subscription, account and workspace turns.
//!
//! When the pipeline refuses a turn for a reason the person can act on (link an
//! account, switch a model on, turn on the Agent switch) the error carries a
//! stable `code` and typed `params` so the shell renders the right card without
//! parsing English sentences.
//!
//! [`RefusalError`] implements `From<String>` so existing `?` sites that return
//! a plain message keep compiling; a plain string becomes an unclassified
//! refusal whose frame has no `error_code` (the shell falls back to showing the
//! text).

use std::collections::HashMap;
use std::fmt;

/// A refusal the pipeline sends to the client instead of a model answer.
#[derive(Debug, Clone)]
pub struct RefusalError {
    /// Stable code the shell switches on (`agent_not_linked`, …).
    /// `None` for legacy refusals that only carry a message.
    pub code: Option<&'static str>,
    /// Typed parameters the shell interpolates into its own sentence.
    pub params: HashMap<&'static str, String>,
    /// English fallback shown by clients that do not know the code.
    pub message: String,
}

impl RefusalError {
    /// A classified refusal with a stable code.
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code: Some(code),
            params: HashMap::new(),
            message: message.into(),
        }
    }

    /// Builder: add a param.
    pub fn param(mut self, key: &'static str, value: impl Into<String>) -> Self {
        self.params.insert(key, value.into());
        self
    }
}

impl fmt::Display for RefusalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

/// A plain string becomes an unclassified refusal (no `error_code` in the
/// frame). This keeps every `?` site that returns `Err(String)` compiling.
impl From<String> for RefusalError {
    fn from(message: String) -> Self {
        Self {
            code: None,
            params: HashMap::new(),
            message,
        }
    }
}

// ── Codes ───────────────────────────────────────────────────────────────

pub const AGENT_NOT_LINKED: &str = "agent_not_linked";
pub const AGENT_SWITCH_OFF: &str = "agent_switch_off";
#[cfg(any(not(feature = "agent-vm"), test))]
pub const AGENT_UNAVAILABLE: &str = "agent_unavailable";
pub const AGENT_MODEL_UNAVAILABLE: &str = "agent_model_unavailable";
pub const AGENT_WORKSPACE_FAILED: &str = "agent_workspace_failed";
pub const ACCOUNT_MODEL_OFF: &str = "account_model_off";
pub const FREE_MODEL_UNAVAILABLE: &str = "free_model_unavailable";
pub const AGENT_LIMIT_REACHED: &str = "agent_limit_reached";
/// The installed agent CLI acted without asking in the approval self-check;
/// its chat turns are refused (params `agent`, `label`, `version`).
pub const AGENT_UNSAFE_VERSION: &str = "agent_unsafe_version";

// ── Refusals raised inside a provider turn ──────────────────────────────

const CODED_PREFIX: &str = "agent_refusal:";
/// The codes and params a provider turn may raise through its error string.
const CODED: [&str; 2] = [AGENT_NOT_LINKED, AGENT_UNSAFE_VERSION];
const CODED_PARAMS: [&str; 3] = ["agent", "label", "version"];

/// The error string a provider returns so the attempt loop ends the turn
/// with this refusal instead of classifying it as a model failure.
#[cfg(any(feature = "agent-vm", test))]
pub(crate) fn coded(code: &'static str, message: &str, params: &[(&'static str, &str)]) -> String {
    let params: serde_json::Map<String, serde_json::Value> = params
        .iter()
        .map(|(key, value)| ((*key).to_string(), serde_json::Value::String((*value).to_string())))
        .collect();
    let body = serde_json::json!({ "code": code, "message": message, "params": params });
    format!("{CODED_PREFIX}{body}")
}

/// The refusal a provider raised with [`coded`]; `None` for any other error
/// and for codes outside [`CODED`].
pub(crate) fn from_coded(error: &str) -> Option<RefusalError> {
    let at = error.find(CODED_PREFIX)?;
    let value: serde_json::Value = serde_json::from_str(error[at + CODED_PREFIX.len()..].trim()).ok()?;
    let code = value.get("code").and_then(serde_json::Value::as_str)?;
    let code = CODED.iter().copied().find(|known| *known == code)?;
    let message = value.get("message").and_then(serde_json::Value::as_str).unwrap_or_default();
    let mut refusal = RefusalError::new(code, message);
    for key in CODED_PARAMS {
        if let Some(param) = value.get("params").and_then(|p| p.get(key)).and_then(serde_json::Value::as_str) {
            refusal = refusal.param(key, param);
        }
    }
    Some(refusal)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_coded_refusal_survives_the_error_string() {
        let error = coded(AGENT_UNSAFE_VERSION, "blocked", &[("agent", "codex-cli"), ("version", "0.172.0")]);
        let refusal = from_coded(&format!("LLM error: {error}")).expect("decoded");
        assert_eq!(refusal.code, Some(AGENT_UNSAFE_VERSION));
        assert_eq!(refusal.params["version"], "0.172.0");
        assert_eq!(refusal.message, "blocked");
        assert!(from_coded("agent_refusal:{\"code\":\"other\"}").is_none());
        assert!(from_coded("plain failure").is_none());
    }

    #[test]
    fn classified_refusal_carries_code_and_params() {
        let err = RefusalError::new(AGENT_NOT_LINKED, "Connect your Claude Code account.")
            .param("agent", "claude-code")
            .param("label", "Claude Code");
        assert_eq!(err.code, Some(AGENT_NOT_LINKED));
        assert_eq!(err.params["agent"], "claude-code");
        assert_eq!(err.params["label"], "Claude Code");
        assert_eq!(err.to_string(), "Connect your Claude Code account.");
    }

    #[test]
    fn plain_string_becomes_unclassified() {
        let err: RefusalError = "some old message".to_string().into();
        assert!(err.code.is_none());
        assert!(err.params.is_empty());
        assert_eq!(err.to_string(), "some old message");
    }

    #[test]
    fn agent_switch_off_refusal() {
        let err = RefusalError::new(AGENT_SWITCH_OFF, "VM is off.")
            .param("agent", "codex-cli")
            .param("label", "Codex");
        assert_eq!(err.code, Some(AGENT_SWITCH_OFF));
        assert_eq!(err.params["agent"], "codex-cli");
    }

    #[test]
    fn agent_unavailable_refusal() {
        let err = RefusalError::new(AGENT_UNAVAILABLE, "Not available.")
            .param("agent", "claude-code")
            .param("label", "Claude Code");
        assert_eq!(err.code, Some(AGENT_UNAVAILABLE));
    }

    #[test]
    fn agent_model_unavailable_refusal() {
        let err = RefusalError::new(AGENT_MODEL_UNAVAILABLE, "Model not available.")
            .param("agent", "claude-code")
            .param("label", "Claude Code")
            .param("model", "opus");
        assert_eq!(err.code, Some(AGENT_MODEL_UNAVAILABLE));
        assert_eq!(err.params["model"], "opus");
    }

    #[test]
    fn agent_workspace_failed_refusal() {
        let err = RefusalError::new(AGENT_WORKSPACE_FAILED, "Could not start.")
            .param("agent", "claude-code")
            .param("label", "Claude Code");
        assert_eq!(err.code, Some(AGENT_WORKSPACE_FAILED));
    }

    #[test]
    fn account_model_off_refusal() {
        let err = RefusalError::new(ACCOUNT_MODEL_OFF, "Model switched off.")
            .param("provider", "anthropic")
            .param("label", "Anthropic")
            .param("model", "claude-opus-5-5");
        assert_eq!(err.code, Some(ACCOUNT_MODEL_OFF));
        assert_eq!(err.params["provider"], "anthropic");
        assert_eq!(err.params["model"], "claude-opus-5-5");
    }

    #[test]
    fn free_model_unavailable_refusal() {
        let err = RefusalError::new(FREE_MODEL_UNAVAILABLE, "Free model is unavailable.");
        assert_eq!(err.code, Some(FREE_MODEL_UNAVAILABLE));
        assert!(err.params.is_empty());
    }
}
