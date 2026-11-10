use crate::engine::VideoEngine;
use crate::models::*;
use crate::requests::*;
use crate::responses::*;

use tracing::info;
use uuid::Uuid;

/// The transcription and caption style one caption pass renders.
#[derive(Debug, Clone, Copy)]
pub struct CaptionRequest<'a> {
    pub project_id: Uuid,
    pub transcription: &'a TranscriptionResponse,
    pub style: &'a str,
    pub max_chars: i32,
    pub font_size: i32,
    pub color: &'a str,
    pub with_background: bool,
}

impl VideoEngine {
    pub async fn generate_preview_frame(
        &self,
        project_id: Uuid,
        at_ms: i64,
        width: i32,
        height: i32,
        output_dir: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let clips = self.get_clips(project_id).await?;

        let clip = clips
            .iter()
            .find(|c| at_ms >= c.start_ms && at_ms < c.start_ms + c.duration_ms)
            .ok_or("No clip at this timestamp")?;

        let seek_time = (at_ms - clip.start_ms + clip.trim_in_ms) as f64 / 1000.0;
        let output_filename = format!("preview_{}_{}.jpg", project_id, at_ms);
        let output_path = format!("{}/{}", output_dir, output_filename);

        let cmd = crate::safe_command::SafeCommand::new("ffmpeg")?
            .arg("-y")?
            .arg("-ss")?
            .arg(format!("{:.3}", seek_time))?
            .arg("-i")?
            .arg(&clip.source_url)?
            .arg("-vframes")?
            .arg("1")?
            .arg("-vf")?
            .arg(format!("scale={}:{}", width, height))?
            .arg("-q:v")?
            .arg("2")?
            .arg(&output_path)?;

        let result = cmd.execute()?;

        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Err(format!("FFmpeg error: {stderr}").into());
        }

        Ok(format!("/video/previews/{}", output_filename))
    }

    pub async fn transcribe_audio(
        &self,
        project_id: Uuid,
        clip_id: Option<Uuid>,
        language: Option<String>,
    ) -> Result<TranscriptionResponse, Box<dyn std::error::Error + Send + Sync>> {
        let clips = self.get_clips(project_id).await?;

        let clip = if let Some(id) = clip_id {
            clips.iter().find(|c| c.id == id).ok_or("Clip not found")?
        } else {
            clips.first().ok_or("No clips in project")?
        };

        let botmodels_url = std::env::var("BOTMODELS_URL").unwrap_or_default();

        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/api/audio/transcribe", botmodels_url))
            .json(&serde_json::json!({
                "audio_url": &clip.source_url,
                "language": language.unwrap_or_else(|| "auto".to_string()),
                "word_timestamps": true,
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let error = response.text().await.unwrap_or_default();
            return Err(format!("Transcription service error: {error}").into());
        }

        let result: serde_json::Value = response.json().await?;

        let segments = segments_from_transcription_json(&result);

        let full_text = segments
            .iter()
            .map(|s| s.text.clone())
            .collect::<Vec<_>>()
            .join(" ");
        let duration_ms = segments.last().map(|s| s.end_ms).unwrap_or(0);

        Ok(TranscriptionResponse {
            segments,
            full_text,
            language: result["language"].as_str().unwrap_or("en").to_string(),
            duration_ms,
        })
    }

    pub async fn generate_captions_from_transcription(
        &self,
        request: CaptionRequest<'_>,
    ) -> Result<Vec<VideoLayer>, diesel::result::Error> {
        let CaptionRequest {
            project_id,
            transcription,
            font_size,
            color,
            with_background,
            ..
        } = request;
        let mut layers = Vec::new();
        let bg_color = if with_background {
            Some("rgba(0,0,0,0.7)")
        } else {
            None
        };

        for segment in &transcription.segments {
            let layer = self
                .add_layer(
                    project_id,
                    AddLayerRequest {
                        name: Some(format!("Caption {}", layers.len() + 1)),
                        layer_type: "text".to_string(),
                        start_ms: Some(segment.start_ms),
                        end_ms: Some(segment.end_ms),
                        x: Some(0.5),
                        y: Some(0.85),
                        width: Some(0.9),
                        height: Some(0.1),
                        properties: Some(serde_json::json!({
                            "content": segment.text.trim(),
                            "font_family": "Arial",
                            "font_size": font_size,
                            "color": color,
                            "text_align": "center",
                            "background_color": bg_color,
                        })),
                    },
                )
                .await?;

            layers.push(layer);
        }

        info!("Generated {} caption layers for project {}", layers.len(), project_id);
        Ok(layers)
    }

    pub async fn text_to_speech(
        &self,
        text: &str,
        voice: &str,
        speed: f32,
        language: &str,
        output_dir: &str,
    ) -> Result<TTSResponse, Box<dyn std::error::Error + Send + Sync>> {
        let botmodels_url = std::env::var("BOTMODELS_URL").unwrap_or_default();

        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/api/audio/tts", botmodels_url))
            .json(&serde_json::json!({
                "text": text,
                "voice": voice,
                "speed": speed,
                "language": language,
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let error = response.text().await.unwrap_or_default();
            return Err(format!("TTS service error: {error}").into());
        }

        let audio_data = response.bytes().await?;
        let filename = format!("tts_{}.mp3", Uuid::new_v4());
        let file_path = format!("{}/{}", output_dir, filename);

        std::fs::write(&file_path, &audio_data)?;

        let duration_ms = self.get_audio_duration(&file_path).unwrap_or(0);

        Ok(TTSResponse {
            audio_url: format!("/video/audio/{}", filename),
            duration_ms,
        })
    }

    fn get_audio_duration(&self, path: &str) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
        let cmd = crate::safe_command::SafeCommand::new("ffprobe")?
            .arg("-v")?
            .arg("error")?
            .arg("-show_entries")?
            .arg("format=duration")?
            .arg("-of")?
            .arg("default=noprint_wrappers=1:nokey=1")?
            .arg(path)?;

        let result = cmd.execute()?;

        let stdout = String::from_utf8_lossy(&result.stdout);
        let duration_secs: f64 = stdout.trim().parse().unwrap_or(0.0);
        Ok((duration_secs * 1000.0) as i64)
    }

    pub async fn detect_scenes(
        &self,
        project_id: Uuid,
        threshold: f32,
        output_dir: &str,
    ) -> Result<SceneDetectionResponse, Box<dyn std::error::Error + Send + Sync>> {
        info!("Detecting scenes for project {project_id} with threshold {threshold}, output_dir: {output_dir}");
        let clips = self.get_clips(project_id).await?;
        let clip = clips.first().ok_or("No clips in project")?;

        let cmd = crate::safe_command::SafeCommand::new("ffmpeg")?
            .arg("-i")?
            .arg(&clip.source_url)?
            .arg("-vf")?
            .arg(format!("select='gt(scene,{})',showinfo", threshold))?
            .arg("-f")?
            .arg("null")?
            .arg("-")?;

        let result = cmd.execute()?;

        let duration_ms = self.get_audio_duration(&clip.source_url).unwrap_or(0);
        let scenes = scenes_from_ffmpeg_log(&String::from_utf8_lossy(&result.stderr), duration_ms);

        info!("Detected {} scenes in project {}", scenes.len(), project_id);
        Ok(SceneDetectionResponse { scenes })
    }
}

/// Decodes the transcription service payload into timeline segments. A missing
/// or non-array `segments` key yields no captions (rather than an error): the
/// service answers `200` with an empty body when the audio has no speech.
fn segments_from_transcription_json(result: &serde_json::Value) -> Vec<TranscriptionSegment> {
    result["segments"]
        .as_array()
        .map(|segments| {
            segments
                .iter()
                .map(|segment| TranscriptionSegment {
                    start_ms: (segment["start"].as_f64().unwrap_or(0.0) * 1000.0) as i64,
                    end_ms: (segment["end"].as_f64().unwrap_or(0.0) * 1000.0) as i64,
                    text: segment["text"].as_str().unwrap_or("").to_string(),
                    confidence: segment["confidence"].as_f64().unwrap_or(0.9) as f32,
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Turns an ffmpeg `showinfo` log into scene ranges. A cut closer than 500 ms
/// to the previous one is folded into it (ffmpeg emits several lines per
/// change and a sub-second scene is not useful on the timeline). The last cut
/// runs to the end of the media, so the returned ranges always cover the tail.
fn scenes_from_ffmpeg_log(stderr: &str, duration_ms: i64) -> Vec<SceneInfo> {
    let mut scenes = Vec::new();
    let mut last_time: f64 = 0.0;

    for line in stderr.lines() {
        if !line.contains("pts_time:") {
            continue;
        }
        let Some(time_str) = line.split("pts_time:").nth(1) else {
            continue;
        };
        let Some(time_end) = time_str.find(char::is_whitespace) else {
            continue;
        };
        let Ok(time) = time_str[..time_end].parse::<f64>() else {
            continue;
        };
        if time > last_time + 0.5 {
            scenes.push(SceneInfo {
                start_ms: (last_time * 1000.0) as i64,
                end_ms: (time * 1000.0) as i64,
                thumbnail_url: None,
                description: None,
            });
            last_time = time;
        }
    }

    if last_time > 0.0 {
        scenes.push(SceneInfo {
            start_ms: (last_time * 1000.0) as i64,
            end_ms: duration_ms,
            thumbnail_url: None,
            description: None,
        });
    }

    scenes
}

#[cfg(test)]
#[path = "effects_tests.rs"]
mod effects_tests;
