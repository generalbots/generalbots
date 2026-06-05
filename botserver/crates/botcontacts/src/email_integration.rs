use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailTracking {
    pub email_id: Uuid,
    pub deal_id: Uuid,
    pub sent_at: DateTime<Utc>,
    pub opened_at: Option<DateTime<Utc>>,
    pub clicked_at: Option<DateTime<Utc>>,
    pub reply_received: bool,
    pub recipient: String,
    pub subject: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedEmailRequest {
    pub deal_id: Uuid,
    pub recipient: String,
    pub subject: String,
    pub template_body: String,
    pub template_vars: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailTrackingEvent {
    pub email_id: Uuid,
    pub event_type: String,
    pub occurred_at: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}

pub struct EmailIntegrationService;

impl EmailIntegrationService {
    pub fn send_tracked_email(
        request: TrackedEmailRequest,
    ) -> Result<EmailTracking, String> {
        let rendered_body = Self::render_template(&request.template_body, &request.template_vars)?;
        let email_id = Uuid::new_v4();

        if request.recipient.is_empty() || !request.recipient.contains('@') {
            return Err("Invalid recipient email address".to_string());
        }

        let tracking = EmailTracking {
            email_id,
            deal_id: request.deal_id,
            sent_at: Utc::now(),
            opened_at: None,
            clicked_at: None,
            reply_received: false,
            recipient: request.recipient,
            subject: request.subject,
        };

        log::info!(
            "Tracked email {email_id} sent to {} for deal {}: {}",
            tracking.recipient,
            tracking.deal_id,
            rendered_body.chars().take(100).collect::<String>()
        );

        Ok(tracking)
    }

    pub fn open_tracking_pixel(email_id: Uuid) -> EmailTrackingEvent {
        EmailTrackingEvent {
            email_id,
            event_type: "open".to_string(),
            occurred_at: Utc::now(),
            metadata: None,
        }
    }

    pub fn click_tracking(
        email_id: Uuid,
        link_url: &str,
    ) -> EmailTrackingEvent {
        EmailTrackingEvent {
            email_id,
            event_type: "click".to_string(),
            occurred_at: Utc::now(),
            metadata: Some(serde_json::json!({
                "url": link_url,
                "user_agent": None::<String>,
            })),
        }
    }

    pub fn process_tracking_event(
        email: &mut EmailTracking,
        event: EmailTrackingEvent,
    ) {
        match event.event_type.as_str() {
            "open" => {
                if email.opened_at.is_none() {
                    email.opened_at = Some(event.occurred_at);
                }
            }
            "click" => {
                if email.clicked_at.is_none() {
                    email.clicked_at = Some(event.occurred_at);
                }
            }
            "reply" => {
                email.reply_received = true;
            }
            _ => {
                log::warn!("Unknown email tracking event type: {}", event.event_type);
            }
        }
    }

    pub fn render_template(
        template: &str,
        vars: &HashMap<String, String>,
    ) -> Result<String, String> {
        let mut result = template.to_string();
        for (key, value) in vars {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }
        if result.contains("{{") && result.contains("}}") {
            log::warn!("Template may contain unresolved variables");
        }
        Ok(result)
    }

    pub fn build_deal_vars(
        deal_title: &str,
        deal_value: f64,
        contact_name: &str,
        company: &str,
        stage: &str,
    ) -> HashMap<String, String> {
        let mut vars = HashMap::new();
        vars.insert("deal_title".to_string(), deal_title.to_string());
        vars.insert("deal_value".to_string(), format!("{:.2}", deal_value));
        vars.insert("contact_name".to_string(), contact_name.to_string());
        vars.insert("company".to_string(), company.to_string());
        vars.insert("stage".to_string(), stage.to_string());
        vars.insert("current_date".to_string(), Utc::now().format("%Y-%m-%d").to_string());
        vars
    }
}

impl Default for EmailIntegrationService {
    fn default() -> Self {
        Self
    }
}
