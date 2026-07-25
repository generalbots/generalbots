



















































use log::{debug, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;





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



    pub fn from_bot_config(target_bot_id: &Uuid) -> Self {
        let mut config = Self::default();

        if let Some(vault_configs) = read_bot_config_from_vault(target_bot_id) {
            for (config_key, config_value) in vault_configs {
                match config_key.as_str() {
                    "bm25-enabled" => {
                        config.enabled = config_value.to_lowercase() == "true";
                        debug!("BM25 enabled: {}", config.enabled);
                    }
                    "bm25-k1" => {
                        config.k1 = config_value.parse().unwrap_or(1.2);
                        debug!("BM25 k1: {}", config.k1);
                    }
                    "bm25-b" => {
                        config.b = config_value.parse().unwrap_or(0.75);
                        debug!("BM25 b: {}", config.b);
                    }
                    "bm25-stemming" => {
                        config.stemming = config_value.to_lowercase() == "true";
                        debug!("BM25 stemming: {}", config.stemming);
                    }
                    "bm25-stopwords" => {
                        config.stopwords = config_value.to_lowercase() == "true";
                        debug!("BM25 stopwords: {}", config.stopwords);
                    }
                    _ => {}
                }
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

fn read_bot_config_from_vault(bot_id: &Uuid) -> Option<HashMap<String, String>> {
    use botcoresecrets::manager::SecretsManager;
    let sm = SecretsManager::get_clone().ok()?;
    if !sm.is_enabled() {
        return None;
    }
    let path = format!("gbo/{}/{}/{}", uuid::Uuid::nil(), uuid::Uuid::nil(), bot_id);
    let sm_clone = sm.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build();
        let result = if let Ok(rt) = rt {
            rt.block_on(async move { sm_clone.get_secret(&path).await.ok() })
        } else {
            None
        };
        let _ = tx.send(result);
    });
    rx.recv().ok().flatten()
}
