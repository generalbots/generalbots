use super::*;

pub(super) fn register_call_keyword(state: &Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(state);
    let user_clone = user.clone();

    if let Err(e) = engine
        .register_custom_syntax(
            ["CALL", "$ident$", "(", "$expr$", ")"],
            false,
            move |context, inputs| {
                let proc_name = inputs[0]
                    .get_string_value()
                    .unwrap_or_default()
                    .to_uppercase();
                let _args = context.eval_expression_tree(&inputs[1])?;

                trace!("CALL {} with args", proc_name);

                // Check for in-memory procedure first
                {
                    let procedures = PROCEDURES.lock().unwrap_or_else(|p| p.into_inner());
                    if procedures.contains_key(&proc_name) {
                        return Ok(Dynamic::UNIT);
                    }
                }

                // Try to execute as .bas file
                call_bas_script(&state_clone, &user_clone, &proc_name)
            },
        )
    {
        log::error!("Failed to register CALL with args syntax: {e}");
    }

    let state_clone2 = Arc::clone(state);
    let user_clone2 = user.clone();

    if let Err(e) = engine
        .register_custom_syntax(["CALL", "$ident$"], false, move |_context, inputs| {
            let proc_name = inputs[0]
                .get_string_value()
                .unwrap_or_default()
                .to_uppercase();

            trace!("CALL {} (no args)", proc_name);

            // Check for in-memory procedure first
            {
                let procedures = PROCEDURES.lock().unwrap_or_else(|p| p.into_inner());
                if procedures.contains_key(&proc_name) {
                    return Ok(Dynamic::UNIT);
                }
            }

            // Try to execute as .bas file
            call_bas_script(&state_clone2, &user_clone2, &proc_name)
        })
    {
        log::error!("Failed to register CALL without args syntax: {e}");
    }
}
/// Validate a BASIC script name before it is used to build a filesystem path.
///
/// Only ASCII alphanumeric characters, `_`, `-` and `/` are allowed. Traversal
/// (`..`), absolute paths (POSIX or Windows drive/UNC) and NUL bytes are
/// rejected. `/` is permitted so bots can organize tools in subdirectories,
/// but every component must be non-empty.
fn is_safe_script_name(name: &str) -> bool {
    if name.is_empty() || name.contains('\0') || name.contains("..") {
        return false;
    }
    if name.starts_with('/') || name.starts_with('\\') || name.contains('\\') {
        return false;
    }
    let bytes = name.as_bytes();
    if bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
        return false;
    }
    name.bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-' || byte == b'/')
        && name.split('/').all(|component| !component.is_empty())
}

pub(super) fn call_bas_script(state: &Arc<dyn BasicRuntime>, user: &UserSession, script_name: &str) -> Result<Dynamic, Box<rhai::EvalAltResult>> {
    if !is_safe_script_name(script_name) {
        return Err(format!("Invalid script name: {}", script_name).into());
    }

    // Get bot name from bot_id
    let bot_name = {
        if let Ok(mut conn) = state.db_pool().get() {
            use botbasic_types::schema::bots::dsl::*;
            bots.filter(id.eq(user.bot_id))
                .select(name)
                .first::<String>(&mut *conn)
                .unwrap_or_else(|_| "default".to_string())
        } else {
            "default".to_string()
        }
    };

    let work_path = std::env::var("GBO_WORK_PATH").unwrap_or_else(|_| "/opt/gbo/work".to_string());
    let dialog_root = PathBuf::from(&work_path)
        .join(format!("{}.gbai", bot_name))
        .join(format!("{}.gbdialog", bot_name));
    let script_path = dialog_root.join(format!("{}.bas", script_name.to_lowercase()));
    if !script_path.starts_with(&dialog_root) {
        return Err(format!("Invalid script path for: {}", script_name).into());
    }

    if !script_path.exists() {
        return Err(format!("Undefined procedure/script: {}", script_name).into());
    }

    let script_content = match std::fs::read_to_string(&script_path) {
        Ok(c) => c,
        Err(e) => return Err(format!("Failed to read script {}: {}", script_name, e).into()),
    };

    trace!("Executing .bas script: {:?}", script_path);

    // Clone necessary data for thread
    let state_clone = state.clone();
    let user_clone = user.clone();

    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let result = state_clone.execute_script(
            user_clone,
            &script_content,
        );
        let _ = tx.send(result);
    });

    match rx.recv_timeout(std::time::Duration::from_secs(60)) {
        Ok(Ok(result)) => Ok(Dynamic::from(result)),
        Ok(Err(e)) => Err(format!("Script error: {}", e).into()),
        Err(_) => Err("Script execution timed out".into()),
    }
}
pub(super) fn register_return_keyword(engine: &mut Engine) {
    if let Err(e) = engine
        .register_custom_syntax(["RETURN", "$expr$"], false, |context, inputs| {
            let value = context.eval_expression_tree(&inputs[0])?;
            trace!("RETURN with value: {:?}", value);

            Err(format!("RETURN:{}", value).into())
        })
    {
        log::error!("Failed to register RETURN with value syntax: {e}");
    }

    if let Err(e) = engine
        .register_custom_syntax(["RETURN"], false, |_context, _inputs| {
            trace!("RETURN (no value)");
            Err("RETURN:".into())
        })
    {
        log::error!("Failed to register RETURN syntax: {e}");
    }
}
pub fn preprocess_subs(input: &str) -> String {
    let mut result = String::new();
    let lines: Vec<&str> = input.lines().collect();
    let mut i = 0;
    let mut in_sub = false;
    let mut sub_name = String::new();
    let mut sub_params: Vec<String> = Vec::new();
    let mut sub_body = String::new();

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();
        let upper_line = trimmed.to_uppercase();

        if upper_line.starts_with("SUB ") && !in_sub {
            in_sub = true;

            let rest = trimmed[4..].trim();
            if let Some(paren_start) = rest.find('(') {
                sub_name = rest[..paren_start].trim().to_uppercase();
                if let Some(paren_end) = rest.find(')') {
                    let params_str = &rest[paren_start + 1..paren_end];
                    sub_params = params_str
                        .split(',')
                        .map(|p| p.trim().to_string())
                        .filter(|p| !p.is_empty())
                        .collect();
                }
            } else {
                sub_name = rest.to_uppercase();
                sub_params.clear();
            }

            sub_body.clear();
            trace!("Found SUB: {} with params: {:?}", sub_name, sub_params);
        } else if upper_line == "END SUB" && in_sub {
            in_sub = false;

            let proc = ProcedureDefinition {
                name: sub_name.clone(),
                params: sub_params.clone(),
                body: sub_body.clone(),
                is_function: false,
            };

            trace!("Registering SUB: {}", sub_name);
            PROCEDURES
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .insert(sub_name.clone(), proc);

            sub_name.clear();
            sub_params.clear();
            sub_body.clear();
        } else if in_sub {
            sub_body.push_str(trimmed);
            sub_body.push('\n');
        } else {
            result.push_str(line);
            result.push('\n');
        }

        i += 1;
    }

    if in_sub {
        trace!("Warning: Unclosed SUB {}", sub_name);
        result.push_str(&sub_body);
    }

    result
}
pub fn preprocess_functions(input: &str) -> String {
    let mut result = String::new();
    let lines: Vec<&str> = input.lines().collect();
    let mut i = 0;
    let mut in_function = false;
    let mut func_name = String::new();
    let mut func_params: Vec<String> = Vec::new();
    let mut func_body = String::new();

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();
        let upper_line = trimmed.to_uppercase();

        if upper_line.starts_with("FUNCTION ") && !in_function {
            in_function = true;

            let rest = trimmed[9..].trim();
            if let Some(paren_start) = rest.find('(') {
                func_name = rest[..paren_start].trim().to_uppercase();
                if let Some(paren_end) = rest.find(')') {
                    let params_str = &rest[paren_start + 1..paren_end];
                    func_params = params_str
                        .split(',')
                        .map(|p| p.trim().to_string())
                        .filter(|p| !p.is_empty())
                        .collect();
                }
            } else {
                func_name = rest.to_uppercase();
                func_params.clear();
            }

            func_body.clear();
            trace!(
                "Found FUNCTION: {} with params: {:?}",
                func_name,
                func_params
            );
        } else if upper_line == "END FUNCTION" && in_function {
            in_function = false;

            let proc = ProcedureDefinition {
                name: func_name.clone(),
                params: func_params.clone(),
                body: func_body.clone(),
                is_function: true,
            };

            trace!("Registering FUNCTION: {}", func_name);
            PROCEDURES
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .insert(func_name.clone(), proc);

            func_name.clear();
            func_params.clear();
            func_body.clear();
        } else if in_function {
            func_body.push_str(trimmed);
            func_body.push('\n');
        } else {
            result.push_str(line);
            result.push('\n');
        }

        i += 1;
    }

    if in_function {
        trace!("Warning: Unclosed FUNCTION {}", func_name);
        result.push_str(&func_body);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::is_safe_script_name;

    #[test]
    fn safe_script_names_are_accepted() {
        for name in ["start", "classify_media", "my-tool", "tools/report", "a/b/c"] {
            assert!(is_safe_script_name(name), "{name} must be accepted");
        }
    }

    #[test]
    fn traversal_payloads_are_rejected() {
        for name in [
            "..",
            "../start",
            "foo/../bar",
            "tools/../../etc/passwd",
            "..\\..\\windows\\system32",
            "foo..bar",
        ] {
            assert!(!is_safe_script_name(name), "{name} must be rejected");
        }
    }

    #[test]
    fn absolute_paths_are_rejected() {
        for name in [
            "/etc/passwd",
            "//server/share/script",
            "C:\\Windows\\system32\\cmd",
            "C:/Windows/cmd",
        ] {
            assert!(!is_safe_script_name(name), "{name} must be rejected");
        }
    }

    #[test]
    fn injection_payloads_are_rejected() {
        for name in [
            "start.bas; rm -rf /",
            "start\u{0000}.bas",
            "start\nCALL evil",
            "$(whoami)",
            "start|id",
            "foo bar",
            "foo.bar",
            "",
        ] {
            assert!(!is_safe_script_name(name), "{name} must be rejected");
        }
    }
}
