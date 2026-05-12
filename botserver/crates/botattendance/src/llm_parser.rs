use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendantTip {
    pub content: String,
    pub rationale: String,
    pub tone: String,
    pub applicable_context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartReply {
    pub content: String,
    pub rationale: String,
    pub tone: String,
    pub confidence: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationSummary {
    pub summary: String,
    pub key_points: Vec<String>,
    pub action_items: Vec<String>,
    pub sentiment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentAnalysis {
    pub sentiment: String,
    pub confidence: f32,
    pub key_emotions: Vec<String>,
    pub suggested_response_tone: String,
}

pub fn parse_tips_response(response: &str) -> Vec<AttendantTip> {
    let json_str = extract_json(response);
    if let Ok(tips) = serde_json::from_str::<Vec<AttendantTip>>(&json_str) {
        return tips;
    }
    response
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.starts_with("- ") || line.starts_with("* ") {
                Some(AttendantTip {
                    content: line[2..].to_string(),
                    rationale: String::new(),
                    tone: "neutral".to_string(),
                    applicable_context: None,
                })
            } else {
                None
            }
        })
        .collect()
}

pub fn parse_polish_response(response: &str, original: &str) -> (String, Vec<String>) {
    let json_str = extract_json(response);
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&json_str) {
        let polished = value["polished"].as_str().unwrap_or(original).to_string();
        let suggestions: Vec<String> = value["suggestions"]
            .as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();
        return (polished, suggestions);
    }
    (response.to_string(), Vec::new())
}

pub fn parse_smart_replies_response(response: &str) -> Vec<SmartReply> {
    let json_str = extract_json(response);
    if let Ok(replies) = serde_json::from_str::<Vec<SmartReply>>(&json_str) {
        return replies;
    }
    vec![SmartReply {
        content: "I understand. Let me help you with that.".to_string(),
        rationale: "Default acknowledgement".to_string(),
        tone: "professional".to_string(),
        confidence: None,
    }]
}

pub fn parse_summary_response(response: &str) -> ConversationSummary {
    let json_str = extract_json(response);
    if let Ok(summary) = serde_json::from_str::<ConversationSummary>(&json_str) {
        return summary;
    }
    ConversationSummary {
        summary: response.lines().take(3).collect::<Vec<_>>().join(" "),
        key_points: Vec::new(),
        action_items: Vec::new(),
        sentiment: "neutral".to_string(),
    }
}

pub fn parse_sentiment_response(response: &str) -> SentimentAnalysis {
    let json_str = extract_json(response);
    if let Ok(analysis) = serde_json::from_str::<SentimentAnalysis>(&json_str) {
        return analysis;
    }
    let response_lower = response.to_lowercase();
    let (sentiment, confidence) = if response_lower.contains("positive") || response_lower.contains("happy") {
        ("positive".to_string(), 0.7)
    } else if response_lower.contains("negative") || response_lower.contains("angry") {
        ("negative".to_string(), 0.7)
    } else {
        ("neutral".to_string(), 0.5)
    };
    SentimentAnalysis {
        sentiment,
        confidence,
        key_emotions: Vec::new(),
        suggested_response_tone: "professional".to_string(),
    }
}

pub fn extract_json(response: &str) -> String {
    let response = response.trim();
    if let Some(start) = response.find("```") {
        if let Some(json_start) = response[start..].find('{') {
            let json_part = &response[start + json_start..];
            if let Some(end) = json_part.find("```") {
                return json_part[..end].trim().to_string();
            }
        }
    }
    if let Some(start) = response.find('{') {
        if let Some(end) = response.rfind('}') {
            return response[start..=end].to_string();
        }
    }
    response.to_string()
}
