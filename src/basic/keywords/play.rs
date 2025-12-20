//! PLAY keyword for content projector/player
//!
//! Opens a modal/projector to display various content types.
//!
//! Syntax:
//! - PLAY "video.mp4"
//! - PLAY "image.png"
//! - PLAY "presentation.pptx"
//! - PLAY "document.pdf"
//! - PLAY "code.rs"
//! - PLAY url
//! - PLAY file WITH OPTIONS "autoplay,loop,fullscreen"

use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::{info, trace};
use rhai::{Dynamic, Engine};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

/// Content types that can be played
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContentType {
    Video,
    Audio,
    Image,
    Presentation,
    Document,
    Code,
    Spreadsheet,
    Pdf,
    Markdown,
    Html,
    Iframe,
    Unknown,
}

impl ContentType {
    /// Detect content type from file extension
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            // Video
            "mp4" | "webm" | "ogg" | "mov" | "avi" | "mkv" | "m4v" => ContentType::Video,
            // Audio
            "mp3" | "wav" | "flac" | "aac" | "m4a" | "wma" => ContentType::Audio,
            // Images
            "jpg" | "jpeg" | "png" | "gif" | "webp" | "svg" | "bmp" | "ico" => ContentType::Image,
            // Presentations
            "pptx" | "ppt" | "odp" | "key" => ContentType::Presentation,
            // Documents
            "docx" | "doc" | "odt" | "rtf" => ContentType::Document,
            // Spreadsheets
            "xlsx" | "xls" | "csv" | "ods" => ContentType::Spreadsheet,
            // PDF
            "pdf" => ContentType::Pdf,
            // Code
            "rs" | "py" | "js" | "ts" | "java" | "c" | "cpp" | "h" | "go" | "rb" | "php"
            | "swift" | "kt" | "scala" | "r" | "sql" | "sh" | "bash" | "zsh" | "ps1" | "yaml"
            | "yml" | "toml" | "json" | "xml" | "bas" | "basic" => ContentType::Code,
            // Markdown
            "md" | "markdown" => ContentType::Markdown,
            // HTML
            "html" | "htm" => ContentType::Html,
            _ => ContentType::Unknown,
        }
    }

    /// Detect content type from MIME type
    pub fn from_mime(mime: &str) -> Self {
        if mime.starts_with("video/") {
            ContentType::Video
        } else if mime.starts_with("audio/") {
            ContentType::Audio
        } else if mime.starts_with("image/") {
            ContentType::Image
        } else if mime == "application/pdf" {
            ContentType::Pdf
        } else if mime.contains("presentation") || mime.contains("powerpoint") {
            ContentType::Presentation
        } else if mime.contains("spreadsheet") || mime.contains("excel") {
            ContentType::Spreadsheet
        } else if mime.contains("document") || mime.contains("word") {
            ContentType::Document
        } else if mime.starts_with("text/") {
            if mime.contains("html") {
                ContentType::Html
            } else if mime.contains("markdown") {
                ContentType::Markdown
            } else {
                ContentType::Code
            }
        } else {
            ContentType::Unknown
        }
    }

    /// Get the player component name for this content type
    pub fn player_component(&self) -> &'static str {
        match self {
            ContentType::Video => "video-player",
            ContentType::Audio => "audio-player",
            ContentType::Image => "image-viewer",
            ContentType::Presentation => "presentation-viewer",
            ContentType::Document => "document-viewer",
            ContentType::Code => "code-viewer",
            ContentType::Spreadsheet => "spreadsheet-viewer",
            ContentType::Pdf => "pdf-viewer",
            ContentType::Markdown => "markdown-viewer",
            ContentType::Html => "html-viewer",
            ContentType::Iframe => "iframe-viewer",
            ContentType::Unknown => "generic-viewer",
        }
    }
}

/// Play options
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayOptions {
    pub autoplay: bool,
    pub loop_content: bool,
    pub fullscreen: bool,
    pub muted: bool,
    pub controls: bool,
    pub start_time: Option<f64>,
    pub end_time: Option<f64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub theme: Option<String>,
    pub line_numbers: Option<bool>,
    pub highlight_lines: Option<Vec<u32>>,
    pub slide: Option<u32>,
    pub page: Option<u32>,
    pub zoom: Option<f64>,
}

impl PlayOptions {
    /// Parse options from a comma-separated string
    pub fn from_string(options_str: &str) -> Self {
        let mut opts = PlayOptions::default();
        opts.controls = true; // Default to showing controls

        for opt in options_str.split(',').map(|s| s.trim().to_lowercase()) {
            match opt.as_str() {
                "autoplay" => opts.autoplay = true,
                "loop" => opts.loop_content = true,
                "fullscreen" => opts.fullscreen = true,
                "muted" => opts.muted = true,
                "nocontrols" => opts.controls = false,
                "linenumbers" => opts.line_numbers = Some(true),
                _ => {
                    // Handle key=value options
                    if let Some((key, value)) = opt.split_once('=') {
                        match key {
                            "start" => opts.start_time = value.parse().ok(),
                            "end" => opts.end_time = value.parse().ok(),
                            "width" => opts.width = value.parse().ok(),
                            "height" => opts.height = value.parse().ok(),
                            "theme" => opts.theme = Some(value.to_string()),
                            "slide" => opts.slide = value.parse().ok(),
                            "page" => opts.page = value.parse().ok(),
                            "zoom" => opts.zoom = value.parse().ok(),
                            "highlight" => {
                                opts.highlight_lines =
                                    Some(value.split('-').filter_map(|s| s.parse().ok()).collect());
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        opts
    }
}

/// Play content request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayContent {
    pub id: Uuid,
    pub session_id: Uuid,
    pub content_type: ContentType,
    pub source: String,
    pub title: Option<String>,
    pub options: PlayOptions,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Response sent to UI to trigger player
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayResponse {
    pub player_id: Uuid,
    pub content_type: ContentType,
    pub component: String,
    pub source_url: String,
    pub title: String,
    pub options: PlayOptions,
    pub metadata: HashMap<String, String>,
}

/// Register the PLAY keyword
pub fn play_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    if let Err(e) = play_simple_keyword(state.clone(), user.clone(), engine) {
        log::error!("Failed to register PLAY keyword: {}", e);
    }
    if let Err(e) = play_with_options_keyword(state.clone(), user.clone(), engine) {
        log::error!("Failed to register PLAY WITH OPTIONS keyword: {}", e);
    }
    if let Err(e) = stop_keyword(state.clone(), user.clone(), engine) {
        log::error!("Failed to register STOP keyword: {}", e);
    }
    if let Err(e) = pause_keyword(state.clone(), user.clone(), engine) {
        log::error!("Failed to register PAUSE keyword: {}", e);
    }
    if let Err(e) = resume_keyword(state.clone(), user.clone(), engine) {
        log::error!("Failed to register RESUME keyword: {}", e);
    }
}

/// PLAY "source"
fn play_simple_keyword(
    state: Arc<AppState>,
    user: UserSession,
    engine: &mut Engine,
) -> Result<(), rhai::ParseError> {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine.register_custom_syntax(&["PLAY", "$expr$"], false, move |context, inputs| {
        let source = context
            .eval_expression_tree(&inputs[0])?
            .to_string()
            .trim_matches('"')
            .to_string();

        trace!("PLAY '{}' for session: {}", source, user_clone.id);

        let state_for_task = Arc::clone(&state_clone);
        let session_id = user_clone.id;

        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
            let result = rt.block_on(async {
                execute_play(&state_for_task, session_id, &source, PlayOptions::default()).await
            });
            let _ = tx.send(result);
        });

        match rx.recv_timeout(std::time::Duration::from_secs(30)) {
            Ok(Ok(response)) => {
                let json = serde_json::to_string(&response).unwrap_or_default();
                Ok(Dynamic::from(json))
            }
            Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                e.into(),
                rhai::Position::NONE,
            ))),
            Err(_) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                "PLAY timed out".into(),
                rhai::Position::NONE,
            ))),
        }
    })?;
    Ok(())
}

/// PLAY "source" WITH OPTIONS "options"
fn play_with_options_keyword(
    state: Arc<AppState>,
    user: UserSession,
    engine: &mut Engine,
) -> Result<(), rhai::ParseError> {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine.register_custom_syntax(
        &["PLAY", "$expr$", "WITH", "OPTIONS", "$expr$"],
        false,
        move |context, inputs| {
            let source = context
                .eval_expression_tree(&inputs[0])?
                .to_string()
                .trim_matches('"')
                .to_string();
            let options_str = context
                .eval_expression_tree(&inputs[1])?
                .to_string()
                .trim_matches('"')
                .to_string();

            let options = PlayOptions::from_string(&options_str);

            trace!(
                "PLAY '{}' WITH OPTIONS '{}' for session: {}",
                source,
                options_str,
                user_clone.id
            );

            let state_for_task = Arc::clone(&state_clone);
            let session_id = user_clone.id;

            let (tx, rx) = std::sync::mpsc::channel();

            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
                let result = rt.block_on(async {
                    execute_play(&state_for_task, session_id, &source, options).await
                });
                let _ = tx.send(result);
            });

            match rx.recv_timeout(std::time::Duration::from_secs(30)) {
                Ok(Ok(response)) => {
                    let json = serde_json::to_string(&response).unwrap_or_default();
                    Ok(Dynamic::from(json))
                }
                Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    e.into(),
                    rhai::Position::NONE,
                ))),
                Err(_) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    "PLAY timed out".into(),
                    rhai::Position::NONE,
                ))),
            }
        },
    )?;
    Ok(())
}

/// STOP - Stop current playback
fn stop_keyword(
    state: Arc<AppState>,
    user: UserSession,
    engine: &mut Engine,
) -> Result<(), rhai::ParseError> {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine.register_custom_syntax(&["STOP"], false, move |_context, _inputs| {
        trace!("STOP playback for session: {}", user_clone.id);

        let state_for_task = Arc::clone(&state_clone);
        let session_id = user_clone.id;

        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
            let result = rt
                .block_on(async { send_player_command(&state_for_task, session_id, "stop").await });
            let _ = tx.send(result);
        });

        match rx.recv_timeout(std::time::Duration::from_secs(10)) {
            Ok(Ok(_)) => Ok(Dynamic::from("Playback stopped")),
            Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                e.into(),
                rhai::Position::NONE,
            ))),
            Err(_) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                "STOP timed out".into(),
                rhai::Position::NONE,
            ))),
        }
    })?;
    Ok(())
}

/// PAUSE - Pause current playback
fn pause_keyword(
    state: Arc<AppState>,
    user: UserSession,
    engine: &mut Engine,
) -> Result<(), rhai::ParseError> {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine.register_custom_syntax(&["PAUSE"], false, move |_context, _inputs| {
        trace!("PAUSE playback for session: {}", user_clone.id);

        let state_for_task = Arc::clone(&state_clone);
        let session_id = user_clone.id;

        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
            let result = rt.block_on(async {
                send_player_command(&state_for_task, session_id, "pause").await
            });
            let _ = tx.send(result);
        });

        match rx.recv_timeout(std::time::Duration::from_secs(10)) {
            Ok(Ok(_)) => Ok(Dynamic::from("Playback paused")),
            Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                e.into(),
                rhai::Position::NONE,
            ))),
            Err(_) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                "PAUSE timed out".into(),
                rhai::Position::NONE,
            ))),
        }
    })?;
    Ok(())
}

/// RESUME - Resume paused playback
fn resume_keyword(
    state: Arc<AppState>,
    user: UserSession,
    engine: &mut Engine,
) -> Result<(), rhai::ParseError> {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine.register_custom_syntax(&["RESUME"], false, move |_context, _inputs| {
        trace!("RESUME playback for session: {}", user_clone.id);

        let state_for_task = Arc::clone(&state_clone);
        let session_id = user_clone.id;

        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
            let result = rt.block_on(async {
                send_player_command(&state_for_task, session_id, "resume").await
            });
            let _ = tx.send(result);
        });

        match rx.recv_timeout(std::time::Duration::from_secs(10)) {
            Ok(Ok(_)) => Ok(Dynamic::from("Playback resumed")),
            Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                e.into(),
                rhai::Position::NONE,
            ))),
            Err(_) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                "RESUME timed out".into(),
                rhai::Position::NONE,
            ))),
        }
    })?;
    Ok(())
}

// Core Functions

/// Execute the PLAY command
async fn execute_play(
    state: &AppState,
    session_id: Uuid,
    source: &str,
    options: PlayOptions,
) -> Result<PlayResponse, String> {
    // Detect content type
    let content_type = detect_content_type(source);

    // Resolve source URL
    let source_url = resolve_source_url(state, session_id, source).await?;

    // Get metadata
    let metadata = get_content_metadata(state, &source_url, &content_type).await?;

    // Create player ID
    let player_id = Uuid::new_v4();

    // Get title from source or metadata
    let title = metadata
        .get("title")
        .cloned()
        .unwrap_or_else(|| extract_title_from_source(source));

    // Build response
    let response = PlayResponse {
        player_id,
        content_type: content_type.clone(),
        component: content_type.player_component().to_string(),
        source_url,
        title,
        options,
        metadata,
    };

    // Send to client via WebSocket
    send_play_to_client(state, session_id, &response).await?;

    info!(
        "Playing {:?} content: {} for session {}",
        response.content_type, source, session_id
    );

    Ok(response)
}

/// Detect content type from source
fn detect_content_type(source: &str) -> ContentType {
    // Check if it's a URL
    if source.starts_with("http://") || source.starts_with("https://") {
        // Check for known video platforms
        if source.contains("youtube.com")
            || source.contains("youtu.be")
            || source.contains("vimeo.com")
        {
            return ContentType::Video;
        }

        // Check for known image hosts
        if source.contains("imgur.com")
            || source.contains("unsplash.com")
            || source.contains("pexels.com")
        {
            return ContentType::Image;
        }

        // Try to detect from URL path extension
        if let Some(path) = source.split('?').next() {
            if let Some(ext) = Path::new(path).extension() {
                return ContentType::from_extension(&ext.to_string_lossy());
            }
        }

        // Default to iframe for unknown URLs
        return ContentType::Iframe;
    }

    // Local file - detect from extension
    if let Some(ext) = Path::new(source).extension() {
        return ContentType::from_extension(&ext.to_string_lossy());
    }

    ContentType::Unknown
}

/// Resolve source to a URL
async fn resolve_source_url(
    _state: &AppState,
    session_id: Uuid,
    source: &str,
) -> Result<String, String> {
    // If already a URL, return as-is
    if source.starts_with("http://") || source.starts_with("https://") {
        return Ok(source.to_string());
    }

    // Check if it's a drive path
    if source.starts_with("/") || source.contains(".gbdrive") {
        // Resolve from drive
        let file_url = format!(
            "/api/drive/file/{}?session={}",
            urlencoding::encode(source),
            session_id
        );
        return Ok(file_url);
    }

    // Check if it's a relative path in current bot's folder
    let file_url = format!(
        "/api/drive/file/{}?session={}",
        urlencoding::encode(source),
        session_id
    );

    Ok(file_url)
}

/// Get content metadata
async fn get_content_metadata(
    _state: &AppState,
    source_url: &str,
    content_type: &ContentType,
) -> Result<HashMap<String, String>, String> {
    let mut metadata = HashMap::new();

    metadata.insert("source".to_string(), source_url.to_string());
    metadata.insert("type".to_string(), format!("{:?}", content_type));

    // Add type-specific metadata
    match content_type {
        ContentType::Video => {
            metadata.insert("player".to_string(), "html5".to_string());
        }
        ContentType::Audio => {
            metadata.insert("player".to_string(), "html5".to_string());
        }
        ContentType::Image => {
            metadata.insert("viewer".to_string(), "lightbox".to_string());
        }
        ContentType::Pdf => {
            metadata.insert("viewer".to_string(), "pdfjs".to_string());
        }
        ContentType::Code => {
            metadata.insert("highlighter".to_string(), "prism".to_string());
        }
        ContentType::Presentation => {
            metadata.insert("viewer".to_string(), "revealjs".to_string());
        }
        ContentType::Spreadsheet => {
            metadata.insert("viewer".to_string(), "handsontable".to_string());
        }
        _ => {}
    }

    Ok(metadata)
}

/// Extract title from source path/URL
fn extract_title_from_source(source: &str) -> String {
    // Extract filename from path or URL
    let path = source.split('?').next().unwrap_or(source);

    Path::new(path)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "Untitled".to_string())
}

/// Send play command to client via WebSocket
async fn send_play_to_client(
    state: &AppState,
    session_id: Uuid,
    response: &PlayResponse,
) -> Result<(), String> {
    let message = serde_json::json!({
        "type": "play",
        "data": response
    });

    let message_str =
        serde_json::to_string(&message).map_err(|e| format!("Failed to serialize: {}", e))?;

    // Send via web adapter
    let bot_response = crate::shared::models::BotResponse {
        bot_id: String::new(),
        user_id: String::new(),
        session_id: session_id.to_string(),
        channel: "web".to_string(),
        content: message_str,
        message_type: crate::shared::message_types::MessageType::BOT_RESPONSE,
        stream_token: None,
        is_complete: true,
        suggestions: Vec::new(),
        context_name: None,
        context_length: 0,
        context_max_length: 0,
    };

    state
        .web_adapter
        .send_message_to_session(&session_id.to_string(), bot_response)
        .await
        .map_err(|e| format!("Failed to send to client: {}", e))?;

    Ok(())
}

/// Send player command (stop/pause/resume) to client
async fn send_player_command(
    state: &AppState,
    session_id: Uuid,
    command: &str,
) -> Result<(), String> {
    let message = serde_json::json!({
        "type": "player_command",
        "command": command
    });

    let message_str =
        serde_json::to_string(&message).map_err(|e| format!("Failed to serialize: {}", e))?;

    // Use web adapter to send message
    let _ = state
        .web_adapter
        .send_message_to_session(
            &session_id.to_string(),
            crate::shared::models::BotResponse {
                bot_id: String::new(),
                user_id: String::new(),
                session_id: session_id.to_string(),
                channel: "web".to_string(),
                content: message_str,
                message_type: crate::shared::message_types::MessageType::BOT_RESPONSE,
                stream_token: None,
                is_complete: true,
                suggestions: Vec::new(),
                context_name: None,
                context_length: 0,
                context_max_length: 0,
            },
        )
        .await;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_type_from_extension() {
        assert_eq!(ContentType::from_extension("mp4"), ContentType::Video);
        assert_eq!(ContentType::from_extension("MP3"), ContentType::Audio);
        assert_eq!(ContentType::from_extension("png"), ContentType::Image);
        assert_eq!(ContentType::from_extension("pdf"), ContentType::Pdf);
        assert_eq!(ContentType::from_extension("rs"), ContentType::Code);
        assert_eq!(
            ContentType::from_extension("pptx"),
            ContentType::Presentation
        );
        assert_eq!(
            ContentType::from_extension("xlsx"),
            ContentType::Spreadsheet
        );
        assert_eq!(ContentType::from_extension("md"), ContentType::Markdown);
    }

    #[test]
    fn test_content_type_from_mime() {
        assert_eq!(ContentType::from_mime("video/mp4"), ContentType::Video);
        assert_eq!(ContentType::from_mime("audio/mpeg"), ContentType::Audio);
        assert_eq!(ContentType::from_mime("image/png"), ContentType::Image);
        assert_eq!(ContentType::from_mime("application/pdf"), ContentType::Pdf);
    }

    #[test]
    fn test_play_options_from_string() {
        let opts = PlayOptions::from_string("autoplay,loop,muted");
        assert!(opts.autoplay);
        assert!(opts.loop_content);
        assert!(opts.muted);
        assert!(!opts.fullscreen);
        assert!(opts.controls);

        let opts = PlayOptions::from_string("fullscreen,nocontrols,start=10,end=60");
        assert!(opts.fullscreen);
        assert!(!opts.controls);
        assert_eq!(opts.start_time, Some(10.0));
        assert_eq!(opts.end_time, Some(60.0));

        let opts = PlayOptions::from_string("theme=dark,zoom=1.5,page=3");
        assert_eq!(opts.theme, Some("dark".to_string()));
        assert_eq!(opts.zoom, Some(1.5));
        assert_eq!(opts.page, Some(3));
    }

    #[test]
    fn test_detect_content_type() {
        assert_eq!(
            detect_content_type("https://youtube.com/watch?v=123"),
            ContentType::Video
        );
        assert_eq!(
            detect_content_type("https://example.com/video.mp4"),
            ContentType::Video
        );
        assert_eq!(
            detect_content_type("https://imgur.com/abc123"),
            ContentType::Image
        );
        assert_eq!(
            detect_content_type("presentation.pptx"),
            ContentType::Presentation
        );
        assert_eq!(detect_content_type("report.pdf"), ContentType::Pdf);
        assert_eq!(detect_content_type("main.rs"), ContentType::Code);
    }

    #[test]
    fn test_extract_title_from_source() {
        assert_eq!(extract_title_from_source("documents/report.pdf"), "report");
        assert_eq!(
            extract_title_from_source("https://example.com/video.mp4?token=abc"),
            "video"
        );
        assert_eq!(
            extract_title_from_source("presentation.pptx"),
            "presentation"
        );
    }

    #[test]
    fn test_player_component() {
        assert_eq!(ContentType::Video.player_component(), "video-player");
        assert_eq!(ContentType::Audio.player_component(), "audio-player");
        assert_eq!(ContentType::Image.player_component(), "image-viewer");
        assert_eq!(ContentType::Pdf.player_component(), "pdf-viewer");
        assert_eq!(ContentType::Code.player_component(), "code-viewer");
    }
}
