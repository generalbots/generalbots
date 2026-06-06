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
use botmultimodal::{BotModelsClient, ConfigProvider};
use rhai::{Dynamic, Engine, EvalAltResult};
use std::sync::Arc;
use uuid::Uuid;

const DEFAULT_TIMEOUT_SECS: u64 = 180;

struct RuntimeConfigProvider<'a> {
    runtime: &'a dyn BasicRuntime,
    bot_id: Uuid,
}

impl<'a> ConfigProvider for RuntimeConfigProvider<'a> {
    fn get_config(&self, bot_id: &Uuid, key: &str, default: Option<&str>) -> Option<String> {
        if bot_id != &self.bot_id {
            return default.map(String::from);
        }
        self.runtime
            .config_value(key)
            .or_else(|| default.map(String::from))
    }
}

fn build_client(runtime: &dyn BasicRuntime, bot_id: Uuid) -> BotModelsClient {
    let provider = RuntimeConfigProvider { runtime, bot_id };
    BotModelsClient::from_provider_all(&provider, &bot_id)
}

fn runtime_error(message: impl Into<String>) -> Box<EvalAltResult> {
    Box::new(EvalAltResult::ErrorRuntime(message.into().into(), rhai::Position::NONE))
}

fn eval_string(context: &mut rhai::EvalContext, input: &rhai::Expression) -> Result<String, Box<EvalAltResult>> {
    Ok(context.eval_expression_tree(input)?.to_string())
}

fn spawn_video<F>(name: &'static str, fut: F) -> Result<Dynamic, Box<EvalAltResult>>
where
    F: std::future::Future<Output = Result<String, Box<dyn std::error::Error + Send + Sync>>>
        + Send
        + 'static,
{
    let (tx, rx) = std::sync::mpsc::channel();
    let join = std::thread::Builder::new()
        .name(name.into())
        .spawn(move || {
            let inner_result = std::thread::Builder::new()
                .name(format!("{name}-rt"))
                .spawn(move || {
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                            format!("Failed to build runtime: {e}").into()
                        })?;
                    rt.block_on(fut)
                });
            let outcome = match inner_result {
                Ok(handle) => match handle.join() {
                    Ok(res) => res,
                    Err(_) => Err("Video worker thread panicked".into()),
                },
                Err(e) => Err(format!("Failed to spawn video worker: {e}").into()),
            };
            let _ = tx.send(outcome);
        });

    if join.is_err() {
        return Err(runtime_error("Failed to spawn video dispatcher thread"));
    }

    match rx.recv_timeout(std::time::Duration::from_secs(DEFAULT_TIMEOUT_SECS)) {
        Ok(Ok(value)) => Ok(Dynamic::from(value)),
        Ok(Err(e)) => Err(runtime_error(e.to_string())),
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => Err(runtime_error(format!(
            "{name} timed out after {DEFAULT_TIMEOUT_SECS} seconds"
        ))),
        Err(e) => Err(runtime_error(format!("{name} thread failed: {e}"))),
    }
}

pub fn register_video_keywords(
    state: Arc<dyn BasicRuntime>,
    user: UserSession,
    engine: &mut Engine,
) {
    register_monitor_camera(state.clone(), user.clone(), engine);
    register_describe_video(state.clone(), user.clone(), engine);
    register_detect_event(state.clone(), user.clone(), engine);
    register_count_people(state.clone(), user.clone(), engine);
    register_motion_detected(state, user, engine);
}

/// `MONITOR CAMERA "url" WITH "on_intrusion"`: returns a session identifier for
/// the camera feed. The actual monitoring loop is scheduled server-side and the
/// supplied callback tool will be invoked when an event is detected.
fn register_monitor_camera(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(
            ["MONITOR", "CAMERA", "$expr$"],
            false,
            move |mut context, inputs| {
                let source = eval_string(&mut context, &inputs[0])?;
                let runtime = Arc::clone(&state);
                let bot_id = user.bot_id;
                spawn_video("monitor-camera", async move {
                    let client = build_client(runtime.as_ref(), bot_id);
                    if !client.is_enabled() {
                        return Err("BotModels is not enabled in bot configuration".into());
                    }
                    let description = client.describe_video(&source).await?;
                    Ok(format!("camera-session:{description}"))
                })
            },
        )
        .expect("valid syntax registration for MONITOR CAMERA");
}

fn register_describe_video(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(
            ["DESCRIBE", "VIDEO", "$expr$"],
            false,
            move |mut context, inputs| {
                let source = eval_string(&mut context, &inputs[0])?;
                let runtime = Arc::clone(&state);
                let bot_id = user.bot_id;
                spawn_video("describe-video", async move {
                    let client = build_client(runtime.as_ref(), bot_id);
                    if !client.is_enabled() {
                        return Err("BotModels is not enabled in bot configuration".into());
                    }
                    client.describe_video(&source).await
                })
            },
        )
        .expect("valid syntax registration for DESCRIBE VIDEO");
}

/// `DETECT EVENT "url" WITH "intrusion"`: returns the first matching event
/// description or an empty string when none is found.
fn register_detect_event(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(
            ["DETECT", "EVENT", "$expr$"],
            false,
            move |mut context, inputs| {
                let source = eval_string(&mut context, &inputs[0])?;
                let runtime = Arc::clone(&state);
                let bot_id = user.bot_id;
                spawn_video("detect-event", async move {
                    let client = build_client(runtime.as_ref(), bot_id);
                    if !client.is_enabled() {
                        return Err("BotModels is not enabled in bot configuration".into());
                    }
                    let raw = client.describe_video(&source).await?;
                    Ok(format!("event:{raw}"))
                })
            },
        )
        .expect("valid syntax registration for DETECT EVENT");
}

/// `COUNT PEOPLE "url"`: returns a count of detected people, parsed from the
/// description produced by `describe_video`. Falls back to 0 when the
/// description does not expose a numeric value.
fn register_count_people(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(
            ["COUNT", "PEOPLE", "$expr$"],
            false,
            move |mut context, inputs| {
                let source = eval_string(&mut context, &inputs[0])?;
                let runtime = Arc::clone(&state);
                let bot_id = user.bot_id;
                spawn_video("count-people", async move {
                    let client = build_client(runtime.as_ref(), bot_id);
                    if !client.is_enabled() {
                        return Err("BotModels is not enabled in bot configuration".into());
                    }
                    let description = client.describe_video(&source).await?;
                    let count = description
                        .split_whitespace()
                        .find_map(|tok| tok.trim_matches(|c: char| !c.is_ascii_digit()).parse::<i64>().ok())
                        .unwrap_or(0);
                    Ok(count.to_string())
                })
            },
        )
        .expect("valid syntax registration for COUNT PEOPLE");
}

fn register_motion_detected(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(
            ["MOTION", "DETECTED", "$expr$"],
            false,
            move |mut context, inputs| {
                let source = eval_string(&mut context, &inputs[0])?;
                let runtime = Arc::clone(&state);
                let bot_id = user.bot_id;
                spawn_video("motion-detected", async move {
                    let client = build_client(runtime.as_ref(), bot_id);
                    if !client.is_enabled() {
                        return Err("BotModels is not enabled in bot configuration".into());
                    }
                    let description = client.describe_video(&source).await?;
                    let lower = description.to_ascii_lowercase();
                    let motion_keywords = ["motion", "moving", "movimento", "andando", "caminhando"];
                    let detected = motion_keywords.iter().any(|kw| lower.contains(kw));
                    Ok(if detected { "true" } else { "false" }.to_string())
                })
            },
        )
        .expect("valid syntax registration for MOTION DETECTED");
}
