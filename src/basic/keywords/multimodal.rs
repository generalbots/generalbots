use crate::multimodal::BotModelsClient;
use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::{error, trace};
use rhai::{Dynamic, Engine};
use std::sync::Arc;
use std::time::Duration;

pub fn register_multimodal_keywords(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    image_keyword(state.clone(), user.clone(), engine);
    video_keyword(state.clone(), user.clone(), engine);
    audio_keyword(state.clone(), user.clone(), engine);
    see_keyword(state, user, engine);
}

pub fn image_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(["IMAGE", "$expr$"], false, move |context, inputs| {
            let prompt = context.eval_expression_tree(&inputs[0])?.to_string();

            trace!("IMAGE keyword: generating image for prompt: {}", prompt);

            let state_for_thread = state.clone();
            let bot_id = user.bot_id;

            let (tx, rx) = std::sync::mpsc::channel();

            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(2)
                    .enable_all()
                    .build();

                let send_err = if let Ok(rt) = rt {
                    let result = rt.block_on(async move {
                        execute_image_generation(state_for_thread, bot_id, prompt).await
                    });
                    tx.send(result).err()
                } else {
                    tx.send(Err("Failed to build tokio runtime".into())).err()
                };

                if send_err.is_some() {
                    error!("Failed to send IMAGE result");
                }
            });

            match rx.recv_timeout(Duration::from_secs(300)) {
                Ok(Ok(result)) => Ok(Dynamic::from(result)),
                Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    e.to_string().into(),
                    rhai::Position::NONE,
                ))),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        "Image generation timed out".into(),
                        rhai::Position::NONE,
                    )))
                }
                Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    format!("IMAGE thread failed: {}", e).into(),
                    rhai::Position::NONE,
                ))),
            }
        })
        .expect("valid syntax registration");
}

async fn execute_image_generation(
    state: Arc<AppState>,
    bot_id: uuid::Uuid,
    prompt: String,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let client = BotModelsClient::from_state(&state, &bot_id);

    if !client.is_enabled() {
        return Err("BotModels is not enabled. Set botmodels-enabled=true in config.csv".into());
    }

    client.generate_image(&prompt).await
}

pub fn video_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(["VIDEO", "$expr$"], false, move |context, inputs| {
            let prompt = context.eval_expression_tree(&inputs[0])?.to_string();

            trace!("VIDEO keyword: generating video for prompt: {}", prompt);

            let state_for_thread = state.clone();
            let bot_id = user.bot_id;

            let (tx, rx) = std::sync::mpsc::channel();

            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(2)
                    .enable_all()
                    .build();

                let send_err = if let Ok(rt) = rt {
                    let result = rt.block_on(async move {
                        execute_video_generation(state_for_thread, bot_id, prompt).await
                    });
                    tx.send(result).err()
                } else {
                    tx.send(Err("Failed to build tokio runtime".into())).err()
                };

                if send_err.is_some() {
                    error!("Failed to send VIDEO result");
                }
            });

            match rx.recv_timeout(Duration::from_secs(600)) {
                Ok(Ok(result)) => Ok(Dynamic::from(result)),
                Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    e.to_string().into(),
                    rhai::Position::NONE,
                ))),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        "Video generation timed out".into(),
                        rhai::Position::NONE,
                    )))
                }
                Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    format!("VIDEO thread failed: {}", e).into(),
                    rhai::Position::NONE,
                ))),
            }
        })
        .expect("valid syntax registration");
}

async fn execute_video_generation(
    state: Arc<AppState>,
    bot_id: uuid::Uuid,
    prompt: String,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let client = BotModelsClient::from_state(&state, &bot_id);

    if !client.is_enabled() {
        return Err("BotModels is not enabled. Set botmodels-enabled=true in config.csv".into());
    }

    client.generate_video(&prompt).await
}

pub fn audio_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(["AUDIO", "$expr$"], false, move |context, inputs| {
            let text = context.eval_expression_tree(&inputs[0])?.to_string();

            trace!("AUDIO keyword: generating speech for text: {}", text);

            let state_for_thread = state.clone();
            let bot_id = user.bot_id;

            let (tx, rx) = std::sync::mpsc::channel();

            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(2)
                    .enable_all()
                    .build();

                let send_err = if let Ok(rt) = rt {
                    let result = rt.block_on(async move {
                        execute_audio_generation(state_for_thread, bot_id, text).await
                    });
                    tx.send(result).err()
                } else {
                    tx.send(Err("Failed to build tokio runtime".into())).err()
                };

                if send_err.is_some() {
                    error!("Failed to send AUDIO result");
                }
            });

            match rx.recv_timeout(Duration::from_secs(120)) {
                Ok(Ok(result)) => Ok(Dynamic::from(result)),
                Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    e.to_string().into(),
                    rhai::Position::NONE,
                ))),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        "Audio generation timed out".into(),
                        rhai::Position::NONE,
                    )))
                }
                Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    format!("AUDIO thread failed: {}", e).into(),
                    rhai::Position::NONE,
                ))),
            }
        })
        .expect("valid syntax registration");
}

async fn execute_audio_generation(
    state: Arc<AppState>,
    bot_id: uuid::Uuid,
    text: String,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let client = BotModelsClient::from_state(&state, &bot_id);

    if !client.is_enabled() {
        return Err("BotModels is not enabled. Set botmodels-enabled=true in config.csv".into());
    }

    client.generate_audio(&text, None, None).await
}

pub fn see_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(["SEE", "$expr$"], false, move |context, inputs| {
            let file_path = context.eval_expression_tree(&inputs[0])?.to_string();

            trace!("SEE keyword: getting caption for file: {}", file_path);

            let state_for_thread = state.clone();
            let bot_id = user.bot_id;

            let (tx, rx) = std::sync::mpsc::channel();

            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(2)
                    .enable_all()
                    .build();

                let send_err = if let Ok(rt) = rt {
                    let result = rt.block_on(async move {
                        execute_see_caption(state_for_thread, bot_id, file_path).await
                    });
                    tx.send(result).err()
                } else {
                    tx.send(Err("Failed to build tokio runtime".into())).err()
                };

                if send_err.is_some() {
                    error!("Failed to send SEE result");
                }
            });

            match rx.recv_timeout(Duration::from_secs(60)) {
                Ok(Ok(result)) => Ok(Dynamic::from(result)),
                Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    e.to_string().into(),
                    rhai::Position::NONE,
                ))),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        "Vision/caption timed out".into(),
                        rhai::Position::NONE,
                    )))
                }
                Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    format!("SEE thread failed: {}", e).into(),
                    rhai::Position::NONE,
                ))),
            }
        })
        .expect("valid syntax registration");
}

async fn execute_see_caption(
    state: Arc<AppState>,
    bot_id: uuid::Uuid,
    file_path: String,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let client = BotModelsClient::from_state(&state, &bot_id);

    if !client.is_enabled() {
        return Err("BotModels is not enabled. Set botmodels-enabled=true in config.csv".into());
    }

    use std::path::Path;
    let path = Path::new(&file_path);
    let is_video = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            let ext_lower = ext.to_lowercase();
            ext_lower == "mp4"
                || ext_lower == "avi"
                || ext_lower == "mov"
                || ext_lower == "webm"
                || ext_lower == "mkv"
        })
        .unwrap_or(false);

    if is_video {
        client.describe_video(&file_path).await
    } else {
        client.describe_image(&file_path).await
    }
}
