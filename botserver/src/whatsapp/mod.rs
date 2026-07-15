pub use botwhatsapp::*;

use std::sync::Arc;

use axum::Router;
use botcore::shared::state::AppState;
use botcore::shared::utils::DbPool;
use botcore::config::ConfigManager;
use diesel::sql_types::Uuid as SqlUuid;
use uuid::Uuid;
use reqwest::Client;

fn get_default_bot_id(pool: &DbPool) -> Uuid {
    use diesel::prelude::*;

    #[derive(diesel::QueryableByName, Debug)]
    struct BotRow {
        #[diesel(sql_type = SqlUuid)]
        id: Uuid,
    }

    if let Ok(mut conn) = pool.get() {
        let result = diesel::sql_query(
            "SELECT id FROM bots ORDER BY created_at ASC LIMIT 1"
        )
        .load::<BotRow>(&mut conn);
        match result {
            Ok(rows) if !rows.is_empty() => rows[0].id,
            _ => Uuid::nil(),
        }
    } else {
        Uuid::nil()
    }
}

pub fn configure(app_state: &Arc<AppState>) -> Router<()> {
    let pool = app_state.conn.clone();
    let bot_id = get_default_bot_id(&pool);
    log::info!("WhatsApp module: using bot_id={}", bot_id);

    let cfg_phone_number_id = get_whatsapp_config(&pool, &bot_id, "whatsapp-phone-number-id")
        .unwrap_or_default();
    let cfg_api_key = get_whatsapp_config(&pool, &bot_id, "whatsapp-api-key")
        .unwrap_or_default();
    let cfg_verify_token = get_whatsapp_config(&pool, &bot_id, "whatsapp-verify-token")
        .unwrap_or_default();
    let cfg_business_account_id = get_whatsapp_config(&pool, &bot_id, "whatsapp-business-account-id")
        .unwrap_or_default();
    let cfg_api_url = get_whatsapp_config(&pool, &bot_id, "whatsapp-api-url")
        .unwrap_or_else(|| "https://graph.facebook.com/v18.0".to_string());

    log::info!("WhatsApp configured: phone_number_id={}", cfg_phone_number_id);
    log::info!("WhatsApp api_key present: {}", !cfg_api_key.is_empty());
    log::info!("WhatsApp verify_token present: {}", !cfg_verify_token.is_empty());
    log::info!("WhatsApp business_account_id={}", cfg_business_account_id);
    log::info!("WhatsApp api_url='{}'", cfg_api_url);

    let http_client = Arc::new(
        Client::builder()
            .user_agent("GeneralBots/1.0")
            .build()
            .expect("Failed to create HTTP client"),
    );

    let sm_cl = http_client.clone();
    let sm_pni = cfg_phone_number_id.clone();
    let sm_key = cfg_api_key.clone();
    let sm_url = cfg_api_url.clone();

    let gc_pni = cfg_phone_number_id.clone();
    let gc_ba = cfg_business_account_id.clone();
    let gc_url = cfg_api_url.clone();

    let sc_vt = cfg_verify_token.clone();
    let sc_ak = cfg_api_key.clone();

    let pm_cl = http_client.clone();
    let pm_pni = cfg_phone_number_id.clone();
    let pm_key = cfg_api_key.clone();
    let pm_url = cfg_api_url.clone();
    let pm_pool = pool.clone();

    let wa_state = Arc::new(WhatsAppState {
        pool,
        send_message: {
            Arc::new(move |to: &str, message: &str, _bot_name: &str| {
                let cl = sm_cl.clone();
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
            })
        },
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
            let pm_burl = pm_url.clone();
            let pm_pni = pm_pni.clone();
            let pm_key = pm_key.clone();
            Arc::new(move |bot_id_str: String, phone: String, content: String| {
                let pool = pm_pool.clone();
                let burl = pm_burl.clone();
                let pni = pm_pni.clone();
                let wkey = pm_key.clone();
                Box::pin(async move {
                    log::info!("WhatsApp process msg from {} for bot '{}': {}", phone, bot_id_str, content);
                    if pni.is_empty() || wkey.is_empty() {
                        log::warn!("WhatsApp not configured, cannot reply");
                        return Ok(());
                    }
                    let bot_id = uuid::Uuid::parse_str(&bot_id_str).unwrap_or_default();
                    let cfg = ConfigManager::new(pool);
                    let llm_url = cfg.get_config(&bot_id, "llm-url", None).ok().filter(|v| !v.is_empty());
                    let llm_key = cfg.get_config(&bot_id, "llm-key", None).ok().filter(|v| !v.is_empty());
                    let llm_model = cfg.get_config(&bot_id, "llm-model", None).ok().filter(|v| !v.is_empty());
                    let system_prompt = cfg.get_config(&bot_id, "system-prompt", None).ok().filter(|v| !v.is_empty());

                    let reply = if let (Some(url), Some(model)) = (&llm_url, &llm_model) {
                        log::info!("Calling LLM for bot {}: model={}", bot_id_str, model);
                        let mut messages = Vec::new();
                        if let Some(sp) = &system_prompt {
                            messages.push(serde_json::json!({"role": "system", "content": sp}));
                        }
                        messages.push(serde_json::json!({"role": "user", "content": content}));
                        let mut req = reqwest::Client::new()
                            .post(url)
                            .json(&serde_json::json!({
                                "model": model,
                                "messages": messages,
                                "max_tokens": 1024,
                            }));
                        if let Some(k) = &llm_key {
                            req = req.header("Authorization", format!("Bearer {}", k));
                        }
                        match req.send().await {
                            Ok(resp) if resp.status().is_success() => {
                                if let Ok(body) = resp.json::<serde_json::Value>().await {
                                    body["choices"][0]["message"]["content"].as_str().unwrap_or("Desculpe, não consegui processar.").to_string()
                                } else {
                                    "Desculpe, erro ao processar resposta.".to_string()
                                }
                            }
                            Ok(resp) => {
                                log::error!("LLM API error: {}", resp.status());
                                format!("Erro LLM: {}", resp.status())
                            }
                            Err(e) => {
                                log::error!("LLM request failed: {}", e);
                                "Desculpe, erro de conexão com o LLM.".to_string()
                            }
                        }
                    } else {
                        log::warn!("Bot {} has no LLM configured", bot_id_str);
                        format!("Echo: {}", content)
                    };

                    let url = format!("{}/{}/messages", burl, pni);
                    let body = serde_json::json!({
                        "messaging_product": "whatsapp",
                        "to": phone,
                        "type": "text",
                        "text": { "body": reply }
                    });
                    match reqwest::Client::new()
                        .post(&url)
                        .header("Authorization", format!("Bearer {}", wkey))
                        .json(&body)
                        .send()
                        .await
                    {
                        Ok(r) => log::info!("WhatsApp reply sent: {} ({})", r.status(), reply.chars().take(50).collect::<String>()),
                        Err(e) => log::error!("Failed to send WhatsApp reply: {:?}", e),
                    }
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

fn get_whatsapp_config(pool: &DbPool, bot_id: &Uuid, key: &str) -> Option<String> {
    let config_manager = ConfigManager::new(pool.clone());
    config_manager.get_config(bot_id, key, None).ok().filter(|v| !v.is_empty())
}