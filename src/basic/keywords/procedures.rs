use crate::core::shared::models::UserSession;
use crate::core::shared::state::AppState;
use botlib::MAX_LOOP_ITERATIONS;
use log::trace;
use rhai::{Dynamic, Engine};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct ProcedureDefinition {
    pub name: String,
    pub params: Vec<String>,
    pub body: String,
    pub is_function: bool,
}

static PROCEDURES: std::sync::LazyLock<Arc<Mutex<HashMap<String, ProcedureDefinition>>>> =
    std::sync::LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));

pub fn register_procedure_keywords(_state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    register_while_wend(engine);
    register_do_loop(engine);
    register_call_keyword(engine);
    register_return_keyword(engine);
}

fn register_while_wend(engine: &mut Engine) {
    engine
        .register_custom_syntax(
            ["WHILE", "$expr$", "$block$", "WEND"],
            true,
            |context, inputs| {
                let condition_expr = &inputs[0];
                let block = &inputs[1];

                let max_iterations = MAX_LOOP_ITERATIONS;
                let mut iterations = 0;

                loop {
                    let condition = context.eval_expression_tree(condition_expr)?;
                    let should_continue = match condition.as_bool() {
                        Ok(b) => b,
                        Err(_) => {
                            if let Ok(n) = condition.as_int() {
                                n != 0
                            } else if let Ok(f) = condition.as_float() {
                                f != 0.0
                            } else if let Ok(s) = condition.clone().into_string() {
                                !s.is_empty() && s.to_lowercase() != "false"
                            } else {
                                !condition.is_unit()
                            }
                        }
                    };

                    if !should_continue {
                        break;
                    }

                    match context.eval_expression_tree(block) {
                        Ok(_) => (),
                        Err(e) => {
                            let err_str = e.to_string();
                            if err_str == "EXIT WHILE" || err_str == "EXIT DO" {
                                break;
                            }
                            return Err(e);
                        }
                    }

                    iterations += 1;
                    if iterations >= max_iterations {
                        return Err(format!(
                            "WHILE loop exceeded maximum iterations ({max_iterations}). Possible infinite loop."
                        )
                        .into());
                    }
                }

                Ok(Dynamic::UNIT)
            },
        )
        .expect("Failed to register WHILE/WEND syntax");

    engine
        .register_custom_syntax(["EXIT", "WHILE"], false, |_context, _inputs| {
            Err("EXIT WHILE".into())
        })
        .expect("Failed to register EXIT WHILE syntax");
}

fn register_do_loop(engine: &mut Engine) {
    engine
        .register_custom_syntax(
            ["DO", "WHILE", "$expr$", "$block$", "LOOP"],
            true,
            |context, inputs| {
                let condition_expr = &inputs[0];
                let block = &inputs[1];

                let max_iterations = MAX_LOOP_ITERATIONS;
                let mut iterations = 0;

                loop {
                    let condition = context.eval_expression_tree(condition_expr)?;
                    let should_continue = eval_bool_condition(&condition);

                    if !should_continue {
                        break;
                    }

                    match context.eval_expression_tree(block) {
                        Ok(_) => (),
                        Err(e) if e.to_string() == "EXIT DO" => break,
                        Err(e) => return Err(e),
                    }

                    iterations += 1;
                    if iterations >= max_iterations {
                        return Err("DO WHILE loop exceeded maximum iterations".into());
                    }
                }

                Ok(Dynamic::UNIT)
            },
        )
        .expect("Failed to register DO WHILE syntax");

    engine
        .register_custom_syntax(
            ["DO", "UNTIL", "$expr$", "$block$", "LOOP"],
            true,
            |context, inputs| {
                let condition_expr = &inputs[0];
                let block = &inputs[1];

                let max_iterations = MAX_LOOP_ITERATIONS;
                let mut iterations = 0;

                loop {
                    let condition = context.eval_expression_tree(condition_expr)?;
                    let should_stop = eval_bool_condition(&condition);

                    if should_stop {
                        break;
                    }

                    match context.eval_expression_tree(block) {
                        Ok(_) => (),
                        Err(e) if e.to_string() == "EXIT DO" => break,
                        Err(e) => return Err(e),
                    }

                    iterations += 1;
                    if iterations >= max_iterations {
                        return Err("DO UNTIL loop exceeded maximum iterations".into());
                    }
                }

                Ok(Dynamic::UNIT)
            },
        )
        .expect("Failed to register DO UNTIL syntax");

    engine
        .register_custom_syntax(
            ["DO", "$block$", "LOOP", "WHILE", "$expr$"],
            true,
            |context, inputs| {
                let block = &inputs[0];
                let condition_expr = &inputs[1];

                let max_iterations = MAX_LOOP_ITERATIONS;
                let mut iterations = 0;

                loop {
                    match context.eval_expression_tree(block) {
                        Ok(_) => (),
                        Err(e) if e.to_string() == "EXIT DO" => break,
                        Err(e) => return Err(e),
                    }

                    let condition = context.eval_expression_tree(condition_expr)?;
                    let should_continue = eval_bool_condition(&condition);

                    if !should_continue {
                        break;
                    }

                    iterations += 1;
                    if iterations >= max_iterations {
                        return Err("DO...LOOP WHILE exceeded maximum iterations".into());
                    }
                }

                Ok(Dynamic::UNIT)
            },
        )
        .expect("Failed to register DO...LOOP WHILE syntax");

    engine
        .register_custom_syntax(
            ["DO", "$block$", "LOOP", "UNTIL", "$expr$"],
            true,
            |context, inputs| {
                let block = &inputs[0];
                let condition_expr = &inputs[1];

                let max_iterations = MAX_LOOP_ITERATIONS;
                let mut iterations = 0;

                loop {
                    match context.eval_expression_tree(block) {
                        Ok(_) => (),
                        Err(e) if e.to_string() == "EXIT DO" => break,
                        Err(e) => return Err(e),
                    }

                    let condition = context.eval_expression_tree(condition_expr)?;
                    let should_stop = eval_bool_condition(&condition);

                    if should_stop {
                        break;
                    }

                    iterations += 1;
                    if iterations >= max_iterations {
                        return Err("DO...LOOP UNTIL exceeded maximum iterations".into());
                    }
                }

                Ok(Dynamic::UNIT)
            },
        )
        .expect("Failed to register DO...LOOP UNTIL syntax");

    engine
        .register_custom_syntax(["EXIT", "DO"], false, |_context, _inputs| {
            Err("EXIT DO".into())
        })
        .expect("Failed to register EXIT DO syntax");
}

fn eval_bool_condition(value: &Dynamic) -> bool {
    match value.as_bool() {
        Ok(b) => b,
        Err(_) => {
            if let Ok(n) = value.as_int() {
                n != 0
            } else if let Ok(f) = value.as_float() {
                f != 0.0
            } else if let Ok(s) = value.clone().into_string() {
                !s.is_empty() && s.to_lowercase() != "false" && s != "0"
            } else {
                !value.is_unit()
            }
        }
    }
}

fn register_call_keyword(engine: &mut Engine) {
    engine
        .register_custom_syntax(
            ["CALL", "$ident$", "(", "$expr$", ")"],
            false,
            |context, inputs| {
                let proc_name = inputs[0]
                    .get_string_value()
                    .unwrap_or_default()
                    .to_uppercase();
                let args = context.eval_expression_tree(&inputs[1])?;

                trace!("CALL {} with args: {:?}", proc_name, args);

                let procedures = PROCEDURES.lock().expect("mutex not poisoned");
                if let Some(proc) = procedures.get(&proc_name) {
                    trace!(
                        "Found procedure: {} (is_function: {})",
                        proc.name,
                        proc.is_function
                    );

                    Ok(Dynamic::UNIT)
                } else {
                    Err(format!("Undefined procedure: {}", proc_name).into())
                }
            },
        )
        .expect("Failed to register CALL with args syntax");

    engine
        .register_custom_syntax(["CALL", "$ident$"], false, |_context, inputs| {
            let proc_name = inputs[0]
                .get_string_value()
                .unwrap_or_default()
                .to_uppercase();

            trace!("CALL {} (no args)", proc_name);

            let procedures = PROCEDURES.lock().expect("mutex not poisoned");
            if procedures.contains_key(&proc_name) {
                Ok(Dynamic::UNIT)
            } else {
                Err(format!("Undefined procedure: {}", proc_name).into())
            }
        })
        .expect("Failed to register CALL without args syntax");
}

fn register_return_keyword(engine: &mut Engine) {
    engine
        .register_custom_syntax(["RETURN", "$expr$"], false, |context, inputs| {
            let value = context.eval_expression_tree(&inputs[0])?;
            trace!("RETURN with value: {:?}", value);

            Err(format!("RETURN:{}", value).into())
        })
        .expect("Failed to register RETURN with value syntax");

    engine
        .register_custom_syntax(["RETURN"], false, |_context, _inputs| {
            trace!("RETURN (no value)");
            Err("RETURN:".into())
        })
        .expect("Failed to register RETURN syntax");
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
            PROCEDURES.lock().expect("mutex not poisoned").insert(sub_name.clone(), proc);

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
            PROCEDURES.lock().expect("mutex not poisoned").insert(func_name.clone(), proc);

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

pub fn preprocess_calls(input: &str) -> String {
    let mut result = String::new();
    let lines: Vec<&str> = input.lines().collect();

    for line in lines {
        let trimmed = line.trim();
        let upper_line = trimmed.to_uppercase();

        if upper_line.starts_with("CALL ") {
            let rest = trimmed[5..].trim();
            let (proc_name, args) = if let Some(paren_start) = rest.find('(') {
                let name = rest[..paren_start].trim().to_uppercase();
                let args_str = if let Some(paren_end) = rest.find(')') {
                    rest[paren_start + 1..paren_end].to_string()
                } else {
                    String::new()
                };
                (name, args_str)
            } else {
                (rest.to_uppercase(), String::new())
            };

            let procedures = PROCEDURES.lock().expect("mutex not poisoned");
            if let Some(proc) = procedures.get(&proc_name) {
                let arg_values: Vec<&str> = if args.is_empty() {
                    Vec::new()
                } else {
                    args.split(',').map(|a| a.trim()).collect()
                };

                result.push_str("// Begin inlined CALL ");
                result.push_str(&proc_name);
                result.push('\n');

                for (i, param) in proc.params.iter().enumerate() {
                    if i < arg_values.len() && !arg_values[i].is_empty() {
                        use std::fmt::Write;
                        let _ = writeln!(result, "let {} = {};", param, arg_values[i]);
                    }
                }

                result.push_str(&proc.body);
                result.push_str("// End inlined CALL ");
                result.push_str(&proc_name);
            } else {
                result.push_str(line);
            }
        } else {
            result.push_str(line);
        }
        result.push('\n');
    }

    result
}

pub fn preprocess_procedures(input: &str) -> String {
    let after_subs = preprocess_subs(input);

    let after_functions = preprocess_functions(&after_subs);

    preprocess_calls(&after_functions)
}

pub fn clear_procedures() {
    PROCEDURES.lock().expect("mutex not poisoned").clear();
}

pub fn get_procedure_names() -> Vec<String> {
    PROCEDURES.lock().expect("mutex not poisoned").keys().cloned().collect()
}

pub fn has_procedure(name: &str) -> bool {
    PROCEDURES
        .lock()
        .expect("mutex not poisoned")
        .contains_key(&name.to_uppercase())
}

pub fn get_procedure(name: &str) -> Option<ProcedureDefinition> {
    PROCEDURES
        .lock()
        .expect("mutex not poisoned")
        .get(&name.to_uppercase())
        .cloned()
}
