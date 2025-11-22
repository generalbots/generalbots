use crate::channels::{
    instagram::InstagramAdapter, teams::TeamsAdapter, whatsapp::WhatsAppAdapter,
};
use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::{error, trace};
use rhai::{Dynamic, Engine};
use serde_json::json;
use std::sync::Arc;

pub fn register_universal_messaging(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    register_talk_to(state.clone(), user.clone(), engine);
    register_send_file_to(state.clone(), user.clone(), engine);
    register_send_to(state.clone(), user.clone(), engine);
    register_broadcast(state.clone(), user.clone(), engine);
}

fn register_talk_to(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);

    engine
        .register_custom_syntax(
            &["TALK", "TO", "$expr$", ",", "$expr$"],
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
        .unwrap();
}

fn register_send_file_to(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_arc = Arc::new(user);

    let user_clone = Arc::clone(&user_arc);
    engine
        .register_custom_syntax(
            &["SEND", "FILE", "TO", "$expr$", ",", "$expr$"],
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
        .unwrap();

    // With caption variant
    let state_clone2 = Arc::clone(&state);
    let user_clone2 = Arc::clone(&user_arc);

    engine
        .register_custom_syntax(
            &["SEND", "FILE", "TO", "$expr$", ",", "$expr$", ",", "$expr$"],
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
        .unwrap();
}

fn register_send_to(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);

    // SEND TO channel:id, message - explicit channel specification
    engine
        .register_custom_syntax(
            &["SEND", "TO", "$expr$", ",", "$expr$"],
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
        .unwrap();
}

fn register_broadcast(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);

    // BROADCAST message TO list
    engine
        .register_custom_syntax(
            &["BROADCAST", "$expr$", "TO", "$expr$"],
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
        .unwrap();
}

// Helper functions
async fn send_message_to_recipient(
    state: Arc<AppState>,
    _user: &UserSession,
    recipient: &str,
    message: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Determine channel and recipient ID from the recipient string
    let (channel, recipient_id) = parse_recipient(state.clone(), recipient).await?;

    match channel.as_str() {
        "whatsapp" => {
            let adapter = WhatsAppAdapter::new(state.clone());
            adapter.send_message(&recipient_id, message).await?;
        }
        "instagram" => {
            let adapter = InstagramAdapter::new(state.clone());
            adapter.send_message(&recipient_id, message).await?;
        }
        "teams" => {
            let adapter = TeamsAdapter::new(state.clone());
            // For Teams, we need conversation ID
            let conversation_id = get_teams_conversation_id(&state, &recipient_id).await?;
            adapter
                .send_message(&conversation_id, &recipient_id, message)
                .await?;
        }
        "web" => {
            // Send to web socket session
            send_web_message(state.clone(), &recipient_id, message).await?;
        }
        "email" => {
            // Send email
            send_email(state.clone(), &recipient_id, message).await?;
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
    _user: &UserSession,
    recipient: &str,
    file: Dynamic,
    caption: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (channel, recipient_id) = parse_recipient(state.clone(), recipient).await?;

    // Convert Dynamic file to bytes
    let file_data = if file.is_string() {
        // If it's a file path, read the file
        let file_path = file.to_string();
        std::fs::read(&file_path)?
    } else {
        return Err("File must be a string path".into());
    };

    match channel.as_str() {
        "whatsapp" => {
            send_whatsapp_file(state, &recipient_id, file_data, caption).await?;
        }
        "instagram" => {
            send_instagram_file(state, &recipient_id, file_data, caption).await?;
        }
        "teams" => {
            send_teams_file(state, &recipient_id, file_data, caption).await?;
        }
        "web" => {
            send_web_file(state, &recipient_id, file_data, caption).await?;
        }
        "email" => {
            send_email_attachment(state, &recipient_id, file_data, caption).await?;
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
    // Check for explicit channel specification (channel:id format)
    if recipient.contains(':') {
        let parts: Vec<&str> = recipient.splitn(2, ':').collect();
        if parts.len() == 2 {
            return Ok((parts[0].to_string(), parts[1].to_string()));
        }
    }

    // Auto-detect channel based on format
    if recipient.starts_with('+') || recipient.chars().all(|c| c.is_numeric()) {
        // Phone number - WhatsApp
        return Ok(("whatsapp".to_string(), recipient.to_string()));
    }

    if recipient.contains('@') {
        // Email address - could be email or Teams
        if recipient.ends_with("@teams.ms") || recipient.contains("@microsoft") {
            return Ok(("teams".to_string(), recipient.to_string()));
        } else {
            return Ok(("email".to_string(), recipient.to_string()));
        }
    }

    // Check if it's a known web session
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

    // Default to current user's channel if available
    Ok(("whatsapp".to_string(), recipient.to_string()))
}

async fn send_to_specific_channel(
    state: Arc<AppState>,
    user: &UserSession,
    target: &str,
    message: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Parse target as channel:recipient format
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
        let recipient_list = recipients.into_array().unwrap();

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

// Channel-specific implementations
async fn send_whatsapp_file(
    state: Arc<AppState>,
    recipient: &str,
    file_data: Vec<u8>,
    caption: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use reqwest::Client;

    let adapter = WhatsAppAdapter::new(state);

    // First, upload the file to WhatsApp
    let upload_url = format!(
        "https://graph.facebook.com/v17.0/{}/media",
        adapter.phone_number_id
    );

    let client = Client::new();
    let form = reqwest::multipart::Form::new()
        .text("messaging_product", "whatsapp")
        .part("file", reqwest::multipart::Part::bytes(file_data));

    let upload_response = client
        .post(&upload_url)
        .bearer_auth(&adapter.access_token)
        .multipart(form)
        .send()
        .await?;

    if !upload_response.status().is_success() {
        return Err("Failed to upload file to WhatsApp".into());
    }

    let upload_result: serde_json::Value = upload_response.json().await?;
    let media_id = upload_result["id"].as_str().ok_or("No media ID returned")?;

    // Send the file message
    let send_url = format!(
        "https://graph.facebook.com/v17.0/{}/messages",
        adapter.phone_number_id
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
        .bearer_auth(&adapter.access_token)
        .json(&payload)
        .send()
        .await?;

    Ok(())
}

async fn send_instagram_file(
    state: Arc<AppState>,
    _recipient: &str,
    _file_data: Vec<u8>,
    _caption: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Instagram file sending implementation
    // Similar to WhatsApp but using Instagram API
    let _adapter = InstagramAdapter::new(state);

    // Upload and send via Instagram Messaging API

    Ok(())
}

async fn send_teams_file(
    state: Arc<AppState>,
    recipient_id: &str,
    file_data: Vec<u8>,
    caption: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let adapter = TeamsAdapter::new(state.clone());

    // Get conversation ID
    let conversation_id = get_teams_conversation_id(&state, recipient_id).await?;

    // Upload to Teams and send as attachment
    let access_token = adapter.get_access_token().await?;
    let url = format!(
        "{}/v3/conversations/{}/activities",
        adapter.service_url.trim_end_matches('/'),
        conversation_id
    );

    // Create attachment activity
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
            "id": adapter.app_id,
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
        .bearer_auth(&access_token)
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
    // Send via websocket to web client
    let web_adapter = Arc::clone(&state.web_adapter);

    let response = crate::shared::models::BotResponse {
        bot_id: "system".to_string(),
        user_id: session_id.to_string(),
        session_id: session_id.to_string(),
        channel: "web".to_string(),
        content: message.to_string(),
        message_type: 1,
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
    // Store file and send URL to web client
    let file_id = uuid::Uuid::new_v4().to_string();
    let file_url = format!("/api/files/{}", file_id);

    // Store file in temporary storage
    if let Some(redis_client) = &state.cache {
        let mut conn = redis_client.get_multiplexed_async_connection().await?;
        let file_key = format!("file:{}", file_id);

        redis::cmd("SET")
            .arg(&file_key)
            .arg(&file_data)
            .arg("EX")
            .arg(3600) // 1 hour TTL
            .query_async::<()>(&mut conn)
            .await?;
    }

    // Send file URL as message
    let message = if !caption.is_empty() {
        format!("{}\n[File: {}]", caption, file_url)
    } else {
        format!("[File: {}]", file_url)
    };

    send_web_message(state, session_id, &message).await
}

async fn send_email(
    state: Arc<AppState>,
    email: &str,
    message: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Send email using the email service
    #[cfg(feature = "email")]
    {
        use crate::email::EmailService;

        let email_service = EmailService::new(state);
        email_service
            .send_email(email, "Message from Bot", message, None)
            .await?;
        Ok(())
    }

    #[cfg(not(feature = "email"))]
    {
        let _ = (state, email, message); // Explicitly use variables to avoid warnings
        error!("Email feature not enabled");
        Err("Email feature not enabled".into())
    }
}

async fn send_email_attachment(
    state: Arc<AppState>,
    email: &str,
    file_data: Vec<u8>,
    caption: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    #[cfg(feature = "email")]
    {
        use crate::email::EmailService;

        let email_service = EmailService::new(state);
        email_service
            .send_email_with_attachment(email, "File from Bot", caption, file_data, "attachment")
            .await?;
        Ok(())
    }

    #[cfg(not(feature = "email"))]
    {
        let _ = (state, email, file_data, caption); // Explicitly use variables to avoid warnings
        error!("Email feature not enabled for attachments");
        Err("Email feature not enabled".into())
    }
}

async fn get_teams_conversation_id(
    state: &Arc<AppState>,
    user_id: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // Get or create Teams conversation ID for user
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

    // Return default or create new conversation
    Ok(user_id.to_string())
}
