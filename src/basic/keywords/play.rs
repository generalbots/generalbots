use crate::core::shared::models::UserSession;
use crate::core::shared::state::AppState;
use log::{info, trace};
use rhai::{Dynamic, Engine};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "mp4" | "webm" | "ogg" | "mov" | "avi" | "mkv" | "m4v" => Self::Video,

            "mp3" | "wav" | "flac" | "aac" | "m4a" | "wma" => Self::Audio,

            "jpg" | "jpeg" | "png" | "gif" | "webp" | "svg" | "bmp" | "ico" => Self::Image,

            "pptx" | "ppt" | "odp" | "key" => Self::Presentation,

            "docx" | "doc" | "odt" | "rtf" => Self::Document,

            "xlsx" | "xls" | "csv" | "ods" => Self::Spreadsheet,

            "pdf" => Self::Pdf,

            "rs" | "py" | "js" | "ts" | "java" | "c" | "cpp" | "h" | "go" | "rb" | "php"
            | "swift" | "kt" | "scala" | "r" | "sql" | "sh" | "bash" | "zsh" | "ps1" | "yaml"
            | "yml" | "toml" | "json" | "xml" | "bas" | "basic" => Self::Code,

            "md" | "markdown" => Self::Markdown,

            "html" | "htm" => Self::Html,
            _ => Self::Unknown,
        }
    }

    pub fn from_mime(mime: &str) -> Self {
        if mime.starts_with("video/") {
            Self::Video
        } else if mime.starts_with("audio/") {
            Self::Audio
        } else if mime.starts_with("image/") {
            Self::Image
        } else if mime == "application/pdf" {
            Self::Pdf
        } else if mime.contains("presentation") || mime.contains("powerpoint") {
            Self::Presentation
        } else if mime.contains("spreadsheet") || mime.contains("excel") {
            Self::Spreadsheet
        } else if mime.contains("document") || mime.contains("word") {
            Self::Document
        } else if mime.starts_with("text/") {
            if mime.contains("html") {
                Self::Html
            } else if mime.contains("markdown") {
                Self::Markdown
            } else {
                Self::Code
            }
        } else {
            Self::Unknown
        }
    }

    pub fn player_component(&self) -> &'static str {
        match self {
            Self::Video => "video-player",
            Self::Audio => "audio-player",
            Self::Image => "image-viewer",
            Self::Presentation => "presentation-viewer",
            Self::Document => "document-viewer",
            Self::Code => "code-viewer",
            Self::Spreadsheet => "spreadsheet-viewer",
            Self::Pdf => "pdf-viewer",
            Self::Markdown => "markdown-viewer",
            Self::Html => "html-viewer",
            Self::Iframe => "iframe-viewer",
            Self::Unknown => "generic-viewer",
        }
    }
}

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
    pub fn from_string(options_str: &str) -> Self {
        let mut opts = Self {
            controls: true,
            ..Self::default()
        };

        for opt in options_str.split(',').map(|s| s.trim().to_lowercase()) {
            match opt.as_str() {
                "autoplay" => opts.autoplay = true,
                "loop" => opts.loop_content = true,
                "fullscreen" => opts.fullscreen = true,
                "muted" => opts.muted = true,
                "nocontrols" => opts.controls = false,
                "linenumbers" => opts.line_numbers = Some(true),
                _ => {
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

pub fn play_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    if let Err(e) = play_simple_keyword(Arc::clone(&state), user.clone(), engine) {
        log::error!("Failed to register PLAY keyword: {e}");
    }
    if let Err(e) = play_with_options_keyword(Arc::clone(&state), user.clone(), engine) {
        log::error!("Failed to register PLAY WITH OPTIONS keyword: {e}");
    }
    if let Err(e) = stop_keyword(Arc::clone(&state), user.clone(), engine) {
        log::error!("Failed to register STOP keyword: {e}");
    }
    if let Err(e) = pause_keyword(Arc::clone(&state), user.clone(), engine) {
        log::error!("Failed to register PAUSE keyword: {e}");
    }
    if let Err(e) = resume_keyword(state, user, engine) {
        log::error!("Failed to register RESUME keyword: {e}");
    }
}

fn play_simple_keyword(
    state: Arc<AppState>,
    user: UserSession,
    engine: &mut Engine,
) -> Result<(), rhai::ParseError> {
    let state_clone = Arc::clone(&state);
    let user_clone = user;

    engine.register_custom_syntax(["PLAY", "$expr$"], false, move |context, inputs| {
        let source = context
            .eval_expression_tree(&inputs[0])?
            .to_string()
            .trim_matches('"')
            .to_string();

        trace!("PLAY '{source}' for session: {}", user_clone.id);

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

fn play_with_options_keyword(
    state: Arc<AppState>,
    user: UserSession,
    engine: &mut Engine,
) -> Result<(), rhai::ParseError> {
    let state_clone = Arc::clone(&state);
    let user_clone = user;

    engine.register_custom_syntax(
        ["PLAY", "$expr$", "WITH", "OPTIONS", "$expr$"],
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
                "PLAY '{source}' WITH OPTIONS '{options_str}' for session: {}",
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

fn stop_keyword(
    state: Arc<AppState>,
    user: UserSession,
    engine: &mut Engine,
) -> Result<(), rhai::ParseError> {
    let state_clone = Arc::clone(&state);
    let user_clone = user;

    engine.register_custom_syntax(["STOP"], false, move |_context, _inputs| {
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

fn pause_keyword(
    state: Arc<AppState>,
    user: UserSession,
    engine: &mut Engine,
) -> Result<(), rhai::ParseError> {
    let state_clone = Arc::clone(&state);
    let user_clone = user;

    engine.register_custom_syntax(["PAUSE"], false, move |_context, _inputs| {
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

fn resume_keyword(
    state: Arc<AppState>,
    user: UserSession,
    engine: &mut Engine,
) -> Result<(), rhai::ParseError> {
    let state_clone = Arc::clone(&state);
    let user_clone = user;

    engine.register_custom_syntax(["RESUME"], false, move |_context, _inputs| {
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

async fn execute_play(
    state: &AppState,
    session_id: Uuid,
    source: &str,
    options: PlayOptions,
) -> Result<PlayResponse, String> {
    let content_type = detect_content_type(source);

    let source_url = resolve_source_url(state, session_id, source)?;

    let metadata = get_content_metadata(state, &source_url, &content_type);

    let player_id = Uuid::new_v4();

    let title = metadata
        .get("title")
        .cloned()
        .unwrap_or_else(|| extract_title_from_source(source));

    let response = PlayResponse {
        player_id,
        content_type: content_type.clone(),
        component: content_type.player_component().to_string(),
        source_url,
        title,
        options,
        metadata,
    };

    send_play_to_client(state, session_id, &response).await?;

    info!(
        "Playing {:?} content: {source} for session {session_id}",
        response.content_type
    );

    Ok(response)
}

pub fn detect_content_type(source: &str) -> ContentType {
    if source.starts_with("http://") || source.starts_with("https://") {
        if source.contains("youtube.com")
            || source.contains("youtu.be")
            || source.contains("vimeo.com")
        {
            return ContentType::Video;
        }

        if source.contains("imgur.com")
            || source.contains("unsplash.com")
            || source.contains("pexels.com")
        {
            return ContentType::Image;
        }

        if let Some(path) = source.split('?').next() {
            if let Some(ext) = Path::new(path).extension() {
                return ContentType::from_extension(&ext.to_string_lossy());
            }
        }

        return ContentType::Iframe;
    }

    if let Some(ext) = Path::new(source).extension() {
        return ContentType::from_extension(&ext.to_string_lossy());
    }

    ContentType::Unknown
}

fn resolve_source_url(_state: &AppState, session_id: Uuid, source: &str) -> Result<String, String> {
    if source.starts_with("http://") || source.starts_with("https://") {
        return Ok(source.to_string());
    }

    if source.starts_with('/') || source.contains(".gbdrive") {
        let file_url = format!(
            "/api/drive/file/{}?session={session_id}",
            urlencoding::encode(source)
        );
        return Ok(file_url);
    }

    let file_url = format!(
        "/api/drive/file/{}?session={session_id}",
        urlencoding::encode(source)
    );

    Ok(file_url)
}

fn get_content_metadata(
    _state: &AppState,
    source_url: &str,
    content_type: &ContentType,
) -> HashMap<String, String> {
    let mut metadata = HashMap::new();

    metadata.insert("source".to_string(), source_url.to_string());
    metadata.insert("type".to_string(), format!("{content_type:?}"));

    match content_type {
        ContentType::Video | ContentType::Audio => {
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

    metadata
}

pub fn extract_title_from_source(source: &str) -> String {
    let path = source.split('?').next().unwrap_or(source);

    Path::new(path)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "Untitled".to_string())
}

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
        serde_json::to_string(&message).map_err(|e| format!("Failed to serialize: {e}"))?;

    let bot_response = crate::core::shared::models::BotResponse {
        bot_id: String::new(),
        user_id: String::new(),
        session_id: session_id.to_string(),
        channel: "web".to_string(),
        content: message_str,
        message_type: crate::core::shared::message_types::MessageType::BOT_RESPONSE,
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
        .map_err(|e| format!("Failed to send to client: {e}"))?;

    Ok(())
}

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
        serde_json::to_string(&message).map_err(|e| format!("Failed to serialize: {e}"))?;

    let _ = state
        .web_adapter
        .send_message_to_session(
            &session_id.to_string(),
            crate::core::shared::models::BotResponse {
                bot_id: String::new(),
                user_id: String::new(),
                session_id: session_id.to_string(),
                channel: "web".to_string(),
                content: message_str,
                message_type: crate::core::shared::message_types::MessageType::BOT_RESPONSE,
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
