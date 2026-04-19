use super::llm_assist_types::*;
use super::llm_assist_config::get_bot_system_prompt;
use super::llm_assist_helpers::*;
use crate::core::config::ConfigManager;
use crate::core::shared::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use log::{error, info};
use std::sync::Arc;
use uuid::Uuid;

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
    let config = crate::attendance::llm_assist_config::LlmAssistConfig::from_config(session.bot_id, &work_path);

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
    let config = crate::attendance::llm_assist_config::LlmAssistConfig::from_config(session.bot_id, &work_path);

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

"{}"#,
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
    let config = crate::attendance::llm_assist_config::LlmAssistConfig::from_config(session.bot_id, &work_path);

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
3. Vary in approach (one empathetic, one solution-focused, one follow_up)
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
        r"Conversation:
{}

Generate 3 reply options for the attendant.",
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
    let config = crate::attendance::llm_assist_config::LlmAssistConfig::from_config(session.bot_id, &work_path);

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
        r"Summarize this conversation:

{}",
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
    let config = crate::attendance::llm_assist_config::LlmAssistConfig::from_config(session.bot_id, &work_path);

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
    let config = crate::attendance::llm_assist_config::LlmAssistConfig::from_config(bot_id, &work_path);

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
