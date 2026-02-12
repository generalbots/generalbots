use crate::core::shared::models::Attachment;
use crate::core::shared::state::AppState;
use uuid::Uuid;

use super::types::InputType;
use super::validators::validate_input;

pub async fn process_hear_input(
    state: &AppState,
    session_id: Uuid,
    variable_name: &str,
    input: &str,
    attachments: Option<Vec<Attachment>>,
) -> Result<(String, Option<serde_json::Value>), String> {
    let wait_data = if let Some(redis_client) = &state.cache {
        if let Ok(mut conn) = redis_client.get_multiplexed_async_connection().await {
            let key = format!("hear:{session_id}:{variable_name}");

            let data: Result<String, _> = redis::cmd("GET").arg(&key).query_async(&mut conn).await;

            match data {
                Ok(json_str) => serde_json::from_str::<serde_json::Value>(&json_str).ok(),
                Err(_) => None,
            }
        } else {
            None
        }
    } else {
        None
    };

    let input_type = wait_data
        .as_ref()
        .and_then(|d| d.get("type"))
        .and_then(|t| t.as_str())
        .unwrap_or("any");

    let options = wait_data
        .as_ref()
        .and_then(|d| d.get("options"))
        .and_then(|o| o.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect::<Vec<_>>()
        });

    let validation_type = if let Some(opts) = options {
        InputType::Menu(opts)
    } else {
        InputType::parse_type(input_type)
    };

    match validation_type {
        InputType::Image | InputType::QrCode => {
            if let Some(atts) = &attachments {
                if let Some(img) = atts
                    .iter()
                    .find(|a| a.mime_type.as_deref().unwrap_or("").starts_with("image/"))
                {
                    if validation_type == InputType::QrCode {
                        return process_qrcode(state, &img.url).await;
                    }
                    return Ok((
                        img.url.clone(),
                        Some(serde_json::json!({ "attachment": img })),
                    ));
                }
            }
            return Err(validation_type.error_message());
        }
        InputType::Audio => {
            if let Some(atts) = &attachments {
                if let Some(audio) = atts
                    .iter()
                    .find(|a| a.mime_type.as_deref().unwrap_or("").starts_with("audio/"))
                {
                    return process_audio_to_text(state, &audio.url).await;
                }
            }
            return Err(validation_type.error_message());
        }
        InputType::Video => {
            if let Some(atts) = &attachments {
                if let Some(video) = atts
                    .iter()
                    .find(|a| a.mime_type.as_deref().unwrap_or("").starts_with("video/"))
                {
                    return process_video_description(state, &video.url).await;
                }
            }
            return Err(validation_type.error_message());
        }
        InputType::File | InputType::Document => {
            if let Some(atts) = &attachments {
                if let Some(doc) = atts.first() {
                    return Ok((
                        doc.url.clone(),
                        Some(serde_json::json!({ "attachment": doc })),
                    ));
                }
            }
            return Err(validation_type.error_message());
        }
        _ => {}
    }

    let result = validate_input(input, &validation_type);

    if result.is_valid {
        if let Some(redis_client) = &state.cache {
            if let Ok(mut conn) = redis_client.get_multiplexed_async_connection().await {
                let key = format!("hear:{session_id}:{variable_name}");
                let _: Result<(), _> = redis::cmd("DEL").arg(&key).query_async(&mut conn).await;
            }
        }

        Ok((result.normalized_value, result.metadata))
    } else {
        Err(result
            .error_message
            .unwrap_or_else(|| validation_type.error_message()))
    }
}

pub async fn process_qrcode(
    state: &AppState,
    image_url: &str,
) -> Result<(String, Option<serde_json::Value>), String> {
    let botmodels_url = {
        let config_url = state.conn.get().ok().and_then(|mut conn| {
            use crate::core::shared::models::schema::bot_memories::dsl::*;
            use diesel::prelude::*;
            bot_memories
                .filter(key.eq("botmodels-url"))
                .select(value)
                .first::<String>(&mut conn)
                .ok()
        });
        config_url.unwrap_or_else(|| {
            std::env::var("BOTMODELS_URL").unwrap_or_else(|_| "http://localhost:8001".to_string())
        })
    };

    let client = reqwest::Client::new();

    let image_data = client
        .get(image_url)
        .send()
        .await
        .map_err(|e| format!("Failed to download image: {}", e))?
        .bytes()
        .await
        .map_err(|e| format!("Failed to fetch image: {e}"))?;

    let response = client
        .post(format!("{botmodels_url}/api/vision/qrcode"))
        .header("Content-Type", "application/octet-stream")
        .body(image_data.to_vec())
        .send()
        .await
        .map_err(|e| format!("Failed to call botmodels: {}", e))?;

    if response.status().is_success() {
        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to read image: {e}"))?;

        if let Some(qr_data) = result.get("data").and_then(|d| d.as_str()) {
            Ok((
                qr_data.to_string(),
                Some(serde_json::json!({
                    "type": "qrcode",
                    "raw": result
                })),
            ))
        } else {
            Err("No QR code found in image".to_string())
        }
    } else {
        Err("Failed to read QR code".to_string())
    }
}

pub async fn process_audio_to_text(
    _state: &AppState,
    audio_url: &str,
) -> Result<(String, Option<serde_json::Value>), String> {
    let botmodels_url =
        std::env::var("BOTMODELS_URL").unwrap_or_else(|_| "http://localhost:8001".to_string());

    let client = reqwest::Client::new();

    let audio_data = client
        .get(audio_url)
        .send()
        .await
        .map_err(|e| format!("Failed to download audio: {}", e))?
        .bytes()
        .await
        .map_err(|e| format!("Failed to read audio: {e}"))?;

    let response = client
        .post(format!("{botmodels_url}/api/speech/to-text"))
        .header("Content-Type", "application/octet-stream")
        .body(audio_data.to_vec())
        .send()
        .await
        .map_err(|e| format!("Failed to call botmodels: {}", e))?;

    if response.status().is_success() {
        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        if let Some(text) = result.get("text").and_then(|t| t.as_str()) {
            Ok((
                text.to_string(),
                Some(serde_json::json!({
                    "type": "audio_transcription",
                    "language": result.get("language"),
                    "confidence": result.get("confidence")
                })),
            ))
        } else {
            Err("Could not transcribe audio".to_string())
        }
    } else {
        Err("Failed to process audio".to_string())
    }
}

pub async fn process_video_description(
    _state: &AppState,
    video_url: &str,
) -> Result<(String, Option<serde_json::Value>), String> {
    let botmodels_url =
        std::env::var("BOTMODELS_URL").unwrap_or_else(|_| "http://localhost:8001".to_string());

    let client = reqwest::Client::new();

    let video_data = client
        .get(video_url)
        .send()
        .await
        .map_err(|e| format!("Failed to download video: {}", e))?
        .bytes()
        .await
        .map_err(|e| format!("Failed to fetch video: {e}"))?;

    let response = client
        .post(format!("{botmodels_url}/api/vision/describe-video"))
        .header("Content-Type", "application/octet-stream")
        .body(video_data.to_vec())
        .send()
        .await
        .map_err(|e| format!("Failed to read video: {e}"))?;

    if response.status().is_success() {
        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        if let Some(description) = result.get("description").and_then(|d| d.as_str()) {
            Ok((
                description.to_string(),
                Some(serde_json::json!({
                    "type": "video_description",
                    "frame_count": result.get("frame_count"),
                    "url": video_url
                })),
            ))
        } else {
            Err("Could not describe video".to_string())
        }
    } else {
        Err("Failed to process video".to_string())
    }
}
