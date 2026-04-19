//! Simple Hallucination Loop Detector
//!
//! Detects when an LLM gets stuck in a repetition loop.
//! Only triggers when the same pattern repeats 50+ times consecutively.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use log::warn;

const THRESHOLD: usize = 50;
const WINDOW: Duration = Duration::from_secs(60);

#[derive(Debug, Clone)]
pub struct HallucinationConfig {
    pub threshold: usize,
    pub window: Duration,
}

impl Default for HallucinationConfig {
    fn default() -> Self {
        Self {
            threshold: THRESHOLD,
            window: WINDOW,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HallucinationDetector {
    config: HallucinationConfig,
    pattern_counts: Arc<Mutex<HashMap<String, (usize, Instant)>>>,
}

impl Default for HallucinationDetector {
    fn default() -> Self {
        Self::new(HallucinationConfig::default())
    }
}

impl HallucinationDetector {
    pub fn new(config: HallucinationConfig) -> Self {
        Self {
            config,
            pattern_counts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Check if a pattern is hallucinating (repeating 50+ times)
    pub async fn check(&self, pattern: &str) -> bool {
        let trimmed = pattern.trim();
        
        // Ignore short patterns
        if trimmed.is_empty() || trimmed.len() < 3 {
            return false;
        }

        // Ignore Markdown formatting patterns
        let md_patterns = ["**", "__", "*", "_", "`", "~~", "---", "***"];
        if md_patterns.contains(&trimmed) {
            return false;
        }

        // Ignore patterns that are just Markdown formatting (e.g., " **", "* ", "__")
        if trimmed.chars().all(|c| c == '*' || c == '_' || c == '`' || c == '~' || c == '-') {
            return false;
        }

        let mut counts = self.pattern_counts.lock().await;
        let now = Instant::now();

        // Clean old entries
        counts.retain(|_, (_, time)| now.duration_since(*time) < self.config.window);

        // Increment count for this pattern
        let (count, _) = counts.entry(trimmed.to_string()).or_insert((0, now));
        *count += 1;

        if *count >= self.config.threshold {
            warn!("Hallucination detected: pattern {:?} repeated {} times", trimmed, count);
            true
        } else {
            false
        }
    }

    /// Reset all counts
    pub async fn reset(&self) {
        let mut counts = self.pattern_counts.lock().await;
        counts.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_no_hallucination_below_threshold() {
        let detector = HallucinationDetector::default();
        for _ in 0..49 {
            assert!(!detector.check("test_pattern").await);
        }
    }

    #[tokio::test]
    async fn test_hallucination_at_threshold() {
        let detector = HallucinationDetector::default();
        for _ in 0..50 {
            detector.check("test_pattern").await;
        }
        assert!(detector.check("test_pattern").await);
    }

    #[tokio::test]
    async fn test_reset() {
        let detector = HallucinationDetector::default();
        for _ in 0..50 {
            detector.check("pattern").await;
        }
        detector.reset().await;
        assert!(!detector.check("pattern").await);
    }
}
