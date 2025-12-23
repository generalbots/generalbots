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













use crate::core::config::ConfigManager;
use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::{error, info, trace};
use rhai::{Dynamic, Engine};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;


#[derive(Debug, Clone, PartialEq)]
pub enum SmsProvider {
    Twilio,
    AwsSns,
    Vonage,
    MessageBird,
    Custom(String),
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SmsPriority {
    Low,
    Normal,
    High,
    Urgent,
}

impl Default for SmsPriority {
    fn default() -> Self {
        SmsPriority::Normal
    }
}

impl From<&str> for SmsPriority {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "low" => SmsPriority::Low,
            "high" => SmsPriority::High,
            "urgent" | "critical" => SmsPriority::Urgent,
            _ => SmsPriority::Normal,
        }
    }
}

impl std::fmt::Display for SmsPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SmsPriority::Low => write!(f, "low"),
            SmsPriority::Normal => write!(f, "normal"),
            SmsPriority::High => write!(f, "high"),
            SmsPriority::Urgent => write!(f, "urgent"),
        }
    }
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


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsSendResult {
    pub success: bool,
    pub message_id: Option<String>,
    pub provider: String,
    pub to: String,
    pub priority: String,
    pub error: Option<String>,
}


pub fn register_sms_keywords(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    register_send_sms_keyword(state.clone(), user.clone(), engine);
    register_send_sms_with_third_arg_keyword(state.clone(), user.clone(), engine);
    register_send_sms_full_keyword(state, user, engine);
}



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
                        map.insert("priority".into(), Dynamic::from(result.priority));
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




pub fn register_send_sms_with_third_arg_keyword(
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
                let third_arg = context.eval_expression_tree(&inputs[2])?.to_string();


                let is_priority = matches!(
                    third_arg.to_lowercase().as_str(),
                    "low" | "normal" | "high" | "urgent" | "critical"
                );

                let (provider_override, priority_override) = if is_priority {
                    (None, Some(third_arg.clone()))
                } else {
                    (Some(third_arg.clone()), None)
                };

                trace!(
                    "SEND_SMS: Sending SMS to {} (third_arg={}, is_priority={})",
                    phone,
                    third_arg,
                    is_priority
                );

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
                                provider_override.as_deref(),
                                priority_override.as_deref(),
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
                        map.insert("priority".into(), Dynamic::from(result.priority));
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



pub fn register_send_sms_full_keyword(
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
                let priority = context.eval_expression_tree(&inputs[3])?.to_string();

                trace!(
                    "SEND_SMS: Sending SMS to {} via {} with priority {}",
                    phone,
                    provider,
                    priority
                );

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
                                Some(&priority),
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
                        map.insert("priority".into(), Dynamic::from(result.priority));
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


    let state_clone2 = Arc::clone(&state);
    let user_clone2 = user.clone();

    engine
        .register_custom_syntax(
            &[
                "SEND_SMS", "$expr$", ",", "$expr$", ",", "$expr$", ",", "$expr$",
            ],
            false,
            move |context, inputs| {
                let phone = context.eval_expression_tree(&inputs[0])?.to_string();
                let message = context.eval_expression_tree(&inputs[1])?.to_string();
                let provider = context.eval_expression_tree(&inputs[2])?.to_string();
                let priority = context.eval_expression_tree(&inputs[3])?.to_string();

                trace!(
                    "SEND_SMS: Sending SMS to {} via {} with priority {}",
                    phone,
                    provider,
                    priority
                );

                let state_for_task = Arc::clone(&state_clone2);
                let user_for_task = user_clone2.clone();

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
                                Some(&priority),
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
                        map.insert("priority".into(), Dynamic::from(result.priority));
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
    priority_override: Option<&str>,
) -> Result<SmsSendResult, Box<dyn std::error::Error + Send + Sync>> {
    let config_manager = ConfigManager::new(state.conn.clone());
    let bot_id = user.bot_id;


    let provider_name = match provider_override {
        Some(p) => p.to_string(),
        None => config_manager
            .get_config(&bot_id, "sms-provider", None)
            .unwrap_or_else(|_| "twilio".to_string()),
    };

    let provider = SmsProvider::from(provider_name.as_str());


    let priority = match priority_override {
        Some(p) => SmsPriority::from(p),
        None => {
            let priority_str = config_manager
                .get_config(&bot_id, "sms-default-priority", None)
                .unwrap_or_else(|_| "normal".to_string());
            SmsPriority::from(priority_str.as_str())
        }
    };


    let normalized_phone = normalize_phone_number(phone);


    if matches!(priority, SmsPriority::High | SmsPriority::Urgent) {
        info!(
            "High priority SMS to {}: priority={}",
            normalized_phone, priority
        );
    }


    let result = match provider {
        SmsProvider::Twilio => {
            send_via_twilio(state, &bot_id, &normalized_phone, message, &priority).await
        }
        SmsProvider::AwsSns => {
            send_via_aws_sns(state, &bot_id, &normalized_phone, message, &priority).await
        }
        SmsProvider::Vonage => {
            send_via_vonage(state, &bot_id, &normalized_phone, message, &priority).await
        }
        SmsProvider::MessageBird => {
            send_via_messagebird(state, &bot_id, &normalized_phone, message, &priority).await
        }
        SmsProvider::Custom(name) => {
            send_via_custom_webhook(state, &bot_id, &name, &normalized_phone, message, &priority)
                .await
        }
    };

    match result {
        Ok(message_id) => {
            info!(
                "SMS sent successfully to {} via {} (priority={}): {}",
                normalized_phone,
                provider_name,
                priority,
                message_id.as_deref().unwrap_or("no-id")
            );
            Ok(SmsSendResult {
                success: true,
                message_id,
                provider: provider_name,
                to: normalized_phone,
                priority: priority.to_string(),
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
                priority: priority.to_string(),
                error: Some(e.to_string()),
            })
        }
    }
}

fn normalize_phone_number(phone: &str) -> String {

    let has_plus = phone.starts_with('+');
    let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();

    if has_plus {
        format!("+{}", digits)
    } else if digits.len() == 10 {

        format!("+1{}", digits)
    } else if digits.len() == 11 && digits.starts_with('1') {

        format!("+{}", digits)
    } else {

        format!("+{}", digits)
    }
}

async fn send_via_twilio(
    state: &AppState,
    bot_id: &Uuid,
    phone: &str,
    message: &str,
    priority: &SmsPriority,
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



    let final_message = match priority {
        SmsPriority::Urgent => format!("[URGENT] {}", message),
        SmsPriority::High => format!("[HIGH] {}", message),
        _ => message.to_string(),
    };

    let params = [
        ("To", phone),
        ("From", from_number.as_str()),
        ("Body", final_message.as_str()),
    ];

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
    priority: &SmsPriority,
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


    let client = reqwest::Client::new();
    let url = format!("https://sns.{}.amazonaws.com/", region);


    let timestamp = chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let _date = &timestamp[..8];



    let sms_type = match priority {
        SmsPriority::High | SmsPriority::Urgent => "Transactional",
        _ => "Promotional",
    };


    let params = [
        ("Action", "Publish"),
        ("PhoneNumber", phone),
        ("Message", message),
        ("Version", "2010-03-31"),
        ("MessageAttributes.entry.1.Name", "AWS.SNS.SMS.SMSType"),
        ("MessageAttributes.entry.1.Value.DataType", "String"),
        ("MessageAttributes.entry.1.Value.StringValue", sms_type),
    ];



    let response = client
        .post(&url)
        .form(&params)
        .header("X-Amz-Date", &timestamp)
        .basic_auth(&access_key, Some(&secret_key))
        .send()
        .await?;

    if response.status().is_success() {
        let body = response.text().await?;

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
    priority: &SmsPriority,
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


    let message_class = match priority {
        SmsPriority::Urgent => Some("0"),
        _ => None,
    };

    let mut body = serde_json::json!({
        "api_key": api_key,
        "api_secret": api_secret,
        "to": phone,
        "from": from_number,
        "text": message
    });

    if let Some(class) = message_class {
        body["message-class"] = serde_json::Value::String(class.to_string());
    }

    let response = client
        .post("https://rest.nexmo.com/sms/json")
        .json(&body)
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
    priority: &SmsPriority,
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


    let type_details = match priority {
        SmsPriority::Urgent => Some(serde_json::json!({"class": 0})),
        SmsPriority::High => Some(serde_json::json!({"class": 1})),
        _ => None,
    };

    let mut body = serde_json::json!({
        "originator": originator,
        "recipients": [phone],
        "body": message
    });

    if let Some(details) = type_details {
        body["typeDetails"] = details;
    }

    let response = client
        .post("https://rest.messagebird.com/messages")
        .header("Authorization", format!("AccessKey {}", api_key))
        .json(&body)
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
    webhook_name: &str,
    phone: &str,
    message: &str,
    priority: &SmsPriority,
) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
    let config_manager = ConfigManager::new(state.conn.clone());

    let webhook_url = config_manager
        .get_config(bot_id, &format!("{}-webhook-url", webhook_name), None)
        .map_err(|_| {
            format!(
                "Custom SMS webhook URL not configured. Set {}-webhook-url in config.",
                webhook_name
            )
        })?;

    let api_key = config_manager
        .get_config(bot_id, &format!("{}-api-key", webhook_name), None)
        .ok();

    let client = reqwest::Client::new();

    let payload = serde_json::json!({
        "to": phone,
        "message": message,
        "provider": webhook_name,
        "priority": priority.to_string()
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
