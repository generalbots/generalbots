//! Service state for the memory crate. The LLM is always injected by the
//! integrator; this crate never constructs a provider itself.

use std::sync::Arc;

use crate::DbPool;

/// Injected chat-completion function: `(system prompt, user prompt, json
/// parameters) -> raw completion text` or an error string.
pub type LlmFn = Arc<dyn Fn(&str, &str, &str) -> Result<String, String> + Send + Sync>;

const SAMPLE_PCT_MAX: u8 = 100;
const DEFAULT_SAMPLE_PCT: u8 = 20;

/// Shared service for memory storage, extraction and recall.
pub struct MemoryService {
    pub pool: DbPool,
    pub llm_generate: LlmFn,
    /// Master switch for the conversation extraction hook.
    pub enabled: bool,
    /// Percentage of turns sampled for extraction (0-100). Explicit
    /// "remember that" phrasing bypasses sampling entirely.
    pub sample_pct: u8,
}

impl MemoryService {
    pub fn new(pool: DbPool, llm_generate: LlmFn) -> Self {
        Self {
            pool,
            llm_generate,
            enabled: true,
            sample_pct: DEFAULT_SAMPLE_PCT,
        }
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn with_sample_pct(mut self, sample_pct: u8) -> Self {
        self.sample_pct = sample_pct.min(SAMPLE_PCT_MAX);
        self
    }
}
