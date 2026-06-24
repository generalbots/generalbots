use botmultimodal::{BotModelsClient, ConfigProvider};
use rhai::EvalAltResult;
use uuid::Uuid;

use botbasic_types::BasicRuntime;

pub const DEFAULT_TIMEOUT_SECS: u64 = 120;

/// ConfigProvider implementation that resolves values from a `BasicRuntime`.
///
/// Used to feed `BotModelsClient::from_provider_all` without depending on
/// `botcore::shared::state::AppState`, keeping the keyword registration
/// path inside `botbasic_ai` (which only sees `Arc<dyn BasicRuntime>`).
pub struct RuntimeConfigProvider<'a> {
    pub runtime: &'a dyn BasicRuntime,
    pub bot_id: Uuid,
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

pub fn build_client(runtime: &dyn BasicRuntime, bot_id: Uuid) -> BotModelsClient {
    let provider = RuntimeConfigProvider { runtime, bot_id };
    BotModelsClient::from_provider_all(&provider, &bot_id)
}

pub fn runtime_error(message: impl Into<String>) -> Box<EvalAltResult> {
    Box::new(EvalAltResult::ErrorRuntime(message.into().into(), rhai::Position::NONE))
}

pub fn spawn_multimodal<F>(name: &'static str, fut: F) -> Result<rhai::Dynamic, Box<EvalAltResult>>
where
    F: std::future::Future<Output = Result<String, Box<dyn std::error::Error + Send + Sync>>>
        + Send
        + 'static,
{
    use rhai::Dynamic;

    let (tx, rx) = std::sync::mpsc::channel();
    let join = std::thread::Builder::new()
        .name(name.into())
        .spawn(move || {
            let result = std::thread::Builder::new()
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
            let outcome = match result {
                Ok(handle) => match handle.join() {
                    Ok(res) => res,
                    Err(_) => Err("Multimodal worker thread panicked".into()),
                },
                Err(e) => Err(format!("Failed to spawn multimodal worker: {e}").into()),
            };
            let _ = tx.send(outcome);
        });

    if join.is_err() {
        return Err(runtime_error("Failed to spawn multimodal dispatcher thread"));
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

pub fn eval_string(
    context: &mut rhai::EvalContext,
    input: &rhai::Expression,
) -> Result<String, Box<EvalAltResult>> {
    Ok(context.eval_expression_tree(input)?.to_string())
}
