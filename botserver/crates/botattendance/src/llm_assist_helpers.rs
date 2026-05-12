use crate::llm_assist_types::*;
use crate::models::UserSession;
use crate::schema::user_sessions;
use crate::schema::message_history;
use crate::AttendanceConfig;
use diesel::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

pub async fn execute_llm_with_context(
    config: &Arc<AttendanceConfig>,
    bot_id: Uuid,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let model = (config.config_get)(&bot_id, "llm-model");
    let key = (config.config_get)(&bot_id, "llm-key");

    let messages = serde_json::json!([
        { "role": "system", "content": system_prompt },
        { "role": "user", "content": user_prompt }
    ]);

    let response = (config.llm_generate)(user_prompt, &messages, &model, &key)?;
    let processed = (config.process_content)(&model, &response);
    Ok(processed)
}

pub async fn get_session(config: &Arc<AttendanceConfig>, session_id: Uuid) -> Result<UserSession, String> {
    let pool = config.pool.clone();
    tokio::task::spawn_blocking(move || {
        let mut db_conn = pool.get().map_err(|e| format!("DB error: {}", e))?;
        user_sessions::table
            .find(session_id)
            .first::<UserSession>(&mut db_conn)
            .map_err(|e| format!("Session not found: {}", e))
    })
    .await
    .map_err(|e| format!("Task error: {}", e))?
}

pub async fn load_conversation_history(
    config: &Arc<AttendanceConfig>,
    session_id: Uuid,
) -> Vec<ConversationMessage> {
    let pool = config.pool.clone();
    let result = tokio::task::spawn_blocking(move || {
        let Ok(mut db_conn) = pool.get() else {
            return Vec::new();
        };
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

pub fn extract_json(response: &str) -> String {
    if let Some(start) = response.find('{') {
        if let Some(end) = response.rfind('}') {
            if end > start { return response[start..=end].to_string(); }
        }
    }
    if let Some(start) = response.find('[') {
        if let Some(end) = response.rfind(']') {
            if end > start { return response[start..=end].to_string(); }
        }
    }
    response.to_string()
}

pub fn generate_fallback_tips(message: &str) -> Vec<AttendantTip> {
    let msg_lower = message.to_lowercase();
    let mut tips = Vec::new();
    if msg_lower.contains("urgent") || msg_lower.contains("asap") || msg_lower.contains("immediately") || msg_lower.contains("emergency") {
        tips.push(AttendantTip { tip_type: TipType::Warning, content: "Customer indicates urgency - prioritize quick response".to_string(), confidence: 0.9, priority: 1 });
    }
    if msg_lower.contains("frustrated") || msg_lower.contains("angry") || msg_lower.contains("ridiculous") || msg_lower.contains("unacceptable") {
        tips.push(AttendantTip { tip_type: TipType::Warning, content: "Customer may be frustrated - use empathetic language".to_string(), confidence: 0.85, priority: 1 });
    }
    if message.contains('?') {
        tips.push(AttendantTip { tip_type: TipType::Intent, content: "Customer is asking a question - provide clear, direct answer".to_string(), confidence: 0.8, priority: 2 });
    }
    if msg_lower.contains("problem") || msg_lower.contains("issue") || msg_lower.contains("not working") || msg_lower.contains("broken") {
        tips.push(AttendantTip { tip_type: TipType::Action, content: "Customer reporting an issue - acknowledge and gather details".to_string(), confidence: 0.8, priority: 2 });
    }
    if msg_lower.contains("thank") || msg_lower.contains("great") || msg_lower.contains("perfect") || msg_lower.contains("awesome") {
        tips.push(AttendantTip { tip_type: TipType::General, content: "Customer is expressing satisfaction - good opportunity to close or upsell".to_string(), confidence: 0.85, priority: 3 });
    }
    if tips.is_empty() {
        tips.push(AttendantTip { tip_type: TipType::General, content: "Read message carefully and respond helpfully".to_string(), confidence: 0.5, priority: 3 });
    }
    tips
}

pub fn generate_fallback_replies() -> Vec<SmartReply> {
    vec![
        SmartReply { text: "Thank you for reaching out! I'd be happy to help you with that. Could you provide me with a bit more detail?".to_string(), tone: "friendly".to_string(), confidence: 0.7, category: "greeting".to_string() },
        SmartReply { text: "I understand your concern. Let me look into this for you right away.".to_string(), tone: "empathetic".to_string(), confidence: 0.7, category: "acknowledgment".to_string() },
        SmartReply { text: "Is there anything else I can help you with today?".to_string(), tone: "professional".to_string(), confidence: 0.7, category: "follow_up".to_string() },
    ]
}

pub fn analyze_sentiment_keywords(message: &str) -> SentimentAnalysis {
    let msg_lower = message.to_lowercase();
    let positive_words = ["thank", "great", "perfect", "awesome", "excellent", "good", "happy", "love", "appreciate", "wonderful", "fantastic", "amazing", "helpful"];
    let negative_words = ["angry", "frustrated", "terrible", "awful", "horrible", "worst", "hate", "disappointed", "unacceptable", "ridiculous", "stupid", "problem", "issue", "broken", "failed", "error"];
    let urgent_words = ["urgent", "asap", "immediately", "emergency", "now", "critical"];
    let positive_count = positive_words.iter().filter(|w| msg_lower.contains(*w)).count();
    let negative_count = negative_words.iter().filter(|w| msg_lower.contains(*w)).count();
    let urgent_count = urgent_words.iter().filter(|w| msg_lower.contains(*w)).count();
    let score = match positive_count.cmp(&negative_count) {
        std::cmp::Ordering::Greater => 0.3 + (positive_count as f32 * 0.2).min(0.7),
        std::cmp::Ordering::Less => -0.3 - (negative_count as f32 * 0.2).min(0.7),
        std::cmp::Ordering::Equal => 0.0,
    };
    let overall = if score > 0.2 { "positive" } else if score < -0.2 { "negative" } else { "neutral" };
    let escalation_risk = if negative_count >= 3 { "high" } else if negative_count >= 1 { "medium" } else { "low" };
    let urgency = if urgent_count >= 2 { "urgent" } else if urgent_count >= 1 { "high" } else { "normal" };
    let emoji = match overall { "positive" => "\u{1f60a}", "negative" => "\u{1f61f}", _ => "\u{1f610}" };
    let mut emotions = Vec::new();
    if negative_count > 0 { emotions.push(Emotion { name: "frustration".to_string(), intensity: (negative_count as f32 * 0.3).min(1.0) }); }
    if positive_count > 0 { emotions.push(Emotion { name: "satisfaction".to_string(), intensity: (positive_count as f32 * 0.3).min(1.0) }); }
    if urgent_count > 0 { emotions.push(Emotion { name: "anxiety".to_string(), intensity: (urgent_count as f32 * 0.4).min(1.0) }); }
    SentimentAnalysis { overall: overall.to_string(), score, emotions, escalation_risk: escalation_risk.to_string(), urgency: urgency.to_string(), emoji: emoji.to_string() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fallback_tips_urgent() {
        let tips = generate_fallback_tips("This is URGENT! I need help immediately!");
        assert!(!tips.is_empty());
        assert!(tips.iter().any(|t| matches!(t.tip_type, TipType::Warning)));
    }

    #[test]
    fn test_sentiment_positive() {
        let sentiment = analyze_sentiment_keywords("Thank you so much! This is great!");
        assert_eq!(sentiment.overall, "positive");
        assert!(sentiment.score > 0.0);
    }

    #[test]
    fn test_sentiment_negative() {
        let sentiment = analyze_sentiment_keywords("This is terrible! I'm very frustrated with this problem.");
        assert_eq!(sentiment.overall, "negative");
        assert!(sentiment.score < 0.0);
    }

    #[test]
    fn test_extract_json() {
        let response = "Here is the result: {\"key\": \"value\"} and some more text.";
        let json = extract_json(response);
        assert_eq!(json, "{\"key\": \"value\"}");
    }

    #[test]
    fn test_fallback_replies() {
        let replies = generate_fallback_replies();
        assert_eq!(replies.len(), 3);
    }
}
