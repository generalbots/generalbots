use botbasic_types::UserSession;
use botbasic_types::BasicRuntime;
use diesel::prelude::*;
use log::{error, info, trace, warn};
use rhai::{Dynamic, Engine};
pub fn use_tool_keyword(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    if let Err(e) = engine
        .register_custom_syntax(["USE", "TOOL", "$expr$"], false, move |context, inputs| {
            let tool_path = context.eval_expression_tree(&inputs[0])?;
            let tool_path_str = tool_path.to_string().trim_matches('"').to_string();
            trace!(
                "USE TOOL command executed: {} for session: {}",
                tool_path_str,
                user_clone.id
            );
            // Strip {bot_name}.gbdialog/ or .gbdialog/ prefix, and .bas suffix
            let tool_name = if let Some(idx) = tool_path_str.find(".gbdialog/") {
                &tool_path_str[idx + 10..] // Skip past ".gbdialog/"
            } else {
                tool_path_str.as_str()
            }
            .strip_suffix(".bas")
            .unwrap_or(tool_path_str.as_str())
            .to_string();
            if tool_name.is_empty() {
                return Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    "Invalid tool name".into(),
                    rhai::Position::NONE,
                )));
            }
            let (tx, rx) = std::sync::mpsc::channel();
            let state_clone_1 = Arc::clone(&state_clone);
            let user_clone_1 = user_clone.clone();
            let tool_name_1 = tool_name.clone();
            std::thread::spawn(move || {
                let result =
                    associate_tool_with_session(&state_clone_1, &user_clone_1, &tool_name_1);
                let _ = tx.send(result);
            });
            let result = rx
                .recv()
                .unwrap_or(Err("Failed to receive result".to_string()));
            match result {
                Ok(message) => Ok(Dynamic::from(message)),
                Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    e.into(),
                    rhai::Position::NONE,
                ))),
            }
        })
    {
        log::error!("Failed to register the custom syntax: {e}");
    }

    // Register use_tool(tool_name) function for preprocessor compatibility
    let state_clone2 = Arc::clone(&state);
    let user_clone2 = user.clone();

    engine.register_fn("use_tool", move |tool_path: &str| -> Dynamic {
        let tool_path_str = tool_path.to_string();
        trace!(
            "use_tool function called: {} for session: {}",
            tool_path_str,
            user_clone2.id
        );
        // Strip {bot_name}.gbdialog/ or .gbdialog/ prefix, and .bas suffix
        let tool_name = if let Some(idx) = tool_path_str.find(".gbdialog/") {
            &tool_path_str[idx + 10..] // Skip past ".gbdialog/"
        } else {
            &tool_path_str
        }
        .strip_suffix(".bas")
        .unwrap_or_else(|| &tool_path_str)
        .to_string();
        if tool_name.is_empty() {
            return Dynamic::from("ERROR: Invalid tool name");
        }
        let (tx, rx) = std::sync::mpsc::channel();
        let state_clone_2 = Arc::clone(&state_clone2);
        let user_clone_2 = user_clone2.clone();
        let tool_name_2 = tool_name.clone();
        std::thread::spawn(move || {
            let result = associate_tool_with_session(&state_clone_2, &user_clone_2, &tool_name_2);
            let _ = tx.send(result);
        });
        let result = rx
            .recv()
            .unwrap_or(Err("Failed to receive result".to_string()));
        match result {
            Ok(message) => Dynamic::from(message),
            Err(e) => Dynamic::from(format!("ERROR: {}", e)),
        }
    });

    // Register USE_TOOL(tool_name) function (uppercase variant)
    let state_clone3 = Arc::clone(&state);
    let user_clone3 = user;

    engine.register_fn("USE_TOOL", move |tool_path: &str| -> Dynamic {
        let tool_path_str = tool_path.to_string();
        trace!(
            "USE_TOOL function called: {} for session: {}",
            tool_path_str,
            user_clone3.id
        );
        // Strip {bot_name}.gbdialog/ or .gbdialog/ prefix, and .bas suffix
        let tool_name = if let Some(idx) = tool_path_str.find(".gbdialog/") {
            &tool_path_str[idx + 10..] // Skip past ".gbdialog/"
        } else {
            &tool_path_str
        }
        .strip_suffix(".bas")
        .unwrap_or_else(|| &tool_path_str)
        .to_string();
        if tool_name.is_empty() {
            return Dynamic::from("ERROR: Invalid tool name");
        }
        let (tx, rx) = std::sync::mpsc::channel();
        let state_clone_3 = Arc::clone(&state_clone3);
        let user_clone_3 = user_clone3.clone();
        let tool_name_3 = tool_name.clone();
        std::thread::spawn(move || {
            let result = associate_tool_with_session(&state_clone_3, &user_clone_3, &tool_name_3);
            let _ = tx.send(result);
        });
        let result = rx
            .recv()
            .unwrap_or(Err("Failed to receive result".to_string()));
        match result {
            Ok(message) => Dynamic::from(message),
            Err(e) => Dynamic::from(format!("ERROR: {}", e)),
        }
    });
}

/// Locates a tool's manifest inside the materialized work tree.
///
/// The org directory is named after the org id (`<uuid>.gborg`) while this
/// lookup used to assume `{bot}.gborg`, so every `USE TOOL` on a desktop
/// install failed with ".mcp.json not found" and the tool was never associated
/// with the session (the button was registered but did nothing).
///
/// The manifest is written by the compiler; when only the source is present it
/// is generated here so a shipped `.bas` tool is usable on a fresh install.
fn resolve_tool_manifest(
    work_root: &Path,
    bot_name: &str,
    tool_name: &str,
) -> Option<std::path::PathBuf> {
    let dialog_rel = format!("{bot_name}.gbai/{bot_name}.gbdialog");
    let manifest = format!("{tool_name}.mcp.json");
    let source = format!("{tool_name}.bas");

    let mut candidates: Vec<std::path::PathBuf> = Vec::new();
    // Historical layout: the org directory carried the bot name.
    candidates.push(work_root.join(format!("{bot_name}.gborg")).join(&dialog_rel));
    // Actual layout: one directory per org id.
    if let Ok(entries) = std::fs::read_dir(work_root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| name.ends_with(".gborg"))
                    .unwrap_or(false)
            {
                candidates.push(path.join(&dialog_rel));
            }
        }
    }

    for dir in candidates {
        let manifest_path = dir.join(&manifest);
        if manifest_path.exists() {
            return Some(manifest_path);
        }
        let source_path = dir.join(&source);
        if source_path.exists() {
            let generated = serde_json::json!({
                "name": tool_name,
                "description": format!("Tool {tool_name}"),
                "input_schema": { "type": "object", "properties": {}, "required": [] }
            });
            match serde_json::to_string_pretty(&generated) {
                Ok(raw) => {
                    if std::fs::write(&manifest_path, raw).is_ok() {
                        info!(
                            "Generated missing tool manifest {:?} from {}",
                            manifest_path, source
                        );
                        return Some(manifest_path);
                    }
                }
                Err(e) => warn!("could not serialize manifest for {}: {e}", tool_name),
            }
        }
    }
    None
}

fn associate_tool_with_session(
    state: &Arc<dyn BasicRuntime>,
    user: &UserSession,
    tool_name: &str,
) -> Result<String, String> {
    use botschema::session_tool_associations;

    // Check if tool's .mcp.json file exists in work directory
    let work_root = botlib::work_path::get_work_path();

    // Get bot name to construct the path
    let bot_name = get_bot_name_from_id(state, &user.bot_id)?;
    let Some(mcp_path) =
        resolve_tool_manifest(Path::new(&work_root), &bot_name, tool_name)
    else {
        warn!(
            "Tool '{}' .mcp.json file not found under {:?} for bot '{}'",
            tool_name, work_root, bot_name
        );
        return Err(format!(
            "Tool '{}' is not available. .mcp.json file not found.",
            tool_name
        ));
    };

    trace!("Checking for tool .mcp.json at: {:?}", mcp_path);

    info!(
        "Tool '{}' .mcp.json found, proceeding with session association",
        tool_name
    );

    let mut conn = state.db_pool().get().map_err(|e| format!("DB error: {}", e))?;
    let association_id = Uuid::new_v4().to_string();
    let session_id_str = user.id.to_string();
    let added_at = chrono::Utc::now().to_rfc3339();
    let insert_result: Result<usize, diesel::result::Error> =
        diesel::insert_into(session_tool_associations::table)
            .values((
                session_tool_associations::id.eq(&association_id),
                session_tool_associations::session_id.eq(&session_id_str),
                session_tool_associations::tool_name.eq(tool_name),
                session_tool_associations::added_at.eq(&added_at),
            ))
            .on_conflict((
                session_tool_associations::session_id,
                session_tool_associations::tool_name,
            ))
            .do_nothing()
            .execute(&mut *conn);
    match insert_result {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                trace!(
                    "Tool '{}' newly associated with session '{}' (user: {}, bot: {})",
                    tool_name,
                    user.id,
                    user.user_id,
                    user.bot_id
                );
                Ok(format!(
                    "Tool '{}' is now available in this conversation",
                    tool_name
                ))
            } else {
                trace!(
                    "Tool '{}' was already associated with session '{}'",
                    tool_name,
                    user.id
                );
                Ok(format!(
                    "Tool '{}' is already available in this conversation",
                    tool_name
                ))
            }
        }
        Err(e) => {
            error!(
                "Failed to associate tool '{}' with session '{}': {}",
                tool_name, user.id, e
            );
            Err(format!("Failed to add tool to session: {}", e))
        }
    }
}
pub fn get_session_tools(
    conn: &mut PgConnection,
    session_id: &Uuid,
) -> Result<Vec<String>, diesel::result::Error> {
    use botschema::session_tool_associations;
    let session_id_str = session_id.to_string();
    session_tool_associations::table
        .filter(session_tool_associations::session_id.eq(&session_id_str))
        .select(session_tool_associations::tool_name)
        .load::<String>(conn)
}
pub fn clear_session_tools(
    conn: &mut PgConnection,
    session_id: &Uuid,
) -> Result<usize, diesel::result::Error> {
    use botschema::session_tool_associations;
    let session_id_str = session_id.to_string();
    diesel::delete(
        session_tool_associations::table
            .filter(session_tool_associations::session_id.eq(&session_id_str)),
    )
    .execute(conn)
}

fn get_bot_name_from_id(state: &Arc<dyn BasicRuntime>, bot_id: &uuid::Uuid) -> Result<String, String> {
    use botschema::bots;
    let mut conn = state.db_pool().get().map_err(|e| format!("DB error: {}", e))?;
    let bot_name: String = bots::table
        .filter(bots::id.eq(bot_id))
        .select(bots::name)
        .first(&mut *conn)
        .map_err(|e| format!("Failed to get bot name for id {}: {}", bot_id, e))?;
    Ok(bot_name)
}

use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;
