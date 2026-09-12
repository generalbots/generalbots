use botmultimodal::{BotModelsClient, ConfigProvider};
use diesel::prelude::*;
use rhai::EvalAltResult;
use std::path::PathBuf;
use uuid::Uuid;

use botbasic_types::schema::bots::dsl as bots_dsl;
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

/// A multimodal `source` resolved to something `BotModelsClient` can read.
///
/// `reference()` is either the original `http(s)` URL or a local filesystem
/// path; when the source was a Drive object the bytes are staged in the system
/// temporary directory and removed by [`ResolvedMedia::cleanup`].
pub struct ResolvedMedia {
    reference: String,
    staged: Option<PathBuf>,
}

impl ResolvedMedia {
    pub fn reference(&self) -> &str {
        &self.reference
    }

    /// Remove the temporary copy staged from Drive, if one was created.
    pub fn cleanup(self) {
        if let Some(path) = self.staged {
            if let Err(e) = std::fs::remove_file(&path) {
                log::debug!("failed to remove staged media {}: {e}", path.display());
            }
        }
    }
}

/// Resolve a multimodal `source` argument for a bot.
///
/// Contract, kept identical to the `GET` keyword and every other `.gbdrive`
/// file keyword (see `botbasic_data::keywords::get::get_from_bucket`):
///
/// * `http(s)://…` — returned unchanged; BotModels fetches the URL itself.
/// * any other string — a Drive-relative path inside the bot's `.gbdrive`
///   (`{bot}.gbai/{bot}.gbdrive/{path}`), matching the Telegram inbound stager.
///   The object is downloaded and staged in the system temp directory.
/// * when the object is absent from Drive (or Drive is not configured) the raw
///   string is returned so a script that staged a local file still works;
///   BotModels then reports the read failure and the caller degrades via
///   `ON ERROR`.
pub async fn resolve_media_source(
    runtime: &dyn BasicRuntime,
    bot_id: Uuid,
    source: &str,
) -> Result<ResolvedMedia, Box<dyn std::error::Error + Send + Sync>> {
    if source.starts_with("http://") || source.starts_with("https://") {
        return Ok(ResolvedMedia {
            reference: source.to_string(),
            staged: None,
        });
    }

    let Some(drive_repo) = runtime.drive_repository() else {
        return Ok(ResolvedMedia {
            reference: source.to_string(),
            staged: None,
        });
    };

    let bot_name: String = {
        let mut conn = runtime
            .db_pool()
            .get()
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                format!("DB error resolving bot name: {e}").into()
            })?;
        bots_dsl::bots
            .filter(bots_dsl::id.eq(&bot_id))
            .select(bots_dsl::name)
            .first(&mut *conn)
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                format!("Failed to resolve bot name for {bot_id}: {e}").into()
            })?
    };

    let bucket = format!("{bot_name}.gbai");
    let prefix = format!("{bot_name}.gbdrive/");
    let object_key = if source.starts_with(&prefix) {
        source.to_string()
    } else if let Some(stripped) = source.strip_prefix("gbdrive/") {
        format!("{prefix}{stripped}")
    } else {
        format!("{prefix}{source}")
    };

    let bytes = match drive_repo.get_object(&bucket, &object_key).await {
        Ok(bytes) => bytes,
        Err(e) => {
            log::debug!(
                "multimodal source '{source}' not found in Drive ({bucket}/{object_key}): {e}"
            );
            return Ok(ResolvedMedia {
                reference: source.to_string(),
                staged: None,
            });
        }
    };

    let path = stage_in_temp(&object_key, bytes)?;
    Ok(ResolvedMedia {
        reference: path.to_string_lossy().into_owned(),
        staged: Some(path),
    })
}

fn stage_in_temp(
    object_key: &str,
    bytes: Vec<u8>,
) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let extension = std::path::Path::new(object_key)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("bin");
    let path = std::env::temp_dir().join(format!("gbmedia-{}.{extension}", Uuid::new_v4()));
    std::fs::write(&path, bytes)?;
    Ok(path)
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
