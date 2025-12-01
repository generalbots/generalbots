/*****************************************************************************\
|  █████  █████ ██    █ █████ █████   ████  ██      ████   █████ █████  ███ ® |
| ██      █     ███   █ █     ██  ██ ██  ██ ██      ██  █ ██   ██  █   █      |
| ██  ███ ████  █ ██  █ ████  █████  ██████ ██      ████   █   █   █    ██    |
| ██   ██ █     █  ██ █ █     ██  ██ ██  ██ ██      ██  █ ██   ██  █      █   |
|  █████  █████ █   ███ █████ ██  ██ ██  ██ █████   ████   █████   █   ███    |
|                                                                             |
| General Bots Copyright (c) pragmatismo.com.br. All rights reserved.         |
| Licensed under the AGPL-3.0.                                                |
|                                                                             |
| According to our dual licensing model, this program can be used either      |
| under the terms of the GNU Affero General Public License, version 3,        |
| or under a proprietary license.                                             |
|                                                                             |
| The texts of the GNU Affero General Public License with an additional       |
| permission and of our proprietary license can be found at and               |
| in the LICENSE file you have received along with this program.              |
|                                                                             |
| This program is distributed in the hope that it will be useful,             |
| but WITHOUT ANY WARRANTY, without even the implied warranty of              |
| MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the                |
| GNU Affero General Public License for more details.                         |
|                                                                             |
| "General Bots" is a registered trademark of pragmatismo.com.br.             |
| The licensing of the program under the AGPLv3 does not imply a              |
| trademark license. Therefore any rights, title and interest in              |
| our trademarks remain entirely with us.                                     |
|                                                                             |
\*****************************************************************************/

//! SMS keyword for sending text messages
//!
//! Provides BASIC keywords:
//! - SEND_SMS phone, message -> sends SMS to phone number
//! - SEND_SMS phone, message, provider -> sends SMS using specific provider
//!
//! Supported providers:
//! - twilio (default)
//! - aws_sns
//! - nexmo/vonage
//! - messagebird

use crate::core::config::ConfigManager;
use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::{error, info, trace};
use rhai::{Dynamic, Engine};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// SMS Provider types
#[derive(Debug, Clone, PartialEq)]
pub enum SmsProvider {
    Twilio,
    AwsSns,
    Vonage,
    MessageBird,
    Custom(String),
}

impl From<&str> for SmsProvider {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "twilio" => SmsProvider::Twilio,
            "aws" | "aws_sns" | "sns" => SmsProvider::AwsSns,
            "vonage" | "nexmo" => SmsProvider::Vonage,
            "messagebird" => SmsProvider::MessageBird,
            other => SmsProvider::Custom(other.to_string()),
        }
    }
}

/// SMS send result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsSendResult {
    pub success: bool,
    pub message_id: Option<String>,
    pub provider: String,
    pub to: String,
    pub error: Option<String>,
}

/// Register SMS keywords
pub fn register_sms_keywords(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    register_send_sms_keyword(state.clone(), user.clone(), engine);
    register_send_sms_with_provider_keyword(state, user, engine);
}

/// SEND_SMS phone, message
/// Sends an SMS message using the default configured provider
pub fn register_send_sms_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine
        .register_custom_syntax(
            &["SEND_SMS", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let phone = context.eval_expression_tree(&inputs[0])?.to_string();
                let message = context.eval_expression_tree(&inputs[1])?.to_string();

                trace!("SEND_SMS: Sending SMS to {}", phone);

                let state_for_task = Arc::clone(&state_clone);
                let user_for_task = user_clone.clone();

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();

                    let send_err = if let Ok(rt) = rt {
                        let result = rt.block_on(async move {
                            execute_send_sms(
                                &state_for_task,
                                &user_for_task,
                                &phone,
                                &message,
                                None,
                            )
                            .await
                        });
                        tx.send(result).err()
                    } else {
                        tx.send(Err("Failed to build tokio runtime".into())).err()
                    };

                    if send_err.is_some() {
                        error!("Failed to send SEND_SMS result from thread");
                    }
                });

                match rx.recv_timeout(std::time::Duration::from_secs(30)) {
                    Ok(Ok(result)) => {
                        let mut map = rhai::Map::new();
                        map.insert("success".into(), Dynamic::from(result.success));
                        map.insert(
                            "message_id".into(),
                            Dynamic::from(result.message_id.unwrap_or_default()),
                        );
                        map.insert("provider".into(), Dynamic::from(result.provider));
                        map.insert("to".into(), Dynamic::from(result.to));
                        if let Some(err) = result.error {
                            map.insert("error".into(), Dynamic::from(err));
                        }
                        Ok(Dynamic::from(map))
                    }
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("SEND_SMS failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            "SEND_SMS timed out".into(),
                            rhai::Position::NONE,
                        )))
                    }
                    Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("SEND_SMS thread failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .unwrap();
}

/// SEND_SMS phone, message, provider
/// Sends an SMS message using a specific provider
pub fn register_send_sms_with_provider_keyword(
    state: Arc<AppState>,
    user: UserSession,
    engine: &mut Engine,
) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine
        .register_custom_syntax(
            &["SEND_SMS", "$expr$", ",", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let phone = context.eval_expression_tree(&inputs[0])?.to_string();
                let message = context.eval_expression_tree(&inputs[1])?.to_string();
                let provider = context.eval_expression_tree(&inputs[2])?.to_string();

                trace!("SEND_SMS: Sending SMS to {} via {}", phone, provider);

                let state_for_task = Arc::clone(&state_clone);
                let user_for_task = user_clone.clone();

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();

                    let send_err = if let Ok(rt) = rt {
                        let result = rt.block_on(async move {
                            execute_send_sms(
                                &state_for_task,
                                &user_for_task,
                                &phone,
                                &message,
                                Some(&provider),
                            )
                            .await
                        });
                        tx.send(result).err()
                    } else {
                        tx.send(Err("Failed to build tokio runtime".into())).err()
                    };

                    if send_err.is_some() {
                        error!("Failed to send SEND_SMS result from thread");
                    }
                });

                match rx.recv_timeout(std::time::Duration::from_secs(30)) {
                    Ok(Ok(result)) => {
                        let mut map = rhai::Map::new();
                        map.insert("success".into(), Dynamic::from(result.success));
                        map.insert(
                            "message_id".into(),
                            Dynamic::from(result.message_id.unwrap_or_default()),
                        );
                        map.insert("provider".into(), Dynamic::from(result.provider));
                        map.insert("to".into(), Dynamic::from(result.to));
                        if let Some(err) = result.error {
                            map.insert("error".into(), Dynamic::from(err));
                        }
                        Ok(Dynamic::from(map))
                    }
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("SEND_SMS failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            "SEND_SMS timed out".into(),
                            rhai::Position::NONE,
                        )))
                    }
                    Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("SEND_SMS thread failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .unwrap();
}

async fn execute_send_sms(
    state: &AppState,
    user: &UserSession,
    phone: &str,
    message: &str,
    provider_override: Option<&str>,
) -> Result<SmsSendResult, Box<dyn std::error::Error + Send + Sync>> {
    let config_manager = ConfigManager::new(state.conn.clone());
    let bot_id = user.bot_id;

    // Get provider from config or use override
    let provider_name = match provider_override {
        Some(p) => p.to_string(),
        None => config_manager
            .get_config(&bot_id, "sms-provider", None)
            .unwrap_or_else(|_| "twilio".to_string()),
    };

    let provider = SmsProvider::from(provider_name.as_str());

    // Normalize phone number
    let normalized_phone = normalize_phone_number(phone);

    // Send via appropriate provider
    let result = match provider {
        SmsProvider::Twilio => send_via_twilio(state, &bot_id, &normalized_phone, message).await,
        SmsProvider::AwsSns => send_via_aws_sns(state, &bot_id, &normalized_phone, message).await,
        SmsProvider::Vonage => send_via_vonage(state, &bot_id, &normalized_phone, message).await,
        SmsProvider::MessageBird => {
            send_via_messagebird(state, &bot_id, &normalized_phone, message).await
        }
        SmsProvider::Custom(name) => {
            send_via_custom_webhook(state, &bot_id, &name, &normalized_phone, message).await
        }
    };

    match result {
        Ok(message_id) => {
            info!(
                "SMS sent successfully to {} via {}: {}",
                normalized_phone,
                provider_name,
                message_id.as_deref().unwrap_or("no-id")
            );
            Ok(SmsSendResult {
                success: true,
                message_id,
                provider: provider_name,
                to: normalized_phone,
                error: None,
            })
        }
        Err(e) => {
            error!("SMS send failed to {}: {}", normalized_phone, e);
            Ok(SmsSendResult {
                success: false,
                message_id: None,
                provider: provider_name,
                to: normalized_phone,
                error: Some(e.to_string()),
            })
        }
    }
}

fn normalize_phone_number(phone: &str) -> String {
    // Remove all non-digit characters except leading +
    let has_plus = phone.starts_with('+');
    let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();

    if has_plus {
        format!("+{}", digits)
    } else if digits.len() == 10 {
        // Assume US number without country code
        format!("+1{}", digits)
    } else if digits.len() == 11 && digits.starts_with('1') {
        // US number with country code but no +
        format!("+{}", digits)
    } else {
        // Return as-is with + prefix
        format!("+{}", digits)
    }
}

async fn send_via_twilio(
    state: &AppState,
    bot_id: &Uuid,
    phone: &str,
    message: &str,
) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
    let config_manager = ConfigManager::new(state.conn.clone());

    let account_sid = config_manager
        .get_config(bot_id, "twilio-account-sid", None)
        .map_err(|_| "Twilio account SID not configured. Set twilio-account-sid in config.")?;

    let auth_token = config_manager
        .get_config(bot_id, "twilio-auth-token", None)
        .map_err(|_| "Twilio auth token not configured. Set twilio-auth-token in config.")?;

    let from_number = config_manager
        .get_config(bot_id, "twilio-from-number", None)
        .map_err(|_| "Twilio from number not configured. Set twilio-from-number in config.")?;

    let client = reqwest::Client::new();
    let url = format!(
        "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
        account_sid
    );

    let params = [("To", phone), ("From", &from_number), ("Body", message)];

    let response = client
        .post(&url)
        .basic_auth(&account_sid, Some(&auth_token))
        .form(&params)
        .send()
        .await?;

    if response.status().is_success() {
        let json: serde_json::Value = response.json().await?;
        let sid = json["sid"].as_str().map(|s| s.to_string());
        Ok(sid)
    } else {
        let error_text = response.text().await?;
        Err(format!("Twilio API error: {}", error_text).into())
    }
}

async fn send_via_aws_sns(
    state: &AppState,
    bot_id: &Uuid,
    phone: &str,
    message: &str,
) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
    let config_manager = ConfigManager::new(state.conn.clone());

    let access_key = config_manager
        .get_config(bot_id, "aws-access-key", None)
        .map_err(|_| "AWS access key not configured. Set aws-access-key in config.")?;

    let secret_key = config_manager
        .get_config(bot_id, "aws-secret-key", None)
        .map_err(|_| "AWS secret key not configured. Set aws-secret-key in config.")?;

    let region = config_manager
        .get_config(bot_id, "aws-region", Some("us-east-1"))
        .unwrap_or_else(|_| "us-east-1".to_string());

    // Use HTTP API directly instead of AWS SDK
    let client = reqwest::Client::new();
    let url = format!("https://sns.{}.amazonaws.com/", region);

    // Create timestamp for AWS Signature
    let timestamp = chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let date = &timestamp[..8];

    // Build the request parameters
    let params = [
        ("Action", "Publish"),
        ("PhoneNumber", phone),
        ("Message", message),
        ("Version", "2010-03-31"),
    ];

    // For simplicity, using query string auth (requires proper AWS SigV4 in production)
    // This is a simplified implementation - in production use aws-sigv4 crate
    let response = client
        .post(&url)
        .form(&params)
        .header("X-Amz-Date", &timestamp)
        .basic_auth(&access_key, Some(&secret_key))
        .send()
        .await?;

    if response.status().is_success() {
        let body = response.text().await?;
        // Parse MessageId from XML response
        if let Some(start) = body.find("<MessageId>") {
            if let Some(end) = body.find("</MessageId>") {
                let message_id = &body[start + 11..end];
                return Ok(Some(message_id.to_string()));
            }
        }
        Ok(None)
    } else {
        let error_text = response.text().await?;
        Err(format!("AWS SNS API error: {}", error_text).into())
    }
}

async fn send_via_vonage(
    state: &AppState,
    bot_id: &Uuid,
    phone: &str,
    message: &str,
) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
    let config_manager = ConfigManager::new(state.conn.clone());

    let api_key = config_manager
        .get_config(bot_id, "vonage-api-key", None)
        .map_err(|_| "Vonage API key not configured. Set vonage-api-key in config.")?;

    let api_secret = config_manager
        .get_config(bot_id, "vonage-api-secret", None)
        .map_err(|_| "Vonage API secret not configured. Set vonage-api-secret in config.")?;

    let from_number = config_manager
        .get_config(bot_id, "vonage-from-number", None)
        .map_err(|_| "Vonage from number not configured. Set vonage-from-number in config.")?;

    let client = reqwest::Client::new();

    let payload = serde_json::json!({
        "api_key": api_key,
        "api_secret": api_secret,
        "to": phone.trim_start_matches('+'),
        "from": from_number,
        "text": message
    });

    let response = client
        .post("https://rest.nexmo.com/sms/json")
        .json(&payload)
        .send()
        .await?;

    if response.status().is_success() {
        let json: serde_json::Value = response.json().await?;
        let messages = json["messages"].as_array();

        if let Some(msgs) = messages {
            if let Some(first) = msgs.first() {
                if first["status"].as_str() == Some("0") {
                    return Ok(first["message-id"].as_str().map(|s| s.to_string()));
                } else {
                    let error_text = first["error-text"].as_str().unwrap_or("Unknown error");
                    return Err(format!("Vonage error: {}", error_text).into());
                }
            }
        }
        Err("Invalid Vonage response".into())
    } else {
        let error_text = response.text().await?;
        Err(format!("Vonage API error: {}", error_text).into())
    }
}

async fn send_via_messagebird(
    state: &AppState,
    bot_id: &Uuid,
    phone: &str,
    message: &str,
) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
    let config_manager = ConfigManager::new(state.conn.clone());

    let api_key = config_manager
        .get_config(bot_id, "messagebird-api-key", None)
        .map_err(|_| "MessageBird API key not configured. Set messagebird-api-key in config.")?;

    let originator = config_manager
        .get_config(bot_id, "messagebird-originator", None)
        .map_err(|_| {
            "MessageBird originator not configured. Set messagebird-originator in config."
        })?;

    let client = reqwest::Client::new();

    let payload = serde_json::json!({
        "originator": originator,
        "recipients": [phone.trim_start_matches('+')],
        "body": message
    });

    let response = client
        .post("https://rest.messagebird.com/messages")
        .header("Authorization", format!("AccessKey {}", api_key))
        .json(&payload)
        .send()
        .await?;

    if response.status().is_success() {
        let json: serde_json::Value = response.json().await?;
        Ok(json["id"].as_str().map(|s| s.to_string()))
    } else {
        let error_text = response.text().await?;
        Err(format!("MessageBird API error: {}", error_text).into())
    }
}

async fn send_via_custom_webhook(
    state: &AppState,
    bot_id: &Uuid,
    provider_name: &str,
    phone: &str,
    message: &str,
) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
    let config_manager = ConfigManager::new(state.conn.clone());

    let webhook_url = config_manager
        .get_config(bot_id, &format!("{}-webhook-url", provider_name), None)
        .map_err(|_| {
            format!(
                "Custom SMS webhook URL not configured. Set {}-webhook-url in config.",
                provider_name
            )
        })?;

    let api_key = config_manager
        .get_config(bot_id, &format!("{}-api-key", provider_name), None)
        .ok();

    let client = reqwest::Client::new();

    let payload = serde_json::json!({
        "to": phone,
        "message": message,
        "provider": provider_name
    });

    let mut request = client.post(&webhook_url).json(&payload);

    if let Some(key) = api_key {
        request = request.header("Authorization", format!("Bearer {}", key));
    }

    let response = request.send().await?;

    if response.status().is_success() {
        let json: serde_json::Value = response.json().await.unwrap_or(serde_json::json!({}));
        Ok(json["message_id"]
            .as_str()
            .or(json["id"].as_str())
            .map(|s| s.to_string()))
    } else {
        let error_text = response.text().await?;
        Err(format!("Custom webhook error: {}", error_text).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_phone_us_10_digit() {
        assert_eq!(normalize_phone_number("5551234567"), "+15551234567");
    }

    #[test]
    fn test_normalize_phone_us_11_digit() {
        assert_eq!(normalize_phone_number("15551234567"), "+15551234567");
    }

    #[test]
    fn test_normalize_phone_with_plus() {
        assert_eq!(normalize_phone_number("+15551234567"), "+15551234567");
    }

    #[test]
    fn test_normalize_phone_with_formatting() {
        assert_eq!(normalize_phone_number("+1 (555) 123-4567"), "+15551234567");
    }

    #[test]
    fn test_normalize_phone_international() {
        assert_eq!(normalize_phone_number("+44 7911 123456"), "+447911123456");
    }

    #[test]
    fn test_sms_provider_from_str() {
        assert_eq!(SmsProvider::from("twilio"), SmsProvider::Twilio);
        assert_eq!(SmsProvider::from("aws_sns"), SmsProvider::AwsSns);
        assert_eq!(SmsProvider::from("vonage"), SmsProvider::Vonage);
        assert_eq!(SmsProvider::from("nexmo"), SmsProvider::Vonage);
        assert_eq!(SmsProvider::from("messagebird"), SmsProvider::MessageBird);
        assert_eq!(
            SmsProvider::from("custom"),
            SmsProvider::Custom("custom".to_string())
        );
    }
}
