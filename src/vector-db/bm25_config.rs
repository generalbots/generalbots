



















































use diesel::prelude::*;
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::shared::utils::DbPool;





#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bm25Config {


    pub enabled: bool,





    pub k1: f32,





    pub b: f32,



    pub stemming: bool,


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


        config.validate();
        config
    }


    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }


    pub fn with_params(k1: f32, b: f32) -> Self {
        let mut config = Self {
            k1,
            b,
            ..Default::default()
        };
        config.validate();
        config
    }


    fn validate(&mut self) {

        if self.k1 < 0.0 {
            warn!("BM25 k1 cannot be negative, setting to default 1.2");
            self.k1 = 1.2;
        } else if self.k1 > 10.0 {
            warn!("BM25 k1 {} is unusually high, capping at 10.0", self.k1);
            self.k1 = 10.0;
        }


        if self.b < 0.0 {
            warn!("BM25 b cannot be negative, setting to 0.0");
            self.b = 0.0;
        } else if self.b > 1.0 {
            warn!("BM25 b cannot exceed 1.0, capping at 1.0");
            self.b = 1.0;
        }
    }


    pub fn is_enabled(&self) -> bool {
        self.enabled
    }


    pub fn has_preprocessing(&self) -> bool {
        self.stemming || self.stopwords
    }


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


pub fn is_stopword(word: &str) -> bool {
    DEFAULT_STOPWORDS.contains(&word.to_lowercase().as_str())
}
