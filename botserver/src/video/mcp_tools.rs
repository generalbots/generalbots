use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

use crate::core::shared::utils::DbPool;

use super::engine::VideoEngine;
use super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> McpToolResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVideoProjectInput {
    pub name: String,
    pub description: Option<String>,
    pub resolution_width: Option<i32>,
    pub resolution_height: Option<i32>,
    pub fps: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddVideoClipInput {
    pub project_id: String,
    pub source_url: String,
    pub name: Option<String>,
    pub at_ms: Option<i64>,
    pub duration_ms: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateCaptionsInput {
    pub project_id: String,
    pub style: Option<String>,
    pub max_chars_per_line: Option<i32>,
    pub font_size: Option<i32>,
    pub color: Option<String>,
    pub with_background: Option<bool>,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportVideoInput {
    pub project_id: String,
    pub format: Option<String>,
    pub quality: Option<String>,
    pub save_to_library: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddTextOverlayInput {
    pub project_id: String,
    pub content: String,
    pub at_ms: Option<i64>,
    pub duration_ms: Option<i64>,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub font_size: Option<i32>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddAudioTrackInput {
    pub project_id: String,
    pub source_url: String,
    pub name: Option<String>,
    pub track_type: Option<String>,
    pub start_ms: Option<i64>,
    pub volume: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectOutput {
    pub project_id: String,
    pub name: String,
    pub resolution: String,
    pub fps: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddClipOutput {
    pub clip_id: String,
    pub project_id: String,
    pub name: String,
    pub start_ms: i64,
    pub duration_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateCaptionsOutput {
    pub project_id: String,
    pub captions_count: usize,
    pub total_duration_ms: i64,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportVideoOutput {
    pub export_id: String,
    pub project_id: String,
    pub status: String,
    pub format: String,
    pub quality: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddTextOverlayOutput {
    pub layer_id: String,
    pub project_id: String,
    pub content: String,
    pub start_ms: i64,
    pub end_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddAudioTrackOutput {
    pub track_id: String,
    pub project_id: String,
    pub name: String,
    pub track_type: String,
}

pub async fn create_video_project_tool(
    db: DbPool,
    input: CreateVideoProjectInput,
) -> McpToolResponse<CreateProjectOutput> {
    let engine = VideoEngine::new(db);

    let req = CreateProjectRequest {
        name: input.name.clone(),
        description: input.description,
        resolution_width: input.resolution_width,
        resolution_height: input.resolution_height,
        fps: input.fps,
    };

    match engine.create_project(None, None, req).await {
        Ok(project) => {
            info!("MCP: Created video project {} ({})", project.name, project.id);
            McpToolResponse::ok(CreateProjectOutput {
                project_id: project.id.to_string(),
                name: project.name,
                resolution: format!("{}x{}", project.resolution_width, project.resolution_height),
                fps: project.fps,
            })
        }
        Err(e) => {
            error!("MCP: Failed to create video project: {e}");
            McpToolResponse::err(format!("Failed to create project: {e}"))
        }
    }
}

pub async fn add_video_clip_tool(
    db: DbPool,
    input: AddVideoClipInput,
) -> McpToolResponse<AddClipOutput> {
    let project_id = match Uuid::parse_str(&input.project_id) {
        Ok(id) => id,
        Err(_) => return McpToolResponse::err("Invalid project_id format"),
    };

    let engine = VideoEngine::new(db);

    let req = AddClipRequest {
        name: input.name,
        source_url: input.source_url,
        at_ms: input.at_ms,
        duration_ms: input.duration_ms,
    };

    match engine.add_clip(project_id, req).await {
        Ok(clip) => {
            info!("MCP: Added clip {} to project {}", clip.id, project_id);
            McpToolResponse::ok(AddClipOutput {
                clip_id: clip.id.to_string(),
                project_id: clip.project_id.to_string(),
                name: clip.name,
                start_ms: clip.start_ms,
                duration_ms: clip.duration_ms,
            })
        }
        Err(e) => {
            error!("MCP: Failed to add clip: {e}");
            McpToolResponse::err(format!("Failed to add clip: {e}"))
        }
    }
}

pub async fn generate_captions_tool(
    db: DbPool,
    input: GenerateCaptionsInput,
) -> McpToolResponse<GenerateCaptionsOutput> {
    let project_id = match Uuid::parse_str(&input.project_id) {
        Ok(id) => id,
        Err(_) => return McpToolResponse::err("Invalid project_id format"),
    };

    let engine = VideoEngine::new(db);

    let transcription = match engine
        .transcribe_audio(project_id, None, input.language.clone())
        .await
    {
        Ok(t) => t,
        Err(e) => {
            error!("MCP: Transcription failed: {e}");
            return McpToolResponse::err(format!("Transcription failed: {e}"));
        }
    };

    let style = input.style.as_deref().unwrap_or("default");
    let max_chars = input.max_chars_per_line.unwrap_or(40);
    let font_size = input.font_size.unwrap_or(32);
    let color = input.color.as_deref().unwrap_or("#FFFFFF");
    let with_bg = input.with_background.unwrap_or(true);

    match engine
        .generate_captions_from_transcription(
            project_id,
            &transcription,
            style,
            max_chars,
            font_size,
            color,
            with_bg,
        )
        .await
    {
        Ok(layers) => {
            info!(
                "MCP: Generated {} captions for project {}",
                layers.len(),
                project_id
            );
            McpToolResponse::ok(GenerateCaptionsOutput {
                project_id: project_id.to_string(),
                captions_count: layers.len(),
                total_duration_ms: transcription.duration_ms,
                language: transcription.language,
            })
        }
        Err(e) => {
            error!("MCP: Failed to generate captions: {e}");
            McpToolResponse::err(format!("Failed to generate captions: {e}"))
        }
    }
}

pub async fn export_video_tool(
    db: DbPool,
    cache: Option<Arc<redis::Client>>,
    input: ExportVideoInput,
) -> McpToolResponse<ExportVideoOutput> {
    let project_id = match Uuid::parse_str(&input.project_id) {
        Ok(id) => id,
        Err(_) => return McpToolResponse::err("Invalid project_id format"),
    };

    let engine = VideoEngine::new(db);

    let format = input.format.clone().unwrap_or_else(|| "mp4".to_string());
    let quality = input.quality.clone().unwrap_or_else(|| "high".to_string());

    let req = ExportRequest {
        format: Some(format.clone()),
        quality: Some(quality.clone()),
        save_to_library: input.save_to_library,
    };

    match engine.start_export(project_id, req, cache.as_ref()).await {
        Ok(export) => {
            info!(
                "MCP: Started export {} for project {}",
                export.id, project_id
            );
            McpToolResponse::ok(ExportVideoOutput {
                export_id: export.id.to_string(),
                project_id: export.project_id.to_string(),
                status: export.status,
                format,
                quality,
            })
        }
        Err(e) => {
            error!("MCP: Failed to start export: {e}");
            McpToolResponse::err(format!("Failed to start export: {e}"))
        }
    }
}

pub async fn add_text_overlay_tool(
    db: DbPool,
    input: AddTextOverlayInput,
) -> McpToolResponse<AddTextOverlayOutput> {
    let project_id = match Uuid::parse_str(&input.project_id) {
        Ok(id) => id,
        Err(_) => return McpToolResponse::err("Invalid project_id format"),
    };

    let engine = VideoEngine::new(db);

    let start_ms = input.at_ms.unwrap_or(0);
    let duration_ms = input.duration_ms.unwrap_or(5000);
    let end_ms = start_ms + duration_ms;

    let req = AddLayerRequest {
        name: Some("Text".to_string()),
        layer_type: "text".to_string(),
        start_ms: Some(start_ms),
        end_ms: Some(end_ms),
        x: input.x.or(Some(0.5)),
        y: input.y.or(Some(0.9)),
        width: Some(0.8),
        height: Some(0.1),
        properties: Some(serde_json::json!({
            "content": input.content,
            "font_family": "Arial",
            "font_size": input.font_size.unwrap_or(48),
            "color": input.color.unwrap_or_else(|| "#FFFFFF".to_string()),
            "text_align": "center",
        })),
    };

    match engine.add_layer(project_id, req).await {
        Ok(layer) => {
            info!("MCP: Added text overlay {} to project {}", layer.id, project_id);
            McpToolResponse::ok(AddTextOverlayOutput {
                layer_id: layer.id.to_string(),
                project_id: layer.project_id.to_string(),
                content: input.content,
                start_ms: layer.start_ms,
                end_ms: layer.end_ms,
            })
        }
        Err(e) => {
            error!("MCP: Failed to add text overlay: {e}");
            McpToolResponse::err(format!("Failed to add text overlay: {e}"))
        }
    }
}

pub async fn add_audio_track_tool(
    db: DbPool,
    input: AddAudioTrackInput,
) -> McpToolResponse<AddAudioTrackOutput> {
    let project_id = match Uuid::parse_str(&input.project_id) {
        Ok(id) => id,
        Err(_) => return McpToolResponse::err("Invalid project_id format"),
    };

    let engine = VideoEngine::new(db);

    let track_type = input.track_type.clone().unwrap_or_else(|| "music".to_string());

    let req = AddAudioRequest {
        name: input.name,
        source_url: input.source_url,
        track_type: Some(track_type.clone()),
        start_ms: input.start_ms,
        duration_ms: None,
        volume: input.volume,
    };

    match engine.add_audio_track(project_id, req).await {
        Ok(track) => {
            info!("MCP: Added audio track {} to project {}", track.id, project_id);
            McpToolResponse::ok(AddAudioTrackOutput {
                track_id: track.id.to_string(),
                project_id: track.project_id.to_string(),
                name: track.name,
                track_type,
            })
        }
        Err(e) => {
            error!("MCP: Failed to add audio track: {e}");
            McpToolResponse::err(format!("Failed to add audio track: {e}"))
        }
    }
}

pub fn get_tool_definitions() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "name": "create_video_project",
            "description": "Create a new video editing project",
            "input_schema": {
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Name of the video project"
                    },
                    "description": {
                        "type": "string",
                        "description": "Optional description of the project"
                    },
                    "resolution_width": {
                        "type": "integer",
                        "description": "Video width in pixels (default: 1920)"
                    },
                    "resolution_height": {
                        "type": "integer",
                        "description": "Video height in pixels (default: 1080)"
                    },
                    "fps": {
                        "type": "integer",
                        "description": "Frames per second (default: 30)"
                    }
                },
                "required": ["name"]
            }
        }),
        serde_json::json!({
            "name": "add_video_clip",
            "description": "Add a video clip to an existing project",
            "input_schema": {
                "type": "object",
                "properties": {
                    "project_id": {
                        "type": "string",
                        "description": "UUID of the project"
                    },
                    "source_url": {
                        "type": "string",
                        "description": "URL or path to the video file"
                    },
                    "name": {
                        "type": "string",
                        "description": "Optional name for the clip"
                    },
                    "at_ms": {
                        "type": "integer",
                        "description": "Position in timeline (milliseconds)"
                    },
                    "duration_ms": {
                        "type": "integer",
                        "description": "Duration of the clip (milliseconds)"
                    }
                },
                "required": ["project_id", "source_url"]
            }
        }),
        serde_json::json!({
            "name": "generate_captions",
            "description": "Generate captions from audio transcription using AI",
            "input_schema": {
                "type": "object",
                "properties": {
                    "project_id": {
                        "type": "string",
                        "description": "UUID of the project"
                    },
                    "style": {
                        "type": "string",
                        "description": "Caption style (default, bold, minimal)"
                    },
                    "max_chars_per_line": {
                        "type": "integer",
                        "description": "Maximum characters per caption line"
                    },
                    "font_size": {
                        "type": "integer",
                        "description": "Font size for captions"
                    },
                    "color": {
                        "type": "string",
                        "description": "Text color (hex format)"
                    },
                    "with_background": {
                        "type": "boolean",
                        "description": "Add background box behind captions"
                    },
                    "language": {
                        "type": "string",
                        "description": "Language code for transcription"
                    }
                },
                "required": ["project_id"]
            }
        }),
        serde_json::json!({
            "name": "export_video",
            "description": "Export a video project to a file, optionally saving to .gbdrive library",
            "input_schema": {
                "type": "object",
                "properties": {
                    "project_id": {
                        "type": "string",
                        "description": "UUID of the project"
                    },
                    "format": {
                        "type": "string",
                        "description": "Output format (mp4, webm, mov)"
                    },
                    "quality": {
                        "type": "string",
                        "description": "Quality preset (low, medium, high, 4k)"
                    },
                    "save_to_library": {
                        "type": "boolean",
                        "description": "Save to .gbdrive/videos library (default: true)"
                    }
                },
                "required": ["project_id"]
            }
        }),
        serde_json::json!({
            "name": "add_text_overlay",
            "description": "Add a text overlay to a video project",
            "input_schema": {
                "type": "object",
                "properties": {
                    "project_id": {
                        "type": "string",
                        "description": "UUID of the project"
                    },
                    "content": {
                        "type": "string",
                        "description": "Text content to display"
                    },
                    "at_ms": {
                        "type": "integer",
                        "description": "Start time in milliseconds"
                    },
                    "duration_ms": {
                        "type": "integer",
                        "description": "Duration to display (milliseconds)"
                    },
                    "x": {
                        "type": "number",
                        "description": "Horizontal position (0.0 to 1.0)"
                    },
                    "y": {
                        "type": "number",
                        "description": "Vertical position (0.0 to 1.0)"
                    },
                    "font_size": {
                        "type": "integer",
                        "description": "Font size in pixels"
                    },
                    "color": {
                        "type": "string",
                        "description": "Text color (hex format)"
                    }
                },
                "required": ["project_id", "content"]
            }
        }),
        serde_json::json!({
            "name": "add_audio_track",
            "description": "Add an audio track to a video project",
            "input_schema": {
                "type": "object",
                "properties": {
                    "project_id": {
                        "type": "string",
                        "description": "UUID of the project"
                    },
                    "source_url": {
                        "type": "string",
                        "description": "URL or path to the audio file"
                    },
                    "name": {
                        "type": "string",
                        "description": "Optional name for the track"
                    },
                    "track_type": {
                        "type": "string",
                        "description": "Type of track (music, narration, sound_effect)"
                    },
                    "start_ms": {
                        "type": "integer",
                        "description": "Start time in timeline (milliseconds)"
                    },
                    "volume": {
                        "type": "number",
                        "description": "Volume level (0.0 to 1.0)"
                    }
                },
                "required": ["project_id", "source_url"]
            }
        }),
    ]
}
