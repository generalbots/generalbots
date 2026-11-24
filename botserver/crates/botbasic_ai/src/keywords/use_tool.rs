use botbasic_types::UserSession;
use botbasic_types::BasicRuntime;
use diesel::prelude::*;
use log::{error, info, trace, warn};
use rhai::{Dynamic, Engine};

/// #2685 — a single, safe path component.
///
/// Bot/tool names are interpolated into work-tree paths. Anything that is not
/// a plain component (traversal, separators, control characters, a leading dot
/// or a doubled dot) must be rejected: the old code fell back to the original
/// operand, so a crafted name such as `bot.gbdialog/../../secret` survived.
fn is_safe_name_component(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 255
        && !name.starts_with('.')
        && !name.contains("..")
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

/// Normalizes a `USE TOOL` operand into a bare tool name.
///
/// The `.bas` suffix is stripped from the *sliced* operand, never from the
/// original string — returning the original was the fallback that let unsafe
/// names through. `None` means "reject"; callers must not fabricate a name.
fn normalize_tool_name(raw: &str) -> Option<&str> {
    let trimmed = raw.trim();
    let after_prefix = match trimmed.find(".gbdialog/") {
        Some(idx) => &trimmed[idx + ".gbdialog/".len()..],
        None => trimmed,
    };
    let name = after_prefix.strip_suffix(".bas").unwrap_or(after_prefix);
    is_safe_name_component(name).then_some(name)
}

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
            let Some(tool_name) = normalize_tool_name(&tool_path_str) else {
                return Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    "Invalid tool name: expected a single component (letters, digits, '-', '_' or '.')"
                        .into(),
                    rhai::Position::NONE,
                )));
            };
            let tool_name = tool_name.to_string();
            let (tx, rx) = std::sync::mpsc::channel();
            let state_clone_1 = Arc::clone(&state_clone);
            let user_clone_1 = user_clone.clone();
            let tool_name_1 = tool_name.clone();
            std::thread::spawn(move || {
                let result =
                    associate_tool_with_session(&state_clone_1, &user_clone_1, &tool_name_1);
                if let Err(e) = tx.send(result) {
                    log::warn!("USE_TOOL keyword: result channel send failed: {e}");
                }
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
        let Some(tool_name) = normalize_tool_name(&tool_path_str) else {
            return Dynamic::from("ERROR: Invalid tool name");
        };
        let tool_name = tool_name.to_string();
        let (tx, rx) = std::sync::mpsc::channel();
        let state_clone_2 = Arc::clone(&state_clone2);
        let user_clone_2 = user_clone2.clone();
        let tool_name_2 = tool_name.clone();
        std::thread::spawn(move || {
            let result = associate_tool_with_session(&state_clone_2, &user_clone_2, &tool_name_2);
            if let Err(e) = tx.send(result) {
                log::warn!("USE_TOOL keyword: result channel send failed: {e}");
            }
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
        let Some(tool_name) = normalize_tool_name(&tool_path_str) else {
            return Dynamic::from("ERROR: Invalid tool name");
        };
        let tool_name = tool_name.to_string();
        let (tx, rx) = std::sync::mpsc::channel();
        let state_clone_3 = Arc::clone(&state_clone3);
        let user_clone_3 = user_clone3.clone();
        let tool_name_3 = tool_name.clone();
        std::thread::spawn(move || {
            let result = associate_tool_with_session(&state_clone_3, &user_clone_3, &tool_name_3);
            if let Err(e) = tx.send(result) {
                log::warn!("USE_TOOL keyword: result channel send failed: {e}");
            }
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
/// The org directory is named after the org id (`<uuid>.gborg`) while older
/// installs used `{bot}.gborg`. The lookup used to scan *every* `.gborg` tree
/// and return the first hit; with colliding bot names across tenants that hit
/// could belong to another org. Resolution is now ordered and deterministic:
/// the session's org directory, then the historical name directory. The
/// last-resort scan is sorted and only accepted when it yields one match.
///
/// The manifest is written by the compiler; when only the source is present it
/// is generated here so a shipped `.bas` tool is usable on a fresh install.
fn resolve_tool_manifest(
    work_root: &Path,
    org_id: Option<Uuid>,
    bot_name: &str,
    tool_name: &str,
) -> Option<std::path::PathBuf> {
    if !is_safe_name_component(bot_name) || !is_safe_name_component(tool_name) {
        warn!("Refusing tool lookup for an unsafe bot/tool name");
        return None;
    }
    let dialog_rel = format!("{bot_name}.gbai/{bot_name}.gbdialog");
    let manifest = format!("{tool_name}.mcp.json");
    let source = format!("{tool_name}.bas");

    let mut dirs: Vec<std::path::PathBuf> = Vec::new();
    if let Some(org_id) = org_id {
        dirs.push(work_root.join(format!("{org_id}.gborg")).join(&dialog_rel));
    }
    dirs.push(work_root.join(format!("{bot_name}.gborg")).join(&dialog_rel));

    for dir in dirs {
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

    // Last resort only: `read_dir` order is arbitrary, so sort and require a
    // unique answer rather than silently picking another tenant's script.
    let mut candidates: Vec<std::path::PathBuf> = Vec::new();
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
                candidates.push(path.join(&dialog_rel).join(&manifest));
            }
        }
    }
    candidates.sort();
    candidates.dedup();
    let matches: Vec<_> = candidates.into_iter().filter(|path| path.exists()).collect();
    match matches.len() {
        1 => matches.into_iter().next(),
        0 => None,
        _ => {
            warn!(
                "Ambiguous tool '{}' for bot '{}': {} manifests; refusing to choose",
                tool_name,
                bot_name,
                matches.len()
            );
            None
        }
    }
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
        resolve_tool_manifest(Path::new(&work_root), user.org_id(), &bot_name, tool_name)
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

#[cfg(test)]
mod tests {
    use super::{is_safe_name_component, normalize_tool_name};

    #[test]
    fn normalizes_prefix_and_suffix_from_the_sliced_operand() {
        assert_eq!(normalize_tool_name("foo"), Some("foo"));
        assert_eq!(normalize_tool_name("foo.bas"), Some("foo"));
        assert_eq!(normalize_tool_name("bot.gbdialog/foo.bas"), Some("foo"));
        // The old `.unwrap_or(tool_path_str)` fallback returned this whole
        // string when `.bas` was absent — the unsafe name that reached disk.
        assert_eq!(normalize_tool_name("bot.gbdialog/foo"), Some("foo"));
        assert_eq!(normalize_tool_name(".gbdialog/foo.bas"), Some("foo"));
        assert_eq!(normalize_tool_name("  foo.bas  "), Some("foo"));
    }

    #[test]
    fn rejects_traversal_and_odd_names() {
        let too_long = "x".repeat(256);
        for raw in [
            "",
            " ",
            ".",
            "..",
            ".hidden",
            "foo..bar",
            "../secret",
            "bot.gbdialog/../../secret.bas",
            "bot.gbdialog/../secret",
            "foo/bar",
            "foo\\bar",
            "foo bar",
            "foo\nbas",
            &too_long,
        ] {
            assert_eq!(normalize_tool_name(raw), None, "accepted {raw:?}");
            assert!(!is_safe_name_component(raw), "accepted {raw:?}");
        }
    }

    #[test]
    fn issue_2664_full_path_never_returns_the_original_operand() {
        // #2664: the old `.unwrap_or(tool_path_str)` fallback returned the
        // *whole* operand when `.bas` was absent, so a full path reached the
        // manifest/script joins. Every full-path spelling must collapse to
        // the bare leaf, never to the operand or any slice containing a
        // separator or the `.gbdialog` prefix.
        for raw in [
            "bot.gbdialog/foo.bas",
            "bot.gbdialog/foo",
            "work/bot.gbai/bot.gbdialog/foo.bas",
            "work/bot.gbai/bot.gbdialog/foo",
        ] {
            let normalized = normalize_tool_name(raw);
            assert_eq!(normalized, Some("foo"), "wrong normalization of {raw:?}");
            if let Some(name) = normalized {
                assert!(!name.contains('/'), "separator leaked: {raw:?}");
                assert!(!name.contains('\\'), "separator leaked: {raw:?}");
                assert!(!name.contains(".gbdialog"), "prefix leaked: {raw:?}");
            }
        }

        // Slicing at the first prefix must still fail closed when what remains
        // is a nested prefix, a separator or an empty leaf: the guard rejects
        // these rather than fabricating a name from the unsafe remainder.
        for raw in [
            "bot.gbdialog/a.gbdialog/foo.bas",
            "bot.gbdialog/foo/bar.bas",
            "bot.gbdialog/.hidden.bas",
            "bot.gbdialog/foo..bar.bas",
            "bot.gbdialog/",
        ] {
            assert_eq!(normalize_tool_name(raw), None, "accepted {raw:?}");
        }
    }
}
