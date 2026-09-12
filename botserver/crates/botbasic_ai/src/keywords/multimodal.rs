/*****************************************************************************\
|  █████  █████ ██    █ █████ █████   ████  ██      ████   █████ █████  ███ ® |
| ██      █     ███   █ █     ██  ██ ██  ██ ██      ██  █ ██   ██  █   █      |
| ██  ███ ████  █ ██  █ ████  █████  ██████ ██      ████   █   █   █    ██    |
| ██   ██ █     █  ██ █ █     ██  ██ ██  ██ ██      ██  █ ██   ██  █      █   |
|  █████  █████ █   ███ █████ ██  ██ ██  ██ █████   ████   █████   █   ███    |
|                                                                             |
| General Bots Copyright (c) pragmatismo.com.br. All rights reserved.         |
| Licensed under the AGPL-3.0.                                                |
|                                                                             |
| According to our dual licensing model, this program can be used either      |
| under the terms of the GNU Affero General Public License, version 3,        |
| or under a proprietary license.                                             |
|                                                                             |
| The texts of the GNU Affero General Public License with an additional       |
| permission and of our proprietary license can be found at and               |
| in the LICENSE file you have received along with this program.              |
|                                                                             |
| This program is distributed in the hope that it will be useful,             |
| but WITHOUT ANY WARRANTY, without even the implied warranty of              |
| MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the                |
| GNU Affero General Public License for more details.                         |
|                                                                             |
| "General Bots" is a registered trademark of pragmatismo.com.br.             |
| The licensing of the program under the AGPLv3 does not imply a              |
| trademark license. Therefore any rights, title and interest in              |
| our trademarks remain entirely with us.                                     |
|                                                                             |
\*****************************************************************************/

use botbasic_types::{BasicRuntime, UserSession};
use rhai::Engine;
use std::sync::Arc;

use super::multimodal_helpers::{build_client, eval_string, spawn_multimodal};

pub fn register_multimodal_keywords(
    state: Arc<dyn BasicRuntime>,
    user: UserSession,
    engine: &mut Engine,
) {
    register_generate_image(state.clone(), user.clone(), engine);
    register_describe_image(state.clone(), user.clone(), engine);
    register_read_text(state.clone(), user.clone(), engine);
    register_scan_barcode(state.clone(), user.clone(), engine);
    register_detect_objects(state.clone(), user.clone(), engine);
    register_read_plate(state.clone(), user.clone(), engine);
    register_detect_damage(state.clone(), user.clone(), engine);
    register_generate_video(state.clone(), user.clone(), engine);
    register_speech_to_text(state.clone(), user.clone(), engine);
    register_text_to_speech(state.clone(), user.clone(), engine);
    register_compare_images(state.clone(), user.clone(), engine);
    register_classify_image(state.clone(), user.clone(), engine);
    register_detect_defects(state.clone(), user.clone(), engine);
    register_detect_faces(state.clone(), user.clone(), engine);
    register_extract_colors(state.clone(), user.clone(), engine);
    register_assess_image(state.clone(), user.clone(), engine);
    register_analyze_image(state.clone(), user.clone(), engine);
}

fn register_generate_image(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    if let Err(e) = engine.register_custom_syntax(
        ["GENERATE", "IMAGE", "$expr$"],
        false,
        move |context, inputs| {
            let prompt = eval_string(context, &inputs[0])?;
            let runtime = Arc::clone(&state);
            let bot_id = user.bot_id;
            spawn_multimodal("generate-image", async move {
                let client = build_client(runtime.as_ref(), bot_id);
                if !client.is_enabled() {
                    return Err("BotModels is not enabled in bot configuration".into());
                }
                client.generate_image(&prompt).await
            })
        },
    ) {
        log::error!("GENERATE IMAGE registration failed: {e}");
    }
}

fn register_describe_image(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    if let Err(e) = engine.register_custom_syntax(
        ["DESCRIBE", "IMAGE", "$expr$"],
        false,
        move |context, inputs| {
            let source = eval_string(context, &inputs[0])?;
            let runtime = Arc::clone(&state);
            let bot_id = user.bot_id;
            spawn_multimodal("describe-image", async move {
                let client = build_client(runtime.as_ref(), bot_id);
                if !client.is_enabled() {
                    return Err("BotModels is not enabled in bot configuration".into());
                }
                client.describe_image(&source).await
            })
        },
    ) {
        log::error!("DESCRIBE IMAGE registration failed: {e}");
    }
}

fn register_read_text(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    if let Err(e) =
        engine.register_custom_syntax(["READ", "TEXT", "$expr$"], false, move |context, inputs| {
            let source = eval_string(context, &inputs[0])?;
            let runtime = Arc::clone(&state);
            let bot_id = user.bot_id;
            spawn_multimodal("read-text", async move {
                let client = build_client(runtime.as_ref(), bot_id);
                if !client.is_enabled() {
                    return Err("BotModels is not enabled in bot configuration".into());
                }
                client.describe_image(&source).await
            })
        })
    {
        log::error!("READ TEXT registration failed: {e}");
    }
}

fn register_scan_barcode(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    if let Err(e) = engine.register_custom_syntax(
        ["SCAN", "BARCODE", "$expr$"],
        false,
        move |context, inputs| {
            let source = eval_string(context, &inputs[0])?;
            let runtime = Arc::clone(&state);
            let bot_id = user.bot_id;
            spawn_multimodal("scan-barcode", async move {
                let client = build_client(runtime.as_ref(), bot_id);
                if !client.is_enabled() {
                    return Err("BotModels is not enabled in bot configuration".into());
                }
                client.scan_barcode(&source).await
            })
        },
    ) {
        log::error!("SCAN BARCODE registration failed: {e}");
    }
}

fn register_detect_objects(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    if let Err(e) = engine.register_custom_syntax(
        ["DETECT", "OBJECTS", "$expr$"],
        false,
        move |context, inputs| {
            let source = eval_string(context, &inputs[0])?;
            let runtime = Arc::clone(&state);
            let bot_id = user.bot_id;
            spawn_multimodal("detect-objects", async move {
                let client = build_client(runtime.as_ref(), bot_id);
                if !client.is_enabled() {
                    return Err("BotModels is not enabled in bot configuration".into());
                }
                client.describe_image(&source).await
            })
        },
    ) {
        log::error!("DETECT OBJECTS registration failed: {e}");
    }
}

fn register_read_plate(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    if let Err(e) = engine.register_custom_syntax(
        ["READ", "PLATE", "$expr$"],
        false,
        move |context, inputs| {
            let source = eval_string(context, &inputs[0])?;
            let runtime = Arc::clone(&state);
            let bot_id = user.bot_id;
            spawn_multimodal("read-plate", async move {
                let client = build_client(runtime.as_ref(), bot_id);
                if !client.is_enabled() {
                    return Err("BotModels is not enabled in bot configuration".into());
                }
                let raw = client.scan_barcode(&source).await?;
                Ok(format!("plate-scan:{raw}"))
            })
        },
    ) {
        log::error!("READ PLATE registration failed: {e}");
    }
}

fn register_detect_damage(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    if let Err(e) = engine.register_custom_syntax(
        ["DETECT", "DAMAGE", "$expr$"],
        false,
        move |context, inputs| {
            let source = eval_string(context, &inputs[0])?;
            let runtime = Arc::clone(&state);
            let bot_id = user.bot_id;
            spawn_multimodal("detect-damage", async move {
                let client = build_client(runtime.as_ref(), bot_id);
                if !client.is_enabled() {
                    return Err("BotModels is not enabled in bot configuration".into());
                }
                let description = client.describe_image(&source).await?;
                Ok(format!("damage-assessment:{description}"))
            })
        },
    ) {
        log::error!("DETECT DAMAGE registration failed: {e}");
    }
}

fn register_generate_video(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    if let Err(e) = engine.register_custom_syntax(
        ["GENERATE", "VIDEO", "$expr$"],
        false,
        move |context, inputs| {
            let prompt = eval_string(context, &inputs[0])?;
            let runtime = Arc::clone(&state);
            let bot_id = user.bot_id;
            spawn_multimodal("generate-video", async move {
                let client = build_client(runtime.as_ref(), bot_id);
                if !client.is_enabled() {
                    return Err("BotModels is not enabled in bot configuration".into());
                }
                client.generate_video(&prompt).await
            })
        },
    ) {
        log::error!("GENERATE VIDEO registration failed: {e}");
    }
}

fn register_speech_to_text(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    if let Err(e) = engine.register_custom_syntax(
        ["SPEECH", "TO", "TEXT", "$expr$"],
        false,
        move |context, inputs| {
            let source = eval_string(context, &inputs[0])?;
            let runtime = Arc::clone(&state);
            let bot_id = user.bot_id;
            spawn_multimodal("speech-to-text", async move {
                let client = build_client(runtime.as_ref(), bot_id);
                if !client.is_enabled() {
                    return Err("BotModels is not enabled in bot configuration".into());
                }
                client.speech_to_text(&source).await
            })
        },
    ) {
        log::error!("SPEECH TO TEXT registration failed: {e}");
    }
}

fn register_text_to_speech(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    if let Err(e) = engine.register_custom_syntax(
        ["TEXT", "TO", "SPEECH", "$expr$"],
        false,
        move |context, inputs| {
            let text = eval_string(context, &inputs[0])?;
            let runtime = Arc::clone(&state);
            let bot_id = user.bot_id;
            spawn_multimodal("text-to-speech", async move {
                let client = build_client(runtime.as_ref(), bot_id);
                if !client.is_enabled() {
                    return Err("BotModels is not enabled in bot configuration".into());
                }
                client.generate_audio(&text, None, Some("pt-BR")).await
            })
        },
    ) {
        log::error!("TEXT TO SPEECH registration failed: {e}");
    }
}

fn register_analyze_image(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    if let Err(e) = engine.register_custom_syntax(
        ["ANALYZE", "IMAGE", "$expr$"],
        false,
        move |context, inputs| {
            let source = eval_string(context, &inputs[0])?;
            let runtime = Arc::clone(&state);
            let bot_id = user.bot_id;
            spawn_multimodal("analyze-image", async move {
                let client = build_client(runtime.as_ref(), bot_id);
                if !client.is_enabled() {
                    return Err("BotModels is not enabled in bot configuration".into());
                }
                client.describe_image(&source).await
            })
        },
    ) {
        log::error!("ANALYZE IMAGE registration failed: {e}");
    }
}

fn register_compare_images(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    if let Err(e) = engine.register_custom_syntax(
        ["COMPARE", "IMAGES", "$expr$", "WITH", "$expr$"],
        false,
        move |context, inputs| {
            let a = eval_string(context, &inputs[0])?;
            let b = eval_string(context, &inputs[1])?;
            let combined = format!("{a}|{b}");
            let runtime = Arc::clone(&state);
            let bot_id = user.bot_id;
            spawn_multimodal("compare-images", async move {
                let client = build_client(runtime.as_ref(), bot_id);
                if !client.is_enabled() {
                    return Err("BotModels is not enabled in bot configuration".into());
                }
                client.describe_image(&combined).await
            })
        },
    ) {
        log::error!("COMPARE IMAGES registration failed: {e}");
    }
}

fn register_classify_image(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    if let Err(e) = engine.register_custom_syntax(
        ["CLASSIFY", "IMAGE", "$expr$"],
        false,
        move |context, inputs| {
            let source = eval_string(context, &inputs[0])?;
            let runtime = Arc::clone(&state);
            let bot_id = user.bot_id;
            spawn_multimodal("classify-image", async move {
                let client = build_client(runtime.as_ref(), bot_id);
                if !client.is_enabled() {
                    return Err("BotModels is not enabled in bot configuration".into());
                }
                client.describe_image(&source).await
            })
        },
    ) {
        log::error!("CLASSIFY IMAGE registration failed: {e}");
    }
}

fn register_detect_defects(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    if let Err(e) = engine.register_custom_syntax(
        ["DETECT", "DEFECTS", "$expr$"],
        false,
        move |context, inputs| {
            let source = eval_string(context, &inputs[0])?;
            let runtime = Arc::clone(&state);
            let bot_id = user.bot_id;
            spawn_multimodal("detect-defects", async move {
                let client = build_client(runtime.as_ref(), bot_id);
                if !client.is_enabled() {
                    return Err("BotModels is not enabled in bot configuration".into());
                }
                client.describe_image(&source).await
            })
        },
    ) {
        log::error!("DETECT DEFECTS registration failed: {e}");
    }
}

fn register_detect_faces(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    if let Err(e) = engine.register_custom_syntax(
        ["DETECT", "FACES", "$expr$"],
        false,
        move |context, inputs| {
            let source = eval_string(context, &inputs[0])?;
            let runtime = Arc::clone(&state);
            let bot_id = user.bot_id;
            spawn_multimodal("detect-faces", async move {
                let client = build_client(runtime.as_ref(), bot_id);
                if !client.is_enabled() {
                    return Err("BotModels is not enabled in bot configuration".into());
                }
                client.describe_image(&source).await
            })
        },
    ) {
        log::error!("DETECT FACES registration failed: {e}");
    }
}

fn register_extract_colors(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    if let Err(e) = engine.register_custom_syntax(
        ["EXTRACT", "COLORS", "$expr$"],
        false,
        move |context, inputs| {
            let source = eval_string(context, &inputs[0])?;
            let runtime = Arc::clone(&state);
            let bot_id = user.bot_id;
            spawn_multimodal("extract-colors", async move {
                let client = build_client(runtime.as_ref(), bot_id);
                if !client.is_enabled() {
                    return Err("BotModels is not enabled in bot configuration".into());
                }
                client.describe_image(&source).await
            })
        },
    ) {
        log::error!("EXTRACT COLORS registration failed: {e}");
    }
}

fn register_assess_image(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    if let Err(e) = engine.register_custom_syntax(
        ["ASSESS", "IMAGE", "$expr$"],
        false,
        move |context, inputs| {
            let source = eval_string(context, &inputs[0])?;
            let runtime = Arc::clone(&state);
            let bot_id = user.bot_id;
            spawn_multimodal("assess-image", async move {
                let client = build_client(runtime.as_ref(), bot_id);
                if !client.is_enabled() {
                    return Err("BotModels is not enabled in bot configuration".into());
                }
                client.describe_image(&source).await
            })
        },
    ) {
        log::error!("ASSESS IMAGE registration failed: {e}");
    }
}

#[cfg(test)]
#[path = "multimodal_tests.rs"]
mod multimodal_tests;
