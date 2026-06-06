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
    engine
        .register_custom_syntax(
            ["GENERATE", "IMAGE", "$expr$"],
            false,
            move |context, inputs| {
                let prompt = eval_string(&context, &inputs[0])?;
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
        )
        .expect("valid syntax registration for GENERATE IMAGE");
}

fn register_describe_image(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(
            ["DESCRIBE", "IMAGE", "$expr$"],
            false,
            move |context, inputs| {
                let source = eval_string(&context, &inputs[0])?;
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
        )
        .expect("valid syntax registration for DESCRIBE IMAGE");
}

fn register_read_text(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(
            ["READ", "TEXT", "$expr$"],
            false,
            move |context, inputs| {
                let source = eval_string(&context, &inputs[0])?;
                let runtime = Arc::clone(&state);
                let bot_id = user.bot_id;
                spawn_multimodal("read-text", async move {
                    let client = build_client(runtime.as_ref(), bot_id);
                    if !client.is_enabled() {
                        return Err("BotModels is not enabled in bot configuration".into());
                    }
                    client.describe_image(&source).await
                })
            },
        )
        .expect("valid syntax registration for READ TEXT");
}

fn register_scan_barcode(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(
            ["SCAN", "BARCODE", "$expr$"],
            false,
            move |context, inputs| {
                let source = eval_string(&context, &inputs[0])?;
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
        )
        .expect("valid syntax registration for SCAN BARCODE");
}

fn register_detect_objects(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(
            ["DETECT", "OBJECTS", "$expr$"],
            false,
            move |context, inputs| {
                let source = eval_string(&context, &inputs[0])?;
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
        )
        .expect("valid syntax registration for DETECT OBJECTS");
}

fn register_read_plate(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(
            ["READ", "PLATE", "$expr$"],
            false,
            move |context, inputs| {
                let source = eval_string(&context, &inputs[0])?;
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
        )
        .expect("valid syntax registration for READ PLATE");
}

fn register_detect_damage(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(
            ["DETECT", "DAMAGE", "$expr$"],
            false,
            move |context, inputs| {
                let source = eval_string(&context, &inputs[0])?;
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
        )
        .expect("valid syntax registration for DETECT DAMAGE");
}

fn register_generate_video(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(
            ["GENERATE", "VIDEO", "$expr$"],
            false,
            move |context, inputs| {
                let prompt = eval_string(&context, &inputs[0])?;
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
        )
        .expect("valid syntax registration for GENERATE VIDEO");
}

fn register_speech_to_text(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(
            ["SPEECH", "TO", "TEXT", "$expr$"],
            false,
            move |context, inputs| {
                let source = eval_string(&context, &inputs[0])?;
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
        )
        .expect("valid syntax registration for SPEECH TO TEXT");
}

fn register_text_to_speech(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(
            ["TEXT", "TO", "SPEECH", "$expr$"],
            false,
            move |context, inputs| {
                let text = eval_string(&context, &inputs[0])?;
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
        )
        .expect("valid syntax registration for TEXT TO SPEECH");
}

fn register_analyze_image(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(
            ["ANALYZE", "IMAGE", "$expr$"],
            false,
            move |context, inputs| {
                let source = eval_string(&context, &inputs[0])?;
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
        )
        .expect("valid syntax registration for ANALYZE IMAGE");
}

fn register_compare_images(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(
            ["COMPARE", "IMAGES", "$expr$", "WITH", "$expr$"],
            false,
            move |context, inputs| {
                let a = eval_string(&context, &inputs[0])?;
                let b = eval_string(&context, &inputs[1])?;
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
        )
        .expect("valid syntax registration for COMPARE IMAGES");
}

fn register_classify_image(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(
            ["CLASSIFY", "IMAGE", "$expr$"],
            false,
            move |context, inputs| {
                let source = eval_string(&context, &inputs[0])?;
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
        )
        .expect("valid syntax registration for CLASSIFY IMAGE");
}

fn register_detect_defects(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(
            ["DETECT", "DEFECTS", "$expr$"],
            false,
            move |context, inputs| {
                let source = eval_string(&context, &inputs[0])?;
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
        )
        .expect("valid syntax registration for DETECT DEFECTS");
}

fn register_detect_faces(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(
            ["DETECT", "FACES", "$expr$"],
            false,
            move |context, inputs| {
                let source = eval_string(&context, &inputs[0])?;
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
        )
        .expect("valid syntax registration for DETECT FACES");
}

fn register_extract_colors(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(
            ["EXTRACT", "COLORS", "$expr$"],
            false,
            move |context, inputs| {
                let source = eval_string(&context, &inputs[0])?;
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
        )
        .expect("valid syntax registration for EXTRACT COLORS");
}

fn register_assess_image(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(
            ["ASSESS", "IMAGE", "$expr$"],
            false,
            move |context, inputs| {
                let source = eval_string(&context, &inputs[0])?;
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
        )
        .expect("valid syntax registration for ASSESS IMAGE");
}

#[cfg(test)]
#[path = "multimodal_tests.rs"]
mod multimodal_tests;
