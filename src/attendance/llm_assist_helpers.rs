use super::llm_assist_types::*;
use crate::core::config::ConfigManager;
use crate::core::shared::state::AppState;
use crate::core::shared::models::UserSession;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

// ============================================================================
// LLM EXECUTION
// ============================================================================

pub async fn execute_llm_with_context(
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

    let messages = json::json!(<
        [
            {
                "role": "system",
                "content": system_prompt
            },
            {
                "role": "user",
                "content": user_prompt
            }
        ]
    >);

    let response = state
        .llm_provider
        .generate(user_prompt, &messages, &model, &key)
        .await?;

    let handler = crate::llm::llm_models::get_handler(&model);
    let processed = handler.process_content(&response);

    Ok(processed)
}

// ============================================================================
// SESSION HELPERS
// ============================================================================

pub async fn get_session(state: &Arc<AppState>, session_id: Uuid) -> Result<UserSession, String> {
    let conn = state.conn.clone();

    tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB error: {}", e))?;

        use crate::core::shared::models::schema::user_sessions;

        user_sessions::table
            .find(session_id)
            .first::<UserSession>(&mut db_conn)
            .map_err(|e| format!("Session not found: {}", e))
    })
    .await
    .map_err(|e| format!("Task error: {}", e))?
}

pub async fn load_conversation_history(
    state: &Arc<AppState>,
    session_id: Uuid,
) -> Vec<ConversationMessage> {
    let conn = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let Ok(mut db_conn) = conn.get() else {
            return Vec::new();
        };

        use crate::core::shared::models::schema::message_history;

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

// ============================================================================
// RESPONSE PARSERS
// ============================================================================

pub fn parse_tips_response(response: &str) -> Vec<AttendantTip> {
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

    if response.trim().is_empty() {
        Vec::new()
    } else {
        vec![AttendantTip {
            tip_type: TipType::General,
            content: response.trim().to_string(),
            confidence: 0.7,
            priority: 2,
        }]
    }
}

pub fn parse_polish_response(response: &str, original: &str) -> (String, Vec<String>) {
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

pub fn parse_smart_replies_response(response: &str) -> Vec<SmartReply> {
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

pub fn parse_summary_response(response: &str) -> ConversationSummary {
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

pub fn parse_sentiment_response(response: &str) -> SentimentAnalysis {
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

pub fn extract_json(response: &str) -> String {
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

// ============================================================================
// FALLBACK FUNCTIONS
// ============================================================================

pub fn generate_fallback_tips(message: &str) -> Vec<AttendantTip> {
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
            content: "Read message carefully and respond helpfully".to_string(),
            confidence: 0.5,
            priority: 3,
        });
    }

    tips
}

pub fn generate_fallback_replies() -> Vec<SmartReply> {
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

pub fn analyze_sentiment_keywords(message: &str) -> SentimentAnalysis {
    let msg_lower = message.to_lowercase();

    let positive_words = [
        "thank", "great", "perfect", "awesome", "excellent", "good", "happy", "love", "appreciate",
        "wonderful", "fantastic", "amazing", "helpful",
    ];
    let negative_words = [
        "angry", "frustrated", "terrible", "awful", "horrible", "worst", "hate", "disappointed",
        "unacceptable", "ridiculous", "stupid", "problem", "issue", "broken", "failed", "error",
    ];
    let urgent_words = ["urgent", "asap", "immediately", "emergency", "now", "critical"];

    let positive_count = positive_words.iter().filter(|w| msg_lower.contains(*w)).count();
    let negative_count = negative_words.iter().filter(|w| msg_lower.contains(*w)).count();
    let urgent_count = urgent_words.iter().filter(|w| msg_lower.contains(*w)).count();

    let score = match positive_count.cmp(&negative_count) {
        std::cmp::Ordering::Greater => 0.3 + (positive_count as f32 * 0.2).min(0.7),
        std::cmp::Ordering::Less => -0.3 - (negative_count as f32 * 0.2).min(0.7),
        std::cmp::Ordering::Equal => 0.0,
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

// ============================================================================
// TESTS
// ============================================================================

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
    fn test_fallback_tips_question() {
        let tips = generate_fallback_tips("How do I reset my password?");
        assert!(!tips.is_empty());
        assert!(tips.iter().any(|t| matches!(t.tip_type, TipType::Intent)));
    }

    #[test]
    fn test_sentiment_positive() {
        let sentiment = analyze_sentiment_keywords("Thank you so much! This is great!");
        assert_eq!(sentiment.overall, "positive");
        assert!(sentiment.score > 0.0);
        assert_eq!(sentiment.escalation_risk, "low");
    }

    #[test]
    fn test_sentiment_negative() {
        let sentiment =
            analyze_sentiment_keywords("This is terrible! I'm very frustrated with this problem.");
        assert_eq!(sentiment.overall, "negative");
        assert!(sentiment.score < 0.0);
        assert!(sentiment.escalation_risk == "medium" || sentiment.escalation_risk == "high");
    }

    #[test]
    fn test_sentiment_urgent() {
        let sentiment = analyze_sentiment_keywords("I need help ASAP! This is urgent!");
        assert!(sentiment.urgency == "high" || sentiment.urgency == "urgent");
    }

    #[test]
    fn test_extract_json() {
        let response = "Here is the result: {\"key\": \"value\"} and some more text.";
        let json = extract_json(&response);
        assert_eq!(json, "{\"key\": \"value\"}");
    }

    #[test]
    fn test_fallback_replies() {
        let replies = generate_fallback_replies();
        assert_eq!(replies.len(), 3);
        assert!(replies.iter().any(|r| r.category == "greeting"));
        assert!(replies.iter().any(|r| r.category == "follow_up"));
    }
}
