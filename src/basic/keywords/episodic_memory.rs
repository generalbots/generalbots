use chrono::{DateTime, Duration, Utc};
use rhai::{Array, Dynamic, Engine, Map};
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    pub id: Uuid,

    pub user_id: Uuid,

    pub bot_id: Uuid,

    pub session_id: Uuid,

    pub summary: String,

    pub key_topics: Vec<String>,

    pub decisions: Vec<String>,

    pub action_items: Vec<ActionItem>,

    pub sentiment: Sentiment,

    pub resolution: ResolutionStatus,

    pub message_count: usize,

    pub message_ids: Vec<Uuid>,

    pub created_at: DateTime<Utc>,

    pub conversation_start: DateTime<Utc>,
    pub conversation_end: DateTime<Utc>,

    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItem {
    pub description: String,

    pub assignee: Option<String>,

    pub due_date: Option<DateTime<Utc>>,

    pub priority: Priority,

    pub completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum Priority {
    Low,
    #[default]
    Medium,
    High,
    Critical,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sentiment {
    pub score: f64,

    pub label: SentimentLabel,

    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum SentimentLabel {
    VeryNegative,
    Negative,
    #[default]
    Neutral,
    Positive,
    VeryPositive,
}


impl Default for Sentiment {
    fn default() -> Self {
        Self {
            score: 0.0,
            label: SentimentLabel::Neutral,
            confidence: 0.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum ResolutionStatus {
    Resolved,
    Unresolved,
    Escalated,
    Pending,
    #[default]
    Unknown,
}


#[derive(Debug, Clone)]
pub struct EpisodicMemoryConfig {
    pub enabled: bool,

    pub threshold: usize,

    pub history: usize,

    pub model: String,

    pub max_episodes: usize,

    pub retention_days: u32,

    pub auto_summarize: bool,
}

impl Default for EpisodicMemoryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            threshold: 4,
            history: 2,
            model: "fast".to_string(),
            max_episodes: 100,
            retention_days: 365,
            auto_summarize: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub id: Uuid,
    pub role: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug)]
pub struct EpisodicMemoryManager {
    config: EpisodicMemoryConfig,
}

impl EpisodicMemoryManager {
    pub fn new(config: EpisodicMemoryConfig) -> Self {
        Self { config }
    }

    pub fn from_config(config_map: &std::collections::HashMap<String, String>) -> Self {
        let config = EpisodicMemoryConfig {
            enabled: config_map
                .get("episodic-memory-enabled")
                .map(|v| v == "true")
                .unwrap_or(true),
            threshold: config_map
                .get("episodic-memory-threshold")
                .and_then(|v| v.parse().ok())
                .unwrap_or(4),
            history: config_map
                .get("episodic-memory-history")
                .and_then(|v| v.parse().ok())
                .unwrap_or(2),
            model: config_map
                .get("episodic-memory-model")
                .cloned()
                .unwrap_or_else(|| "fast".to_string()),
            max_episodes: config_map
                .get("episodic-memory-max-episodes")
                .and_then(|v| v.parse().ok())
                .unwrap_or(100),
            retention_days: config_map
                .get("episodic-memory-retention-days")
                .and_then(|v| v.parse().ok())
                .unwrap_or(365),
            auto_summarize: config_map
                .get("episodic-memory-auto-summarize")
                .map(|v| v == "true")
                .unwrap_or(true),
        };
        Self::new(config)
    }

    pub fn should_summarize(&self, message_count: usize) -> bool {
        self.config.enabled && self.config.auto_summarize && message_count >= self.config.threshold
    }

    pub fn get_history_to_keep(&self) -> usize {
        self.config.history
    }

    pub fn get_threshold(&self) -> usize {
        self.config.threshold
    }

    pub fn generate_summary_prompt(&self, messages: &[ConversationMessage]) -> String {
        let formatted_messages = messages
            .iter()
            .map(|m| {
                format!(
                    "[{}] {}: {}",
                    m.timestamp.format("%H:%M"),
                    m.role,
                    m.content
                )
            })
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

    pub fn parse_summary_response(
        &self,
        response: &str,
        messages: &[ConversationMessage],
        user_id: Uuid,
        bot_id: Uuid,
        session_id: Uuid,
    ) -> Result<Episode, String> {
        let json_str = extract_json(response)?;

        let parsed: serde_json::Value =
            serde_json::from_str(&json_str).map_err(|e| format!("Failed to parse JSON: {}", e))?;

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

    pub fn get_retention_cutoff(&self) -> DateTime<Utc> {
        Utc::now() - Duration::days(i64::from(self.config.retention_days))
    }
}

pub fn extract_json(response: &str) -> Result<String, String> {
    if let Some(start) = response.find("```json") {
        if let Some(end) = response[start + 7..].find("```") {
            return Ok(response[start + 7..start + 7 + end].trim().to_string());
        }
    }

    if let Some(start) = response.find("```") {
        let after_start = start + 3;

        let json_start = response[after_start..]
            .find('\n')
            .map(|i| after_start + i + 1)
            .unwrap_or(after_start);
        if let Some(end) = response[json_start..].find("```") {
            return Ok(response[json_start..json_start + end].trim().to_string());
        }
    }

    if let Some(start) = response.find('{') {
        if let Some(end) = response.rfind('}') {
            if end > start {
                return Ok(response[start..=end].to_string());
            }
        }
    }

    Err("No JSON found in response".to_string())
}

impl Episode {
    pub fn to_dynamic(&self) -> Dynamic {
        let mut map = Map::new();

        map.insert("id".into(), self.id.to_string().into());
        map.insert("user_id".into(), self.user_id.to_string().into());
        map.insert("bot_id".into(), self.bot_id.to_string().into());
        map.insert("session_id".into(), self.session_id.to_string().into());
        map.insert("summary".into(), self.summary.clone().into());

        let topics: Array = self
            .key_topics
            .iter()
            .map(|t| Dynamic::from(t.clone()))
            .collect();
        map.insert("key_topics".into(), topics.into());

        let decisions: Array = self
            .decisions
            .iter()
            .map(|d| Dynamic::from(d.clone()))
            .collect();
        map.insert("decisions".into(), decisions.into());

        let action_items: Array = self
            .action_items
            .iter()
            .map(|a| {
                let mut item_map = Map::new();
                item_map.insert("description".into(), a.description.clone().into());
                item_map.insert(
                    "assignee".into(),
                    a.assignee.clone().unwrap_or_default().into(),
                );
                item_map.insert(
                    "priority".into(),
                    format!("{:?}", a.priority).to_lowercase().into(),
                );
                item_map.insert("completed".into(), a.completed.into());
                Dynamic::from(item_map)
            })
            .collect();
        map.insert("action_items".into(), action_items.into());

        let mut sentiment_map = Map::new();
        sentiment_map.insert("score".into(), self.sentiment.score.into());
        sentiment_map.insert(
            "label".into(),
            format!("{:?}", self.sentiment.label).to_lowercase().into(),
        );
        sentiment_map.insert("confidence".into(), self.sentiment.confidence.into());
        map.insert("sentiment".into(), sentiment_map.into());

        map.insert(
            "resolution".into(),
            format!("{:?}", self.resolution).to_lowercase().into(),
        );
        map.insert("message_count".into(), (self.message_count as i64).into());
        map.insert("created_at".into(), self.created_at.to_rfc3339().into());
        map.insert(
            "conversation_start".into(),
            self.conversation_start.to_rfc3339().into(),
        );
        map.insert(
            "conversation_end".into(),
            self.conversation_end.to_rfc3339().into(),
        );

        Dynamic::from(map)
    }
}

pub fn register_episodic_memory_keywords(engine: &mut Engine) {
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

pub mod sql {
    pub const INSERT_EPISODE: &str = r"
        INSERT INTO conversation_episodes (
            id, user_id, bot_id, session_id, summary, key_topics, decisions,
            action_items, sentiment, resolution, message_count, message_ids,
            conversation_start, conversation_end, metadata, created_at
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16
        )
    ";

    pub const GET_EPISODES_BY_USER: &str = r"
        SELECT * FROM conversation_episodes
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT $2
    ";

    pub const GET_EPISODES_BY_SESSION: &str = r"
        SELECT * FROM conversation_episodes
        WHERE session_id = $1
        ORDER BY created_at DESC
    ";

    pub const SEARCH_EPISODES: &str = r"
        SELECT * FROM conversation_episodes
        WHERE user_id = $1
        AND (
            to_tsvector('english', summary) @@ plainto_tsquery('english', $2)
            OR key_topics @> $3::jsonb
        )
        ORDER BY created_at DESC
        LIMIT $4
    ";

    pub const DELETE_OLD_EPISODES: &str = r"
        DELETE FROM conversation_episodes
        WHERE created_at < $1
    ";

    pub const COUNT_USER_EPISODES: &str = r"
        SELECT COUNT(*) FROM conversation_episodes
        WHERE user_id = $1
    ";

    pub const DELETE_OLDEST_EPISODES: &str = r"
        DELETE FROM conversation_episodes
        WHERE id IN (
            SELECT id FROM conversation_episodes
            WHERE user_id = $1
            ORDER BY created_at ASC
            LIMIT $2
        )
    ";
}
