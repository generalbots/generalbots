use anyhow::Result;
use log::{error, info};
use crate::minutes::types::{Transcription, TranscriptionSegment, SpeakerEntry};
use uuid::Uuid;
use chrono::Utc;

pub struct RealSttTranscriber;

impl RealSttTranscriber {
    pub async fn transcribe(
        recording_path: &str,
        language: &str,
        api_base_url: &str,
        api_key: &str,
    ) -> Result<Transcription> {
        info!("Transcribing recording at {recording_path} via botmodels STT");

        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{api_base_url}/api/speech/totext"))
            .header("Authorization", format!("Bearer {api_key}"))
            .json(&serde_json::json!({
                "audio_url": recording_path,
                "language": language,
                "diarization": true,
                "timestamp_granularity": "segment"
            }))
            .send()
            .await
            .map_err(|e| {
                error!("STT API request failed: {e}");
                anyhow::anyhow!("STT request failed: {e}")
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            error!("STT API returned {status}: {body}");
            return Err(anyhow::anyhow!("STT API error {status}: {body}"));
        }

        let stt_result: serde_json::Value = resp.json().await
            .map_err(|e| anyhow::anyhow!("Failed to parse STT response: {e}"))?;

        let segments: Vec<TranscriptionSegment> = stt_result["segments"]
            .as_array()
            .map(|arr| {
                arr.iter().map(|s| {
                    TranscriptionSegment {
                        start: s["start"].as_f64().unwrap_or(0.0),
                        end: s["end"].as_f64().unwrap_or(0.0),
                        speaker: s["speaker"].as_str().unwrap_or("unknown").to_string(),
                        text: s["text"].as_str().unwrap_or("").to_string(),
                    }
                }).collect()
            })
            .unwrap_or_default();

        let full_text: String = segments.iter().map(|s| s.text.as_str()).collect::<Vec<&str>>().join(" ");

        let speaker_map: std::collections::BTreeMap<String, usize> = {
            let mut m = std::collections::BTreeMap::new();
            for seg in &segments {
                *m.entry(seg.speaker.clone()).or_insert(0) += 1;
            }
            m
        };

        let speakers: Vec<SpeakerEntry> = speaker_map.into_iter().enumerate().map(|(i, (id, count))| {
            SpeakerEntry {
                id: id.clone(),
                name: format!("Speaker {}", i + 1),
                segments_count: count,
            }
        }).collect();

        info!("Transcription complete: {} segments, {} speakers", segments.len(), speakers.len());

        Ok(Transcription {
            id: Uuid::new_v4(),
            recording_id: Uuid::nil(),
            full_text,
            segments,
            speakers,
            language: language.to_string(),
            confidence: stt_result["confidence"].as_f64().unwrap_or(0.0),
            created_at: Utc::now(),
        })
    }

    pub async fn transcribe_fallback(recording_path: &str) -> Result<Transcription> {
        info!("Using fallback transcription (no STT configured) for {recording_path}");
        Ok(Transcription {
            id: Uuid::new_v4(),
            recording_id: Uuid::nil(),
            full_text: "[Transcription not available — STT not configured]".to_string(),
            segments: vec![],
            speakers: vec![],
            language: "unknown".to_string(),
            confidence: 0.0,
            created_at: Utc::now(),
        })
    }
}
