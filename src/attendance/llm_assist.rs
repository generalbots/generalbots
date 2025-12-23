




































use crate::core::config::ConfigManager;
use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use diesel::prelude::*;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;




#[derive(Debug, Clone, Default)]
pub struct LlmAssistConfig {

    pub tips_enabled: bool,

    pub polish_enabled: bool,

    pub smart_replies_enabled: bool,

    pub auto_summary_enabled: bool,

    pub sentiment_enabled: bool,

    pub bot_system_prompt: Option<String>,

    pub bot_description: Option<String>,
}

impl LlmAssistConfig {

    pub fn from_config(bot_id: Uuid, work_path: &str) -> Self {
        let config_path = PathBuf::from(work_path)
            .join(format!("{}.gbai", bot_id))
            .join("config.csv");

        let alt_path = PathBuf::from(work_path).join("config.csv");

        let path = if config_path.exists() {
            config_path
        } else if alt_path.exists() {
            alt_path
        } else {
            return Self::default();
        };

        let mut config = Self::default();

        if let Ok(content) = std::fs::read_to_string(&path) {
            for line in content.lines() {
                let line_lower = line.to_lowercase();
                let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();

                if parts.len() < 2 {
                    continue;
                }

                let key = parts[0].to_lowercase();
                let value = parts[1];

                match key.as_str() {
                    "attendant-llm-tips" => {
                        config.tips_enabled = value.to_lowercase() == "true";
                    }
                    "attendant-polish-message" => {
                        config.polish_enabled = value.to_lowercase() == "true";
                    }
                    "attendant-smart-replies" => {
                        config.smart_replies_enabled = value.to_lowercase() == "true";
                    }
                    "attendant-auto-summary" => {
                        config.auto_summary_enabled = value.to_lowercase() == "true";
                    }
                    "attendant-sentiment-analysis" => {
                        config.sentiment_enabled = value.to_lowercase() == "true";
                    }
                    "bot-description" | "bot_description" => {
                        config.bot_description = Some(value.to_string());
                    }
                    "bot-system-prompt" | "system-prompt" => {
                        config.bot_system_prompt = Some(value.to_string());
                    }
                    _ => {}
                }
            }
        }

        info!(
            "LLM Assist config loaded: tips={}, polish={}, replies={}, summary={}, sentiment={}",
            config.tips_enabled,
            config.polish_enabled,
            config.smart_replies_enabled,
            config.auto_summary_enabled,
            config.sentiment_enabled
        );

        config
    }


    pub fn any_enabled(&self) -> bool {
        self.tips_enabled
            || self.polish_enabled
            || self.smart_replies_enabled
            || self.auto_summary_enabled
            || self.sentiment_enabled
    }
}




#[derive(Debug, Deserialize)]
pub struct TipRequest {
    pub session_id: Uuid,
    pub customer_message: String,

    #[serde(default)]
    pub history: Vec<ConversationMessage>,
}


#[derive(Debug, Deserialize)]
pub struct PolishRequest {
    pub session_id: Uuid,
    pub message: String,

    #[serde(default = "default_tone")]
    pub tone: String,
}

fn default_tone() -> String {
    "professional".to_string()
}


#[derive(Debug, Deserialize)]
pub struct SmartRepliesRequest {
    pub session_id: Uuid,
    #[serde(default)]
    pub history: Vec<ConversationMessage>,
}


#[derive(Debug, Deserialize)]
pub struct SummaryRequest {
    pub session_id: Uuid,
}


#[derive(Debug, Deserialize)]
pub struct SentimentRequest {
    pub session_id: Uuid,
    pub message: String,
    #[serde(default)]
    pub history: Vec<ConversationMessage>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub role: String,
    pub content: String,
    pub timestamp: Option<String>,
}


#[derive(Debug, Serialize)]
pub struct TipResponse {
    pub success: bool,
    pub tips: Vec<AttendantTip>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}


#[derive(Debug, Clone, Serialize)]
pub struct AttendantTip {
    pub tip_type: TipType,
    pub content: String,
    pub confidence: f32,
    pub priority: i32,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TipType {

    Intent,

    Action,

    Warning,

    Knowledge,

    History,

    General,
}


#[derive(Debug, Serialize)]
pub struct PolishResponse {
    pub success: bool,
    pub original: String,
    pub polished: String,
    pub changes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}


#[derive(Debug, Serialize)]
pub struct SmartRepliesResponse {
    pub success: bool,
    pub replies: Vec<SmartReply>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}


#[derive(Debug, Clone, Serialize)]
pub struct SmartReply {
    pub text: String,
    pub tone: String,
    pub confidence: f32,
    pub category: String,
}


#[derive(Debug, Serialize)]
pub struct SummaryResponse {
    pub success: bool,
    pub summary: ConversationSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}


#[derive(Debug, Clone, Serialize, Default)]
pub struct ConversationSummary {
    pub brief: String,
    pub key_points: Vec<String>,
    pub customer_needs: Vec<String>,
    pub unresolved_issues: Vec<String>,
    pub sentiment_trend: String,
    pub recommended_action: String,
    pub message_count: i32,
    pub duration_minutes: i32,
}


#[derive(Debug, Serialize)]
pub struct SentimentResponse {
    pub success: bool,
    pub sentiment: SentimentAnalysis,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}


#[derive(Debug, Clone, Serialize, Default)]
pub struct SentimentAnalysis {
    pub overall: String,
    pub score: f32,
    pub emotions: Vec<Emotion>,
    pub escalation_risk: String,
    pub urgency: String,
    pub emoji: String,
}


#[derive(Debug, Clone, Serialize)]
pub struct Emotion {
    pub name: String,
    pub intensity: f32,
}




async fn execute_llm_with_context(
    state: &Arc<AppState>,
    bot_id: Uuid,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let config_manager = ConfigManager::new(state.conn.clone());

    let model = config_manager
        .get_config(&bot_id, "llm-model", None)
        .unwrap_or_else(|_| {
            config_manager
                .get_config(&Uuid::nil(), "llm-model", None)
                .unwrap_or_default()
        });

    let key = config_manager
        .get_config(&bot_id, "llm-key", None)
        .unwrap_or_else(|_| {
            config_manager
                .get_config(&Uuid::nil(), "llm-key", None)
                .unwrap_or_default()
        });


    let messages = serde_json::json!([
        {
            "role": "system",
            "content": system_prompt
        },
        {
            "role": "user",
            "content": user_prompt
        }
    ]);

    let response = state
        .llm_provider
        .generate(user_prompt, &messages, &model, &key)
        .await?;


    let handler = crate::llm::llm_models::get_handler(&model);
    let processed = handler.process_content(&response);

    Ok(processed)
}


fn get_bot_system_prompt(bot_id: Uuid, work_path: &str) -> String {

    let config = LlmAssistConfig::from_config(bot_id, work_path);
    if let Some(prompt) = config.bot_system_prompt {
        return prompt;
    }


    let start_bas_path = PathBuf::from(work_path)
        .join(format!("{}.gbai", bot_id))
        .join(format!("{}.gbdialog", bot_id))
        .join("start.bas");

    if let Ok(content) = std::fs::read_to_string(&start_bas_path) {

        let mut description_lines = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("REM ") || trimmed.starts_with("' ") {
                let comment = trimmed.trim_start_matches("REM ").trim_start_matches("' ");
                description_lines.push(comment);
            } else if !trimmed.is_empty() {
                break;
            }
        }
        if !description_lines.is_empty() {
            return description_lines.join(" ");
        }
    }


    "You are a professional customer service assistant. Be helpful, empathetic, and solution-oriented. Maintain a friendly but professional tone.".to_string()
}





pub async fn generate_tips(
    State(state): State<Arc<AppState>>,
    Json(request): Json<TipRequest>,
) -> (StatusCode, Json<TipResponse>) {
    info!("Generating tips for session {}", request.session_id);


    let session_result = get_session(&state, request.session_id).await;
    let session = match session_result {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::NOT_FOUND,
                Json(TipResponse {
                    success: false,
                    tips: vec![],
                    error: Some(e),
                }),
            )
        }
    };


    let work_path = std::env::var("WORK_PATH").unwrap_or_else(|_| "./work".to_string());
    let config = LlmAssistConfig::from_config(session.bot_id, &work_path);

    if !config.tips_enabled {
        return (
            StatusCode::OK,
            Json(TipResponse {
                success: true,
                tips: vec![],
                error: Some("Tips feature is disabled".to_string()),
            }),
        );
    }


    let history_context = request
        .history
        .iter()
        .map(|m| format!("{}: {}", m.role, m.content))
        .collect::<Vec<_>>()
        .join("\n");

    let bot_prompt = get_bot_system_prompt(session.bot_id, &work_path);

    let system_prompt = format!(
        r#"You are an AI assistant helping a human customer service attendant.
The bot they are replacing has this personality: {}

Your job is to provide helpful tips to the attendant based on the customer's message.

Analyze the customer message and provide 2-4 actionable tips. For each tip, classify it as:
- intent: What the customer wants
- action: Suggested action for attendant
- warning: Sentiment or escalation concern
- knowledge: Relevant info they should know
- history: Insight from conversation history
- general: General helpful advice

Respond in JSON format:
{{
    "tips": [
        {{"type": "intent", "content": "...", "confidence": 0.9, "priority": 1}},
        {{"type": "action", "content": "...", "confidence": 0.8, "priority": 2}}
    ]
}}"#,
        bot_prompt
    );

    let user_prompt = format!(
        r#"Conversation history:
{}

Latest customer message: "{}"

Provide tips for the attendant."#,
        history_context, request.customer_message
    );

    match execute_llm_with_context(&state, session.bot_id, &system_prompt, &user_prompt).await {
        Ok(response) => {

            let tips = parse_tips_response(&response);
            (
                StatusCode::OK,
                Json(TipResponse {
                    success: true,
                    tips,
                    error: None,
                }),
            )
        }
        Err(e) => {
            error!("LLM error generating tips: {}", e);

            (
                StatusCode::OK,
                Json(TipResponse {
                    success: true,
                    tips: generate_fallback_tips(&request.customer_message),
                    error: Some(format!("LLM unavailable, using fallback: {}", e)),
                }),
            )
        }
    }
}



pub async fn polish_message(
    State(state): State<Arc<AppState>>,
    Json(request): Json<PolishRequest>,
) -> (StatusCode, Json<PolishResponse>) {
    info!("Polishing message for session {}", request.session_id);

    let session_result = get_session(&state, request.session_id).await;
    let session = match session_result {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::NOT_FOUND,
                Json(PolishResponse {
                    success: false,
                    original: request.message.clone(),
                    polished: request.message.clone(),
                    changes: vec![],
                    error: Some(e),
                }),
            )
        }
    };

    let work_path = std::env::var("WORK_PATH").unwrap_or_else(|_| "./work".to_string());
    let config = LlmAssistConfig::from_config(session.bot_id, &work_path);

    if !config.polish_enabled {
        return (
            StatusCode::OK,
            Json(PolishResponse {
                success: true,
                original: request.message.clone(),
                polished: request.message.clone(),
                changes: vec![],
                error: Some("Polish feature is disabled".to_string()),
            }),
        );
    }

    let bot_prompt = get_bot_system_prompt(session.bot_id, &work_path);

    let system_prompt = format!(
        r#"You are a professional editor helping a customer service attendant.
The service has this tone: {}

Your job is to polish the attendant's message to be more {} while:
1. Fixing grammar and spelling errors
2. Improving clarity and flow
3. Maintaining the original meaning
4. Keeping it natural (not robotic)

Respond in JSON format:
{{
    "polished": "The improved message",
    "changes": ["Changed X to Y", "Fixed grammar in..."]
}}"#,
        bot_prompt, request.tone
    );

    let user_prompt = format!(
        r#"Polish this message with a {} tone:

"{}""#,
        request.tone, request.message
    );

    match execute_llm_with_context(&state, session.bot_id, &system_prompt, &user_prompt).await {
        Ok(response) => {
            let (polished, changes) = parse_polish_response(&response, &request.message);
            (
                StatusCode::OK,
                Json(PolishResponse {
                    success: true,
                    original: request.message.clone(),
                    polished,
                    changes,
                    error: None,
                }),
            )
        }
        Err(e) => {
            error!("LLM error polishing message: {}", e);
            (
                StatusCode::OK,
                Json(PolishResponse {
                    success: false,
                    original: request.message.clone(),
                    polished: request.message.clone(),
                    changes: vec![],
                    error: Some(format!("LLM error: {}", e)),
                }),
            )
        }
    }
}



pub async fn generate_smart_replies(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SmartRepliesRequest>,
) -> (StatusCode, Json<SmartRepliesResponse>) {
    info!(
        "Generating smart replies for session {}",
        request.session_id
    );

    let session_result = get_session(&state, request.session_id).await;
    let session = match session_result {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::NOT_FOUND,
                Json(SmartRepliesResponse {
                    success: false,
                    replies: vec![],
                    error: Some(e),
                }),
            )
        }
    };

    let work_path = std::env::var("WORK_PATH").unwrap_or_else(|_| "./work".to_string());
    let config = LlmAssistConfig::from_config(session.bot_id, &work_path);

    if !config.smart_replies_enabled {
        return (
            StatusCode::OK,
            Json(SmartRepliesResponse {
                success: true,
                replies: vec![],
                error: Some("Smart replies feature is disabled".to_string()),
            }),
        );
    }

    let history_context = request
        .history
        .iter()
        .map(|m| format!("{}: {}", m.role, m.content))
        .collect::<Vec<_>>()
        .join("\n");

    let bot_prompt = get_bot_system_prompt(session.bot_id, &work_path);

    let system_prompt = format!(
        r#"You are an AI assistant helping a customer service attendant craft responses.
The service has this personality: {}

Generate exactly 3 reply suggestions that:
1. Are contextually appropriate
2. Sound natural and human (not robotic)
3. Vary in approach (one empathetic, one solution-focused, one follow-up)
4. Are ready to send (no placeholders like [name])

Respond in JSON format:
{{
    "replies": [
        {{"text": "...", "tone": "empathetic", "confidence": 0.9, "category": "answer"}},
        {{"text": "...", "tone": "professional", "confidence": 0.85, "category": "solution"}},
        {{"text": "...", "tone": "friendly", "confidence": 0.8, "category": "follow_up"}}
    ]
}}"#,
        bot_prompt
    );

    let user_prompt = format!(
        r#"Conversation:
{}

Generate 3 reply options for the attendant."#,
        history_context
    );

    match execute_llm_with_context(&state, session.bot_id, &system_prompt, &user_prompt).await {
        Ok(response) => {
            let replies = parse_smart_replies_response(&response);
            (
                StatusCode::OK,
                Json(SmartRepliesResponse {
                    success: true,
                    replies,
                    error: None,
                }),
            )
        }
        Err(e) => {
            error!("LLM error generating smart replies: {}", e);
            (
                StatusCode::OK,
                Json(SmartRepliesResponse {
                    success: true,
                    replies: generate_fallback_replies(),
                    error: Some(format!("LLM unavailable, using fallback: {}", e)),
                }),
            )
        }
    }
}



pub async fn generate_summary(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<Uuid>,
) -> (StatusCode, Json<SummaryResponse>) {
    info!("Generating summary for session {}", session_id);

    let session_result = get_session(&state, session_id).await;
    let session = match session_result {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::NOT_FOUND,
                Json(SummaryResponse {
                    success: false,
                    summary: ConversationSummary::default(),
                    error: Some(e),
                }),
            )
        }
    };

    let work_path = std::env::var("WORK_PATH").unwrap_or_else(|_| "./work".to_string());
    let config = LlmAssistConfig::from_config(session.bot_id, &work_path);

    if !config.auto_summary_enabled {
        return (
            StatusCode::OK,
            Json(SummaryResponse {
                success: true,
                summary: ConversationSummary::default(),
                error: Some("Auto-summary feature is disabled".to_string()),
            }),
        );
    }


    let history = load_conversation_history(&state, session_id).await;

    if history.is_empty() {
        return (
            StatusCode::OK,
            Json(SummaryResponse {
                success: true,
                summary: ConversationSummary {
                    brief: "No messages in conversation yet".to_string(),
                    ..Default::default()
                },
                error: None,
            }),
        );
    }

    let history_text = history
        .iter()
        .map(|m| format!("{}: {}", m.role, m.content))
        .collect::<Vec<_>>()
        .join("\n");

    let bot_prompt = get_bot_system_prompt(session.bot_id, &work_path);

    let system_prompt = format!(
        r#"You are an AI assistant helping a customer service attendant understand a conversation.
The bot/service personality is: {}

Analyze the conversation and provide a comprehensive summary.

Respond in JSON format:
{{
    "brief": "One sentence summary",
    "key_points": ["Point 1", "Point 2"],
    "customer_needs": ["Need 1", "Need 2"],
    "unresolved_issues": ["Issue 1"],
    "sentiment_trend": "improving/stable/declining",
    "recommended_action": "What the attendant should do next"
}}"#,
        bot_prompt
    );

    let user_prompt = format!(
        r#"Summarize this conversation:

{}"#,
        history_text
    );

    match execute_llm_with_context(&state, session.bot_id, &system_prompt, &user_prompt).await {
        Ok(response) => {
            let mut summary = parse_summary_response(&response);
            summary.message_count = history.len() as i32;


            if let (Some(first_ts), Some(last_ts)) = (
                history.first().and_then(|m| m.timestamp.as_ref()),
                history.last().and_then(|m| m.timestamp.as_ref()),
            ) {
                if let (Ok(first), Ok(last)) = (
                    chrono::DateTime::parse_from_rfc3339(first_ts),
                    chrono::DateTime::parse_from_rfc3339(last_ts),
                ) {
                    summary.duration_minutes = (last - first).num_minutes() as i32;
                }
            }

            (
                StatusCode::OK,
                Json(SummaryResponse {
                    success: true,
                    summary,
                    error: None,
                }),
            )
        }
        Err(e) => {
            error!("LLM error generating summary: {}", e);
            (
                StatusCode::OK,
                Json(SummaryResponse {
                    success: false,
                    summary: ConversationSummary {
                        brief: format!("Conversation with {} messages", history.len()),
                        message_count: history.len() as i32,
                        ..Default::default()
                    },
                    error: Some(format!("LLM error: {}", e)),
                }),
            )
        }
    }
}



pub async fn analyze_sentiment(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SentimentRequest>,
) -> impl IntoResponse {
    info!("Analyzing sentiment for session {}", request.session_id);

    let session_result = get_session(&state, request.session_id).await;
    let session = match session_result {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::NOT_FOUND,
                Json(SentimentResponse {
                    success: false,
                    sentiment: SentimentAnalysis::default(),
                    error: Some(e),
                }),
            )
        }
    };

    let work_path = std::env::var("WORK_PATH").unwrap_or_else(|_| "./work".to_string());
    let config = LlmAssistConfig::from_config(session.bot_id, &work_path);

    if !config.sentiment_enabled {

        let sentiment = analyze_sentiment_keywords(&request.message);
        return (
            StatusCode::OK,
            Json(SentimentResponse {
                success: true,
                sentiment,
                error: Some("LLM sentiment disabled, using keyword analysis".to_string()),
            }),
        );
    }

    let history_context = request
        .history
        .iter()
        .take(5)
        .map(|m| format!("{}: {}", m.role, m.content))
        .collect::<Vec<_>>()
        .join("\n");

    let system_prompt = r#"You are a sentiment analysis expert. Analyze the customer's emotional state.

Consider:
1. Overall sentiment (positive/neutral/negative)
2. Specific emotions present
3. Risk of escalation
4. Urgency level

Respond in JSON format:
{
    "overall": "positive|neutral|negative",
    "score": 0.5,
    "emotions": [{"name": "frustration", "intensity": 0.7}],
    "escalation_risk": "low|medium|high",
    "urgency": "low|normal|high|urgent",
    "emoji": "😐"
}"#;

    let user_prompt = format!(
        r#"Recent conversation:
{}

Current message to analyze: "{}"

Analyze the customer's sentiment."#,
        history_context, request.message
    );

    match execute_llm_with_context(&state, session.bot_id, system_prompt, &user_prompt).await {
        Ok(response) => {
            let sentiment = parse_sentiment_response(&response);
            (
                StatusCode::OK,
                Json(SentimentResponse {
                    success: true,
                    sentiment,
                    error: None,
                }),
            )
        }
        Err(e) => {
            error!("LLM error analyzing sentiment: {}", e);
            let sentiment = analyze_sentiment_keywords(&request.message);
            (
                StatusCode::OK,
                Json(SentimentResponse {
                    success: true,
                    sentiment,
                    error: Some(format!("LLM unavailable, using fallback: {}", e)),
                }),
            )
        }
    }
}



pub async fn get_llm_config(
    State(_state): State<Arc<AppState>>,
    Path(bot_id): Path<Uuid>,
) -> impl IntoResponse {
    let work_path = std::env::var("WORK_PATH").unwrap_or_else(|_| "./work".to_string());
    let config = LlmAssistConfig::from_config(bot_id, &work_path);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "tips_enabled": config.tips_enabled,
            "polish_enabled": config.polish_enabled,
            "smart_replies_enabled": config.smart_replies_enabled,
            "auto_summary_enabled": config.auto_summary_enabled,
            "sentiment_enabled": config.sentiment_enabled,
            "any_enabled": config.any_enabled()
        })),
    )
}




pub async fn process_attendant_command(
    state: &Arc<AppState>,
    attendant_phone: &str,
    command: &str,
    current_session: Option<Uuid>,
) -> Result<String, String> {
    let parts: Vec<&str> = command.trim().split_whitespace().collect();
    if parts.is_empty() {
        return Err("Empty command".to_string());
    }

    let cmd = parts[0].to_lowercase();
    let args: Vec<&str> = parts[1..].to_vec();

    match cmd.as_str() {
        "/queue" | "/fila" => handle_queue_command(state).await,
        "/take" | "/pegar" => handle_take_command(state, attendant_phone).await,
        "/status" => handle_status_command(state, attendant_phone, args).await,
        "/transfer" | "/transferir" => handle_transfer_command(state, current_session, args).await,
        "/resolve" | "/resolver" => handle_resolve_command(state, current_session).await,
        "/tips" | "/dicas" => handle_tips_command(state, current_session).await,
        "/polish" | "/polir" => {
            let message = args.join(" ");
            handle_polish_command(state, current_session, &message).await
        }
        "/replies" | "/respostas" => handle_replies_command(state, current_session).await,
        "/summary" | "/resumo" => handle_summary_command(state, current_session).await,
        "/help" | "/ajuda" => Ok(get_help_text()),
        _ => Err(format!(
            "Unknown command: {}. Type /help for available commands.",
            cmd
        )),
    }
}

async fn handle_queue_command(state: &Arc<AppState>) -> Result<String, String> {

    let conn = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| e.to_string())?;

        use crate::shared::models::schema::user_sessions;

        let sessions: Vec<UserSession> = user_sessions::table
            .filter(
                user_sessions::context_data
                    .retrieve_as_text("needs_human")
                    .eq("true"),
            )
            .filter(
                user_sessions::context_data
                    .retrieve_as_text("status")
                    .ne("resolved"),
            )
            .order(user_sessions::updated_at.desc())
            .limit(10)
            .load(&mut db_conn)
            .map_err(|e| e.to_string())?;

        Ok::<Vec<UserSession>, String>(sessions)
    })
    .await
    .map_err(|e| e.to_string())??;

    if result.is_empty() {
        return Ok(" *Queue is empty*\nNo conversations waiting for attention.".to_string());
    }

    let mut response = format!(" *Queue* ({} waiting)\n\n", result.len());

    for (i, session) in result.iter().enumerate() {
        let name = session
            .context_data
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");
        let channel = session
            .context_data
            .get("channel")
            .and_then(|v| v.as_str())
            .unwrap_or("web");
        let status = session
            .context_data
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("waiting");

        response.push_str(&format!(
            "{}. *{}* ({})\n   Status: {} | ID: {}\n\n",
            i + 1,
            name,
            channel,
            status,
            &session.id.to_string()[..8]
        ));
    }

    response.push_str("Type `/take` to take the next conversation.");

    Ok(response)
}

async fn handle_take_command(
    state: &Arc<AppState>,
    attendant_phone: &str,
) -> Result<String, String> {
    let conn = state.conn.clone();
    let phone = attendant_phone.to_string();

    let result = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| e.to_string())?;

        use crate::shared::models::schema::user_sessions;


        let session: Option<UserSession> = user_sessions::table
            .filter(
                user_sessions::context_data
                    .retrieve_as_text("needs_human")
                    .eq("true"),
            )
            .filter(
                user_sessions::context_data
                    .retrieve_as_text("status")
                    .eq("waiting"),
            )
            .order(user_sessions::updated_at.asc())
            .first(&mut db_conn)
            .optional()
            .map_err(|e| e.to_string())?;

        if let Some(session) = session {

            let mut ctx = session.context_data.clone();
            ctx["assigned_to_phone"] = serde_json::json!(phone);
            ctx["status"] = serde_json::json!("assigned");
            ctx["assigned_at"] = serde_json::json!(Utc::now().to_rfc3339());

            diesel::update(user_sessions::table.filter(user_sessions::id.eq(session.id)))
                .set(user_sessions::context_data.eq(&ctx))
                .execute(&mut db_conn)
                .map_err(|e| e.to_string())?;

            let name = session
                .context_data
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown");

            Ok::<String, String>(format!(
                " *Conversation assigned*\n\nCustomer: *{}*\nSession: {}\n\nYou can now respond to this customer. Their messages will be forwarded to you.",
                name,
                &session.id.to_string()[..8]
            ))
        } else {
            Ok::<String, String>(" No conversations waiting in queue.".to_string())
        }
    })
    .await
    .map_err(|e| e.to_string())??;

    Ok(result)
}

async fn handle_status_command(
    state: &Arc<AppState>,
    attendant_phone: &str,
    args: Vec<&str>,
) -> Result<String, String> {
    if args.is_empty() {
        return Ok(
            " *Status Options*\n\n`/status online` - Available\n`/status busy` - In conversation\n`/status away` - Temporarily away\n`/status offline` - Not available"
                .to_string(),
        );
    }

    let status = args[0].to_lowercase();
    let (emoji, text, status_value) = match status.as_str() {
        "online" => ("", "Online - Available for conversations", "online"),
        "busy" => ("", "Busy - Handling conversations", "busy"),
        "away" => ("", "Away - Temporarily unavailable", "away"),
        "offline" => ("", "Offline - Not available", "offline"),
        _ => {
            return Err(format!(
                "Invalid status: {}. Use online, busy, away, or offline.",
                status
            ))
        }
    };



    let conn = state.conn.clone();
    let phone = attendant_phone.to_string();
    let status_val = status_value.to_string();

    let update_result = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| e.to_string())?;

        use crate::shared::models::schema::user_sessions;



        let sessions: Vec<UserSession> = user_sessions::table
            .filter(
                user_sessions::context_data
                    .retrieve_as_text("assigned_to_phone")
                    .eq(&phone),
            )
            .load(&mut db_conn)
            .map_err(|e| e.to_string())?;

        let session_count = sessions.len();
        for session in sessions {
            let mut ctx = session.context_data.clone();
            ctx["attendant_status"] = serde_json::json!(status_val);
            ctx["attendant_status_updated_at"] = serde_json::json!(Utc::now().to_rfc3339());

            diesel::update(user_sessions::table.filter(user_sessions::id.eq(session.id)))
                .set(user_sessions::context_data.eq(&ctx))
                .execute(&mut db_conn)
                .map_err(|e| e.to_string())?;
        }

        Ok::<usize, String>(session_count)
    })
    .await
    .map_err(|e| e.to_string())?;

    match update_result {
        Ok(count) => {
            info!(
                "Attendant {} set status to {} ({} sessions updated)",
                attendant_phone, status_value, count
            );
            Ok(format!("{} Status set to *{}*", emoji, text))
        }
        Err(e) => {
            warn!("Failed to persist status for {}: {}", attendant_phone, e);

            Ok(format!("{} Status set to *{}*", emoji, text))
        }
    }
}

async fn handle_transfer_command(
    state: &Arc<AppState>,
    current_session: Option<Uuid>,
    args: Vec<&str>,
) -> Result<String, String> {
    let session_id = current_session.ok_or("No active conversation to transfer")?;

    if args.is_empty() {
        return Err("Usage: `/transfer @attendant_name` or `/transfer department`".to_string());
    }

    let target = args.join(" ");
    let target_clean = target.trim_start_matches('@').to_string();


    let conn = state.conn.clone();
    let target_attendant = target_clean.clone();

    let transfer_result = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| e.to_string())?;

        use crate::shared::models::schema::user_sessions;


        let session: UserSession = user_sessions::table
            .find(session_id)
            .first(&mut db_conn)
            .map_err(|e| format!("Session not found: {}", e))?;


        let mut ctx = session.context_data.clone();
        let previous_attendant = ctx
            .get("assigned_to_phone")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        ctx["transferred_from"] = serde_json::json!(previous_attendant);
        ctx["transfer_target"] = serde_json::json!(target_attendant);
        ctx["transferred_at"] = serde_json::json!(Utc::now().to_rfc3339());
        ctx["status"] = serde_json::json!("pending_transfer");


        ctx["assigned_to_phone"] = serde_json::Value::Null;
        ctx["assigned_to"] = serde_json::Value::Null;


        ctx["needs_human"] = serde_json::json!(true);

        diesel::update(user_sessions::table.filter(user_sessions::id.eq(session_id)))
            .set((
                user_sessions::context_data.eq(&ctx),
                user_sessions::updated_at.eq(Utc::now()),
            ))
            .execute(&mut db_conn)
            .map_err(|e| format!("Failed to update session: {}", e))?;

        Ok::<String, String>(previous_attendant)
    })
    .await
    .map_err(|e| e.to_string())??;

    info!(
        "Session {} transferred from {} to {}",
        session_id, transfer_result, target_clean
    );

    Ok(format!(
        " *Transfer initiated*\n\nSession {} is being transferred to *{}*.\n\nThe conversation is now in the queue for the target attendant. They will be notified when they check their queue.",
        &session_id.to_string()[..8],
        target_clean
    ))
}

async fn handle_resolve_command(
    state: &Arc<AppState>,
    current_session: Option<Uuid>,
) -> Result<String, String> {
    let session_id = current_session.ok_or("No active conversation to resolve")?;

    let conn = state.conn.clone();
    tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| e.to_string())?;

        use crate::shared::models::schema::user_sessions;

        let session: UserSession = user_sessions::table
            .find(session_id)
            .first(&mut db_conn)
            .map_err(|e| e.to_string())?;

        let mut ctx = session.context_data.clone();
        ctx["status"] = serde_json::json!("resolved");
        ctx["needs_human"] = serde_json::json!(false);
        ctx["resolved_at"] = serde_json::json!(Utc::now().to_rfc3339());

        diesel::update(user_sessions::table.filter(user_sessions::id.eq(session_id)))
            .set(user_sessions::context_data.eq(&ctx))
            .execute(&mut db_conn)
            .map_err(|e| e.to_string())?;

        Ok::<(), String>(())
    })
    .await
    .map_err(|e| e.to_string())??;

    Ok(format!(
        " *Conversation resolved*\n\nSession {} has been marked as resolved. The customer will be returned to bot mode.",
        &session_id.to_string()[..8]
    ))
}

async fn handle_tips_command(
    state: &Arc<AppState>,
    current_session: Option<Uuid>,
) -> Result<String, String> {
    let session_id = current_session.ok_or("No active conversation. Use /take first.")?;


    let history = load_conversation_history(state, session_id).await;

    if history.is_empty() {
        return Ok(
            " No messages yet. Tips will appear when the customer sends a message.".to_string(),
        );
    }

    let last_customer_msg = history
        .iter()
        .rev()
        .find(|m| m.role == "customer")
        .map(|m| m.content.clone())
        .unwrap_or_default();

    let request = TipRequest {
        session_id,
        customer_message: last_customer_msg,
        history,
    };


    let (_, Json(tip_response)) = generate_tips(State(state.clone()), Json(request)).await;

    if tip_response.tips.is_empty() {
        return Ok(" No specific tips for this conversation yet.".to_string());
    }

    let mut result = " *Tips for this conversation*\n\n".to_string();

    for tip in tip_response.tips {
        let emoji = match tip.tip_type {
            TipType::Intent => "",
            TipType::Action => "",
            TipType::Warning => "",
            TipType::Knowledge => "",
            TipType::History => "",
            TipType::General => "",
        };
        result.push_str(&format!("{} {}\n\n", emoji, tip.content));
    }

    Ok(result)
}

async fn handle_polish_command(
    state: &Arc<AppState>,
    current_session: Option<Uuid>,
    message: &str,
) -> Result<String, String> {
    let session_id = current_session.ok_or("No active conversation")?;

    if message.is_empty() {
        return Err("Usage: `/polish Your message here`".to_string());
    }

    let request = PolishRequest {
        session_id,
        message: message.to_string(),
        tone: "professional".to_string(),
    };

    let (_, Json(polish_response)) = polish_message(State(state.clone()), Json(request)).await;

    if !polish_response.success {
        return Err(polish_response
            .error
            .unwrap_or("Failed to polish message".to_string()));
    }

    let mut result = " *Polished message*\n\n".to_string();
    result.push_str(&format!("_{}_\n\n", polish_response.polished));

    if !polish_response.changes.is_empty() {
        result.push_str("Changes:\n");
        for change in polish_response.changes {
            result.push_str(&format!("• {}\n", change));
        }
    }

    result.push_str("\n_Copy and send, or edit as needed._");

    Ok(result)
}

async fn handle_replies_command(
    state: &Arc<AppState>,
    current_session: Option<Uuid>,
) -> Result<String, String> {
    let session_id = current_session.ok_or("No active conversation")?;

    let history = load_conversation_history(state, session_id).await;

    let request = SmartRepliesRequest {
        session_id,
        history,
    };

    let (_, Json(replies_response)) = generate_smart_replies(State(state.clone()), Json(request)).await;

    if replies_response.replies.is_empty() {
        return Ok(" No reply suggestions available.".to_string());
    }

    let mut result = " *Suggested replies*\n\n".to_string();

    for (i, reply) in replies_response.replies.iter().enumerate() {
        result.push_str(&format!(
            "*{}. {}*\n_{}_\n\n",
            i + 1,
            reply.tone.to_uppercase(),
            reply.text
        ));
    }

    result.push_str("_Copy any reply or use as inspiration._");

    Ok(result)
}

async fn handle_summary_command(
    state: &Arc<AppState>,
    current_session: Option<Uuid>,
) -> Result<String, String> {
    let session_id = current_session.ok_or("No active conversation")?;

    let (_, Json(summary_response)) = generate_summary(State(state.clone()), Path(session_id)).await;

    if !summary_response.success {
        return Err(summary_response
            .error
            .unwrap_or("Failed to generate summary".to_string()));
    }

    let summary = summary_response.summary;

    let mut result = " *Conversation Summary*\n\n".to_string();
    result.push_str(&format!("{}\n\n", summary.brief));

    if !summary.key_points.is_empty() {
        result.push_str("*Key Points:*\n");
        for point in &summary.key_points {
            result.push_str(&format!("• {}\n", point));
        }
        result.push('\n');
    }

    if !summary.customer_needs.is_empty() {
        result.push_str("*Customer Needs:*\n");
        for need in &summary.customer_needs {
            result.push_str(&format!("• {}\n", need));
        }
        result.push('\n');
    }

    if !summary.unresolved_issues.is_empty() {
        result.push_str("*Unresolved:*\n");
        for issue in &summary.unresolved_issues {
            result.push_str(&format!("• {}\n", issue));
        }
        result.push('\n');
    }

    result.push_str(&format!(
        " {} messages | {} minutes | Sentiment: {}",
        summary.message_count, summary.duration_minutes, summary.sentiment_trend
    ));

    if !summary.recommended_action.is_empty() {
        result.push_str(&format!(
            "\n\n *Recommended:* {}",
            summary.recommended_action
        ));
    }

    Ok(result)
}

fn get_help_text() -> String {
    r#" *Attendant Commands*

*Queue Management:*
`/queue` - View waiting conversations
`/take` - Take next conversation
`/transfer @name` - Transfer conversation
`/resolve` - Mark as resolved
`/status [online|busy|away|offline]`

*AI Assistance:*
`/tips` - Get tips for current conversation
`/polish <message>` - Improve your message
`/replies` - Get smart reply suggestions
`/summary` - Get conversation summary

*Other:*
`/help` - Show this help

_Portuguese: /fila, /pegar, /transferir, /resolver, /dicas, /polir, /respostas, /resumo, /ajuda_"#
        .to_string()
}




async fn get_session(state: &Arc<AppState>, session_id: Uuid) -> Result<UserSession, String> {
    let conn = state.conn.clone();

    tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB error: {}", e))?;

        use crate::shared::models::schema::user_sessions;

        user_sessions::table
            .find(session_id)
            .first::<UserSession>(&mut db_conn)
            .map_err(|e| format!("Session not found: {}", e))
    })
    .await
    .map_err(|e| format!("Task error: {}", e))?
}


async fn load_conversation_history(
    state: &Arc<AppState>,
    session_id: Uuid,
) -> Vec<ConversationMessage> {
    let conn = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };

        use crate::shared::models::schema::message_history;

        let messages: Vec<(String, i32, chrono::NaiveDateTime)> = message_history::table
            .filter(message_history::session_id.eq(session_id))
            .select((
                message_history::content_encrypted,
                message_history::role,
                message_history::created_at,
            ))
            .order(message_history::created_at.asc())
            .limit(50)
            .load(&mut db_conn)
            .unwrap_or_default();

        messages
            .into_iter()
            .map(|(content, role, timestamp)| ConversationMessage {
                role: match role {
                    0 => "customer".to_string(),
                    1 => "bot".to_string(),
                    2 => "attendant".to_string(),
                    _ => "system".to_string(),
                },
                content,
                timestamp: Some(timestamp.and_utc().to_rfc3339()),
            })
            .collect()
    })
    .await
    .unwrap_or_default();

    result
}


fn parse_tips_response(response: &str) -> Vec<AttendantTip> {

    let json_str = extract_json(response);

    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json_str) {
        if let Some(tips_array) = parsed.get("tips").and_then(|t| t.as_array()) {
            return tips_array
                .iter()
                .filter_map(|tip| {
                    let tip_type = match tip
                        .get("type")
                        .and_then(|t| t.as_str())
                        .unwrap_or("general")
                    {
                        "intent" => TipType::Intent,
                        "action" => TipType::Action,
                        "warning" => TipType::Warning,
                        "knowledge" => TipType::Knowledge,
                        "history" => TipType::History,
                        _ => TipType::General,
                    };

                    Some(AttendantTip {
                        tip_type,
                        content: tip.get("content").and_then(|c| c.as_str())?.to_string(),
                        confidence: tip
                            .get("confidence")
                            .and_then(|c| c.as_f64())
                            .unwrap_or(0.8) as f32,
                        priority: tip.get("priority").and_then(|p| p.as_i64()).unwrap_or(2) as i32,
                    })
                })
                .collect();
        }
    }


    if !response.trim().is_empty() {
        vec![AttendantTip {
            tip_type: TipType::General,
            content: response.trim().to_string(),
            confidence: 0.7,
            priority: 2,
        }]
    } else {
        Vec::new()
    }
}


fn parse_polish_response(response: &str, original: &str) -> (String, Vec<String>) {
    let json_str = extract_json(response);

    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json_str) {
        let polished = parsed
            .get("polished")
            .and_then(|p| p.as_str())
            .unwrap_or(original)
            .to_string();

        let changes = parsed
            .get("changes")
            .and_then(|c| c.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        return (polished, changes);
    }


    (
        response.trim().to_string(),
        vec!["Message improved".to_string()],
    )
}


fn parse_smart_replies_response(response: &str) -> Vec<SmartReply> {
    let json_str = extract_json(response);

    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json_str) {
        if let Some(replies_array) = parsed.get("replies").and_then(|r| r.as_array()) {
            return replies_array
                .iter()
                .filter_map(|reply| {
                    Some(SmartReply {
                        text: reply.get("text").and_then(|t| t.as_str())?.to_string(),
                        tone: reply
                            .get("tone")
                            .and_then(|t| t.as_str())
                            .unwrap_or("professional")
                            .to_string(),
                        confidence: reply
                            .get("confidence")
                            .and_then(|c| c.as_f64())
                            .unwrap_or(0.8) as f32,
                        category: reply
                            .get("category")
                            .and_then(|c| c.as_str())
                            .unwrap_or("answer")
                            .to_string(),
                    })
                })
                .collect();
        }
    }

    generate_fallback_replies()
}


fn parse_summary_response(response: &str) -> ConversationSummary {
    let json_str = extract_json(response);

    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json_str) {
        return ConversationSummary {
            brief: parsed
                .get("brief")
                .and_then(|b| b.as_str())
                .unwrap_or("Conversation summary")
                .to_string(),
            key_points: parsed
                .get("key_points")
                .and_then(|k| k.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            customer_needs: parsed
                .get("customer_needs")
                .and_then(|c| c.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            unresolved_issues: parsed
                .get("unresolved_issues")
                .and_then(|u| u.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            sentiment_trend: parsed
                .get("sentiment_trend")
                .and_then(|s| s.as_str())
                .unwrap_or("stable")
                .to_string(),
            recommended_action: parsed
                .get("recommended_action")
                .and_then(|r| r.as_str())
                .unwrap_or("")
                .to_string(),
            ..Default::default()
        };
    }

    ConversationSummary {
        brief: response.trim().to_string(),
        ..Default::default()
    }
}


fn parse_sentiment_response(response: &str) -> SentimentAnalysis {
    let json_str = extract_json(response);

    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json_str) {
        let emotions = parsed
            .get("emotions")
            .and_then(|e| e.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|e| {
                        Some(Emotion {
                            name: e.get("name").and_then(|n| n.as_str())?.to_string(),
                            intensity: e.get("intensity").and_then(|i| i.as_f64()).unwrap_or(0.5)
                                as f32,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        return SentimentAnalysis {
            overall: parsed
                .get("overall")
                .and_then(|o| o.as_str())
                .unwrap_or("neutral")
                .to_string(),
            score: parsed.get("score").and_then(|s| s.as_f64()).unwrap_or(0.0) as f32,
            emotions,
            escalation_risk: parsed
                .get("escalation_risk")
                .and_then(|e| e.as_str())
                .unwrap_or("low")
                .to_string(),
            urgency: parsed
                .get("urgency")
                .and_then(|u| u.as_str())
                .unwrap_or("normal")
                .to_string(),
            emoji: parsed
                .get("emoji")
                .and_then(|e| e.as_str())
                .unwrap_or("😐")
                .to_string(),
        };
    }

    SentimentAnalysis::default()
}


fn extract_json(response: &str) -> String {

    if let Some(start) = response.find('{') {
        if let Some(end) = response.rfind('}') {
            if end > start {
                return response[start..=end].to_string();
            }
        }
    }


    if let Some(start) = response.find('[') {
        if let Some(end) = response.rfind(']') {
            if end > start {
                return response[start..=end].to_string();
            }
        }
    }

    response.to_string()
}


fn generate_fallback_tips(message: &str) -> Vec<AttendantTip> {
    let msg_lower = message.to_lowercase();
    let mut tips = Vec::new();


    if msg_lower.contains("urgent")
        || msg_lower.contains("asap")
        || msg_lower.contains("immediately")
        || msg_lower.contains("emergency")
    {
        tips.push(AttendantTip {
            tip_type: TipType::Warning,
            content: "Customer indicates urgency - prioritize quick response".to_string(),
            confidence: 0.9,
            priority: 1,
        });
    }


    if msg_lower.contains("frustrated")
        || msg_lower.contains("angry")
        || msg_lower.contains("ridiculous")
        || msg_lower.contains("unacceptable")
    {
        tips.push(AttendantTip {
            tip_type: TipType::Warning,
            content: "Customer may be frustrated - use empathetic language".to_string(),
            confidence: 0.85,
            priority: 1,
        });
    }


    if message.contains('?') {
        tips.push(AttendantTip {
            tip_type: TipType::Intent,
            content: "Customer is asking a question - provide clear, direct answer".to_string(),
            confidence: 0.8,
            priority: 2,
        });
    }


    if msg_lower.contains("problem")
        || msg_lower.contains("issue")
        || msg_lower.contains("not working")
        || msg_lower.contains("broken")
    {
        tips.push(AttendantTip {
            tip_type: TipType::Action,
            content: "Customer reporting an issue - acknowledge and gather details".to_string(),
            confidence: 0.8,
            priority: 2,
        });
    }


    if msg_lower.contains("thank")
        || msg_lower.contains("great")
        || msg_lower.contains("perfect")
        || msg_lower.contains("awesome")
    {
        tips.push(AttendantTip {
            tip_type: TipType::General,
            content: "Customer is expressing satisfaction - good opportunity to close or upsell"
                .to_string(),
            confidence: 0.85,
            priority: 3,
        });
    }


    if tips.is_empty() {
        tips.push(AttendantTip {
            tip_type: TipType::General,
            content: "Read the message carefully and respond helpfully".to_string(),
            confidence: 0.5,
            priority: 3,
        });
    }

    tips
}


fn generate_fallback_replies() -> Vec<SmartReply> {
    vec![
        SmartReply {
            text: "Thank you for reaching out! I'd be happy to help you with that. Could you provide me with a bit more detail?".to_string(),
            tone: "friendly".to_string(),
            confidence: 0.7,
            category: "greeting".to_string(),
        },
        SmartReply {
            text: "I understand your concern. Let me look into this for you right away.".to_string(),
            tone: "empathetic".to_string(),
            confidence: 0.7,
            category: "acknowledgment".to_string(),
        },
        SmartReply {
            text: "Is there anything else I can help you with today?".to_string(),
            tone: "professional".to_string(),
            confidence: 0.7,
            category: "follow_up".to_string(),
        },
    ]
}


fn analyze_sentiment_keywords(message: &str) -> SentimentAnalysis {
    let msg_lower = message.to_lowercase();

    let positive_words = [
        "thank",
        "great",
        "perfect",
        "awesome",
        "excellent",
        "good",
        "happy",
        "love",
        "appreciate",
        "wonderful",
        "fantastic",
        "amazing",
        "helpful",
    ];
    let negative_words = [
        "angry",
        "frustrated",
        "terrible",
        "awful",
        "horrible",
        "worst",
        "hate",
        "disappointed",
        "unacceptable",
        "ridiculous",
        "stupid",
        "problem",
        "issue",
        "broken",
        "failed",
        "error",
    ];
    let urgent_words = [
        "urgent",
        "asap",
        "immediately",
        "emergency",
        "now",
        "critical",
    ];

    let positive_count = positive_words
        .iter()
        .filter(|w| msg_lower.contains(*w))
        .count();
    let negative_count = negative_words
        .iter()
        .filter(|w| msg_lower.contains(*w))
        .count();
    let urgent_count = urgent_words
        .iter()
        .filter(|w| msg_lower.contains(*w))
        .count();

    let score = if positive_count > negative_count {
        0.3 + (positive_count as f32 * 0.2).min(0.7)
    } else if negative_count > positive_count {
        -0.3 - (negative_count as f32 * 0.2).min(0.7)
    } else {
        0.0
    };

    let overall = if score > 0.2 {
        "positive"
    } else if score < -0.2 {
        "negative"
    } else {
        "neutral"
    };

    let escalation_risk = if negative_count >= 3 {
        "high"
    } else if negative_count >= 1 {
        "medium"
    } else {
        "low"
    };

    let urgency = if urgent_count >= 2 {
        "urgent"
    } else if urgent_count >= 1 {
        "high"
    } else {
        "normal"
    };

    let emoji = match overall {
        "positive" => "😊",
        "negative" => "😟",
        _ => "😐",
    };

    let mut emotions = Vec::new();
    if negative_count > 0 {
        emotions.push(Emotion {
            name: "frustration".to_string(),
            intensity: (negative_count as f32 * 0.3).min(1.0),
        });
    }
    if positive_count > 0 {
        emotions.push(Emotion {
            name: "satisfaction".to_string(),
            intensity: (positive_count as f32 * 0.3).min(1.0),
        });
    }
    if urgent_count > 0 {
        emotions.push(Emotion {
            name: "anxiety".to_string(),
            intensity: (urgent_count as f32 * 0.4).min(1.0),
        });
    }

    SentimentAnalysis {
        overall: overall.to_string(),
        score,
        emotions,
        escalation_risk: escalation_risk.to_string(),
        urgency: urgency.to_string(),
        emoji: emoji.to_string(),
    }
}


