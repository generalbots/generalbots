//! Multi-profile LLM configuration resolution for issue #1173.
//!
//! Resolution order:
//! 1. `GB_LLM_PROFILES` environment variable containing a JSON array of profiles;
//! 2. JSON file referenced by `GB_LLM_PROFILES_FILE`;
//! 3. Legacy single-provider fallback built from the very same environment
//!    variables the existing single-provider path reads today (`LLM_URL`,
//!    `LLM_KEY` with `OPENAI_API_KEY` fallback, `LLM_MODEL`), producing one
//!    profile identified as `"default"`.

use log::info;
use serde::{Deserialize, Serialize};

/// A single routable LLM endpoint description.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LlmProfile {
    /// Stable identifier used by the router, breaker logs and council attribution.
    #[serde(default = "default_profile_id")]
    pub id: String,
    /// Full base URL of an OpenAI-compatible chat-completions endpoint.
    #[serde(default)]
    pub url: String,
    /// Model identifier sent to the provider.
    #[serde(default)]
    pub model: String,
    /// Bearer API key for the provider.
    #[serde(default)]
    pub key: String,
    /// Capability tags used for routing filters (for example "vision", "tools").
    #[serde(default)]
    pub caps: Vec<String>,
    /// Monetary cost per 1000 tokens; used when `prefer_cheap` routing is requested.
    #[serde(default)]
    pub cost_per_1k: f64,
    /// Ordering weight; higher values are attempted first unless cost overrides.
    #[serde(default)]
    pub priority: i32,
}

fn default_profile_id() -> String {
    "default".to_string()
}

impl Default for LlmProfile {
    fn default() -> Self {
        Self {
            id: default_profile_id(),
            url: String::new(),
            model: String::new(),
            key: String::new(),
            caps: Vec::new(),
            cost_per_1k: 0.0,
            priority: 0,
        }
    }
}

impl LlmProfile {
    /// Builds the legacy single-provider profile from the environment variables
    /// used by the pre-existing call path (`LLM_URL`, `LLM_KEY` or
    /// `OPENAI_API_KEY`, `LLM_MODEL`). Returns `None` when no URL is configured,
    /// mirroring the current behavior of skipping the provider entirely.
    pub fn legacy_from_env() -> Option<Self> {
        let url = std::env::var("LLM_URL").unwrap_or_default();
        if url.is_empty() {
            return None;
        }
        let mut key = std::env::var("LLM_KEY").unwrap_or_default();
        if key.is_empty() {
            key = std::env::var("OPENAI_API_KEY").unwrap_or_default();
        }
        let model = std::env::var("LLM_MODEL").unwrap_or_default();
        Some(Self {
            url,
            key,
            model,
            ..Self::default()
        })
    }

    fn parse_many(raw: &str, origin: &str) -> Vec<Self> {
        match serde_json::from_str::<Vec<Self>>(raw) {
            Ok(profiles) => {
                info!("Loaded {} LLM profile(s) from {}", profiles.len(), origin);
                profiles
            }
            Err(e) => {
                log::error!("Invalid LLM profiles JSON in {origin}: {e}");
                Vec::new()
            }
        }
    }

    /// True when the profile declares every requested capability.
    pub fn matches_caps(&self, caps: &[&str]) -> bool {
        caps.iter()
            .all(|c| self.caps.iter().any(|pc| pc.eq_ignore_ascii_case(c)))
    }

    /// Effective ordering rank; higher sorts first. When `prefer_cheap` is set,
    /// cheaper profiles are boosted relative to more expensive ones.
    pub fn effective_priority(&self, prefer_cheap: bool) -> i64 {
        if prefer_cheap && self.cost_per_1k > 0.0 {
            self.priority as i64 - (self.cost_per_1k * 1000.0) as i64
        } else {
            self.priority as i64
        }
    }
}

/// Loads all configured LLM profiles following the documented resolution order.
/// Entries without a `url` are discarded because they cannot serve requests.
pub fn load_profiles() -> Vec<LlmProfile> {
    let mut candidates = Vec::new();
    if let Ok(raw) = std::env::var("GB_LLM_PROFILES") {
        if !raw.trim().is_empty() {
            candidates.extend(LlmProfile::parse_many(&raw, "GB_LLM_PROFILES"));
        }
    }
    if candidates.is_empty() {
        if let Ok(path) = std::env::var("GB_LLM_PROFILES_FILE") {
            if !path.is_empty() {
                match std::fs::read_to_string(&path) {
                    Ok(raw) => {
                        candidates.extend(LlmProfile::parse_many(&raw, &path));
                    }
                    Err(e) => {
                        log::error!("Unable to read GB_LLM_PROFILES_FILE {path}: {e}");
                    }
                }
            }
        }
    }
    if candidates.is_empty() {
        if let Some(legacy) = LlmProfile::legacy_from_env() {
            info!("Using legacy single-provider LLM configuration as profile 'default'");
            candidates.push(legacy);
        }
    }
    candidates.retain(|p| !p.url.trim().is_empty());
    candidates
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_json_receives_defaults() {
        let parsed: LlmProfile =
            serde_json::from_str(r#"{"id": "p1", "url": "http://localhost/v1"}"#)
                .expect("partial profile must deserialize");
        assert_eq!(parsed.id, "p1");
        assert_eq!(parsed.url, "http://localhost/v1");
        assert_eq!(parsed.model, "");
        assert_eq!(parsed.key, "");
        assert!(parsed.caps.is_empty());
        assert_eq!(parsed.cost_per_1k, 0.0);
        assert_eq!(parsed.priority, 0);
    }

    #[test]
    fn capability_match_is_case_insensitive_and_all_or_nothing() {
        let profile = LlmProfile {
            caps: vec!["vision".to_string(), "tools".to_string()],
            ..LlmProfile::default()
        };
        assert!(profile.matches_caps(&["VISION"]));
        assert!(!profile.matches_caps(&["vision", "audio"]));
    }

    #[test]
    fn cheap_preference_boosts_low_cost_profiles() {
        let cheap = LlmProfile {
            priority: 10,
            cost_per_1k: 0.001,
            ..LlmProfile::default()
        };
        let pricey = LlmProfile {
            priority: 11,
            cost_per_1k: 5.0,
            ..LlmProfile::default()
        };
        assert!(pricey.effective_priority(false) > cheap.effective_priority(false));
        assert!(cheap.effective_priority(true) > pricey.effective_priority(true));
    }

    #[test]
    fn legacy_without_url_is_absent() {
        // The test process does not define LLM_URL; absence yields None.
        if std::env::var("LLM_URL").is_err() {
            assert!(LlmProfile::legacy_from_env().is_none());
        }
    }
}
