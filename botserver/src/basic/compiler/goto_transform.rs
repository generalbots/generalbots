use log::{trace, warn};
use std::collections::HashSet;
use std::fmt::Write;

#[derive(Debug, Clone)]
struct LabeledBlock {
    name: String,
    lines: Vec<String>,
    next_label: Option<String>,
}

pub fn has_goto_constructs(source: &str) -> bool {
    for line in source.lines() {
        let trimmed = line.trim();
        let upper = trimmed.to_uppercase();

        if is_label_line(trimmed) {
            return true;
        }

        if upper.contains("GOTO ") && !upper.contains("ON ERROR GOTO") {
            return true;
        }
    }
    false
}

fn is_label_line(line: &str) -> bool {
    let trimmed = line.trim();

    if !trimmed.ends_with(':') {
        return false;
    }

    if trimmed.starts_with('\'') || trimmed.starts_with("REM ") || trimmed.starts_with("//") {
        return false;
    }

    let label_part = trimmed.trim_end_matches(':');

    if label_part.is_empty() {
        return false;
    }

    let upper = label_part.to_uppercase();
    if upper == "CASE" || upper == "DEFAULT" || upper == "ELSE" {
        return false;
    }

    let first_char = label_part.chars().next().unwrap_or_default();
    if !first_char.is_alphabetic() && first_char != '_' {
        return false;
    }

    label_part.chars().all(|c| c.is_alphanumeric() || c == '_')
}

fn extract_label(line: &str) -> Option<String> {
    if is_label_line(line) {
        Some(line.trim().trim_end_matches(':').to_string())
    } else {
        None
    }
}

pub fn transform_goto(source: &str) -> String {
    let lines: Vec<&str> = source.lines().collect();

    let mut labels: HashSet<String> = HashSet::new();
    let mut goto_targets: HashSet<String> = HashSet::new();
    let mut has_goto = false;

    for line in &lines {
        let trimmed = line.trim();
        let upper = trimmed.to_uppercase();

        if let Some(label) = extract_label(trimmed) {
            labels.insert(label);
        }

        if upper.contains("GOTO ") && !upper.contains("ON ERROR GOTO") {
            has_goto = true;

            if let Some(target) = extract_goto_target(trimmed) {
                goto_targets.insert(target);
            }
        }
    }

    if !has_goto {
        return source.to_string();
    }

    warn!(
        "⚠️  GOTO detected in BASIC script. Consider using event-driven patterns with ON keyword instead."
    );
    warn!("   Example: ON INSERT OF \"table\" ... END ON");
    warn!("   See documentation: https://docs.generalbots.com/06-gbdialog/keyword-on.html");

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

    let blocks = split_into_blocks(&lines, &labels);

    generate_state_machine(&blocks)
}

fn split_into_blocks(lines: &[&str], labels: &HashSet<String>) -> Vec<LabeledBlock> {
    let mut blocks: Vec<LabeledBlock> = Vec::new();
    let mut current_label = "__start".to_string();
    let mut current_lines: Vec<String> = Vec::new();
    let mut label_order: Vec<String> = vec!["__start".to_string()];

    for line in lines {
        let trimmed = line.trim();

        if trimmed.is_empty()
            || trimmed.starts_with('\'')
            || trimmed.starts_with("//")
            || trimmed.starts_with("REM ")
        {
            if !trimmed.is_empty() {
                current_lines.push(format!(
                    "// {}",
                    trimmed.trim_start_matches(&['\'', '/'][..])
                ));
            }
            continue;
        }

        if let Some(label) = extract_label(trimmed) {
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

            current_label.clone_from(&label);
            label_order.push(label);
            current_lines.clear();
            continue;
        }

        current_lines.push(trimmed.to_string());
    }

    if !current_lines.is_empty() || blocks.is_empty() {
        blocks.push(LabeledBlock {
            name: current_label,
            lines: current_lines,
            next_label: None,
        });
    }

    let label_order_vec: Vec<_> = label_order.iter().collect();
    for (i, block) in blocks.iter_mut().enumerate() {
        if block.next_label.is_none() && i + 1 < label_order_vec.len() {
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

fn extract_goto_target(line: &str) -> Option<String> {
    let upper = line.to_uppercase();

    if let Some(pos) = upper.find("GOTO ") {
        let rest = &line[pos + 5..];
        let target = rest.split_whitespace().next()?;
        return Some(target.trim_matches(|c| c == '"' || c == '\'').to_string());
    }

    None
}

fn generate_state_machine(blocks: &[LabeledBlock]) -> String {
    let mut output = String::new();

    output.push_str(
        "// ⚠️ WARNING: This code uses GOTO which is transformed into a state machine.\n",
    );
    output.push_str("// Consider using event-driven patterns with ON keyword instead:\n");
    output.push_str("//   ON INSERT OF \"table\" ... END ON\n");
    output.push_str("// See: https://docs.generalbots.com/06-gbdialog/keyword-on.html\n\n");

    let start_label = if blocks.is_empty() {
        "__start"
    } else {
        &blocks[0].name
    };

    let _ = writeln!(output, "let __goto_label = \"{start_label}\";");
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
        let _ = writeln!(output, "    if __goto_label == \"{}\" {{", block.name);

        for line in &block.lines {
            let transformed = transform_line(line);

            for transformed_line in transformed.lines() {
                if !transformed_line.trim().is_empty() {
                    let _ = writeln!(output, "        {transformed_line}");
                }
            }
        }

        match &block.next_label {
            Some(next) => {
                let _ = writeln!(output, "        __goto_label = \"{next}\"; continue;");
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

fn transform_line(line: &str) -> String {
    let trimmed = line.trim();
    let upper = trimmed.to_uppercase();

    if upper.contains("ON ERROR GOTO") {
        return trimmed.to_string();
    }

    if upper.starts_with("GOTO ") {
        let target = trimmed[5..].trim();
        return format!("__goto_label = \"{}\"; continue;", target);
    }

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

    trimmed.to_string()
}
