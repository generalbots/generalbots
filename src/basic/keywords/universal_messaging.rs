use crate::core::bot::channels::{
    instagram::InstagramAdapter, teams::TeamsAdapter, whatsapp::WhatsAppAdapter, ChannelAdapter,
};
use crate::shared::message_types::MessageType;
use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::{error, trace};
use rhai::{Dynamic, Engine};
use serde_json::json;
use std::sync::Arc;

pub fn register_universal_messaging(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    register_send_file_to(state.clone(), user.clone(), engine);
    register_send_to(state.clone(), user.clone(), engine);
    register_broadcast(state, user, engine);
}

// DEPRECATED: TALK TO functionality moved to hear_talk.rs talk_keyword function
// to avoid syntax conflicts between TALK and TALK TO
/*
fn register_talk_to(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);

    engine
        .register_custom_syntax(
            ["TALK", "TO", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let recipient = context.eval_expression_tree(&inputs[0])?.to_string();
                let message = context.eval_expression_tree(&inputs[1])?.to_string();

                trace!("TALK TO: Sending message to {}", recipient);

                let state_for_send = Arc::clone(&state_clone);
                let user_for_send = user.clone();

                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        send_message_to_recipient(
                            state_for_send,
                            &user_for_send,
                            &recipient,
                            &message,
                        )
                        .await
                    })
                })
                .map_err(|e| format!("Failed to send message: {}", e))?;

                Ok(Dynamic::UNIT)
            },
        )
        .expect("valid syntax registration");
}
*/

fn register_send_file_to(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_arc = Arc::new(user);

    let user_clone = Arc::clone(&user_arc);
    engine
        .register_custom_syntax(
            ["SEND", "FILE", "TO", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let recipient = context.eval_expression_tree(&inputs[0])?.to_string();
                let file = context.eval_expression_tree(&inputs[1])?;

                trace!("SEND FILE TO: Sending file to {}", recipient);

                let state_for_send = Arc::clone(&state_clone);
                let user_for_send = Arc::clone(&user_clone);

                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        send_file_to_recipient(state_for_send, &user_for_send, &recipient, file)
                            .await
                    })
                })
                .map_err(|e| format!("Failed to send file: {}", e))?;

                Ok(Dynamic::UNIT)
            },
        )
        .expect("valid syntax registration");

    let state_clone2 = Arc::clone(&state);
    let user_clone2 = Arc::clone(&user_arc);

    engine
        .register_custom_syntax(
            ["SEND", "FILE", "TO", "$expr$", ",", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let recipient = context.eval_expression_tree(&inputs[0])?.to_string();
                let file = context.eval_expression_tree(&inputs[1])?;
                let caption = context.eval_expression_tree(&inputs[2])?.to_string();

                trace!("SEND FILE TO: Sending file with caption to {}", recipient);

                let state_for_send = Arc::clone(&state_clone2);
                let user_for_send = Arc::clone(&user_clone2);

                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        send_file_with_caption_to_recipient(
                            state_for_send,
                            &user_for_send,
                            &recipient,
                            file,
                            &caption,
                        )
                        .await
                    })
                })
                .map_err(|e| format!("Failed to send file: {}", e))?;

                Ok(Dynamic::UNIT)
            },
        )
        .expect("valid syntax registration");
}

fn register_send_to(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);

    engine
        .register_custom_syntax(
            ["SEND", "TO", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let target = context.eval_expression_tree(&inputs[0])?.to_string();
                let message = context.eval_expression_tree(&inputs[1])?.to_string();

                trace!("SEND TO: {} with message", target);

                let state_for_send = Arc::clone(&state_clone);
                let user_for_send = user.clone();

                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        send_to_specific_channel(state_for_send, &user_for_send, &target, &message)
                            .await
                    })
                })
                .map_err(|e| format!("Failed to send: {}", e))?;

                Ok(Dynamic::UNIT)
            },
        )
        .expect("valid syntax registration");
}

fn register_broadcast(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);

    engine
        .register_custom_syntax(
            ["BROADCAST", "$expr$", "TO", "$expr$"],
            false,
            move |context, inputs| {
                let message = context.eval_expression_tree(&inputs[0])?.to_string();
                let recipients = context.eval_expression_tree(&inputs[1])?;

                trace!("BROADCAST: Sending to multiple recipients");

                let state_for_send = Arc::clone(&state_clone);
                let user_for_send = user.clone();

                let results = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        broadcast_message(state_for_send, &user_for_send, &message, recipients)
                            .await
                    })
                })
                .map_err(|e| format!("Failed to broadcast: {}", e))?;

                Ok(results)
            },
        )
        .expect("valid syntax registration");
}

pub async fn send_message_to_recipient(
    state: Arc<AppState>,
    user: &UserSession,
    recipient: &str,
    message: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (channel, recipient_id) = parse_recipient(state.clone(), recipient).await?;

    match channel.as_str() {
        "whatsapp" => {
            let adapter = WhatsAppAdapter::new(state.conn.clone(), user.bot_id);
            let response = crate::shared::models::BotResponse {
                bot_id: "default".to_string(),
                session_id: user.id.to_string(),
                user_id: recipient_id.clone(),
                channel: "whatsapp".to_string(),
                content: message.to_string(),
                message_type: MessageType::EXTERNAL,
                stream_token: None,
                is_complete: true,
                suggestions: vec![],
                context_name: None,
                context_length: 0,
                context_max_length: 0,
            };
            adapter.send_message(response).await?;
        }
        "instagram" => {
            let adapter = InstagramAdapter::new();
            let response = crate::shared::models::BotResponse {
                bot_id: "default".to_string(),
                session_id: user.id.to_string(),
                user_id: recipient_id.clone(),
                channel: "instagram".to_string(),
                content: message.to_string(),
                message_type: MessageType::EXTERNAL,
                stream_token: None,
                is_complete: true,
                suggestions: vec![],
                context_name: None,
                context_length: 0,
                context_max_length: 0,
            };
            adapter.send_message(response).await?;
        }
        "teams" => {
            let adapter = TeamsAdapter::new(state.conn.clone(), user.bot_id);
            let response = crate::shared::models::BotResponse {
                bot_id: "default".to_string(),
                session_id: user.id.to_string(),
                user_id: recipient_id.clone(),
                channel: "teams".to_string(),
                content: message.to_string(),
                message_type: MessageType::EXTERNAL,
                stream_token: None,
                is_complete: true,
                suggestions: vec![],
                context_name: None,
                context_length: 0,
                context_max_length: 0,
            };
            adapter.send_message(response).await?;
        }
        "web" => {
            send_web_message(state.clone(), &recipient_id, message).await?;
        }
        "email" => {
            send_email(state.clone(), &recipient_id, message)?;
        }
        _ => {
            error!("Unknown channel: {}", channel);
            return Err(format!("Unknown channel: {}", channel).into());
        }
    }

    Ok(())
}

async fn send_file_to_recipient(
    state: Arc<AppState>,
    user: &UserSession,
    recipient: &str,
    file: Dynamic,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    send_file_with_caption_to_recipient(state, user, recipient, file, "").await
}

async fn send_file_with_caption_to_recipient(
    state: Arc<AppState>,
    user: &UserSession,
    recipient: &str,
    file: Dynamic,
    caption: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (channel, recipient_id) = parse_recipient(state.clone(), recipient).await?;

    let file_data = if file.is_string() {
        let file_path = file.to_string();
        std::fs::read(&file_path)?
    } else {
        return Err("File must be a string path".into());
    };

    match channel.as_str() {
        "whatsapp" => {
            send_whatsapp_file(state, user, &recipient_id, file_data, caption).await?;
        }
        "instagram" => {
            send_instagram_file(state, user, &recipient_id, file_data, caption).await?;
        }
        "teams" => {
            send_teams_file(state, user, &recipient_id, file_data, caption).await?;
        }
        "web" => {
            send_web_file(state, &recipient_id, file_data, caption).await?;
        }
        "email" => {
            send_email_attachment(state, &recipient_id, file_data, caption)?;
        }
        _ => {
            return Err(format!("Unsupported channel for file sending: {}", channel).into());
        }
    }

    Ok(())
}

async fn parse_recipient(
    state: Arc<AppState>,
    recipient: &str,
) -> Result<(String, String), Box<dyn std::error::Error + Send + Sync>> {
    if recipient.contains(':') {
        let parts: Vec<&str> = recipient.splitn(2, ':').collect();
        if parts.len() == 2 {
            return Ok((parts[0].to_string(), parts[1].to_string()));
        }
    }

    if recipient.starts_with('+') || recipient.chars().all(|c| c.is_numeric()) {
        return Ok(("whatsapp".to_string(), recipient.to_string()));
    }

    if recipient.contains('@') {
        if recipient.ends_with("@teams.ms") || recipient.contains("@microsoft") {
            return Ok(("teams".to_string(), recipient.to_string()));
        }
        return Ok(("email".to_string(), recipient.to_string()));
    }

    if let Some(redis_client) = &state.cache {
        let mut conn = redis_client.get_multiplexed_async_connection().await?;
        let web_session_key = format!("web_session:{}", recipient);

        if redis::cmd("EXISTS")
            .arg(&web_session_key)
            .query_async::<bool>(&mut conn)
            .await?
        {
            return Ok(("web".to_string(), recipient.to_string()));
        }
    }

    Ok(("whatsapp".to_string(), recipient.to_string()))
}

async fn send_to_specific_channel(
    state: Arc<AppState>,
    user: &UserSession,
    target: &str,
    message: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    send_message_to_recipient(state, user, target, message).await
}

async fn broadcast_message(
    state: Arc<AppState>,
    user: &UserSession,
    message: &str,
    recipients: Dynamic,
) -> Result<Dynamic, Box<dyn std::error::Error>> {
    let mut results = Vec::new();

    if recipients.is_array() {
        let recipient_list = match recipients.into_array() {
            Ok(arr) => arr,
            Err(_) => return Ok(Dynamic::from("[]")),
        };

        for recipient in recipient_list {
            let recipient_str = recipient.to_string();

            match send_message_to_recipient(state.clone(), user, &recipient_str, message).await {
                Ok(_) => {
                    results.push(json!({
                        "recipient": recipient_str,
                        "status": "sent"
                    }));
                }
                Err(e) => {
                    results.push(json!({
                        "recipient": recipient_str,
                        "status": "failed",
                        "error": e.to_string()
                    }));
                }
            }
        }
    }

    Ok(Dynamic::from(serde_json::to_string(&results)?))
}

async fn send_whatsapp_file(
    state: Arc<AppState>,
    user: &UserSession,
    recipient: &str,
    file_data: Vec<u8>,
    caption: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use reqwest::Client;

    let _adapter = WhatsAppAdapter::new(state.conn.clone(), user.bot_id);

    let phone_number_id = "";
    let upload_url = format!("https://graph.facebook.com/v17.0/{}/media", phone_number_id);

    let client = Client::new();
    let form = reqwest::multipart::Form::new()
        .text("messaging_product", "whatsapp")
        .part("file", reqwest::multipart::Part::bytes(file_data));

    let access_token = "";
    let upload_response = client
        .post(&upload_url)
        .bearer_auth(access_token)
        .multipart(form)
        .send()
        .await?;

    if !upload_response.status().is_success() {
        return Err("Failed to upload file to WhatsApp".into());
    }

    let upload_result: serde_json::Value = upload_response.json().await?;
    let media_id = upload_result["id"].as_str().ok_or("No media ID returned")?;

    let send_url = format!(
        "https://graph.facebook.com/v17.0/{}/messages",
        phone_number_id
    );

    let payload = json!({
        "messaging_product": "whatsapp",
        "to": recipient,
        "type": "document",
        "document": {
            "id": media_id,
            "caption": caption
        }
    });

    client
        .post(&send_url)
        .bearer_auth(access_token)
        .json(&payload)
        .send()
        .await?;

    Ok(())
}

async fn send_instagram_file(
    state: Arc<AppState>,
    user: &UserSession,
    recipient_id: &str,
    file_data: Vec<u8>,
    caption: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let adapter = InstagramAdapter::new();

    let file_key = format!("temp/instagram/{}_{}.bin", user.id, uuid::Uuid::new_v4());

    if let Some(s3) = &state.drive {
        s3.put_object()
            .bucket("uploads")
            .key(&file_key)
            .body(aws_sdk_s3::primitives::ByteStream::from(file_data))
            .send()
            .await?;

        let file_url = format!("https://s3.amazonaws.com/uploads/{}", file_key);

        adapter
            .send_media_message(recipient_id, &file_url, "file")
            .await?;

        if !caption.is_empty() {
            adapter
                .send_instagram_message(recipient_id, caption)
                .await?;
        }

        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
            if let Some(s3) = &state.drive {
                let _ = s3
                    .delete_object()
                    .bucket("uploads")
                    .key(&file_key)
                    .send()
                    .await;
            }
        });
    }

    Ok(())
}

async fn send_teams_file(
    state: Arc<AppState>,
    user: &UserSession,
    recipient_id: &str,
    file_data: Vec<u8>,
    caption: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _adapter = TeamsAdapter::new(state.conn.clone(), user.bot_id);

    let conversation_id = get_teams_conversation_id(&state, recipient_id).await?;

    let access_token = "";
    let service_url = "https://smba.trafficmanager.net/apis".to_string();
    let url = format!(
        "{}/v3/conversations/{}/activities",
        service_url.trim_end_matches('/'),
        conversation_id
    );

    use base64::{engine::general_purpose::STANDARD, Engine};
    let attachment = json!({
        "contentType": "application/octet-stream",
        "contentUrl": format!("data:application/octet-stream;base64,{}", STANDARD.encode(&file_data)),
        "name": "attachment"
    });

    let activity = json!({
        "type": "message",
        "text": caption,
        "from": {
            "id": "",
            "name": "Bot"
        },
        "conversation": {
            "id": conversation_id
        },
        "recipient": {
            "id": recipient_id
        },
        "attachments": [attachment]
    });

    use reqwest::Client;
    let client = Client::new();
    client
        .post(&url)
        .bearer_auth(access_token)
        .json(&activity)
        .send()
        .await?;

    Ok(())
}

async fn send_web_message(
    state: Arc<AppState>,
    session_id: &str,
    message: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let web_adapter = Arc::clone(&state.web_adapter);

    let response = crate::shared::models::BotResponse {
        bot_id: "system".to_string(),
        user_id: session_id.to_string(),
        session_id: session_id.to_string(),
        channel: "web".to_string(),
        content: message.to_string(),
        message_type: MessageType::USER,
        stream_token: None,
        is_complete: true,
        suggestions: Vec::new(),
        context_name: None,
        context_length: 0,
        context_max_length: 0,
    };

    web_adapter
        .send_message_to_session(session_id, response)
        .await?;

    Ok(())
}

async fn send_web_file(
    state: Arc<AppState>,
    session_id: &str,
    file_data: Vec<u8>,
    caption: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let file_id = uuid::Uuid::new_v4().to_string();
    let file_url = format!("/api/files/{}", file_id);

    if let Some(redis_client) = &state.cache {
        let mut conn = redis_client.get_multiplexed_async_connection().await?;
        let file_key = format!("file:{}", file_id);

        redis::cmd("SET")
            .arg(&file_key)
            .arg(&file_data)
            .arg("EX")
            .arg(3600)
            .query_async::<()>(&mut conn)
            .await?;
    }

    let message = if caption.is_empty() {
        format!("[File: {}]", file_url)
    } else {
        format!("{}\n[File: {}]", caption, file_url)
    };

    send_web_message(state, session_id, &message).await
}

fn send_email(
    state: Arc<AppState>,
    email: &str,
    message: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    #[cfg(feature = "mail")]
    {
        use crate::email::EmailService;

        let email_service = EmailService::new(state);
        email_service.send_email(email, "Message from Bot", message, None)?;
        Ok(())
    }

    #[cfg(not(feature = "mail"))]
    {
        let _ = (state, email, message);
        error!("Email feature not enabled");
        Err("Email feature not enabled".into())
    }
}

fn send_email_attachment(
    state: Arc<AppState>,
    email: &str,
    file_data: Vec<u8>,
    caption: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    #[cfg(feature = "mail")]
    {
        use crate::email::EmailService;

        let email_service = EmailService::new(state);
        email_service.send_email_with_attachment(
            email,
            "File from Bot",
            caption,
            file_data,
            "attachment",
        )?;
        Ok(())
    }

    #[cfg(not(feature = "mail"))]
    {
        let _ = (state, email, file_data, caption);
        error!("Email feature not enabled for attachments");
        Err("Email feature not enabled".into())
    }
}

async fn get_teams_conversation_id(
    state: &Arc<AppState>,
    user_id: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    if let Some(redis_client) = &state.cache {
        let mut conn = redis_client.get_multiplexed_async_connection().await?;
        let key = format!("teams_conversation:{}", user_id);

        if let Ok(conversation) = redis::cmd("GET")
            .arg(&key)
            .query_async::<String>(&mut conn)
            .await
        {
            return Ok(conversation);
        }
    }

    Ok(user_id.to_string())
}
