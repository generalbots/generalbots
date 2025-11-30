//! Episodic Memory - Conversation Summaries
//!
//! This module provides episodic memory capabilities that compress long conversations
//! into summaries for efficient context management. Episodic memory enables:
//!
//! - Automatic conversation summarization
//! - Key topic extraction
//! - Decision and action item tracking
//! - Long-term memory without context overflow
//!
//! ## BASIC Keywords
//!
//! ```basic
//! ' Create episode summary manually
//! summary = CREATE EPISODE SUMMARY
//!
//! ' Get recent episodes for a user
//! episodes = GET EPISODES(10)
//!
//! ' Search episodes by topic
//! related = SEARCH EPISODES "billing issues"
//!
//! ' Clear old episodes
//! CLEAR EPISODES OLDER THAN 30
//! ```
//!
//! ## Config.csv Properties
//!
//! ```csv
//! name,value
//! episodic-memory-enabled,true
//! episodic-summary-threshold,20
//! episodic-summary-model,fast
//! episodic-max-episodes,100
//! episodic-retention-days,365
//! episodic-auto-summarize,true
//! ```

use chrono::{DateTime, Duration, Utc};
use rhai::{Dynamic, Engine, EvalAltResult, Map, Array};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::state::AppState;

/// Episode summary structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    /// Unique episode identifier
    pub id: Uuid,
    /// User ID this episode belongs to
    pub user_id: Uuid,
    /// Bot ID that created the episode
    pub bot_id: Uuid,
    /// Session/conversation ID
    pub session_id: Uuid,
    /// Condensed summary of the conversation
    pub summary: String,
    /// Key topics discussed
    pub key_topics: Vec<String>,
    /// Decisions made during conversation
    pub decisions: Vec<String>,
    /// Action items identified
    pub action_items: Vec<ActionItem>,
    /// Sentiment analysis result
    pub sentiment: Sentiment,
    /// Resolution status
    pub resolution: ResolutionStatus,
    /// Number of messages summarized
    pub message_count: usize,
    /// Original message IDs (for reference)
    pub message_ids: Vec<Uuid>,
    /// When the episode was created
    pub created_at: DateTime<Utc>,
    /// Time range of original conversation
    pub conversation_start: DateTime<Utc>,
    pub conversation_end: DateTime<Utc>,
    /// Metadata
    pub metadata: serde_json::Value,
}

/// Action item extracted from conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItem {
    /// Description of the action
    pub description: String,
    /// Who is responsible
    pub assignee: Option<String>,
    /// Due date if mentioned
    pub due_date: Option<DateTime<Utc>>,
    /// Priority level
    pub priority: Priority,
    /// Completion status
    pub completed: bool,
}

/// Priority levels for action items
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Medium
    }
}

/// Sentiment analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sentiment {
    /// Overall sentiment score (-1.0 to 1.0)
    pub score: f64,
    /// Sentiment label
    pub label: SentimentLabel,
    /// Confidence in the assessment
    pub confidence: f64,
}

/// Sentiment labels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SentimentLabel {
    VeryNegative,
    Negative,
    Neutral,
    Positive,
    VeryPositive,
}

impl Default for SentimentLabel {
    fn default() -> Self {
        SentimentLabel::Neutral
    }
}

impl Default for Sentiment {
    fn default() -> Self {
        Sentiment {
            score: 0.0,
            label: SentimentLabel::Neutral,
            confidence: 0.5,
        }
    }
}

/// Resolution status of the conversation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ResolutionStatus {
    Resolved,
    Unresolved,
    Escalated,
    Pending,
    Unknown,
}

impl Default for ResolutionStatus {
    fn default() -> Self {
        ResolutionStatus::Unknown
    }
}

/// Configuration for episodic memory
#[derive(Debug, Clone)]
pub struct EpisodicMemoryConfig {
    /// Whether episodic memory is enabled
    pub enabled: bool,
    /// Message count threshold before auto-summarization
    pub summary_threshold: usize,
    /// Model to use for summarization
    pub summary_model: String,
    /// Maximum episodes to keep per user
    pub max_episodes: usize,
    /// Days to retain episodes
    pub retention_days: u32,
    /// Whether to auto-summarize conversations
    pub auto_summarize: bool,
}

impl Default for EpisodicMemoryConfig {
    fn default() -> Self {
        EpisodicMemoryConfig {
            enabled: true,
            summary_threshold: 20,
            summary_model: "fast".to_string(),
            max_episodes: 100,
            retention_days: 365,
            auto_summarize: true,
        }
    }
}

/// Message structure for summarization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub id: Uuid,
    pub role: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

/// Episodic Memory Manager
pub struct EpisodicMemoryManager {
    config: EpisodicMemoryConfig,
}

impl EpisodicMemoryManager {
    /// Create a new episodic memory manager
    pub fn new(config: EpisodicMemoryConfig) -> Self {
        EpisodicMemoryManager { config }
    }

    /// Create from config map
    pub fn from_config(config_map: &std::collections::HashMap<String, String>) -> Self {
        let config = EpisodicMemoryConfig {
            enabled: config_map
                .get("episodic-memory-enabled")
                .map(|v| v == "true")
                .unwrap_or(true),
            summary_threshold: config_map
                .get("episodic-summary-threshold")
                .and_then(|v| v.parse().ok())
                .unwrap_or(20),
            summary_model: config_map
                .get("episodic-summary-model")
                .cloned()
                .unwrap_or_else(|| "fast".to_string()),
            max_episodes: config_map
                .get("episodic-max-episodes")
                .and_then(|v| v.parse().ok())
                .unwrap_or(100),
            retention_days: config_map
                .get("episodic-retention-days")
                .and_then(|v| v.parse().ok())
                .unwrap_or(365),
            auto_summarize: config_map
                .get("episodic-auto-summarize")
                .map(|v| v == "true")
                .unwrap_or(true),
        };
        EpisodicMemoryManager::new(config)
    }

    /// Check if auto-summarization should trigger
    pub fn should_summarize(&self, message_count: usize) -> bool {
        self.config.enabled
            && self.config.auto_summarize
            && message_count >= self.config.summary_threshold
    }

    /// Generate the summarization prompt
    pub fn generate_summary_prompt(&self, messages: &[ConversationMessage]) -> String {
        let formatted_messages = messages
            .iter()
            .map(|m| format!("[{}] {}: {}", m.timestamp.format("%H:%M"), m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"Analyze and summarize this conversation. Extract key information.

CONVERSATION:
{}

Respond with valid JSON only:
{{
    "summary": "A concise 2-3 sentence summary of the conversation",
    "key_topics": ["topic1", "topic2"],
    "decisions": ["decision1", "decision2"],
    "action_items": [
        {{"description": "action description", "assignee": "user/bot/null", "priority": "low/medium/high/critical"}}
    ],
    "sentiment": {{
        "score": 0.0,
        "label": "very_negative/negative/neutral/positive/very_positive",
        "confidence": 0.8
    }},
    "resolution": "resolved/unresolved/escalated/pending/unknown"
}}"#,
            formatted_messages
        )
    }

    /// Parse LLM response into episode data
    pub fn parse_summary_response(
        &self,
        response: &str,
        messages: &[ConversationMessage],
        user_id: Uuid,
        bot_id: Uuid,
        session_id: Uuid,
    ) -> Result<Episode, String> {
        // Try to extract JSON from response
        let json_str = extract_json(response)?;

        let parsed: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        let summary = parsed["summary"]
            .as_str()
            .unwrap_or("Conversation summary unavailable")
            .to_string();

        let key_topics: Vec<String> = parsed["key_topics"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let decisions: Vec<String> = parsed["decisions"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let action_items: Vec<ActionItem> = parsed["action_items"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| {
                        Some(ActionItem {
                            description: v["description"].as_str()?.to_string(),
                            assignee: v["assignee"].as_str().map(String::from),
                            due_date: None,
                            priority: match v["priority"].as_str().unwrap_or("medium") {
                                "low" => Priority::Low,
                                "high" => Priority::High,
                                "critical" => Priority::Critical,
                                _ => Priority::Medium,
                            },
                            completed: false,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let sentiment = Sentiment {
            score: parsed["sentiment"]["score"].as_f64().unwrap_or(0.0),
            label: match parsed["sentiment"]["label"].as_str().unwrap_or("neutral") {
                "very_negative" => SentimentLabel::VeryNegative,
                "negative" => SentimentLabel::Negative,
                "positive" => SentimentLabel::Positive,
                "very_positive" => SentimentLabel::VeryPositive,
                _ => SentimentLabel::Neutral,
            },
            confidence: parsed["sentiment"]["confidence"].as_f64().unwrap_or(0.5),
        };

        let resolution = match parsed["resolution"].as_str().unwrap_or("unknown") {
            "resolved" => ResolutionStatus::Resolved,
            "unresolved" => ResolutionStatus::Unresolved,
            "escalated" => ResolutionStatus::Escalated,
            "pending" => ResolutionStatus::Pending,
            _ => ResolutionStatus::Unknown,
        };

        let conversation_start = messages
            .first()
            .map(|m| m.timestamp)
            .unwrap_or_else(Utc::now);
        let conversation_end = messages
            .last()
            .map(|m| m.timestamp)
            .unwrap_or_else(Utc::now);

        Ok(Episode {
            id: Uuid::new_v4(),
            user_id,
            bot_id,
            session_id,
            summary,
            key_topics,
            decisions,
            action_items,
            sentiment,
            resolution,
            message_count: messages.len(),
            message_ids: messages.iter().map(|m| m.id).collect(),
            created_at: Utc::now(),
            conversation_start,
            conversation_end,
            metadata: serde_json::json!({}),
        })
    }

    /// Get retention cutoff date
    pub fn get_retention_cutoff(&self) -> DateTime<Utc> {
        Utc::now() - Duration::days(self.config.retention_days as i64)
    }
}

/// Extract JSON from LLM response (handles markdown code blocks)
fn extract_json(response: &str) -> Result<String, String> {
    // Try to find JSON in code blocks first
    if let Some(start) = response.find("```json") {
        if let Some(end) = response[start + 7..].find("```") {
            return Ok(response[start + 7..start + 7 + end].trim().to_string());
        }
    }

    // Try to find JSON in generic code blocks
    if let Some(start) = response.find("```") {
        let after_start = start + 3;
        // Skip language identifier if present
        let json_start = response[after_start..]
            .find('\n')
            .map(|i| after_start + i + 1)
            .unwrap_or(after_start);
        if let Some(end) = response[json_start..].find("```") {
            return Ok(response[json_start..json_start + end].trim().to_string());
        }
    }

    // Try to find raw JSON (starts with {)
    if let Some(start) = response.find('{') {
        if let Some(end) = response.rfind('}') {
            if end > start {
                return Ok(response[start..=end].to_string());
            }
        }
    }

    Err("No JSON found in response".to_string())
}

/// Convert Episode to Rhai Dynamic
impl Episode {
    pub fn to_dynamic(&self) -> Dynamic {
        let mut map = Map::new();

        map.insert("id".into(), self.id.to_string().into());
        map.insert("user_id".into(), self.user_id.to_string().into());
        map.insert("bot_id".into(), self.bot_id.to_string().into());
        map.insert("session_id".into(), self.session_id.to_string().into());
        map.insert("summary".into(), self.summary.clone().into());

        let topics: Array = self.key_topics
            .iter()
            .map(|t| Dynamic::from(t.clone()))
            .collect();
        map.insert("key_topics".into(), topics.into());

        let decisions: Array = self.decisions
            .iter()
            .map(|d| Dynamic::from(d.clone()))
            .collect();
        map.insert("decisions".into(), decisions.into());

        let action_items: Array = self.action_items
            .iter()
            .map(|a| {
                let mut item_map = Map::new();
                item_map.insert("description".into(), a.description.clone().into());
                item_map.insert("assignee".into(), a.assignee.clone().unwrap_or_default().into());
                item_map.insert("priority".into(), format!("{:?}", a.priority).to_lowercase().into());
                item_map.insert("completed".into(), a.completed.into());
                Dynamic::from(item_map)
            })
            .collect();
        map.insert("action_items".into(), action_items.into());

        let mut sentiment_map = Map::new();
        sentiment_map.insert("score".into(), self.sentiment.score.into());
        sentiment_map.insert("label".into(), format!("{:?}", self.sentiment.label).to_lowercase().into());
        sentiment_map.insert("confidence".into(), self.sentiment.confidence.into());
        map.insert("sentiment".into(), sentiment_map.into());

        map.insert("resolution".into(), format!("{:?}", self.resolution).to_lowercase().into());
        map.insert("message_count".into(), (self.message_count as i64).into());
        map.insert("created_at".into(), self.created_at.to_rfc3339().into());
        map.insert("conversation_start".into(), self.conversation_start.to_rfc3339().into());
        map.insert("conversation_end".into(), self.conversation_end.to_rfc3339().into());

        Dynamic::from(map)
    }
}

/// Register episodic memory keywords with Rhai engine
pub fn register_episodic_memory_keywords(engine: &mut Engine) {
    // CREATE EPISODE SUMMARY - creates a summary of current conversation
    // This is typically called from the runtime with state access

    // Helper functions for working with episodes in scripts
    engine.register_fn("episode_summary", |episode: Map| -> String {
        episode
            .get("summary")
            .and_then(|v| v.clone().try_cast::<String>())
            .unwrap_or_default()
    });

    engine.register_fn("episode_topics", |episode: Map| -> Array {
        episode
            .get("key_topics")
            .and_then(|v| v.clone().try_cast::<Array>())
            .unwrap_or_default()
    });

    engine.register_fn("episode_decisions", |episode: Map| -> Array {
        episode
            .get("decisions")
            .and_then(|v| v.clone().try_cast::<Array>())
            .unwrap_or_default()
    });

    engine.register_fn("episode_action_items", |episode: Map| -> Array {
        episode
            .get("action_items")
            .and_then(|v| v.clone().try_cast::<Array>())
            .unwrap_or_default()
    });

    engine.register_fn("episode_sentiment_score", |episode: Map| -> f64 {
        episode
            .get("sentiment")
            .and_then(|v| v.clone().try_cast::<Map>())
            .and_then(|m| m.get("score").and_then(|s| s.clone().try_cast::<f64>()))
            .unwrap_or(0.0)
    });

    engine.register_fn("episode_was_resolved", |episode: Map| -> bool {
        episode
            .get("resolution")
            .and_then(|v| v.clone().try_cast::<String>())
            .map(|s| s == "resolved")
            .unwrap_or(false)
    });

    info!("Episodic memory keywords registered");
}

/// SQL for creating episodic memory tables
pub const EPISODIC_MEMORY_SCHEMA: &str = r#"
-- Conversation episodes (summaries)
CREATE TABLE IF NOT EXISTS conversation_episodes (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    session_id UUID NOT NULL,
    summary TEXT NOT NULL,
    key_topics JSONB NOT NULL DEFAULT '[]',
    decisions JSONB NOT NULL DEFAULT '[]',
    action_items JSONB NOT NULL DEFAULT '[]',
    sentiment JSONB NOT NULL DEFAULT '{"score": 0, "label": "neutral", "confidence": 0.5}',
    resolution VARCHAR(50) NOT NULL DEFAULT 'unknown',
    message_count INTEGER NOT NULL DEFAULT 0,
    message_ids JSONB NOT NULL DEFAULT '[]',
    conversation_start TIMESTAMP WITH TIME ZONE NOT NULL,
    conversation_end TIMESTAMP WITH TIME ZONE NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_episodes_user_id ON conversation_episodes(user_id);
CREATE INDEX IF NOT EXISTS idx_episodes_bot_id ON conversation_episodes(bot_id);
CREATE INDEX IF NOT EXISTS idx_episodes_session_id ON conversation_episodes(session_id);
CREATE INDEX IF NOT EXISTS idx_episodes_created_at ON conversation_episodes(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_episodes_key_topics ON conversation_episodes USING GIN(key_topics);
CREATE INDEX IF NOT EXISTS idx_episodes_resolution ON conversation_episodes(resolution);

-- Full-text search on summaries
CREATE INDEX IF NOT EXISTS idx_episodes_summary_fts ON conversation_episodes
    USING GIN(to_tsvector('english', summary));
"#;

/// SQL for episode operations
pub mod sql {
    pub const INSERT_EPISODE: &str = r#"
        INSERT INTO conversation_episodes (
            id, user_id, bot_id, session_id, summary, key_topics, decisions,
            action_items, sentiment, resolution, message_count, message_ids,
            conversation_start, conversation_end, metadata, created_at
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16
        )
    "#;

    pub const GET_EPISODES_BY_USER: &str = r#"
        SELECT * FROM conversation_episodes
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT $2
    "#;

    pub const GET_EPISODES_BY_SESSION: &str = r#"
        SELECT * FROM conversation_episodes
        WHERE session_id = $1
        ORDER BY created_at DESC
    "#;

    pub const SEARCH_EPISODES: &str = r#"
        SELECT * FROM conversation_episodes
        WHERE user_id = $1
        AND (
            to_tsvector('english', summary) @@ plainto_tsquery('english', $2)
            OR key_topics @> $3::jsonb
        )
        ORDER BY created_at DESC
        LIMIT $4
    "#;

    pub const DELETE_OLD_EPISODES: &str = r#"
        DELETE FROM conversation_episodes
        WHERE created_at < $1
    "#;

    pub const COUNT_USER_EPISODES: &str = r#"
        SELECT COUNT(*) FROM conversation_episodes
        WHERE user_id = $1
    "#;

    pub const DELETE_OLDEST_EPISODES: &str = r#"
        DELETE FROM conversation_episodes
        WHERE id IN (
            SELECT id FROM conversation_episodes
            WHERE user_id = $1
            ORDER BY created_at ASC
            LIMIT $2
        )
    "#;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = EpisodicMemoryConfig::default();
        assert!(config.enabled);
        assert_eq!(config.summary_threshold, 20);
        assert_eq!(config.max_episodes, 100);
    }

    #[test]
    fn test_should_summarize() {
        let manager = EpisodicMemoryManager::new(EpisodicMemoryConfig {
            enabled: true,
            summary_threshold: 10,
            auto_summarize: true,
            ..Default::default()
        });

        assert!(!manager.should_summarize(5));
        assert!(manager.should_summarize(10));
        assert!(manager.should_summarize(15));
    }

    #[test]
    fn test_extract_json() {
        // Test with code block
        let response = "Here's the summary:\n```json\n{\"summary\": \"test\"}\n```\n";
        assert!(extract_json(response).is_ok());

        // Test with raw JSON
        let response = "The result is {\"summary\": \"test\"}";
        assert!(extract_json(response).is_ok());
    }

    #[test]
    fn test_generate_summary_prompt() {
        let manager = EpisodicMemoryManager::new(EpisodicMemoryConfig::default());
        let messages = vec![
            ConversationMessage {
                id: Uuid::new_v4(),
                role: "user".to_string(),
                content: "Hello".to_string(),
                timestamp: Utc::now(),
            },
        ];

        let prompt = manager.generate_summary_prompt(&messages);
        assert!(prompt.contains("CONVERSATION:"));
        assert!(prompt.contains("Hello"));
    }

    #[test]
    fn test_parse_summary_response() {
        let manager = EpisodicMemoryManager::new(EpisodicMemoryConfig::default());
        let response = r#"{
            "summary": "User asked about billing",
            "key_topics": ["billing", "payment"],
            "decisions": [],
            "action_items": [],
            "sentiment": {"score": 0.5, "label": "positive", "confidence": 0.8},
            "resolution": "resolved"
        }"#;

        let messages = vec![
            ConversationMessage {
                id: Uuid::new_v4(),
                role: "user".to_string(),
                content: "What's my balance?".to_string(),
                timestamp: Utc::now(),
            },
        ];

        let episode = manager.parse_summary_response(
            response,
            &messages,
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        );

        assert!(episode.is_ok());
        let ep = episode.unwrap();
        assert_eq!(ep.summary, "User asked about billing");
        assert_eq!(ep.key_topics, vec!["billing", "payment"]);
        assert_eq!(ep.resolution, ResolutionStatus::Resolved);
    }

    #[test]
    fn test_episode_to_dynamic() {
        let episode = Episode {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            bot_id: Uuid::new_v4(),
            session_id: Uuid::new_v4(),
            summary: "Test summary".to_string(),
            key_topics: vec!["topic1".to_string()],
            decisions: vec![],
            action_items: vec![],
            sentiment: Sentiment::default(),
            resolution: ResolutionStatus::Resolved,
            message_count: 5,
            message_ids: vec![],
            created_at: Utc::now(),
            conversation_start: Utc::now(),
            conversation_end: Utc::now(),
            metadata: serde_json::json!({}),
        };

        let dynamic = episode.to_dynamic();
        assert!(dynamic.is::<Map>());
    }
}
