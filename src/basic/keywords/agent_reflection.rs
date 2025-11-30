//! Agent Reflection Module
//!
//! Implements self-improvement loops for bots through periodic reflection
//! on conversation quality, performance metrics, and learned patterns.
//!
//! BASIC Keywords:
//! - SET BOT REFLECTION true/false
//! - SET BOT REFLECTION_INTERVAL 10
//! - REFLECT ON "conversation_quality"
//! - GET REFLECTION INSIGHTS
//!
//! Config.csv properties:
//! ```csv
//! bot-reflection-enabled,true
//! bot-reflection-interval,10
//! bot-reflection-prompt,Analyze conversation quality and suggest improvements
//! bot-improvement-auto-apply,false
//! ```

use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use diesel::prelude::*;
use log::{debug, error, info, trace, warn};
use rhai::{Dynamic, Engine};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Types of reflection that can be performed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReflectionType {
    /// Analyze conversation quality and user satisfaction
    ConversationQuality,
    /// Analyze response accuracy and relevance
    ResponseAccuracy,
    /// Analyze tool usage effectiveness
    ToolUsage,
    /// Analyze knowledge retrieval performance
    KnowledgeRetrieval,
    /// Analyze overall bot performance
    Performance,
    /// Custom reflection with user-defined prompt
    Custom(String),
}

impl Default for ReflectionType {
    fn default() -> Self {
        ReflectionType::ConversationQuality
    }
}

impl From<&str> for ReflectionType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "conversation_quality" | "conversation" | "quality" => {
                ReflectionType::ConversationQuality
            }
            "response_accuracy" | "accuracy" | "response" => ReflectionType::ResponseAccuracy,
            "tool_usage" | "tools" => ReflectionType::ToolUsage,
            "knowledge_retrieval" | "knowledge" | "retrieval" => ReflectionType::KnowledgeRetrieval,
            "performance" | "overall" => ReflectionType::Performance,
            _ => ReflectionType::Custom(s.to_string()),
        }
    }
}

impl ReflectionType {
    /// Get the prompt template for this reflection type
    pub fn prompt_template(&self) -> String {
        match self {
            ReflectionType::ConversationQuality => {
                r#"Analyze the following conversation and evaluate:
1. User satisfaction indicators (positive/negative sentiment)
2. Conversation flow and coherence
3. Whether user's questions were fully addressed
4. Response tone appropriateness
5. Areas for improvement

Conversation:
{conversation}

Provide your analysis in JSON format:
{
    "score": 0-10,
    "satisfaction_indicators": ["..."],
    "strengths": ["..."],
    "weaknesses": ["..."],
    "improvements": ["..."],
    "patterns_noticed": ["..."]
}"#
                .to_string()
            }
            ReflectionType::ResponseAccuracy => {
                r#"Analyze the accuracy and relevance of responses in this conversation:
1. Were responses factually accurate?
2. Were responses relevant to the questions?
3. Were there any hallucinations or incorrect information?
4. Was the level of detail appropriate?
5. Were sources/references needed but not provided?

Conversation:
{conversation}

Provide your analysis in JSON format:
{
    "accuracy_score": 0-10,
    "relevance_score": 0-10,
    "factual_errors": ["..."],
    "hallucinations": ["..."],
    "missing_information": ["..."],
    "improvements": ["..."]
}"#
                .to_string()
            }
            ReflectionType::ToolUsage => r#"Analyze tool usage in this conversation:
1. Were tools used appropriately?
2. Were there missed opportunities to use tools?
3. Did tool outputs meet user needs?
4. Were there any tool errors or failures?
5. Could tools have been combined more effectively?

Conversation:
{conversation}

Tools Available:
{tools}

Provide your analysis in JSON format:
{
    "tool_effectiveness_score": 0-10,
    "tools_used": ["..."],
    "missed_opportunities": ["..."],
    "effective_uses": ["..."],
    "ineffective_uses": ["..."],
    "recommendations": ["..."]
}"#
            .to_string(),
            ReflectionType::KnowledgeRetrieval => {
                r#"Analyze knowledge base retrieval in this conversation:
1. Were relevant documents retrieved?
2. Was the context provided to the LLM appropriate?
3. Were there questions that should have used KB but didn't?
4. Was there irrelevant context that confused responses?
5. Were there knowledge gaps identified?

Conversation:
{conversation}

Retrieved Context:
{context}

Provide your analysis in JSON format:
{
    "retrieval_score": 0-10,
    "relevant_retrievals": ["..."],
    "irrelevant_retrievals": ["..."],
    "missed_retrievals": ["..."],
    "knowledge_gaps": ["..."],
    "improvements": ["..."]
}"#
                .to_string()
            }
            ReflectionType::Performance => {
                r#"Provide an overall performance analysis of this conversation:
1. Response quality and helpfulness
2. Efficiency (number of turns to resolve issues)
3. User engagement level
4. Task completion rate
5. Overall effectiveness

Conversation:
{conversation}

Provide your analysis in JSON format:
{
    "overall_score": 0-10,
    "response_quality": 0-10,
    "efficiency": 0-10,
    "engagement": 0-10,
    "task_completion": 0-10,
    "key_insights": ["..."],
    "critical_improvements": ["..."],
    "positive_patterns": ["..."]
}"#
                .to_string()
            }
            ReflectionType::Custom(prompt) => prompt.clone(),
        }
    }
}

/// Configuration for agent reflection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionConfig {
    /// Whether reflection is enabled
    pub enabled: bool,
    /// Number of interactions between reflections
    pub interval: u32,
    /// Custom reflection prompt override
    pub custom_prompt: Option<String>,
    /// Whether to automatically apply improvements
    pub auto_apply: bool,
    /// Minimum score threshold for triggering improvements
    pub improvement_threshold: f32,
    /// Maximum number of insights to store
    pub max_insights: usize,
    /// Types of reflection to perform
    pub reflection_types: Vec<ReflectionType>,
}

impl Default for ReflectionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interval: 10,
            custom_prompt: None,
            auto_apply: false,
            improvement_threshold: 6.0,
            max_insights: 100,
            reflection_types: vec![ReflectionType::ConversationQuality],
        }
    }
}

impl ReflectionConfig {
    /// Load configuration from bot settings
    pub fn from_bot_config(state: &AppState, bot_id: Uuid) -> Self {
        let mut config = Self::default();

        if let Ok(mut conn) = state.conn.get() {
            #[derive(QueryableByName)]
            struct ConfigRow {
                #[diesel(sql_type = diesel::sql_types::Text)]
                config_key: String,
                #[diesel(sql_type = diesel::sql_types::Text)]
                config_value: String,
            }

            let configs: Vec<ConfigRow> = diesel::sql_query(
                "SELECT config_key, config_value FROM bot_configuration \
                 WHERE bot_id = $1 AND config_key LIKE 'bot-reflection-%' OR config_key LIKE 'bot-improvement-%'",
            )
            .bind::<diesel::sql_types::Uuid, _>(bot_id)
            .load(&mut conn)
            .unwrap_or_default();

            for row in configs {
                match row.config_key.as_str() {
                    "bot-reflection-enabled" => {
                        config.enabled = row.config_value.to_lowercase() == "true";
                    }
                    "bot-reflection-interval" => {
                        config.interval = row.config_value.parse().unwrap_or(10);
                    }
                    "bot-reflection-prompt" => {
                        config.custom_prompt = Some(row.config_value);
                    }
                    "bot-improvement-auto-apply" => {
                        config.auto_apply = row.config_value.to_lowercase() == "true";
                    }
                    "bot-improvement-threshold" => {
                        config.improvement_threshold = row.config_value.parse().unwrap_or(6.0);
                    }
                    "bot-reflection-types" => {
                        config.reflection_types = row
                            .config_value
                            .split(';')
                            .map(|s| ReflectionType::from(s.trim()))
                            .collect();
                    }
                    _ => {}
                }
            }
        }

        config
    }
}

/// Result of a reflection operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionResult {
    /// Unique identifier for this reflection
    pub id: Uuid,
    /// Bot that performed the reflection
    pub bot_id: Uuid,
    /// Session that triggered the reflection
    pub session_id: Uuid,
    /// Type of reflection performed
    pub reflection_type: ReflectionType,
    /// Overall score (0-10)
    pub score: f32,
    /// Key insights discovered
    pub insights: Vec<String>,
    /// Suggested improvements
    pub improvements: Vec<String>,
    /// Positive patterns noticed
    pub positive_patterns: Vec<String>,
    /// Areas of concern
    pub concerns: Vec<String>,
    /// Raw JSON response from LLM
    pub raw_response: String,
    /// Timestamp of reflection
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Number of messages analyzed
    pub messages_analyzed: usize,
}

impl ReflectionResult {
    pub fn new(bot_id: Uuid, session_id: Uuid, reflection_type: ReflectionType) -> Self {
        Self {
            id: Uuid::new_v4(),
            bot_id,
            session_id,
            reflection_type,
            score: 0.0,
            insights: Vec::new(),
            improvements: Vec::new(),
            positive_patterns: Vec::new(),
            concerns: Vec::new(),
            raw_response: String::new(),
            timestamp: chrono::Utc::now(),
            messages_analyzed: 0,
        }
    }

    /// Parse LLM response into structured result
    pub fn from_llm_response(
        bot_id: Uuid,
        session_id: Uuid,
        reflection_type: ReflectionType,
        response: &str,
        messages_analyzed: usize,
    ) -> Self {
        let mut result = Self::new(bot_id, session_id, reflection_type);
        result.raw_response = response.to_string();
        result.messages_analyzed = messages_analyzed;

        // Try to parse JSON response
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(response) {
            // Extract score
            result.score = json
                .get("score")
                .or_else(|| json.get("overall_score"))
                .and_then(|v| v.as_f64())
                .unwrap_or(5.0) as f32;

            // Extract insights
            if let Some(insights) = json.get("key_insights").or_else(|| json.get("insights")) {
                if let Some(arr) = insights.as_array() {
                    result.insights = arr
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();
                }
            }

            // Extract improvements
            if let Some(improvements) = json
                .get("improvements")
                .or_else(|| json.get("critical_improvements"))
                .or_else(|| json.get("recommendations"))
            {
                if let Some(arr) = improvements.as_array() {
                    result.improvements = arr
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();
                }
            }

            // Extract positive patterns
            if let Some(patterns) = json
                .get("positive_patterns")
                .or_else(|| json.get("strengths"))
                .or_else(|| json.get("effective_uses"))
            {
                if let Some(arr) = patterns.as_array() {
                    result.positive_patterns = arr
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();
                }
            }

            // Extract concerns
            if let Some(concerns) = json
                .get("weaknesses")
                .or_else(|| json.get("concerns"))
                .or_else(|| json.get("factual_errors"))
                .or_else(|| json.get("knowledge_gaps"))
            {
                if let Some(arr) = concerns.as_array() {
                    result.concerns = arr
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();
                }
            }
        } else {
            // If not valid JSON, try to extract insights from plain text
            warn!("Reflection response was not valid JSON, extracting from plain text");
            result.insights = extract_insights_from_text(response);
            result.score = 5.0; // Default middle score
        }

        result
    }

    /// Check if this reflection indicates need for improvement
    pub fn needs_improvement(&self, threshold: f32) -> bool {
        self.score < threshold
    }

    /// Get a summary of the reflection
    pub fn summary(&self) -> String {
        format!(
            "Reflection Score: {:.1}/10\n\
             Messages Analyzed: {}\n\
             Key Insights: {}\n\
             Improvements Needed: {}\n\
             Positive Patterns: {}",
            self.score,
            self.messages_analyzed,
            self.insights.len(),
            self.improvements.len(),
            self.positive_patterns.len()
        )
    }
}

/// Extract insights from plain text response
fn extract_insights_from_text(text: &str) -> Vec<String> {
    let mut insights = Vec::new();

    // Look for numbered lists
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(|c: char| c.is_ascii_digit())
            || trimmed.starts_with('-')
            || trimmed.starts_with('•')
            || trimmed.starts_with('*')
        {
            let cleaned = trimmed
                .trim_start_matches(|c: char| {
                    c.is_ascii_digit() || c == '.' || c == '-' || c == '•' || c == '*'
                })
                .trim();
            if !cleaned.is_empty() && cleaned.len() > 10 {
                insights.push(cleaned.to_string());
            }
        }
    }

    // If no list items found, split by sentences
    if insights.is_empty() {
        for sentence in text.split(|c| c == '.' || c == '!' || c == '?') {
            let trimmed = sentence.trim();
            if trimmed.len() > 20 && trimmed.len() < 200 {
                insights.push(format!("{}.", trimmed));
            }
        }
    }

    insights.truncate(10); // Limit to 10 insights
    insights
}

/// Agent Reflection Engine
pub struct ReflectionEngine {
    state: Arc<AppState>,
    config: ReflectionConfig,
    bot_id: Uuid,
}

impl ReflectionEngine {
    pub fn new(state: Arc<AppState>, bot_id: Uuid) -> Self {
        let config = ReflectionConfig::from_bot_config(&state, bot_id);
        Self {
            state,
            config,
            bot_id,
        }
    }

    pub fn with_config(state: Arc<AppState>, bot_id: Uuid, config: ReflectionConfig) -> Self {
        Self {
            state,
            config,
            bot_id,
        }
    }

    /// Perform reflection on recent conversation history
    pub async fn reflect(
        &self,
        session_id: Uuid,
        reflection_type: ReflectionType,
    ) -> Result<ReflectionResult, String> {
        if !self.config.enabled {
            return Err("Reflection is not enabled for this bot".to_string());
        }

        // Get recent conversation history
        let history = self.get_recent_history(session_id, 20).await?;

        if history.is_empty() {
            return Err("No conversation history to analyze".to_string());
        }

        let messages_count = history.len();

        // Build reflection prompt
        let prompt = self.build_reflection_prompt(&reflection_type, &history)?;

        // Call LLM for reflection
        let response = self.call_llm_for_reflection(&prompt).await?;

        // Parse response into structured result
        let result = ReflectionResult::from_llm_response(
            self.bot_id,
            session_id,
            reflection_type,
            &response,
            messages_count,
        );

        // Store reflection result
        self.store_reflection(&result).await?;

        // Auto-apply improvements if configured
        if self.config.auto_apply && result.needs_improvement(self.config.improvement_threshold) {
            self.apply_improvements(&result).await?;
        }

        info!(
            "Reflection completed for bot {} session {}: score {:.1}",
            self.bot_id, session_id, result.score
        );

        Ok(result)
    }

    /// Get recent conversation history
    async fn get_recent_history(
        &self,
        session_id: Uuid,
        limit: usize,
    ) -> Result<Vec<ConversationMessage>, String> {
        let mut conn = self
            .state
            .conn
            .get()
            .map_err(|e| format!("Failed to acquire database connection: {}", e))?;

        #[derive(QueryableByName)]
        struct MessageRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            role: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            content: String,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            created_at: chrono::DateTime<chrono::Utc>,
        }

        let messages: Vec<MessageRow> = diesel::sql_query(
            "SELECT role, content, created_at FROM conversation_messages \
             WHERE session_id = $1 ORDER BY created_at DESC LIMIT $2",
        )
        .bind::<diesel::sql_types::Uuid, _>(session_id)
        .bind::<diesel::sql_types::Integer, _>(limit as i32)
        .load(&mut conn)
        .unwrap_or_default();

        let history: Vec<ConversationMessage> = messages
            .into_iter()
            .rev() // Reverse to get chronological order
            .map(|m| ConversationMessage {
                role: m.role,
                content: m.content,
                timestamp: m.created_at,
            })
            .collect();

        Ok(history)
    }

    /// Build the reflection prompt
    fn build_reflection_prompt(
        &self,
        reflection_type: &ReflectionType,
        history: &[ConversationMessage],
    ) -> Result<String, String> {
        let template = if let Some(custom) = &self.config.custom_prompt {
            custom.clone()
        } else {
            reflection_type.prompt_template()
        };

        // Format conversation history
        let conversation = history
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

        let prompt = template
            .replace("{conversation}", &conversation)
            .replace("{tools}", "Standard tools available")
            .replace("{context}", "Retrieved context from knowledge base");

        Ok(prompt)
    }

    /// Call LLM for reflection analysis
    async fn call_llm_for_reflection(&self, prompt: &str) -> Result<String, String> {
        // Get LLM configuration
        let (llm_url, llm_model, llm_key) = self.get_llm_config().await?;

        let client = reqwest::Client::new();

        let messages = serde_json::json!([
            {
                "role": "system",
                "content": "You are an AI performance analyst. Analyze conversations and provide structured feedback in JSON format. Be objective and constructive."
            },
            {
                "role": "user",
                "content": prompt
            }
        ]);

        let request_body = serde_json::json!({
            "model": llm_model,
            "messages": messages,
            "temperature": 0.3,
            "max_tokens": 1000
        });

        let response = client
            .post(&format!("{}/v1/chat/completions", llm_url))
            .header("Authorization", format!("Bearer {}", llm_key))
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("LLM request failed: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("LLM error: {}", error_text));
        }

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse LLM response: {}", e))?;

        let content = result["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(content)
    }

    /// Get LLM configuration
    async fn get_llm_config(&self) -> Result<(String, String, String), String> {
        let mut conn = self
            .state
            .conn
            .get()
            .map_err(|e| format!("Failed to acquire database connection: {}", e))?;

        #[derive(QueryableByName)]
        struct ConfigRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            config_key: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            config_value: String,
        }

        let configs: Vec<ConfigRow> = diesel::sql_query(
            "SELECT config_key, config_value FROM bot_configuration \
             WHERE bot_id = $1 AND config_key IN ('llm-url', 'llm-model', 'llm-key')",
        )
        .bind::<diesel::sql_types::Uuid, _>(self.bot_id)
        .load(&mut conn)
        .unwrap_or_default();

        let mut llm_url = "http://localhost:8081".to_string();
        let mut llm_model = "default".to_string();
        let mut llm_key = "none".to_string();

        for config in configs {
            match config.config_key.as_str() {
                "llm-url" => llm_url = config.config_value,
                "llm-model" => llm_model = config.config_value,
                "llm-key" => llm_key = config.config_value,
                _ => {}
            }
        }

        Ok((llm_url, llm_model, llm_key))
    }

    /// Store reflection result in database
    async fn store_reflection(&self, result: &ReflectionResult) -> Result<(), String> {
        let mut conn = self
            .state
            .conn
            .get()
            .map_err(|e| format!("Failed to acquire database connection: {}", e))?;

        let insights_json = serde_json::to_string(&result.insights).unwrap_or_default();
        let improvements_json = serde_json::to_string(&result.improvements).unwrap_or_default();
        let patterns_json = serde_json::to_string(&result.positive_patterns).unwrap_or_default();
        let concerns_json = serde_json::to_string(&result.concerns).unwrap_or_default();
        let reflection_type_str =
            serde_json::to_string(&result.reflection_type).unwrap_or_default();

        diesel::sql_query(
            "INSERT INTO bot_reflections \
             (id, bot_id, session_id, reflection_type, score, insights, improvements, \
              positive_patterns, concerns, raw_response, messages_analyzed, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
        )
        .bind::<diesel::sql_types::Uuid, _>(result.id)
        .bind::<diesel::sql_types::Uuid, _>(result.bot_id)
        .bind::<diesel::sql_types::Uuid, _>(result.session_id)
        .bind::<diesel::sql_types::Text, _>(&reflection_type_str)
        .bind::<diesel::sql_types::Float, _>(result.score)
        .bind::<diesel::sql_types::Text, _>(&insights_json)
        .bind::<diesel::sql_types::Text, _>(&improvements_json)
        .bind::<diesel::sql_types::Text, _>(&patterns_json)
        .bind::<diesel::sql_types::Text, _>(&concerns_json)
        .bind::<diesel::sql_types::Text, _>(&result.raw_response)
        .bind::<diesel::sql_types::Integer, _>(result.messages_analyzed as i32)
        .bind::<diesel::sql_types::Timestamptz, _>(result.timestamp)
        .execute(&mut conn)
        .map_err(|e| format!("Failed to store reflection: {}", e))?;

        Ok(())
    }

    /// Apply improvements based on reflection results
    async fn apply_improvements(&self, result: &ReflectionResult) -> Result<(), String> {
        // Store improvements as bot memory for future reference
        let mut conn = self
            .state
            .conn
            .get()
            .map_err(|e| format!("Failed to acquire database connection: {}", e))?;

        for improvement in &result.improvements {
            let key = format!("improvement_{}", Uuid::new_v4());
            let now = chrono::Utc::now();

            diesel::sql_query(
                "INSERT INTO bot_memories (id, bot_id, key, value, created_at, updated_at) \
                 VALUES ($1, $2, $3, $4, $5, $6) \
                 ON CONFLICT (bot_id, key) DO UPDATE SET value = $4, updated_at = $6",
            )
            .bind::<diesel::sql_types::Uuid, _>(Uuid::new_v4())
            .bind::<diesel::sql_types::Uuid, _>(self.bot_id)
            .bind::<diesel::sql_types::Text, _>(&key)
            .bind::<diesel::sql_types::Text, _>(improvement)
            .bind::<diesel::sql_types::Timestamptz, _>(now)
            .bind::<diesel::sql_types::Timestamptz, _>(now)
            .execute(&mut conn)
            .ok();
        }

        info!(
            "Applied {} improvements for bot {}",
            result.improvements.len(),
            self.bot_id
        );

        Ok(())
    }

    /// Get recent reflection insights
    pub async fn get_insights(&self, limit: usize) -> Result<Vec<ReflectionResult>, String> {
        let mut conn = self
            .state
            .conn
            .get()
            .map_err(|e| format!("Failed to acquire database connection: {}", e))?;

        #[derive(QueryableByName)]
        struct ReflectionRow {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            session_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Text)]
            reflection_type: String,
            #[diesel(sql_type = diesel::sql_types::Float)]
            score: f32,
            #[diesel(sql_type = diesel::sql_types::Text)]
            insights: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            improvements: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            positive_patterns: String,
            #[diesel(sql_type = diesel::sql_types::Integer)]
            messages_analyzed: i32,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            created_at: chrono::DateTime<chrono::Utc>,
        }

        let rows: Vec<ReflectionRow> = diesel::sql_query(
            "SELECT id, session_id, reflection_type, score, insights, improvements, \
             positive_patterns, messages_analyzed, created_at \
             FROM bot_reflections WHERE bot_id = $1 ORDER BY created_at DESC LIMIT $2",
        )
        .bind::<diesel::sql_types::Uuid, _>(self.bot_id)
        .bind::<diesel::sql_types::Integer, _>(limit as i32)
        .load(&mut conn)
        .unwrap_or_default();

        let results: Vec<ReflectionResult> = rows
            .into_iter()
            .map(|row| {
                let reflection_type: ReflectionType =
                    serde_json::from_str(&row.reflection_type).unwrap_or_default();
                let insights: Vec<String> = serde_json::from_str(&row.insights).unwrap_or_default();
                let improvements: Vec<String> =
                    serde_json::from_str(&row.improvements).unwrap_or_default();
                let positive_patterns: Vec<String> =
                    serde_json::from_str(&row.positive_patterns).unwrap_or_default();

                ReflectionResult {
                    id: row.id,
                    bot_id: self.bot_id,
                    session_id: row.session_id,
                    reflection_type,
                    score: row.score,
                    insights,
                    improvements,
                    positive_patterns,
                    concerns: Vec::new(),
                    raw_response: String::new(),
                    timestamp: row.created_at,
                    messages_analyzed: row.messages_analyzed as usize,
                }
            })
            .collect();

        Ok(results)
    }

    /// Check if reflection should be triggered based on interaction count
    pub async fn should_reflect(&self, session_id: Uuid) -> bool {
        if !self.config.enabled {
            return false;
        }

        // Get message count for session
        if let Ok(mut conn) = self.state.conn.get() {
            #[derive(QueryableByName)]
            struct CountRow {
                #[diesel(sql_type = diesel::sql_types::BigInt)]
                count: i64,
            }

            let result: Option<CountRow> = diesel::sql_query(
                "SELECT COUNT(*) as count FROM conversation_messages WHERE session_id = $1",
            )
            .bind::<diesel::sql_types::Uuid, _>(session_id)
            .get_result(&mut conn)
            .ok();

            if let Some(row) = result {
                return row.count > 0 && row.count as u32 % self.config.interval == 0;
            }
        }

        false
    }
}

/// Conversation message for analysis
#[derive(Debug, Clone)]
struct ConversationMessage {
    pub role: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Register reflection keywords with the engine
pub fn register_reflection_keywords(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    set_bot_reflection_keyword(state.clone(), user.clone(), engine);
    reflect_on_keyword(state.clone(), user.clone(), engine);
    get_reflection_insights_keyword(state.clone(), user.clone(), engine);
}

/// SET BOT REFLECTION true/false
pub fn set_bot_reflection_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine
        .register_custom_syntax(
            &["SET", "BOT", "REFLECTION", "$expr$"],
            false,
            move |context, inputs| {
                let value = context
                    .eval_expression_tree(&inputs[0])?
                    .to_string()
                    .to_lowercase();

                let enabled = value == "true" || value == "1" || value == "on";

                trace!(
                    "SET BOT REFLECTION {} for bot: {}",
                    enabled,
                    user_clone.bot_id
                );

                let state_for_task = Arc::clone(&state_clone);
                let bot_id = user_clone.bot_id;

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
                    let result = rt.block_on(async {
                        set_reflection_enabled(&state_for_task, bot_id, enabled).await
                    });
                    let _ = tx.send(result);
                });

                match rx.recv_timeout(std::time::Duration::from_secs(10)) {
                    Ok(Ok(msg)) => Ok(Dynamic::from(msg)),
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        e.into(),
                        rhai::Position::NONE,
                    ))),
                    Err(_) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        "SET BOT REFLECTION timed out".into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .expect("Failed to register SET BOT REFLECTION syntax");
}

/// REFLECT ON "type"
pub fn reflect_on_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine
        .register_custom_syntax(
            &["REFLECT", "ON", "$expr$"],
            false,
            move |context, inputs| {
                let reflection_type_str = context
                    .eval_expression_tree(&inputs[0])?
                    .to_string()
                    .trim_matches('"')
                    .to_string();

                let reflection_type = ReflectionType::from(reflection_type_str.as_str());

                trace!(
                    "REFLECT ON {:?} for session: {}",
                    reflection_type,
                    user_clone.id
                );

                let state_for_task = Arc::clone(&state_clone);
                let session_id = user_clone.id;
                let bot_id = user_clone.bot_id;

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
                    let result = rt.block_on(async {
                        let engine = ReflectionEngine::new(state_for_task, bot_id);
                        engine.reflect(session_id, reflection_type).await
                    });
                    let _ = tx.send(result);
                });

                match rx.recv_timeout(std::time::Duration::from_secs(60)) {
                    Ok(Ok(result)) => Ok(Dynamic::from(result.summary())),
                    Ok(Err(e)) => Ok(Dynamic::from(format!("Reflection failed: {}", e))),
                    Err(_) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        "REFLECT ON timed out".into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .expect("Failed to register REFLECT ON syntax");
}

/// GET REFLECTION INSIGHTS()
pub fn get_reflection_insights_keyword(
    state: Arc<AppState>,
    user: UserSession,
    engine: &mut Engine,
) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine.register_fn("GET REFLECTION INSIGHTS", move || -> rhai::Array {
        let state = Arc::clone(&state_clone);
        let bot_id = user_clone.bot_id;

        let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
        let result = rt.block_on(async {
            let engine = ReflectionEngine::new(state, bot_id);
            engine.get_insights(10).await
        });

        match result {
            Ok(insights) => insights
                .into_iter()
                .map(|i| Dynamic::from(i.summary()))
                .collect(),
            Err(_) => rhai::Array::new(),
        }
    });
}

/// Helper function to set reflection enabled state
async fn set_reflection_enabled(
    state: &AppState,
    bot_id: Uuid,
    enabled: bool,
) -> Result<String, String> {
    let mut conn = state
        .conn
        .get()
        .map_err(|e| format!("Failed to acquire database connection: {}", e))?;

    let now = chrono::Utc::now();
    let new_id = Uuid::new_v4();
    let value = if enabled { "true" } else { "false" };

    diesel::sql_query(
        "INSERT INTO bot_configuration (id, bot_id, config_key, config_value, config_type, created_at, updated_at) \
         VALUES ($1, $2, 'bot-reflection-enabled', $3, 'boolean', $4, $4) \
         ON CONFLICT (bot_id, config_key) DO UPDATE SET config_value = $3, updated_at = $4",
    )
    .bind::<diesel::sql_types::Uuid, _>(new_id)
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .bind::<diesel::sql_types::Text, _>(value)
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .execute(&mut conn)
    .map_err(|e| format!("Failed to set reflection enabled: {}", e))?;

    Ok(format!(
        "Bot reflection {}",
        if enabled { "enabled" } else { "disabled" }
    ))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reflection_type_from_str() {
        assert_eq!(
            ReflectionType::from("conversation_quality"),
            ReflectionType::ConversationQuality
        );
        assert_eq!(
            ReflectionType::from("quality"),
            ReflectionType::ConversationQuality
        );
        assert_eq!(
            ReflectionType::from("tool_usage"),
            ReflectionType::ToolUsage
        );
        assert_eq!(
            ReflectionType::from("performance"),
            ReflectionType::Performance
        );
    }

    #[test]
    fn test_reflection_config_default() {
        let config = ReflectionConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.interval, 10);
        assert!(!config.auto_apply);
    }

    #[test]
    fn test_reflection_result_new() {
        let bot_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let result = ReflectionResult::new(bot_id, session_id, ReflectionType::ConversationQuality);

        assert_eq!(result.bot_id, bot_id);
        assert_eq!(result.session_id, session_id);
        assert_eq!(result.score, 0.0);
        assert!(result.insights.is_empty());
    }

    #[test]
    fn test_reflection_result_from_json() {
        let json_response = r#"{
            "score": 7.5,
            "key_insights": ["Users prefer concise responses", "Technical questions need more detail"],
            "improvements": ["Add more examples", "Improve response time"],
            "positive_patterns": ["Good greeting", "Clear explanations"]
        }"#;

        let result = ReflectionResult::from_llm_response(
            Uuid::new_v4(),
            Uuid::new_v4(),
            ReflectionType::ConversationQuality,
            json_response,
            10,
        );

        assert_eq!(result.score, 7.5);
        assert_eq!(result.insights.len(), 2);
        assert_eq!(result.improvements.len(), 2);
        assert_eq!(result.positive_patterns.len(), 2);
    }

    #[test]
    fn test_reflection_result_needs_improvement() {
        let mut result =
            ReflectionResult::new(Uuid::new_v4(), Uuid::new_v4(), ReflectionType::Performance);

        result.score = 5.0;
        assert!(result.needs_improvement(6.0));

        result.score = 8.0;
        assert!(!result.needs_improvement(6.0));
    }

    #[test]
    fn test_extract_insights_from_text() {
        let text = "Here are some insights:\n\
                    1. Users prefer short responses\n\
                    2. Technical questions need examples\n\
                    - Consider adding more context\n\
                    • Improve response time";

        let insights = extract_insights_from_text(text);
        assert!(!insights.is_empty());
    }

    #[test]
    fn test_reflection_type_prompt_template() {
        let template = ReflectionType::ConversationQuality.prompt_template();
        assert!(template.contains("{conversation}"));
        assert!(template.contains("JSON format"));
    }

    #[test]
    fn test_reflection_result_summary() {
        let mut result =
            ReflectionResult::new(Uuid::new_v4(), Uuid::new_v4(), ReflectionType::Performance);
        result.score = 7.5;
        result.messages_analyzed = 15;
        result.insights = vec!["Insight 1".to_string(), "Insight 2".to_string()];
        result.improvements = vec!["Improvement 1".to_string()];

        let summary = result.summary();
        assert!(summary.contains("7.5"));
        assert!(summary.contains("15"));
        assert!(summary.contains("2")); // insights count
        assert!(summary.contains("1")); // improvements count
    }
}
