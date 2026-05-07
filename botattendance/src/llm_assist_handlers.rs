use crate::llm_assist_config::{get_bot_system_prompt, LlmAssistConfig};
use crate::llm_assist_helpers::*;
use crate::llm_assist_types::*;
use crate::llm_parser;
use crate::AttendanceConfig;
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
    State(config): State<Arc<AttendanceConfig>>,
    Json(request): Json<TipRequest>,
) -> (StatusCode, Json<TipResponse>) {
    info!("Generating tips for session {}", request.session_id);
    let session_result = get_session(&config, request.session_id).await;
    let session = match session_result {
        Ok(s) => s,
        Err(e) => return (StatusCode::NOT_FOUND, Json(TipResponse { success: false, tips: vec![], error: Some(e) })),
    };
    let work_path = std::env::var("WORK_PATH").unwrap_or_else(|_| "./work".to_string());
    let llm_config = LlmAssistConfig::from_config(session.bot_id, &work_path);
    if !llm_config.tips_enabled {
        return (StatusCode::OK, Json(TipResponse { success: true, tips: vec![], error: Some("Tips feature is disabled".to_string()) }));
    }
    let history_context = request.history.iter().map(|m| format!("{}: {}", m.role, m.content)).collect::<Vec<_>>().join("\n");
    let bot_prompt = get_bot_system_prompt(session.bot_id, &work_path);
    let system_prompt = format!(
        "You are an AI assistant helping a human customer service attendant.\nThe bot they are replacing has this personality: {}\n\nYour job is to provide helpful tips to the attendant based on the customer's message.\n\nAnalyze the customer message and provide 2-4 actionable tips. For each tip, classify it as:\n- intent: What the customer wants\n- action: Suggested action for attendant\n- warning: Sentiment or escalation concern\n- knowledge: Relevant info they should know\n- history: Insight from conversation history\n- general: General helpful advice\n\nRespond in JSON format:\n{{\n  \"tips\": [\n    {{\"type\": \"intent\", \"content\": \"...\", \"confidence\": 0.9, \"priority\": 1}},\n    {{\"type\": \"action\", \"content\": \"...\", \"confidence\": 0.8, \"priority\": 2}}\n  ]\n}}",
        bot_prompt
    );
    let user_prompt = format!("Conversation history:\n{}\n\nLatest customer message: \"{}\"\n\nProvide tips for the attendant.", history_context, request.customer_message);
    match execute_llm_with_context(&config, session.bot_id, &system_prompt, &user_prompt).await {
        Ok(response) => {
            let tips = llm_parser::parse_tips_response(&response);
            (StatusCode::OK, Json(TipResponse { success: true, tips, error: None }))
        }
        Err(e) => {
            error!("LLM error generating tips: {}", e);
            (StatusCode::OK, Json(TipResponse { success: true, tips: generate_fallback_tips(&request.customer_message), error: Some(format!("LLM unavailable, using fallback: {}", e)) }))
        }
    }
}

pub async fn polish_message(
    State(config): State<Arc<AttendanceConfig>>,
    Json(request): Json<PolishRequest>,
) -> (StatusCode, Json<PolishResponse>) {
    info!("Polishing message for session {}", request.session_id);
    let session_result = get_session(&config, request.session_id).await;
    let session = match session_result {
        Ok(s) => s,
        Err(e) => return (StatusCode::NOT_FOUND, Json(PolishResponse { success: false, original: request.message.clone(), polished: request.message.clone(), changes: vec![], error: Some(e) })),
    };
    let work_path = std::env::var("WORK_PATH").unwrap_or_else(|_| "./work".to_string());
    let llm_config = LlmAssistConfig::from_config(session.bot_id, &work_path);
    if !llm_config.polish_enabled {
        return (StatusCode::OK, Json(PolishResponse { success: true, original: request.message.clone(), polished: request.message.clone(), changes: vec![], error: Some("Polish feature is disabled".to_string()) }));
    }
    let bot_prompt = get_bot_system_prompt(session.bot_id, &work_path);
    let system_prompt = format!(
        "You are a professional editor helping a customer service attendant.\nThe service has this tone: {}\n\nYour job is to polish the attendant's message to be more {} while:\n1. Fixing grammar and spelling errors\n2. Improving clarity and flow\n3. Maintaining the original meaning\n4. Keeping it natural (not robotic)\n\nRespond in JSON format:\n{{\n  \"polished\": \"The improved message\",\n  \"changes\": [\"Changed X to Y\", \"Fixed grammar in...\"]\n}}",
        bot_prompt, request.tone
    );
    let user_prompt = format!("Polish this message with a {} tone:\n\n\"{}", request.tone, request.message);
    match execute_llm_with_context(&config, session.bot_id, &system_prompt, &user_prompt).await {
        Ok(response) => {
            let (polished, changes) = llm_parser::parse_polish_response(&response, &request.message);
            (StatusCode::OK, Json(PolishResponse { success: true, original: request.message.clone(), polished, changes, error: None }))
        }
        Err(e) => {
            error!("LLM error polishing message: {}", e);
            (StatusCode::OK, Json(PolishResponse { success: false, original: request.message.clone(), polished: request.message.clone(), changes: vec![], error: Some(format!("LLM error: {}", e)) }))
        }
    }
}

pub async fn generate_smart_replies(
    State(config): State<Arc<AttendanceConfig>>,
    Json(request): Json<SmartRepliesRequest>,
) -> (StatusCode, Json<SmartRepliesResponse>) {
    info!("Generating smart replies for session {}", request.session_id);
    let session_result = get_session(&config, request.session_id).await;
    let session = match session_result {
        Ok(s) => s,
        Err(e) => return (StatusCode::NOT_FOUND, Json(SmartRepliesResponse { success: false, replies: vec![], error: Some(e) })),
    };
    let work_path = std::env::var("WORK_PATH").unwrap_or_else(|_| "./work".to_string());
    let llm_config = LlmAssistConfig::from_config(session.bot_id, &work_path);
    if !llm_config.smart_replies_enabled {
        return (StatusCode::OK, Json(SmartRepliesResponse { success: true, replies: vec![], error: Some("Smart replies feature is disabled".to_string()) }));
    }
    let history_context = request.history.iter().map(|m| format!("{}: {}", m.role, m.content)).collect::<Vec<_>>().join("\n");
    let bot_prompt = get_bot_system_prompt(session.bot_id, &work_path);
    let system_prompt = format!(
        "You are an AI assistant helping a customer service attendant craft responses.\nThe service has this personality: {}\n\nGenerate exactly 3 reply suggestions that:\n1. Are contextually appropriate\n2. Sound natural and human (not robotic)\n3. Vary in approach (one empathetic, one solution-focused, one follow_up)\n4. Are ready to send (no placeholders like [name])\n\nRespond in JSON format:\n{{\n  \"replies\": [\n    {{\"text\": \"...\", \"tone\": \"empathetic\", \"confidence\": 0.9, \"category\": \"answer\"}},\n    {{\"text\": \"...\", \"tone\": \"professional\", \"confidence\": 0.85, \"category\": \"solution\"}},\n    {{\"text\": \"...\", \"tone\": \"friendly\", \"confidence\": 0.8, \"category\": \"follow_up\"}}\n  ]\n}}",
        bot_prompt
    );
    let user_prompt = format!("Conversation:\n{}\n\nGenerate 3 reply options for the attendant.", history_context);
    match execute_llm_with_context(&config, session.bot_id, &system_prompt, &user_prompt).await {
        Ok(response) => {
            let replies = llm_parser::parse_smart_replies_response(&response);
            (StatusCode::OK, Json(SmartRepliesResponse { success: true, replies, error: None }))
        }
        Err(e) => {
            error!("LLM error generating smart replies: {}", e);
            (StatusCode::OK, Json(SmartRepliesResponse { success: true, replies: generate_fallback_replies(), error: Some(format!("LLM unavailable, using fallback: {}", e)) }))
        }
    }
}

pub async fn generate_summary(
    State(config): State<Arc<AttendanceConfig>>,
    Path(session_id): Path<Uuid>,
) -> (StatusCode, Json<SummaryResponse>) {
    info!("Generating summary for session {}", session_id);
    let session_result = get_session(&config, session_id).await;
    let session = match session_result {
        Ok(s) => s,
        Err(e) => return (StatusCode::NOT_FOUND, Json(SummaryResponse { success: false, summary: ConversationSummary::default(), error: Some(e) })),
    };
    let work_path = std::env::var("WORK_PATH").unwrap_or_else(|_| "./work".to_string());
    let llm_config = LlmAssistConfig::from_config(session.bot_id, &work_path);
    if !llm_config.auto_summary_enabled {
        return (StatusCode::OK, Json(SummaryResponse { success: true, summary: ConversationSummary::default(), error: Some("Auto-summary feature is disabled".to_string()) }));
    }
    let history = load_conversation_history(&config, session_id).await;
    if history.is_empty() {
        return (StatusCode::OK, Json(SummaryResponse { success: true, summary: ConversationSummary { brief: "No messages in conversation yet".to_string(), ..Default::default() }, error: None }));
    }
    let history_text = history.iter().map(|m| format!("{}: {}", m.role, m.content)).collect::<Vec<_>>().join("\n");
    let bot_prompt = get_bot_system_prompt(session.bot_id, &work_path);
    let system_prompt = format!(
        "You are an AI assistant helping a customer service attendant understand a conversation.\nThe bot/service personality is: {}\n\nAnalyze the conversation and provide a comprehensive summary.\n\nRespond in JSON format:\n{{\n  \"brief\": \"One sentence summary\",\n  \"key_points\": [\"Point 1\", \"Point 2\"],\n  \"customer_needs\": [\"Need 1\", \"Need 2\"],\n  \"unresolved_issues\": [\"Issue 1\"],\n  \"sentiment_trend\": \"improving/stable/declining\",\n  \"recommended_action\": \"What the attendant should do next\"\n}}",
        bot_prompt
    );
    let user_prompt = format!("Summarize this conversation:\n\n{}", history_text);
    match execute_llm_with_context(&config, session.bot_id, &system_prompt, &user_prompt).await {
        Ok(response) => {
            let mut summary = llm_parser::parse_summary_response(&response);
            summary.message_count = history.len() as i32;
            if let (Some(first_ts), Some(last_ts)) = (history.first().and_then(|m| m.timestamp.as_ref()), history.last().and_then(|m| m.timestamp.as_ref())) {
                if let (Ok(first), Ok(last)) = (chrono::DateTime::parse_from_rfc3339(first_ts), chrono::DateTime::parse_from_rfc3339(last_ts)) {
                    summary.duration_minutes = (last - first).num_minutes() as i32;
                }
            }
            (StatusCode::OK, Json(SummaryResponse { success: true, summary, error: None }))
        }
        Err(e) => {
            error!("LLM error generating summary: {}", e);
            (StatusCode::OK, Json(SummaryResponse { success: false, summary: ConversationSummary { brief: format!("Conversation with {} messages", history.len()), message_count: history.len() as i32, ..Default::default() }, error: Some(format!("LLM error: {}", e)) }))
        }
    }
}

pub async fn analyze_sentiment(
    State(config): State<Arc<AttendanceConfig>>,
    Json(request): Json<SentimentRequest>,
) -> impl IntoResponse {
    info!("Analyzing sentiment for session {}", request.session_id);
    let session_result = get_session(&config, request.session_id).await;
    let session = match session_result {
        Ok(s) => s,
        Err(e) => return (StatusCode::NOT_FOUND, Json(SentimentResponse { success: false, sentiment: SentimentAnalysis::default(), error: Some(e) })),
    };
    let work_path = std::env::var("WORK_PATH").unwrap_or_else(|_| "./work".to_string());
    let llm_config = LlmAssistConfig::from_config(session.bot_id, &work_path);
    if !llm_config.sentiment_enabled {
        let sentiment = analyze_sentiment_keywords(&request.message);
        return (StatusCode::OK, Json(SentimentResponse { success: true, sentiment, error: Some("LLM sentiment disabled, using keyword analysis".to_string()) }));
    }
    let history_context = request.history.iter().take(5).map(|m| format!("{}: {}", m.role, m.content)).collect::<Vec<_>>().join("\n");
    let system_prompt = "You are a sentiment analysis expert. Analyze the customer's emotional state.\n\nConsider:\n1. Overall sentiment (positive/neutral/negative)\n2. Specific emotions present\n3. Risk of escalation\n4. Urgency level\n\nRespond in JSON format:\n{\n  \"overall\": \"positive|neutral|negative\",\n  \"score\": 0.5,\n  \"emotions\": [{\"name\": \"frustration\", \"intensity\": 0.7}],\n  \"escalation_risk\": \"low|medium|high\",\n  \"urgency\": \"low|normal|high|urgent\",\n  \"emoji\": \"neutral\"\n}";
    let user_prompt = format!("Recent conversation:\n{}\n\nCurrent message to analyze: \"{}\"\n\nAnalyze the customer's sentiment.", history_context, request.message);
    match execute_llm_with_context(&config, session.bot_id, system_prompt, &user_prompt).await {
        Ok(response) => {
            let sentiment = llm_parser::parse_sentiment_response(&response);
            (StatusCode::OK, Json(SentimentResponse { success: true, sentiment, error: None }))
        }
        Err(e) => {
            error!("LLM error analyzing sentiment: {}", e);
            let sentiment = analyze_sentiment_keywords(&request.message);
            (StatusCode::OK, Json(SentimentResponse { success: true, sentiment, error: Some(format!("LLM unavailable, using fallback: {}", e)) }))
        }
    }
}

pub async fn get_llm_config(
    State(_config): State<Arc<AttendanceConfig>>,
    Path(bot_id): Path<Uuid>,
) -> impl IntoResponse {
    let work_path = std::env::var("WORK_PATH").unwrap_or_else(|_| "./work".to_string());
    let llm_config = LlmAssistConfig::from_config(bot_id, &work_path);
    (StatusCode::OK, Json(serde_json::json!({
        "tips_enabled": llm_config.tips_enabled,
        "polish_enabled": llm_config.polish_enabled,
        "smart_replies_enabled": llm_config.smart_replies_enabled,
        "auto_summary_enabled": llm_config.auto_summary_enabled,
        "sentiment_enabled": llm_config.sentiment_enabled,
        "any_enabled": llm_config.any_enabled()
    })))
}
