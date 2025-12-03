//! BM25 Configuration for Tantivy-based sparse retrieval
//!
//! This module provides configuration for BM25 text search powered by Tantivy.
//! Tantivy is a full-text search engine library (like Lucene) that implements
//! the BM25 ranking algorithm.
//!
//! # Config.csv Parameters
//!
//! | Parameter | Default | Description |
//! |-----------|---------|-------------|
//! | `bm25-enabled` | `true` | Enable/disable BM25 sparse search |
//! | `bm25-k1` | `1.2` | Term frequency saturation (0.5-3.0 typical) |
//! | `bm25-b` | `0.75` | Document length normalization (0.0-1.0) |
//! | `bm25-stemming` | `true` | Apply stemming to terms |
//! | `bm25-stopwords` | `true` | Filter common stopwords |
//!
//! # Example config.csv
//!
//! ```csv
//! bm25-enabled,true
//! bm25-k1,1.2
//! bm25-b,0.75
//! bm25-stemming,true
//! bm25-stopwords,true
//! ```
//!
//! # Switching BM25 On/Off
//!
//! To **disable** BM25 sparse search (use only dense/embedding search):
//! ```csv
//! bm25-enabled,false
//! ```
//!
//! To **enable** BM25 with custom tuning:
//! ```csv
//! bm25-enabled,true
//! bm25-k1,1.5
//! bm25-b,0.5
//! ```
//!
//! # How It Works
//!
//! When `bm25-enabled=true`:
//! - Hybrid search uses BOTH BM25 (keyword) + Qdrant (embedding) results
//! - Results are merged using Reciprocal Rank Fusion (RRF)
//! - Good for queries where exact keyword matches matter
//!
//! When `bm25-enabled=false`:
//! - Only dense (embedding) search via Qdrant is used
//! - Faster but may miss exact keyword matches
//! - Better for semantic/conceptual queries

use diesel::prelude::*;
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::shared::utils::DbPool;

/// Configuration for BM25 sparse retrieval (powered by Tantivy)
///
/// BM25 (Best Matching 25) is a ranking function used for information retrieval.
/// This configuration controls the Tantivy-based BM25 implementation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bm25Config {
    /// Whether BM25 sparse search is enabled
    /// When false, only dense (embedding) search is used
    pub enabled: bool,

    /// Term frequency saturation parameter (typically 1.2-2.0)
    /// - Higher values: more weight to term frequency
    /// - Lower values: diminishing returns for repeated terms
    /// - Tantivy default: 1.2
    pub k1: f32,

    /// Document length normalization parameter (0.0-1.0)
    /// - 0.0: no length normalization
    /// - 1.0: full length normalization (penalizes long documents)
    /// - Tantivy default: 0.75
    pub b: f32,

    /// Whether to apply stemming to terms before indexing/searching
    /// Stemming reduces words to their root form (e.g., "running" → "run")
    pub stemming: bool,

    /// Whether to filter out common stopwords (e.g., "the", "a", "is")
    pub stopwords: bool,
}

impl Default for Bm25Config {
    fn default() -> Self {
        Self {
            enabled: true,
            k1: 1.2,
            b: 0.75,
            stemming: true,
            stopwords: true,
        }
    }
}

impl Bm25Config {
    /// Load BM25 configuration from bot_configuration table
    ///
    /// Reads parameters: `bm25-enabled`, `bm25-k1`, `bm25-b`, `bm25-stemming`, `bm25-stopwords`
    pub fn from_bot_config(pool: &DbPool, target_bot_id: &Uuid) -> Self {
        let mut config = Self::default();

        let mut conn = match pool.get() {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to get database connection for BM25 config: {}", e);
                return config;
            }
        };

        #[derive(QueryableByName)]
        struct ConfigRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            config_key: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            config_value: String,
        }

        let configs: Vec<ConfigRow> = diesel::sql_query(
            "SELECT config_key, config_value FROM bot_configuration \
             WHERE bot_id = $1 AND config_key LIKE 'bm25-%'",
        )
        .bind::<diesel::sql_types::Uuid, _>(target_bot_id)
        .load(&mut conn)
        .unwrap_or_default();

        for row in configs {
            match row.config_key.as_str() {
                "bm25-enabled" => {
                    config.enabled = row.config_value.to_lowercase() == "true";
                    debug!("BM25 enabled: {}", config.enabled);
                }
                "bm25-k1" => {
                    config.k1 = row.config_value.parse().unwrap_or(1.2);
                    debug!("BM25 k1: {}", config.k1);
                }
                "bm25-b" => {
                    config.b = row.config_value.parse().unwrap_or(0.75);
                    debug!("BM25 b: {}", config.b);
                }
                "bm25-stemming" => {
                    config.stemming = row.config_value.to_lowercase() == "true";
                    debug!("BM25 stemming: {}", config.stemming);
                }
                "bm25-stopwords" => {
                    config.stopwords = row.config_value.to_lowercase() == "true";
                    debug!("BM25 stopwords: {}", config.stopwords);
                }
                _ => {}
            }
        }

        // Validate and clamp values
        config.validate();
        config
    }

    /// Create config with BM25 disabled (dense-only search)
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }

    /// Create config with custom k1 and b parameters
    pub fn with_params(k1: f32, b: f32) -> Self {
        let mut config = Self {
            k1,
            b,
            ..Default::default()
        };
        config.validate();
        config
    }

    /// Validate and clamp configuration values to sensible ranges
    fn validate(&mut self) {
        // k1 should be positive, typically between 0.5 and 3.0
        if self.k1 < 0.0 {
            warn!("BM25 k1 cannot be negative, setting to default 1.2");
            self.k1 = 1.2;
        } else if self.k1 > 10.0 {
            warn!("BM25 k1 {} is unusually high, capping at 10.0", self.k1);
            self.k1 = 10.0;
        }

        // b should be between 0.0 and 1.0
        if self.b < 0.0 {
            warn!("BM25 b cannot be negative, setting to 0.0");
            self.b = 0.0;
        } else if self.b > 1.0 {
            warn!("BM25 b cannot exceed 1.0, capping at 1.0");
            self.b = 1.0;
        }
    }

    /// Check if BM25 should be used in hybrid search
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Check if text preprocessing is enabled
    pub fn has_preprocessing(&self) -> bool {
        self.stemming || self.stopwords
    }

    /// Get a description of the current configuration
    pub fn describe(&self) -> String {
        if self.enabled {
            format!(
                "BM25(k1={}, b={}, stemming={}, stopwords={})",
                self.k1, self.b, self.stemming, self.stopwords
            )
        } else {
            "BM25(disabled)".to_string()
        }
    }
}

/// Common English stopwords for filtering
/// Used when `bm25-stopwords=true`
pub const DEFAULT_STOPWORDS: &[&str] = &[
    "a", "an", "and", "are", "as", "at", "be", "by", "for", "from", "has", "he",
    "in", "is", "it", "its", "of", "on", "or", "that", "the", "to", "was", "were",
    "will", "with", "this", "but", "they", "have", "had", "what", "when", "where",
    "who", "which", "why", "how", "all", "each", "every", "both", "few", "more",
    "most", "other", "some", "such", "no", "nor", "not", "only", "own", "same",
    "so", "than", "too", "very", "just", "can", "should", "now", "do", "does",
    "did", "done", "been", "being", "would", "could", "might", "must", "shall",
    "may", "am", "your", "our", "their", "his", "her", "my", "me", "him", "them",
    "us", "you", "i", "we", "she", "if", "then", "else", "about", "into", "over",
    "after", "before", "between", "under", "again", "further", "once",
];

/// Check if a word is a common stopword
pub fn is_stopword(word: &str) -> bool {
    DEFAULT_STOPWORDS.contains(&word.to_lowercase().as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Bm25Config::default();
        assert!(config.enabled);
        assert!((config.k1 - 1.2).abs() < f32::EPSILON);
        assert!((config.b - 0.75).abs() < f32::EPSILON);
        assert!(config.stemming);
        assert!(config.stopwords);
    }

    #[test]
    fn test_disabled_config() {
        let config = Bm25Config::disabled();
        assert!(!config.enabled);
        assert!(!config.is_enabled());
    }

    #[test]
    fn test_with_params() {
        let config = Bm25Config::with_params(1.5, 0.5);
        assert!((config.k1 - 1.5).abs() < f32::EPSILON);
        assert!((config.b - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_validation_negative_k1() {
        let mut config = Bm25Config {
            k1: -1.0,
            ..Default::default()
        };
        config.validate();
        assert!((config.k1 - 1.2).abs() < f32::EPSILON);
    }

    #[test]
    fn test_validation_high_k1() {
        let mut config = Bm25Config {
            k1: 15.0,
            ..Default::default()
        };
        config.validate();
        assert!((config.k1 - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_validation_b_range() {
        let mut config = Bm25Config {
            b: -0.5,
            ..Default::default()
        };
        config.validate();
        assert!(config.b.abs() < f32::EPSILON);

        let mut config2 = Bm25Config {
            b: 1.5,
            ..Default::default()
        };
        config2.validate();
        assert!((config2.b - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_has_preprocessing() {
        let config = Bm25Config::default();
        assert!(config.has_preprocessing());

        let no_preprocess = Bm25Config {
            stemming: false,
            stopwords: false,
            ..Default::default()
        };
        assert!(!no_preprocess.has_preprocessing());
    }

    #[test]
    fn test_describe() {
        let config = Bm25Config::default();
        let desc = config.describe();
        assert!(desc.contains("k1=1.2"));
        assert!(desc.contains("b=0.75"));

        let disabled = Bm25Config::disabled();
        assert_eq!(disabled.describe(), "BM25(disabled)");
    }

    #[test]
    fn test_is_stopword() {
        assert!(is_stopword("the"));
        assert!(is_stopword("THE"));
        assert!(is_stopword("and"));
        assert!(is_stopword("is"));
        assert!(!is_stopword("algorithm"));
        assert!(!is_stopword("rust"));
        assert!(!is_stopword("tantivy"));
    }

    #[test]
    fn test_stopwords_list() {
        assert!(!DEFAULT_STOPWORDS.is_empty());
        assert!(DEFAULT_STOPWORDS.len() > 80);
    }
}
