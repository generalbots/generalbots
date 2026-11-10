//! Docs verbs whose endpoints are one route per template/export format (#2037).
//!
//! `docs.new` and `docs.export` cannot use the generic verb→endpoint binding:
//! Docs registers a separate route per template and per export format, so the
//! verb's `template`/`format` selects the registered path. Split out of
//! `execute_command.rs` to keep that dispatcher under the 450-line limit.

use super::*;

/// True when `name` is one of the docs verbs this module executes.
pub(super) fn handles(name: &str) -> bool {
    matches!(name, "docs.new" | "docs.export")
}

/// Executes one docs verb against the registered per-format route.
pub(super) async fn execute(
    state: &Arc<AppState>,
    user_id: Uuid,
    bot_uuid: Uuid,
    name: &str,
    params: &serde_json::Map<String, Value>,
) -> Result<Value, String> {
    let obj = params;
    let str_of = |key: &str| obj.get(key).and_then(|v| v.as_str()).map(|s| s.to_string());
    match name {
        // #2037 — Docs publishes one route per template and per export format,
        // so the verb's `template`/`format` selects the registered path instead
        // of inventing a query parameter the handler does not read.
        "docs.new" => {
            let template = str_of("template").unwrap_or_else(|| "blank".to_string());
            let path = crate::apps::commands::docs_template_path(&template.to_lowercase())?;
            exec_docs_path(state, user_id, bot_uuid, path, serde_json::Map::new()).await
        }
        "docs.export" => {
            let file_id =
                str_of("file_id").ok_or_else(|| "params.file_id is required".to_string())?;
            let format = str_of("format").unwrap_or_else(|| "pdf".to_string());
            let path = crate::apps::commands::docs_export_path(&format.to_lowercase())?;
            let mut params = serde_json::Map::new();
            params.insert("id".to_string(), Value::String(file_id));
            exec_docs_path(state, user_id, bot_uuid, path, params).await
        }
        _ => Err(format!("unknown docs command '{name}'")),
    }
}

async fn exec_docs_path(
    state: &Arc<AppState>,
    user_id: Uuid,
    bot_uuid: Uuid,
    path: &str,
    params: serde_json::Map<String, Value>,
) -> Result<Value, String> {
    crate::core::bot::api_exec::exec_endpoint(
        state,
        user_id,
        Some(bot_uuid),
        "GET",
        path,
        &params,
    )
    .await
}
