use axum::Router;
use std::sync::Arc;

use botcore::shared::state::AppState;
use botfacebook::state::FacebookState;

fn strip_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out
}

pub fn configure(app_state: &Arc<AppState>) -> Router {
    let pool = app_state.conn.clone();
    let (default_bot_id, default_bot_name) = (uuid::Uuid::nil(), "default".to_string());

    let fb_state = FacebookState {
        secrets: Arc::new(move |key: &str| -> Result<String, String> {
            match key {
                "fb_verify_token" => Ok("fb-verify-2026".to_string()),
                "fb_page_token" => Err(format!("Token '{}' not configured", key)),
                _ => Err(format!("Secret key '{}' not found", key)),
            }
        }),
        send_message: {
            let token = String::new();
            Arc::new(move |recipient: &str, text: &str, bot_name: &str| {
                let token = token.clone();
                let recipient = recipient.to_string();
                let text = text.to_string();
                let bot_name = bot_name.to_string();
                Box::pin(async move {
                    if token.is_empty() {
                        log::warn!("Facebook page token not configured, skipping send to {}", recipient);
                        return Ok(());
                    }
                    log::info!("[FB send] would send to {}: {} (bot={})", recipient, text.chars().take(80).collect::<String>(), bot_name);
                    Ok(())
                })
            })
        },
        process_message: {
            let state = app_state.clone();
            Arc::new(move |bot_id_str: &str, sender_id: &str, content: &str, session_id_str: &str, bot_name: &str| {
                let state = state.clone();
                let bot_id_str = bot_id_str.to_string();
                let sender_id = sender_id.to_string();
                let content = content.to_string();
                let session_id_str = session_id_str.to_string();
                let bot_name = bot_name.to_string();
                Box::pin(async move {
                    let session_id = uuid::Uuid::parse_str(&session_id_str).unwrap_or_else(|_| uuid::Uuid::new_v4());
                    let user_id = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_DNS, format!("fb:{}", sender_id).as_bytes());

                    let msg = botlib::models::UserMessage::text(
                        &bot_name, user_id.to_string(), session_id.to_string(),
                        "facebook", &content,
                    );

                    struct FbBufferedSink {
                        buffer: Arc<tokio::sync::Mutex<String>>,
                        sender_id: String,
                    }

                    #[async_trait::async_trait]
                    impl crate::core::bot::pipeline::sink::ChannelSink for FbBufferedSink {
                        async fn send_bot_response(&self, response: &botlib::models::BotResponse) -> Result<(), crate::core::bot::pipeline::PipelineError> {
                            let mut buf = self.buffer.lock().await;
                            let clean = strip_html(&response.content);
                            buf.push_str(&clean);
                            if response.is_complete {
                                let full = std::mem::take(&mut *buf);
                                if !full.is_empty() {
                                    log::info!("[FB response] to {}: {}", self.sender_id, full.chars().take(80).collect::<String>());
                                }
                            }
                            Ok(())
                        }
                        async fn send_error(&self, _session_id: &str, message: &str) -> Result<(), crate::core::bot::pipeline::PipelineError> {
                            log::warn!("[FB error] to {}: {}", self.sender_id, message.chars().take(80).collect::<String>());
                            Ok(())
                        }
                        fn channel_type(&self) -> &str { "facebook" }
                        fn supports_streaming(&self) -> bool { false }
                        fn supports_suggestions(&self) -> bool { false }
                    }

                    let sink = FbBufferedSink {
                        buffer: Arc::new(tokio::sync::Mutex::new(String::new())),
                        sender_id: sender_id.clone(),
                    };

                    let _ = crate::core::bot::pipeline::exec::run_pipeline_for_channel(&state, &msg, &sink).await;
                    Ok(())
                })
            })
        },
        find_bot: Arc::new(move |_sender_id: &str| {
            let _ = pool.clone();
            Box::pin(async move { Some("cristo".to_string()) })
        }),
        get_default_bot: Arc::new(move || (default_bot_id, default_bot_name.clone())),
    };

    botfacebook::configure_facebook_routes().with_state(Arc::new(fb_state))
}
