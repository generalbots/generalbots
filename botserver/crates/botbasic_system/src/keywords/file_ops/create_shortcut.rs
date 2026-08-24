//! CREATE SHORTCUT keyword (#1149): writes a `.shortcut` file into the
//! user's Drive `Desktop/` folder. The desktop shell lists `Desktop/*.shortcut`
//! as icons; double-click deep-links to the target path in Drive.
//!
//! File format (version 1):
//!   { "kind": "gb-shortcut", "version": 1,
//!     "target": { "path": "<drive path>" }, "created_at": "<RFC3339>" }

use botbasic_types::{BasicRuntime, UserSession};
use rhai::{Dynamic, Engine};
use std::sync::Arc;

use crate::keywords::file_ops::basic_io::execute_create_file;

pub fn register_create_shortcut_keyword(
    state: Arc<dyn BasicRuntime>,
    user: UserSession,
    engine: &mut Engine,
) {
    let state_clone = Arc::clone(&state);
    let user_clone = user;

    engine
        .register_custom_syntax(
            ["CREATE", "SHORTCUT", "$expr$", "FOR", "$expr$"],
            false,
            move |context, inputs| {
                let name = context.eval_expression_tree(&inputs[0])?.to_string();
                let target = context.eval_expression_tree(&inputs[1])?.to_string();

                let safe_name: String = name
                    .chars()
                    .map(|c| if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' { c } else { '_' })
                    .collect();
                let file_name = if safe_name.ends_with(".shortcut") {
                    safe_name
                } else {
                    format!("{safe_name}.shortcut")
                };
                let path = format!("Desktop/{file_name}");

                let payload = serde_json::json!({
                    "kind": "gb-shortcut",
                    "version": 1,
                    "target": { "path": target },
                    "created_at": chrono::Utc::now().to_rfc3339(),
                });
                let content = serde_json::to_string_pretty(&payload).unwrap_or_default();

                let state_for_task = Arc::clone(&state_clone);
                let user_for_task = user_clone.clone();
                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();
                    let send_err = if let Ok(rt) = rt {
                        let result = rt.block_on(async move {
                            execute_create_file(&state_for_task, &user_for_task, &path, &content).await
                        });
                        tx.send(result).err()
                    } else {
                        tx.send(Err("Failed to build tokio runtime".into())).err()
                    };
                    if send_err.is_some() {
                        log::error!("Failed to send CREATE SHORTCUT result from thread");
                    }
                });

                match rx.recv_timeout(std::time::Duration::from_secs(30)) {
                    Ok(Ok(_)) => Ok(Dynamic::from(path)),
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("CREATE SHORTCUT failed: {e}").into(),
                        rhai::Position::NONE,
                    ))),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            "CREATE SHORTCUT timed out".into(),
                            rhai::Position::NONE,
                        )))
                    }
                }
            },
        )
        .map_err(|e| log::error!("CREATE SHORTCUT registration failed: {e}"))
        .ok();
}
