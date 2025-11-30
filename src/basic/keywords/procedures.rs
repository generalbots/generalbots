//! Procedure and control flow keywords for BASIC interpreter
//!
//! This module provides SUB, FUNCTION, CALL, WHILE/WEND, DO/LOOP, and RETURN keywords
//! for structured programming in General Bots BASIC.

use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::trace;
use once_cell::sync::Lazy;
use rhai::{Dynamic, Engine};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Storage for defined procedures (SUB and FUNCTION)
#[derive(Clone, Debug)]
pub struct ProcedureDefinition {
    pub name: String,
    pub params: Vec<String>,
    pub body: String,
    pub is_function: bool, // true = FUNCTION (returns value), false = SUB (no return)
}

/// Global procedure registry
static PROCEDURES: Lazy<Arc<Mutex<HashMap<String, ProcedureDefinition>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

/// Register all procedure and control flow keywords
pub fn register_procedure_keywords(_state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    register_while_wend(engine);
    register_do_loop(engine);
    register_call_keyword(engine);
    register_return_keyword(engine);
}

/// Register WHILE/WEND loop construct
///
/// Syntax:
/// ```basic
/// WHILE condition
///     ' statements
/// WEND
/// ```
fn register_while_wend(engine: &mut Engine) {
    engine
        .register_custom_syntax(
            &["WHILE", "$expr$", "$block$", "WEND"],
            true,
            |context, inputs| {
                let condition_expr = &inputs[0];
                let block = &inputs[1];

                let max_iterations = 100_000; // Safety limit
                let mut iterations = 0;

                loop {
                    // Evaluate condition
                    let condition = context.eval_expression_tree(condition_expr)?;
                    let should_continue = match condition.as_bool() {
                        Ok(b) => b,
                        Err(_) => {
                            // Try to convert to bool: non-zero numbers are true, non-empty strings are true
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

                    // Execute block
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
                            "WHILE loop exceeded maximum iterations ({}). Possible infinite loop.",
                            max_iterations
                        )
                        .into());
                    }
                }

                Ok(Dynamic::UNIT)
            },
        )
        .expect("Failed to register WHILE/WEND syntax");

    // EXIT WHILE keyword
    engine
        .register_custom_syntax(&["EXIT", "WHILE"], false, |_context, _inputs| {
            Err("EXIT WHILE".into())
        })
        .expect("Failed to register EXIT WHILE syntax");
}

/// Register DO/LOOP constructs
///
/// Syntax variants:
/// ```basic
/// DO WHILE condition
///     ' statements
/// LOOP
///
/// DO
///     ' statements
/// LOOP WHILE condition
///
/// DO UNTIL condition
///     ' statements
/// LOOP
///
/// DO
///     ' statements
/// LOOP UNTIL condition
/// ```
fn register_do_loop(engine: &mut Engine) {
    // DO WHILE ... LOOP (condition at start)
    engine
        .register_custom_syntax(
            &["DO", "WHILE", "$expr$", "$block$", "LOOP"],
            true,
            |context, inputs| {
                let condition_expr = &inputs[0];
                let block = &inputs[1];

                let max_iterations = 100_000;
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

    // DO UNTIL ... LOOP (condition at start, inverted)
    engine
        .register_custom_syntax(
            &["DO", "UNTIL", "$expr$", "$block$", "LOOP"],
            true,
            |context, inputs| {
                let condition_expr = &inputs[0];
                let block = &inputs[1];

                let max_iterations = 100_000;
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

    // DO ... LOOP WHILE (condition at end - always executes at least once)
    engine
        .register_custom_syntax(
            &["DO", "$block$", "LOOP", "WHILE", "$expr$"],
            true,
            |context, inputs| {
                let block = &inputs[0];
                let condition_expr = &inputs[1];

                let max_iterations = 100_000;
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

    // DO ... LOOP UNTIL (condition at end, inverted)
    engine
        .register_custom_syntax(
            &["DO", "$block$", "LOOP", "UNTIL", "$expr$"],
            true,
            |context, inputs| {
                let block = &inputs[0];
                let condition_expr = &inputs[1];

                let max_iterations = 100_000;
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

    // EXIT DO keyword
    engine
        .register_custom_syntax(&["EXIT", "DO"], false, |_context, _inputs| {
            Err("EXIT DO".into())
        })
        .expect("Failed to register EXIT DO syntax");
}

/// Helper function to evaluate a Dynamic value as a boolean
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

/// Register CALL keyword for invoking procedures
///
/// Syntax:
/// ```basic
/// CALL procedure_name(arg1, arg2, ...)
/// CALL procedure_name
/// ```
fn register_call_keyword(engine: &mut Engine) {
    // CALL with parentheses and arguments
    engine
        .register_custom_syntax(
            &["CALL", "$ident$", "(", "$expr$", ")"],
            false,
            |context, inputs| {
                let proc_name = inputs[0]
                    .get_string_value()
                    .unwrap_or_default()
                    .to_uppercase();
                let args = context.eval_expression_tree(&inputs[1])?;

                trace!("CALL {} with args: {:?}", proc_name, args);

                let procedures = PROCEDURES.lock().unwrap();
                if let Some(proc) = procedures.get(&proc_name) {
                    trace!(
                        "Found procedure: {} (is_function: {})",
                        proc.name,
                        proc.is_function
                    );
                    // Note: Actual execution happens through preprocessing
                    // This runtime check ensures the procedure exists
                    Ok(Dynamic::UNIT)
                } else {
                    Err(format!("Undefined procedure: {}", proc_name).into())
                }
            },
        )
        .expect("Failed to register CALL with args syntax");

    // CALL without arguments
    engine
        .register_custom_syntax(&["CALL", "$ident$"], false, |_context, inputs| {
            let proc_name = inputs[0]
                .get_string_value()
                .unwrap_or_default()
                .to_uppercase();

            trace!("CALL {} (no args)", proc_name);

            let procedures = PROCEDURES.lock().unwrap();
            if procedures.contains_key(&proc_name) {
                Ok(Dynamic::UNIT)
            } else {
                Err(format!("Undefined procedure: {}", proc_name).into())
            }
        })
        .expect("Failed to register CALL without args syntax");
}

/// Register RETURN keyword for functions
///
/// Syntax:
/// ```basic
/// RETURN expression
/// RETURN
/// ```
fn register_return_keyword(engine: &mut Engine) {
    // RETURN with value
    engine
        .register_custom_syntax(&["RETURN", "$expr$"], false, |context, inputs| {
            let value = context.eval_expression_tree(&inputs[0])?;
            trace!("RETURN with value: {:?}", value);
            // Store return value and signal return
            Err(format!("RETURN:{}", value).into())
        })
        .expect("Failed to register RETURN with value syntax");

    // RETURN without value (for SUB)
    engine
        .register_custom_syntax(&["RETURN"], false, |_context, _inputs| {
            trace!("RETURN (no value)");
            Err("RETURN:".into())
        })
        .expect("Failed to register RETURN syntax");
}

// ============================================================================
// PREPROCESSING FUNCTIONS
// These run at compile time to extract SUB/FUNCTION definitions
// ============================================================================

/// Preprocess SUB definitions from source code
/// Converts SUB/END SUB blocks into callable units
///
/// Syntax:
/// ```basic
/// SUB MyProcedure(param1, param2)
///     ' statements
/// END SUB
/// ```
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

            // Parse SUB name and parameters
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

            // Register the procedure
            let proc = ProcedureDefinition {
                name: sub_name.clone(),
                params: sub_params.clone(),
                body: sub_body.clone(),
                is_function: false,
            };

            trace!("Registering SUB: {}", sub_name);
            PROCEDURES.lock().unwrap().insert(sub_name.clone(), proc);

            // Don't output SUB definition to main code
            sub_name.clear();
            sub_params.clear();
            sub_body.clear();
        } else if in_sub {
            // Collect SUB body
            sub_body.push_str(trimmed);
            sub_body.push('\n');
        } else {
            // Regular code - pass through with original indentation
            result.push_str(line);
            result.push('\n');
        }

        i += 1;
    }

    if in_sub {
        // Unclosed SUB - emit warning and include remaining code
        trace!("Warning: Unclosed SUB {}", sub_name);
        result.push_str(&sub_body);
    }

    result
}

/// Preprocess FUNCTION definitions from source code
/// Converts FUNCTION/END FUNCTION blocks into callable units that return values
///
/// Syntax:
/// ```basic
/// FUNCTION MyFunction(param1, param2)
///     ' statements
///     RETURN value
/// END FUNCTION
/// ```
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

            // Parse FUNCTION name and parameters
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

            // Register the function
            let proc = ProcedureDefinition {
                name: func_name.clone(),
                params: func_params.clone(),
                body: func_body.clone(),
                is_function: true,
            };

            trace!("Registering FUNCTION: {}", func_name);
            PROCEDURES.lock().unwrap().insert(func_name.clone(), proc);

            // Don't output FUNCTION definition to main code
            func_name.clear();
            func_params.clear();
            func_body.clear();
        } else if in_function {
            // Collect FUNCTION body
            func_body.push_str(trimmed);
            func_body.push('\n');
        } else {
            // Regular code - pass through
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

/// Preprocess CALL statements to inline procedure code
/// This expands CALL statements by substituting the procedure body
pub fn preprocess_calls(input: &str) -> String {
    let mut result = String::new();
    let lines: Vec<&str> = input.lines().collect();

    for line in lines {
        let trimmed = line.trim();
        let upper_line = trimmed.to_uppercase();

        if upper_line.starts_with("CALL ") {
            // Parse CALL statement
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

            // Look up procedure and inline it
            let procedures = PROCEDURES.lock().unwrap();
            if let Some(proc) = procedures.get(&proc_name) {
                // Parse arguments
                let arg_values: Vec<&str> = if args.is_empty() {
                    Vec::new()
                } else {
                    args.split(',').map(|a| a.trim()).collect()
                };

                // Create variable assignments for parameters
                result.push_str("// Begin inlined CALL ");
                result.push_str(&proc_name);
                result.push('\n');

                for (i, param) in proc.params.iter().enumerate() {
                    if i < arg_values.len() && !arg_values[i].is_empty() {
                        result.push_str(&format!("let {} = {};\n", param, arg_values[i]));
                    }
                }

                // Inline the procedure body
                result.push_str(&proc.body);
                result.push_str("// End inlined CALL ");
                result.push_str(&proc_name);
                result.push('\n');
            } else {
                // Keep original CALL for runtime error handling
                result.push_str(line);
                result.push('\n');
            }
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }

    result
}

/// Full preprocessing pipeline for procedures
pub fn preprocess_procedures(input: &str) -> String {
    // First pass: extract SUB definitions
    let after_subs = preprocess_subs(input);

    // Second pass: extract FUNCTION definitions
    let after_functions = preprocess_functions(&after_subs);

    // Third pass: expand CALL statements
    preprocess_calls(&after_functions)
}

/// Clear all registered procedures (useful for testing and bot reload)
pub fn clear_procedures() {
    PROCEDURES.lock().unwrap().clear();
}

/// Get a list of all registered procedure names
pub fn get_procedure_names() -> Vec<String> {
    PROCEDURES.lock().unwrap().keys().cloned().collect()
}

/// Check if a procedure is registered
pub fn has_procedure(name: &str) -> bool {
    PROCEDURES
        .lock()
        .unwrap()
        .contains_key(&name.to_uppercase())
}

/// Get a procedure definition by name
pub fn get_procedure(name: &str) -> Option<ProcedureDefinition> {
    PROCEDURES
        .lock()
        .unwrap()
        .get(&name.to_uppercase())
        .cloned()
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() {
        clear_procedures();
    }

    #[test]
    fn test_preprocess_sub() {
        setup();

        let input = r#"
x = 1
SUB MySub(a, b)
    TALK a + b
END SUB
y = 2
"#;

        let result = preprocess_subs(input);

        // SUB should be extracted
        assert!(!result.contains("SUB MySub"));
        assert!(!result.contains("END SUB"));
        assert!(result.contains("x = 1"));
        assert!(result.contains("y = 2"));

        // Procedure should be registered
        assert!(has_procedure("MYSUB"));
        let proc = get_procedure("MYSUB").unwrap();
        assert_eq!(proc.params.len(), 2);
        assert!(!proc.is_function);
    }

    #[test]
    fn test_preprocess_function() {
        setup();

        let input = r#"
FUNCTION Add(a, b)
    RETURN a + b
END FUNCTION
result = Add(1, 2)
"#;

        let result = preprocess_functions(input);

        // FUNCTION should be extracted
        assert!(!result.contains("FUNCTION Add"));
        assert!(!result.contains("END FUNCTION"));
        assert!(result.contains("result = Add(1, 2)"));

        // Procedure should be registered
        assert!(has_procedure("ADD"));
        let proc = get_procedure("ADD").unwrap();
        assert!(proc.is_function);
    }

    #[test]
    fn test_preprocess_sub_no_params() {
        setup();

        let input = r#"
SUB PrintHello
    TALK "Hello"
END SUB
"#;

        preprocess_subs(input);

        assert!(has_procedure("PRINTHELLO"));
        let proc = get_procedure("PRINTHELLO").unwrap();
        assert!(proc.params.is_empty());
    }

    #[test]
    fn test_preprocess_call() {
        setup();

        // First register a SUB
        let sub_input = r#"
SUB Greet(name)
    TALK "Hello " + name
END SUB
"#;
        preprocess_subs(sub_input);

        // Then preprocess CALL
        let call_input = "CALL Greet(\"World\")";
        let result = preprocess_calls(call_input);

        // Should contain parameter assignment and body
        assert!(result.contains("let name = \"World\""));
        assert!(result.contains("TALK \"Hello \" + name"));
    }

    #[test]
    fn test_eval_bool_condition() {
        assert!(eval_bool_condition(&Dynamic::from(true)));
        assert!(!eval_bool_condition(&Dynamic::from(false)));
        assert!(eval_bool_condition(&Dynamic::from(1)));
        assert!(!eval_bool_condition(&Dynamic::from(0)));
        assert!(eval_bool_condition(&Dynamic::from(1.5)));
        assert!(!eval_bool_condition(&Dynamic::from(0.0)));
        assert!(eval_bool_condition(&Dynamic::from("hello")));
        assert!(!eval_bool_condition(&Dynamic::from("")));
        assert!(!eval_bool_condition(&Dynamic::from("false")));
        assert!(!eval_bool_condition(&Dynamic::from("0")));
    }

    #[test]
    fn test_clear_procedures() {
        setup();

        let input = "SUB Test\n    TALK \"test\"\nEND SUB";
        preprocess_subs(input);

        assert!(has_procedure("TEST"));

        clear_procedures();

        assert!(!has_procedure("TEST"));
    }

    #[test]
    fn test_full_pipeline() {
        setup();

        let input = r#"
SUB SendGreeting(name, greeting)
    TALK greeting + ", " + name + "!"
END SUB

FUNCTION Calculate(x, y)
    result = x * y + 10
    RETURN result
END FUNCTION

' Main code
CALL SendGreeting("User", "Hello")
total = Calculate(5, 3)
"#;

        let result = preprocess_procedures(input);

        // Should have inlined the CALL
        assert!(result.contains("let name = \"User\""));
        assert!(result.contains("let greeting = \"Hello\""));

        // Original definitions should be gone
        assert!(!result.contains("SUB SendGreeting"));
        assert!(!result.contains("END SUB"));
        assert!(!result.contains("FUNCTION Calculate"));
        assert!(!result.contains("END FUNCTION"));

        // Both should be registered
        assert!(has_procedure("SENDGREETING"));
        assert!(has_procedure("CALCULATE"));
    }
}
