//! Hallucination Loop Detector
//!
//! Detects when an LLM gets stuck in a repetition loop (hallucination).
//! This module provides detection for all channels (web, WhatsApp, Telegram, etc.).

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Configuration for hallucination detection
#[derive(Debug, Clone)]
pub struct HallucinationConfig {
    /// Minimum text length before detection starts
    pub min_text_length: usize,
    /// Pattern lengths to check (in characters)
    pub pattern_lengths: Vec<usize>,
    /// Number of consecutive repetitions to trigger detection
    pub consecutive_threshold: usize,
    /// Number of total occurrences in recent text to trigger detection
    pub occurrence_threshold: usize,
    /// Recent text window size for occurrence counting
    pub recent_text_window: usize,
    /// Number of identical tokens to trigger detection
    pub identical_token_threshold: usize,
    /// Common words to ignore (won't trigger detection when repeated)
    pub ignore_words: Vec<String>,
}

/// Default list of common words that shouldn't trigger hallucination detection
const DEFAULT_IGNORE_WORDS: &[&str] = &[
    "the", "a", "an", "is", "are", "was", "were", "be", "been", "being",
    "have", "has", "had", "do", "does", "did", "will", "would", "could", "should",
    "may", "might", "must", "shall", "can", "need", "dare", "ought", "used",
    "to", "of", "in", "for", "on", "with", "at", "by", "from", "as",
    "into", "through", "during", "before", "after", "above", "below", "between",
    "and", "but", "or", "nor", "so", "yet", "both", "either", "neither",
    "not", "only", "own", "same", "than", "too", "very", "just",
    "de", "da", "do", "das", "dos", "e", "é", "em", "no", "na", "nos", "nas",
    "para", "por", "com", "sem", "sobre", "entre", "após", "antes", "depois",
    "que", "se", "ou", "mas", "porém", "como", "assim", "também", "ainda",
    "um", "uma", "uns", "umas", "o", "a", "os", "as",
];

impl Default for HallucinationConfig {
    fn default() -> Self {
        Self {
            min_text_length: 50,
            pattern_lengths: vec![3, 4, 5, 6, 8, 10, 15, 20],
            consecutive_threshold: 10,  // Increased from 5 to 10
            occurrence_threshold: 15,   // Increased from 8 to 15
            recent_text_window: 500,
            identical_token_threshold: 15,  // Increased from 10 to 15
            ignore_words: DEFAULT_IGNORE_WORDS.iter().map(|s| s.to_string()).collect(),
        }
    }
}

/// State for tracking hallucination during streaming
#[derive(Debug)]
pub struct HallucinationDetector {
    config: HallucinationConfig,
    last_content_hash: u64,
    identical_count: usize,
    detected: bool,
    detected_pattern: Option<String>,
}

impl Default for HallucinationDetector {
    fn default() -> Self {
        Self::new(HallucinationConfig::default())
    }
}

impl HallucinationDetector {
    /// Create a new detector with custom configuration
    pub fn new(config: HallucinationConfig) -> Self {
        Self {
            config,
            last_content_hash: 0,
            identical_count: 0,
            detected: false,
            detected_pattern: None,
        }
    }

    /// Check if hallucination has been detected
    pub fn is_detected(&self) -> bool {
        self.detected
    }

    /// Get the detected pattern if any
    pub fn get_detected_pattern(&self) -> Option<&str> {
        self.detected_pattern.as_deref()
    }

    /// Get the detected pattern as owned String
    pub fn get_detected_pattern_owned(&self) -> Option<String> {
        self.detected_pattern.clone()
    }

    /// Check a new token/chunk for hallucination patterns
    /// Returns true if hallucination is detected
    pub fn check_token(&mut self, token: &str) -> bool {
        if self.detected {
            return true;
        }

        // Check for identical token repetition
        if !token.trim().is_empty() {
            let mut hasher = DefaultHasher::new();
            token.hash(&mut hasher);
            let content_hash = hasher.finish();

            if content_hash == self.last_content_hash {
                self.identical_count += 1;
                if self.identical_count >= self.config.identical_token_threshold {
                    log::warn!(
                        "LLM hallucination detected: identical token repeated {} times: {:?}",
                        self.identical_count,
                        token
                    );
                    self.detected = true;
                    self.detected_pattern = Some(format!("{} ({}x)", token.trim(), self.identical_count));
                    return true;
                }
            } else {
                self.identical_count = 0;
            }
            self.last_content_hash = content_hash;
        }

        false
    }

    /// Check accumulated text for repetition patterns
    /// Returns Some(pattern) if hallucination is detected
    pub fn check_text(&mut self, text: &str) -> Option<String> {
        if self.detected {
            return self.detected_pattern.clone();
        }

        // Skip detection for short texts
        if text.len() < self.config.min_text_length {
            return None;
        }

        // Check for repeated patterns of various lengths
        for pattern_len in &self.config.pattern_lengths {
            if text.len() < *pattern_len * 5 {
                continue;
            }

            // Get the last pattern to check
            let chars: Vec<char> = text.chars().collect();
            let start = chars.len().saturating_sub(*pattern_len);
            let pattern: String = chars[start..].iter().collect();
            let pattern_str = pattern.trim();

            if pattern_str.is_empty() || pattern_str.len() < 2 {
                continue;
            }

            // Ignore common Markdown separators
            if pattern_str == "---" || pattern_str == "***" || pattern_str == "___" {
                continue;
            }

            // Count how many times this pattern appears consecutively at the end
            let mut count = 0;
            let mut search_text = text;

            while search_text.ends_with(pattern_str) || search_text.ends_with(&pattern) {
                count += 1;
                if count >= self.config.consecutive_threshold {
                    // Found threshold repetitions - likely hallucination
                    log::warn!(
                        "LLM hallucination loop detected: pattern {:?} repeated {} times consecutively",
                        pattern_str,
                        count
                    );
                    self.detected = true;
                    self.detected_pattern = Some(pattern_str.to_string());
                    return self.detected_pattern.clone();
                }
                // Remove one occurrence and continue checking
                if search_text.ends_with(pattern_str) {
                    search_text = &search_text[..search_text.len().saturating_sub(pattern_str.len())];
                } else {
                    search_text = &search_text[..search_text.len().saturating_sub(pattern.len())];
                }
            }

            // Alternative: count total occurrences in recent text
            let recent_start = chars.len().saturating_sub(self.config.recent_text_window);
            let recent_text: String = chars[recent_start..].iter().collect();
            let total_count = recent_text.matches(pattern_str).count();
            if total_count >= self.config.occurrence_threshold && pattern_str.len() >= 3 {
                log::warn!(
                    "LLM hallucination loop detected: pattern {:?} appears {} times in recent {} chars",
                    pattern_str,
                    total_count,
                    self.config.recent_text_window
                );
                self.detected = true;
                self.detected_pattern = Some(format!("{} ({}x)", pattern_str, total_count));
                return self.detected_pattern.clone();
            }
        }

        None
    }

    /// Combined check: both token and accumulated text
    /// Returns true if hallucination detected
    pub fn check(&mut self, token: &str, accumulated_text: &str) -> bool {
        // First check token repetition
        if self.check_token(token) {
            return true;
        }

        // Then check accumulated text for patterns
        if self.check_text(accumulated_text).is_some() {
            return true;
        }

        false
    }

    /// Reset the detector state (for new conversations)
    pub fn reset(&mut self) {
        self.last_content_hash = 0;
        self.identical_count = 0;
        self.detected = false;
        self.detected_pattern = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identical_token_detection() {
        let mut detector = HallucinationDetector::default();

        // Same token repeated
        for _ in 0..9 {
            assert!(!detector.check_token("GBJ2KP"));
        }
        // 10th repetition should trigger
        assert!(detector.check_token("GBJ2KP"));
    }

    #[test]
    fn test_pattern_repetition() {
        let mut detector = HallucinationDetector::default();

        // Build text with repeated pattern
        let repeated = "XYZ123 ".repeat(6);
        let result = detector.check_text(&repeated);

        assert!(result.is_some());
        assert!(detector.is_detected());
    }

    #[test]
    fn test_normal_text_not_detected() {
        let mut detector = HallucinationDetector::default();

        let normal_text = "This is a normal response without any repetition patterns. \
                          The LLM is generating coherent text that makes sense.";

        assert!(!detector.check_token("normal"));
        assert!(detector.check_text(normal_text).is_none());
        assert!(!detector.is_detected());
    }

    #[test]
    fn test_reset() {
        let mut detector = HallucinationDetector::default();

        // Trigger detection
        for _ in 0..10 {
            detector.check_token("REPEAT");
        }
        assert!(detector.is_detected());

        // Reset
        detector.reset();
        assert!(!detector.is_detected());
        assert!(detector.get_detected_pattern().is_none());
    }
}
