use super::types::ValidationResult;
use super::types::ValidationError;
use super::types::ValidationWarning;

pub fn validate_basic_code(code: &str) -> ValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    let lines: Vec<&str> = code.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let line_num = i + 1;
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('\'') || trimmed.starts_with("REM ") {
            continue;
        }

        let upper = trimmed.to_uppercase();

        if upper.starts_with("IF ") && !upper.contains(" THEN") {
            errors.push(ValidationError {
                line: line_num,
                column: 1,
                message: "IF statement missing THEN keyword".to_string(),
                node_id: None,
            });
        }

        if upper.starts_with("FOR ") && !upper.contains(" TO ") {
            errors.push(ValidationError {
                line: line_num,
                column: 1,
                message: "FOR statement missing TO keyword".to_string(),
                node_id: None,
            });
        }

        let quote_count = trimmed.chars().filter(|c| *c == '"').count();
        if quote_count % 2 != 0 {
            errors.push(ValidationError {
                line: line_num,
                column: trimmed.find('"').unwrap_or(0) + 1,
                message: "Unclosed string literal".to_string(),
                node_id: None,
            });
        }

        if upper.starts_with("GOTO ") {
            warnings.push(ValidationWarning {
                line: line_num,
                message: "GOTO statements can make code harder to maintain".to_string(),
                node_id: None,
            });
        }

        if trimmed.len() > 120 {
            warnings.push(ValidationWarning {
                line: line_num,
                message: "Line exceeds recommended length of 120 characters".to_string(),
                node_id: None,
            });
        }
    }

    let mut if_count = 0i32;
    let mut for_count = 0i32;
    let mut sub_count = 0i32;

    for line in &lines {
        let upper = line.to_uppercase();
        let trimmed = upper.trim();

        if trimmed.starts_with("IF ") && !trimmed.ends_with(" THEN") && trimmed.contains(" THEN") {
        } else if trimmed.starts_with("IF ") {
            if_count += 1;
        } else if trimmed == "END IF" || trimmed == "ENDIF" {
            if_count -= 1;
        }

        if trimmed.starts_with("FOR ") {
            for_count += 1;
        } else if trimmed == "NEXT" || trimmed.starts_with("NEXT ") {
            for_count -= 1;
        }

        if trimmed.starts_with("SUB ") {
            sub_count += 1;
        } else if trimmed == "END SUB" {
            sub_count -= 1;
        }
    }

    if if_count > 0 {
        errors.push(ValidationError {
            line: lines.len(),
            column: 1,
            message: format!("{} unclosed IF statement(s)", if_count),
            node_id: None,
        });
    }

    if for_count > 0 {
        errors.push(ValidationError {
            line: lines.len(),
            column: 1,
            message: format!("{} unclosed FOR loop(s)", for_count),
            node_id: None,
        });
    }

    if sub_count > 0 {
        errors.push(ValidationError {
            line: lines.len(),
            column: 1,
            message: format!("{} unclosed SUB definition(s)", sub_count),
            node_id: None,
        });
    }

    ValidationResult {
        valid: errors.is_empty(),
        errors,
        warnings,
    }
}
