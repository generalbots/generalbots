use chrono::{DateTime, Utc};
use log::{info, warn};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailHeaders {
    pub message_id: String,
    pub from: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub subject: String,
    pub date: DateTime<Utc>,
    pub in_reply_to: Option<String>,
    pub references: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailTriagingResult {
    pub category: EmailCategory,
    pub confidence: f32,
    pub reasoning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EmailCategory {
    Urgent,
    MeetingRequest,
    Informational,
    Spam,
}

impl std::fmt::Display for EmailCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmailCategory::Urgent => write!(f, "urgent"),
            EmailCategory::MeetingRequest => write!(f, "meeting_request"),
            EmailCategory::Informational => write!(f, "informational"),
            EmailCategory::Spam => write!(f, "spam"),
        }
    }
}

impl std::str::FromStr for EmailCategory {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "urgent" => Ok(EmailCategory::Urgent),
            "meeting_request" | "meeting request" | "meeting" => Ok(EmailCategory::MeetingRequest),
            "informational" | "info" => Ok(EmailCategory::Informational),
            "spam" => Ok(EmailCategory::Spam),
            _ => Err(format!("Unknown email category: {s}")),
        }
    }
}

pub async fn triage_email(
    headers: &EmailHeaders,
    body_text: &str,
) -> EmailTriagingResult {
    let llm_result = call_llm_triage(headers, body_text).await;

    match llm_result {
        Ok(result) => {
            info!(
                "Email triaged: category={}, confidence={:.2}, subject={}",
                result.category, result.confidence, headers.subject
            );
            result
        }
        Err(e) => {
            warn!("LLM triage failed for {}: {e}. Using fallback.", headers.subject);
            fallback_triage(headers, body_text)
        }
    }
}

async fn call_llm_triage(
    headers: &EmailHeaders,
    body_text: &str,
) -> Result<EmailTriagingResult, String> {
    let llm_url = match std::env::var("BOT_EMAIL_LLM_URL") {
        Ok(u) if !u.is_empty() => u,
        _ => return Err("BOT_EMAIL_LLM_URL not set".into()),
    };

    let prompt = format!(
        "You are an email triage assistant. Classify the following email into EXACTLY one category: \
         urgent, meeting_request, informational, spam.\n\n\
         From: {from}\nTo: {to}\nSubject: {subject}\nDate: {date}\n\n\
         Body:\n{body}\n\n\
         Respond with a JSON object: {{\"category\": \"...\", \"confidence\": 0.0..1.0, \"reasoning\": \"...\"}}",
        from = headers.from,
        to = headers.to.join(", "),
        subject = headers.subject,
        date = headers.date.to_rfc3339(),
        body = body_text.chars().take(2000).collect::<String>(),
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {e}"))?;

    let resp = client
        .post(&llm_url)
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": std::env::var("BOT_EMAIL_LLM_MODEL").unwrap_or_else(|_| "default".into()),
            "messages": [{"role": "user", "content": prompt}],
            "max_tokens": 150,
            "temperature": 0.1,
        }))
        .send()
        .await
        .map_err(|e| format!("LLM request failed: {e}"))?;

    let text = resp
        .text()
        .await
        .map_err(|e| format!("Failed to read LLM response: {e}"))?;

    let text = text.trim();

    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(text) {
        let cat_str = parsed
            .get("category")
            .and_then(|c| c.as_str())
            .unwrap_or("informational");
        let confidence = parsed
            .get("confidence")
            .and_then(|c| c.as_f64())
            .unwrap_or(0.5) as f32;
        let reasoning = parsed
            .get("reasoning")
            .and_then(|r| r.as_str())
            .unwrap_or("")
            .to_string();

        let category = cat_str
            .parse::<EmailCategory>()
            .unwrap_or(EmailCategory::Informational);

        return Ok(EmailTriagingResult {
            category,
            confidence: confidence.clamp(0.0, 1.0),
            reasoning,
        });
    }

    let lower = text.to_lowercase();
    let category = if lower.contains("urgent") {
        EmailCategory::Urgent
    } else if lower.contains("meeting") {
        EmailCategory::MeetingRequest
    } else if lower.contains("spam") {
        EmailCategory::Spam
    } else {
        EmailCategory::Informational
    };

    Ok(EmailTriagingResult {
        category,
        confidence: 0.7,
        reasoning: "Parsed from raw LLM text output".into(),
    })
}

fn fallback_triage(headers: &EmailHeaders, body_text: &str) -> EmailTriagingResult {
    let text = format!(
        "{} {} {}",
        headers.subject.to_lowercase(),
        headers.from.to_lowercase(),
        body_text.chars().take(500).collect::<String>().to_lowercase()
    );

    if text.contains("urgent")
        || text.contains("asap")
        || text.contains("deadline")
        || text.contains("immediately")
    {
        return EmailTriagingResult {
            category: EmailCategory::Urgent,
            confidence: 0.75,
            reasoning: "Keyword match: urgent/asap/deadline".into(),
        };
    }

    if text.contains("meeting")
        || text.contains("calendar")
        || text.contains("schedule")
        || text.contains("invite")
        || text.contains("appointment")
        || text.contains("conference")
    {
        return EmailTriagingResult {
            category: EmailCategory::MeetingRequest,
            confidence: 0.8,
            reasoning: "Keyword match: meeting/calendar/schedule/invite".into(),
        };
    }

    if text.contains("unsubscribe")
        || text.contains("newsletter")
        || text.contains("marketing")
        || text.contains("promotion")
    {
        return EmailTriagingResult {
            category: EmailCategory::Spam,
            confidence: 0.85,
            reasoning: "Keyword match: unsubscribe/newsletter/marketing".into(),
        };
    }

    EmailTriagingResult {
        category: EmailCategory::Informational,
        confidence: 0.6,
        reasoning: "No specific keywords found, classified as informational".into(),
    }
}

pub fn should_skip_triage(headers: &EmailHeaders) -> bool {
    if headers.to.is_empty() {
        return true;
    }
    for addr in &headers.to {
        if addr.contains("noreply@")
            || addr.contains("no-reply@")
            || addr.contains("notifications@")
        {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_headers(subject: &str, from: &str) -> EmailHeaders {
        EmailHeaders {
            message_id: "<test@example.com>".into(),
            from: from.into(),
            to: vec!["user@example.com".into()],
            cc: vec![],
            subject: subject.into(),
            date: Utc::now(),
            in_reply_to: None,
            references: vec![],
        }
    }

    #[test]
    fn test_fallback_urgent() {
        let headers = make_headers("URGENT: Server down", "admin@company.com");
        let result = fallback_triage(&headers, "The production server is down, please fix ASAP!");
        assert_eq!(result.category, EmailCategory::Urgent);
        assert!(result.confidence > 0.5);
    }

    #[test]
    fn test_fallback_meeting() {
        let headers = make_headers("Meeting: Project Review", "manager@company.com");
        let result = fallback_triage(&headers, "Let's schedule a meeting for next week.");
        assert_eq!(result.category, EmailCategory::MeetingRequest);
    }

    #[test]
    fn test_fallback_spam() {
        let headers = make_headers("You won!", "spammer@spam.com");
        let result = fallback_triage(&headers, "Click here to unsubscribe from our newsletter.");
        assert_eq!(result.category, EmailCategory::Spam);
    }

    #[test]
    fn test_fallback_informational() {
        let headers = make_headers("Weekly Report", "team@company.com");
        let result = fallback_triage(&headers, "Here is the weekly report for review.");
        assert_eq!(result.category, EmailCategory::Informational);
    }

    #[test]
    fn test_should_skip_noreply() {
        let headers = EmailHeaders {
            message_id: "<test@no-reply.com>".into(),
            from: "no-reply@service.com".into(),
            to: vec!["no-reply@service.com".into()],
            cc: vec![],
            subject: "Notification".into(),
            date: Utc::now(),
            in_reply_to: None,
            references: vec![],
        };
        assert!(should_skip_triage(&headers));
    }

    #[test]
    fn test_category_display() {
        assert_eq!(EmailCategory::Urgent.to_string(), "urgent");
        assert_eq!(EmailCategory::MeetingRequest.to_string(), "meeting_request");
        assert_eq!(EmailCategory::Informational.to_string(), "informational");
        assert_eq!(EmailCategory::Spam.to_string(), "spam");
    }

    #[test]
    fn test_category_parse() {
        assert_eq!("urgent".parse::<EmailCategory>().ok(), Some(EmailCategory::Urgent));
        assert_eq!("Meeting".parse::<EmailCategory>().ok(), Some(EmailCategory::MeetingRequest));
        assert_eq!("SPAM".parse::<EmailCategory>().ok(), Some(EmailCategory::Spam));
        assert!("unknown_category".parse::<EmailCategory>().is_err());
    }
}
