pub use botwhatsapp::*;

use std::sync::Arc;

use axum::Router;
use botcore::shared::state::AppState;
use botcore::shared::utils::DbPool;
use uuid::Uuid;

use diesel::RunQueryDsl;
use crate::core::bot::pipeline::{self, ChannelSink, PipelineError, PipelineResult};

fn get_default_bot_id_simple(pool: &DbPool) -> Uuid {
    if let Ok(mut conn) = pool.get_timeout(std::time::Duration::from_secs(2)) {
        #[derive(diesel::QueryableByName)]
        struct BotRow {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
        }
        diesel::sql_query("SELECT id FROM bots ORDER BY created_at ASC LIMIT 1")
            .get_result::<BotRow>(&mut conn).ok().map(|r| r.id).unwrap_or_default()
    } else {
        Uuid::nil()
    }
}

struct WABufferedSink {
    send_msg: botwhatsapp::state::SendMessageFn,
    bot_name: String,
    phone: String,
    buffer: Arc<tokio::sync::Mutex<String>>,
}

#[async_trait::async_trait]
impl ChannelSink for WABufferedSink {
    async fn send_bot_response(&self, response: &botlib::models::BotResponse) -> PipelineResult<()> {
        let mut buf = self.buffer.lock().await;
        buf.push_str(&response.content);
        if response.is_complete {
            let full = std::mem::take(&mut *buf);
            if !full.is_empty() {
                (self.send_msg)(&self.phone, &full, &self.bot_name).await
                    .map_err(|e| PipelineError::Transport(e))?;
            }
        }
        Ok(())
    }

    async fn send_error(&self, _session_id: &str, message: &str) -> PipelineResult<()> {
        let mut buf = self.buffer.lock().await;
        buf.push_str(message);
        let full = std::mem::take(&mut *buf);
        if !full.is_empty() {
            (self.send_msg)(&self.phone, &full, &self.bot_name).await
                .map_err(|e| PipelineError::Transport(e))?;
        }
        Ok(())
    }

    fn channel_type(&self) -> &str { "whatsapp" }
    fn supports_streaming(&self) -> bool { false }
    fn supports_suggestions(&self) -> bool { false }
}

fn load_wa_config(pool: &DbPool, bot_id: &Uuid, key: &str) -> Option<String> {
    if let Ok(mut conn) = pool.get_timeout(std::time::Duration::from_secs(3)) {
        #[derive(diesel::QueryableByName)]
        struct Cv {
            #[diesel(sql_type = diesel::sql_types::Text)]
            v: String,
        }
        diesel::sql_query("SELECT config_value v FROM bot_configuration WHERE bot_id = $1 AND config_key = $2 LIMIT 1")
            .bind::<diesel::sql_types::Uuid, _>(bot_id)
            .bind::<diesel::sql_types::Text, _>(key)
            .get_result::<Cv>(&mut conn).ok().map(|r| r.v)
    } else { None }
}

pub fn configure(app_state: &Arc<AppState>) -> Router<()> {
    let pool = app_state.conn.clone();
    let bot_id = get_default_bot_id_simple(&pool);
    log::info!("WhatsApp module: using bot_id={}", bot_id);

    let cfg_phone_number_id = load_wa_config(&pool, &bot_id, "whatsapp-phone-number-id").unwrap_or_default();
    let cfg_api_key = load_wa_config(&pool, &bot_id, "whatsapp-api-key").unwrap_or_default();
    let cfg_verify_token = load_wa_config(&pool, &bot_id, "whatsapp-verify-token").unwrap_or_default();
    let cfg_business_account_id = load_wa_config(&pool, &bot_id, "whatsapp-business-account-id").unwrap_or_default();
    let cfg_api_url = load_wa_config(&pool, &bot_id, "whatsapp-api-url")
        .unwrap_or_else(|| "https://graph.facebook.com/v18.0".to_string());

    let sm_pni = cfg_phone_number_id.clone();
    let sm_key = cfg_api_key.clone();
    let sm_url = cfg_api_url.clone();

    let send_message_fn: botwhatsapp::state::SendMessageFn = Arc::new(move |to: &str, message: &str, _bot_name: &str| {
        let pni = sm_pni.clone();
        let key = sm_key.clone();
        let burl = sm_url.clone();
        let to = to.to_string();
        let message = message.to_string();
        Box::pin(async move {
            if pni.is_empty() || key.is_empty() {
                log::warn!("WhatsApp not configured: missing phone_number_id or api_key");
                return Err("WhatsApp not configured".to_string());
            }
            let url = format!("{}/{}/messages", burl, pni);
            let body = serde_json::json!({
                "messaging_product": "whatsapp",
                "to": to,
                "type": "text",
                "text": { "body": message }
            });
            let cl = reqwest::Client::builder()
                .user_agent("GeneralBots/1.0")
                .build().unwrap_or_default();
            let resp = cl
                .post(&url)
                .header("Authorization", format!("Bearer {}", key))
                .json(&body)
                .send()
                .await
                .map_err(|e| format!("HTTP send error: {}", e))?;
            let status = resp.status();
            if status.is_success() {
                log::info!("WhatsApp message sent to {}", to);
                Ok(())
            } else {
                let body_text = resp.text().await.unwrap_or_default();
                let err_msg = format!("WhatsApp API error {}: {}", status, body_text);
                log::error!("{}", err_msg);
                Err(err_msg)
            }
        })
    });

    let sm_for_process = send_message_fn.clone();

    let gc_pni = cfg_phone_number_id.clone();
    let gc_ba = cfg_business_account_id.clone();
    let gc_url = cfg_api_url.clone();

    let sc_vt = cfg_verify_token.clone();
    let sc_ak = cfg_api_key.clone();

    let wa_state = Arc::new(WhatsAppState {
        pool,
        send_message: send_message_fn,
        get_default_bot: Arc::new(move |_c: &mut diesel::PgConnection| {
            (bot_id, "default".to_string())
        }),
        find_bot: Arc::new(move |_phone: &str| (bot_id, "default".to_string())),
        get_config: {
            Arc::new(move |key: &str| -> Result<String, String> {
                match key {
                    "whatsapp_api_url" => Ok(gc_url.clone()),
                    "whatsapp_phone_number_id" => Ok(gc_pni.clone()),
                    "whatsapp_business_account_id" => Ok(gc_ba.clone()),
                    _ => Err(format!("Config key '{}' not found", key)),
                }
            })
        },
        secrets: {
            Arc::new(move |key: &str| -> Result<String, String> {
                match key {
                    "whatsapp_verify_token" => Ok(sc_vt.clone()),
                    "whatsapp_api_key" => Ok(sc_ak.clone()),
                    _ => Err(format!("Secret key '{}' not found", key)),
                }
            })
        },
        transcribe_audio: Arc::new(|_data: &[u8]| {
            Box::pin(async move { Err("Audio transcription not available".to_string()) })
        }),
        process_message: {
            let app_state = app_state.clone();
            let sm_fn = sm_for_process.clone();
            Arc::new(move |bot_id_str: String, phone: String, content: String| {
                let state = app_state.clone();
                let sm = sm_fn.clone();
                Box::pin(async move {
                    log::info!("WhatsApp process msg from {} for bot '{}': {}", phone, bot_id_str, content);

                    let session_id = Uuid::new_v4();
                    let user_id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, format!("wa:{}", phone).as_bytes());

                    let msg = botlib::models::UserMessage::text(
                        bot_id_str.clone(), user_id.to_string(), session_id.to_string(),
                        "whatsapp", &content,
                    );

                    let sink = WABufferedSink {
                        send_msg: sm,
                        bot_name: bot_id_str.clone(),
                        phone: phone.clone(),
                        buffer: Arc::new(tokio::sync::Mutex::new(String::new())),
                    };

                    let _ = pipeline::run_pipeline_for_channel(
                        &state, &msg, &sink,
                    ).await;

                    Ok(())
                })
            })
        },
        user_lookup: Arc::new(|_identifier: &str| {
            Box::pin(async move { Ok(None::<String>) })
        }),
        user_create: Arc::new(
            |_identifier: &str, _display_name: &str, _email: &str, _phone: Option<&str>| {
                Box::pin(async move { Ok("00000000-0000-0000-0000-000000000000".to_string()) })
            },
        ),
    });

    botwhatsapp::configure_whatsapp_routes().with_state(wa_state)
}