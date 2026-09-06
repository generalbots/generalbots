use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppChannel {
    pub phone_number_id: String,
    pub business_account_id: String,
    pub api_key: String,
    pub webhook_secret: String,
    pub template_namespace: String,
}

impl WhatsAppChannel {
    pub fn new(phone_number_id: &str, business_account_id: &str, api_key: &str) -> Self {
        Self {
            phone_number_id: phone_number_id.to_string(),
            business_account_id: business_account_id.to_string(),
            api_key: api_key.to_string(),
            webhook_secret: String::new(),
            template_namespace: String::new(),
        }
    }

    pub fn with_webhook(mut self, secret: &str) -> Self {
        self.webhook_secret = secret.to_string();
        self
    }

    pub fn with_template_namespace(mut self, ns: &str) -> Self {
        self.template_namespace = ns.to_string();
        self
    }

    fn api_url(&self, endpoint: &str) -> String {
        format!(
            "https://graph.facebook.com/v18.0/{}/{}",
            self.phone_number_id, endpoint
        )
    }

    pub fn send_text(&self, to: &str, text: &str) -> Result<(), String> {
        let url = self.api_url("messages");
        let body = serde_json::json!({
            "messaging_product": "whatsapp",
            "to": to,
            "type": "text",
            "text": { "body": text }
        });
        self.post_request(&url, body)
    }

    pub fn send_template(&self, to: &str, template: &str, params: &[&str]) -> Result<(), String> {
        let components: Vec<serde_json::Value> = params.iter().enumerate().map(|(i, p)| {
            serde_json::json!({
                "type": "body",
                "parameters": [{
                    "type": "text",
                    "text": p,
                    "index": i
                }]
            })
        }).collect();

        let url = self.api_url("messages");
        let body = serde_json::json!({
            "messaging_product": "whatsapp",
            "to": to,
            "type": "template",
            "template": {
                "name": template,
                "language": { "code": "pt_BR" },
                "components": components
            }
        });
        self.post_request(&url, body)
    }

    pub fn send_media(&self, to: &str, media_url: &str, media_type: &str) -> Result<(), String> {
        let media_key = match media_type {
            "image" => "image",
            "audio" => "audio",
            "video" => "video",
            "document" => "document",
            _ => return Err(format!("Unsupported media type: {}", media_type)),
        };

        let url = self.api_url("messages");
        let body = serde_json::json!({
            "messaging_product": "whatsapp",
            "to": to,
            "type": media_key,
            media_key: { "link": media_url }
        });
        self.post_request(&url, body)
    }

    pub fn mark_as_read(&self, message_id: &str) -> Result<(), String> {
        let url = self.api_url("messages");
        let body = serde_json::json!({
            "messaging_product": "whatsapp",
            "status": "read",
            "message_id": message_id
        });
        self.post_request(&url, body)
    }

    pub fn verify_webhook(&self, mode: &str, token: &str, challenge: &str) -> Result<String, String> {
        if mode == "subscribe" && token == self.webhook_secret {
            Ok(challenge.to_string())
        } else {
            Err("Webhook verification failed".to_string())
        }
    }

    fn post_request(&self, url: &str, body: serde_json::Value) -> Result<(), String> {
        // Bearer credentials must only ever reach the WhatsApp Graph host.
        let parsed = url::Url::parse(url).map_err(|e| format!("invalid Graph URL: {e}"))?;
        if parsed.host_str() != Some("graph.facebook.com") || parsed.scheme() != "https" {
            return Err("WhatsApp Graph requests must target graph.facebook.com".to_string());
        }
        let client = reqwest::blocking::Client::new();
        let resp = client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if resp.status().is_success() {
            Ok(())
        } else {
            let status = resp.status();
            let text = resp.text().unwrap_or_default();
            Err(format!("WhatsApp API error ({}): {}", status, text))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_new() {
        let ch = WhatsAppChannel::new("123", "456", "tok_abc");
        assert_eq!(ch.phone_number_id, "123");
        assert_eq!(ch.business_account_id, "456");
    }

    #[test]
    fn test_verify_webhook_success() {
        let ch = WhatsAppChannel::new("1", "2", "3").with_webhook("secret123");
        let result = ch.verify_webhook("subscribe", "secret123", "challenge_val");
        assert_eq!(result, Ok("challenge_val".to_string()));
    }

    #[test]
    fn test_verify_webhook_failure() {
        let ch = WhatsAppChannel::new("1", "2", "3").with_webhook("secret123");
        let result = ch.verify_webhook("subscribe", "wrong", "challenge_val");
        assert!(result.is_err());
    }

    #[test]
    fn test_unsupported_media() {
        let ch = WhatsAppChannel::new("1", "2", "3");
        let result = ch.send_media("551199999", "http://ex.com/file", "unsupported");
        assert!(result.is_err());
    }
}
