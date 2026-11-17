//! Classifies LLM provider errors into stable codes for the client (#2562).
//!
//! The codes are intentionally coarse: they tell the *person* what went wrong
//! without leaking internal details (endpoints, keys, raw upstream bodies).

use serde::{Deserialize, Serialize};

/// Stable error codes the client can switch on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LlmErrorCode {
    /// The model endpoint could not be reached (DNS, connection refused, TLS).
    ModelUnreachable,
    /// The model endpoint rejected the credentials (HTTP 401 / 403).
    ModelAuth,
    /// The model endpoint returned a rate-limit response (HTTP 429).
    ModelRateLimited,
    /// The model endpoint returned a server error (HTTP 5xx or malformed body).
    ModelError,
    /// The turn timed out before the model answered.
    Timeout,
}

impl LlmErrorCode {
    /// A short, safe, human-readable message for this code (English).
    /// The client translates it; this is the fallback.
    pub fn safe_message(&self) -> &'static str {
        match self {
            Self::ModelUnreachable => "Could not reach the language model.",
            Self::ModelAuth => "The language model rejected the credentials.",
            Self::ModelRateLimited => "The language model is rate-limiting requests.",
            Self::ModelError => "The language model returned an error.",
            Self::Timeout => "The request timed out.",
        }
    }

    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::ModelUnreachable | Self::ModelRateLimited | Self::ModelError | Self::Timeout
        )
    }
}

/// Classifies an error string from the LLM provider into a stable code.
///
/// The heuristics match against the `Display` output of `reqwest`, `hyper`,
/// `std::io` and the provider wrappers in `botlib/botllm`. They are
/// intentionally broad: a misclassified error is harmless (the person sees
/// "model error" instead of "model unreachable"), but a *missing* error is
/// invisible.
pub fn classify(error: &str) -> LlmErrorCode {
    let lower = error.to_lowercase();

    // Connection-level failures
    if lower.contains("connection refused")
        || lower.contains("connect error")
        || lower.contains("dns error")
        || lower.contains("name or service not known")
        || lower.contains("no such host")
        || lower.contains("couldn't resolve")
        || lower.contains("network is unreachable")
        || lower.contains("connection reset")
        || lower.contains("broken pipe")
        || lower.contains("tls")
        || lower.contains("ssl")
        || lower.contains("certificate")
        // reqwest on Windows reports a refused connection only as "error
        // sending request for url (...)", and the provider gives up with
        // "failed after retries" (it retries connection failures only): a
        // stopped local engine must read as unreachable, not as a model error.
        || lower.contains("error sending request")
        || lower.contains("failed after retries")
    {
        return LlmErrorCode::ModelUnreachable;
    }

    // HTTP status codes
    if lower.contains("401") || lower.contains("403") || lower.contains("unauthorized")
        || lower.contains("forbidden") || lower.contains("invalid api key")
        || lower.contains("invalid_api_key") || lower.contains("authentication")
    {
        return LlmErrorCode::ModelAuth;
    }

    if lower.contains("429") || lower.contains("rate limit") || lower.contains("rate_limit")
        || lower.contains("too many requests") || lower.contains("quota")
    {
        return LlmErrorCode::ModelRateLimited;
    }

    if lower.contains("timeout") || lower.contains("timed out") || lower.contains("deadline") {
        return LlmErrorCode::Timeout;
    }

    // 5xx or anything else from the model
    LlmErrorCode::ModelError
}

/// The sentinel prefix sent through the stream channel when the provider fails.
/// The attempt loop detects this prefix and extracts the error message.
pub const ERROR_SENTINEL: &str = "__llm_error__:";

/// Wraps an error message in the sentinel format for the stream channel.
pub fn sentinel_message(error: &str) -> String {
    format!("{ERROR_SENTINEL}{error}")
}

/// Checks whether a chunk is an error sentinel and extracts the error message.
pub fn parse_sentinel(chunk: &str) -> Option<&str> {
    chunk.strip_prefix(ERROR_SENTINEL)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_connection_refused() {
        assert_eq!(
            classify("error sending request: connection refused"),
            LlmErrorCode::ModelUnreachable
        );
    }

    #[test]
    fn classify_a_stopped_local_engine() {
        assert_eq!(
            classify("HTTP error: error sending request for url (http://localhost:8081/v1/chat/completions)"),
            LlmErrorCode::ModelUnreachable
        );
        assert_eq!(
            classify("LLM stream request failed after retries"),
            LlmErrorCode::ModelUnreachable
        );
    }

    #[test]
    fn classify_dns() {
        assert_eq!(
            classify("error sending request: dns error: Name or service not known"),
            LlmErrorCode::ModelUnreachable
        );
    }

    #[test]
    fn classify_401() {
        assert_eq!(
            classify("HTTP 401 Unauthorized"),
            LlmErrorCode::ModelAuth
        );
    }

    #[test]
    fn classify_invalid_key() {
        assert_eq!(
            classify("Invalid API key provided"),
            LlmErrorCode::ModelAuth
        );
    }

    #[test]
    fn classify_429() {
        assert_eq!(
            classify("HTTP 429 Too Many Requests"),
            LlmErrorCode::ModelRateLimited
        );
    }

    #[test]
    fn classify_rate_limit() {
        assert_eq!(
            classify("Rate limit exceeded. Please retry after 30 seconds."),
            LlmErrorCode::ModelRateLimited
        );
    }

    #[test]
    fn classify_timeout() {
        assert_eq!(
            classify("operation timed out"),
            LlmErrorCode::Timeout
        );
    }

    #[test]
    fn classify_500() {
        assert_eq!(
            classify("Internal server error"),
            LlmErrorCode::ModelError
        );
    }

    #[test]
    fn classify_generic() {
        assert_eq!(
            classify("something went wrong"),
            LlmErrorCode::ModelError
        );
    }

    #[test]
    fn sentinel_roundtrip() {
        let msg = sentinel_message("connection refused");
        assert_eq!(parse_sentinel(&msg), Some("connection refused"));
    }

    #[test]
    fn non_sentinel_returns_none() {
        assert_eq!(parse_sentinel("normal chunk text"), None);
    }
}
