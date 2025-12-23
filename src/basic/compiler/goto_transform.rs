//! GOTO/Label transformation for BASIC
//!
//! Transforms GOTO-based control flow into a state machine that Rhai can execute.
//!
//! # Warning
//!
//! While GOTO is supported for backward compatibility, it is **strongly recommended**
//! to use event-driven patterns with the `ON` keyword instead:
//!
//! ```basic
//! ' ❌ OLD WAY - GOTO loop (not recommended)
//! mainLoop:
//!     data = FIND "sensors", "processed = false"
//!     WAIT 5
//! GOTO mainLoop
//!
//! ' ✅ NEW WAY - Event-driven with ON (recommended)
//! ON INSERT OF "sensors"
//!     data = GET LAST "sensors"
//!     ' Process data reactively
//! END ON
//! ```
//!
//! Benefits of ON over GOTO:
//! - More efficient (no polling)
//! - Cleaner code structure
//! - Better integration with LLM tools
//! - Automatic resource management

use log::{trace, warn};
use std::collections::HashSet;
 
/// Represents a labeled block of code
#[derive(Debug, Clone)]
struct LabeledBlock {
    name: String,
    lines: Vec<String>,
    next_label: Option<String>, // Fall-through label
}

/// Check if source contains GOTO statements or labels
pub fn has_goto_constructs(source: &str) -> bool {
    for line in source.lines() {
        let trimmed = line.trim();
        let upper = trimmed.to_uppercase();

        // Label detection: "labelname:" at start of line (not a string, not a comment)
        if is_label_line(trimmed) {
            return true;
        }

        // GOTO detection (but not ON ERROR GOTO)
        if upper.contains("GOTO ") && !upper.contains("ON ERROR GOTO") {
            return true;
        }
    }
    false
}

/// Check if a line is a label definition
fn is_label_line(line: &str) -> bool {
    let trimmed = line.trim();

    // Must end with : and not contain spaces (except it could be indented)
    if !trimmed.ends_with(':') {
        return false;
    }

    // Skip if it's a comment
    if trimmed.starts_with('\'') || trimmed.starts_with("REM ") || trimmed.starts_with("//") {
        return false;
    }

    // Get the label name (everything before the colon)
    let label_part = trimmed.trim_end_matches(':');

    // Must be a valid identifier (alphanumeric + underscore, not starting with number)
    if label_part.is_empty() {
        return false;
    }

    // Check it's not a CASE statement or other construct
    let upper = label_part.to_uppercase();
    if upper == "CASE" || upper == "DEFAULT" || upper == "ELSE" {
        return false;
    }

    // Must be a simple identifier
    let first_char = label_part.chars().next().unwrap();
    if !first_char.is_alphabetic() && first_char != '_' {
        return false;
    }

    label_part.chars().all(|c| c.is_alphanumeric() || c == '_')
}

/// Extract label name from a label line
fn extract_label(line: &str) -> Option<String> {
    if is_label_line(line) {
        Some(line.trim().trim_end_matches(':').to_string())
    } else {
        None
    }
}

/// Transform BASIC code with GOTO/labels into Rhai-compatible state machine
///
/// This function emits warnings when GOTO is detected, recommending the use of
/// ON keyword patterns instead.
pub fn transform_goto(source: &str) -> String {
    let lines: Vec<&str> = source.lines().collect();

    // First pass: find all labels and GOTO statements
    let mut labels: HashSet<String> = HashSet::new();
    let mut goto_targets: HashSet<String> = HashSet::new();
    let mut has_goto = false;

    for line in &lines {
        let trimmed = line.trim();
        let upper = trimmed.to_uppercase();

        // Label detection
        if let Some(label) = extract_label(trimmed) {
            labels.insert(label);
        }

        // GOTO detection (but not ON ERROR GOTO)
        if upper.contains("GOTO ") && !upper.contains("ON ERROR GOTO") {
            has_goto = true;

            // Extract target label
            if let Some(target) = extract_goto_target(trimmed) {
                goto_targets.insert(target);
            }
        }
    }

    // No GOTO? Return unchanged
    if !has_goto {
        return source.to_string();
    }

    // Emit warning about GOTO usage
    warn!(
        "⚠️  GOTO detected in BASIC script. Consider using event-driven patterns with ON keyword instead."
    );
    warn!("   Example: ON INSERT OF \"table\" ... END ON");
    warn!("   See documentation: https://docs.generalbots.com/06-gbdialog/keyword-on.html");

    // Check for undefined labels
    for target in &goto_targets {
        if !labels.contains(target) {
            warn!("⚠️  GOTO references undefined label: {}", target);
        }
    }

    trace!(
        "Transforming GOTO: {} labels found, {} GOTO statements",
        labels.len(),
        goto_targets.len()
    );

    // Second pass: split code into labeled blocks
    let blocks = split_into_blocks(&lines, &labels);

    // Third pass: generate state machine
    generate_state_machine(&blocks)
}

/// Split source lines into labeled blocks
fn split_into_blocks(lines: &[&str], labels: &HashSet<String>) -> Vec<LabeledBlock> {
    let mut blocks: Vec<LabeledBlock> = Vec::new();
    let mut current_label = "__start".to_string();
    let mut current_lines: Vec<String> = Vec::new();
    let mut label_order: Vec<String> = vec!["__start".to_string()];

    for line in lines {
        let trimmed = line.trim();

        // Skip empty lines and comments in block splitting (but keep them in output)
        if trimmed.is_empty()
            || trimmed.starts_with('\'')
            || trimmed.starts_with("//")
            || trimmed.starts_with("REM ")
        {
            // Keep comments in the current block
            if !trimmed.is_empty() {
                current_lines.push(format!(
                    "// {}",
                    trimmed.trim_start_matches(&['\'', '/'][..])
                ));
            }
            continue;
        }

        // New label starts a new block
        if let Some(label) = extract_label(trimmed) {
            // Save previous block if it has content
            if !current_lines.is_empty() || current_label != "__start" || blocks.is_empty() {
                let next_label = if labels.contains(&label) {
                    Some(label.clone())
                } else {
                    None
                };

                blocks.push(LabeledBlock {
                    name: current_label.clone(),
                    lines: current_lines.clone(),
                    next_label,
                });
            }

            current_label = label.clone();
            label_order.push(label);
            current_lines.clear();
            continue;
        }

        current_lines.push(trimmed.to_string());
    }

    // Save final block
    if !current_lines.is_empty() || blocks.is_empty() {
        blocks.push(LabeledBlock {
            name: current_label,
            lines: current_lines,
            next_label: None, // End of program
        });
    }

    // Fix up next_label references based on order
    let label_order_vec: Vec<_> = label_order.iter().collect();
    for (i, block) in blocks.iter_mut().enumerate() {
        if block.next_label.is_none() && i + 1 < label_order_vec.len() {
            // Check if there's a block after this one
            let current_idx = label_order_vec.iter().position(|l| **l == block.name);
            if let Some(idx) = current_idx {
                if idx + 1 < label_order_vec.len() {
                    block.next_label = Some(label_order_vec[idx + 1].to_string());
                }
            }
        }
    }

    blocks
}

/// Extract the target label from a GOTO statement
fn extract_goto_target(line: &str) -> Option<String> {
    let upper = line.to_uppercase();

    // Simple GOTO
    if let Some(pos) = upper.find("GOTO ") {
        let rest = &line[pos + 5..];
        let target = rest.trim().split_whitespace().next()?;
        return Some(target.trim_matches(|c| c == '"' || c == '\'').to_string());
    }

    None
}

/// Generate the state machine code
fn generate_state_machine(blocks: &[LabeledBlock]) -> String {
    let mut output = String::new();

    // Add warning comment at the top
    output.push_str(
        "// ⚠️ WARNING: This code uses GOTO which is transformed into a state machine.\n",
    );
    output.push_str("// Consider using event-driven patterns with ON keyword instead:\n");
    output.push_str("//   ON INSERT OF \"table\" ... END ON\n");
    output.push_str("// See: https://docs.generalbots.com/06-gbdialog/keyword-on.html\n\n");

    // Determine start label
    let start_label = if blocks.is_empty() {
        "__start"
    } else {
        &blocks[0].name
    };

    output.push_str(&format!("let __goto_label = \"{}\";\n", start_label));
    output.push_str("let __goto_iterations = 0;\n");
    output.push_str("let __goto_max_iterations = 1000000;\n\n");
    output.push_str("while __goto_label != \"__exit\" {\n");
    output.push_str("    __goto_iterations += 1;\n");
    output.push_str("    if __goto_iterations > __goto_max_iterations {\n");
    output.push_str(
        "        throw \"GOTO loop exceeded maximum iterations. Possible infinite loop.\";\n",
    );
    output.push_str("    }\n\n");

    for block in blocks {
        output.push_str(&format!("    if __goto_label == \"{}\" {{\n", block.name));

        for line in &block.lines {
            let transformed = transform_line(line);
            // Indent the transformed line
            for transformed_line in transformed.lines() {
                if !transformed_line.trim().is_empty() {
                    output.push_str(&format!("        {}\n", transformed_line));
                }
            }
        }

        // Fall-through to next label or exit
        match &block.next_label {
            Some(next) => {
                output.push_str(&format!("        __goto_label = \"{}\"; continue;\n", next));
            }
            None => {
                output.push_str("        __goto_label = \"__exit\";\n");
            }
        }

        output.push_str("    }\n\n");
    }

    output.push_str("}\n");
    output
}

/// Transform a single line, handling GOTO and IF...GOTO
fn transform_line(line: &str) -> String {
    let trimmed = line.trim();
    let upper = trimmed.to_uppercase();

    // Skip ON ERROR GOTO - that's handled separately
    if upper.contains("ON ERROR GOTO") {
        return trimmed.to_string();
    }

    // Simple GOTO at start of line
    if upper.starts_with("GOTO ") {
        let target = trimmed[5..].trim();
        return format!("__goto_label = \"{}\"; continue;", target);
    }

    // IF ... THEN GOTO ... (single line if)
    if upper.starts_with("IF ") && upper.contains(" THEN GOTO ") {
        if let Some(then_pos) = upper.find(" THEN GOTO ") {
            let condition = &trimmed[3..then_pos].trim();
            let target = trimmed[then_pos + 11..].trim();
            return format!(
                "if {} {{ __goto_label = \"{}\"; continue; }}",
                condition, target
            );
        }
    }

    // IF ... THEN ... (might contain GOTO after THEN)
    if upper.starts_with("IF ") && upper.contains(" THEN ") {
        if let Some(then_pos) = upper.find(" THEN ") {
            let after_then = &trimmed[then_pos + 6..];
            let after_then_upper = after_then.trim().to_uppercase();

            if after_then_upper.starts_with("GOTO ") {
                let condition = &trimmed[3..then_pos].trim();
                let target = after_then.trim()[5..].trim();
                return format!(
                    "if {} {{ __goto_label = \"{}\"; continue; }}",
                    condition, target
                );
            }
        }
    }

    // IF ... GOTO ... (without THEN - some BASIC dialects support this)
    if upper.starts_with("IF ") && upper.contains(" GOTO ") && !upper.contains(" THEN ") {
        if let Some(goto_pos) = upper.find(" GOTO ") {
            let condition = &trimmed[3..goto_pos].trim();
            let target = trimmed[goto_pos + 6..].trim();
            return format!(
                "if {} {{ __goto_label = \"{}\"; continue; }}",
                condition, target
            );
        }
    }

    // Not a GOTO line, return as-is
    trimmed.to_string()
}
