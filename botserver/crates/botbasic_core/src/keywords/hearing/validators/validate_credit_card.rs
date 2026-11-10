use super::*;

pub(super) fn validate_credit_card(input: &str) -> ValidationResult {
    let digits: String = input.chars().filter(|c| c.is_ascii_digit()).collect();

    if digits.len() < 13 || digits.len() > 19 {
        return ValidationResult::invalid(InputType::CreditCard.error_message());
    }

    let mut sum = 0;
    let mut double = false;

    for c in digits.chars().rev() {
        let mut digit = c.to_digit(10).unwrap_or(0);
        if double {
            digit *= 2;
            if digit > 9 {
                digit -= 9;
            }
        }
        sum += digit;
        double = !double;
    }

    if sum % 10 != 0 {
        return ValidationResult::invalid("Invalid card number".to_string());
    }

    let card_type = if digits.starts_with('4') {
        "Visa"
    } else if digits.starts_with("51")
        || digits.starts_with("52")
        || digits.starts_with("53")
        || digits.starts_with("54")
        || digits.starts_with("55")
    {
        "Mastercard"
    } else if digits.starts_with("34") || digits.starts_with("37") {
        "American Express"
    } else if digits.starts_with("36") || digits.starts_with("38") {
        "Diners Club"
    } else if digits.starts_with("6011") || digits.starts_with("65") {
        "Discover"
    } else {
        "Unknown"
    };

    let masked = format!(
        "{} **** **** {}",
        &digits[0..4],
        &digits[digits.len() - 4..]
    );

    ValidationResult::valid_with_metadata(
        masked.clone(),
        serde_json::json!({
            "masked": masked,
            "last_four": &digits[digits.len()-4..],
            "card_type": card_type
        }),
    )
}
pub(super) fn validate_password(input: &str) -> ValidationResult {
    if input.len() < 8 {
        return ValidationResult::invalid("Password must be at least 8 characters".to_string());
    }

    let has_upper = input.chars().any(|c| c.is_uppercase());
    let has_lower = input.chars().any(|c| c.is_lowercase());
    let has_digit = input.chars().any(|c| c.is_ascii_digit());
    let has_special = input.chars().any(|c| !c.is_alphanumeric());

    let strength = match (has_upper, has_lower, has_digit, has_special) {
        (true, true, true, true) => "strong",
        (true, true, true, false) | (true, true, false, true) | (true, false, true, true) => {
            "medium"
        }
        _ => "weak",
    };

    ValidationResult::valid_with_metadata(
        "[PASSWORD SET]".to_string(),
        serde_json::json!({
            "strength": strength,
            "length": input.len()
        }),
    )
}
pub(super) fn validate_menu(input: &str, options: &[String]) -> ValidationResult {
    let lower_input = input.to_lowercase().trim().to_string();

    for (i, opt) in options.iter().enumerate() {
        if opt.to_lowercase() == lower_input {
            return ValidationResult::valid_with_metadata(
                opt.clone(),
                serde_json::json!({ "index": i, "value": opt }),
            );
        }
    }

    if let Ok(num) = lower_input.parse::<usize>() {
        if num >= 1 && num <= options.len() {
            let selected = &options[num - 1];
            return ValidationResult::valid_with_metadata(
                selected.clone(),
                serde_json::json!({ "index": num - 1, "value": selected }),
            );
        }
    }

    let matches: Vec<&String> = options
        .iter()
        .filter(|opt| opt.to_lowercase().contains(&lower_input))
        .collect();

    if matches.len() == 1 {
        let idx = options.iter().position(|o| o == matches[0]).unwrap_or(0);
        return ValidationResult::valid_with_metadata(
            matches[0].clone(),
            serde_json::json!({ "index": idx, "value": matches[0] }),
        );
    }

    let opts = options.join(", ");
    ValidationResult::invalid(format!("Please select one of: {opts}"))
}

#[cfg(test)]
mod credit_card_tests {
    use super::*;

    #[test]
    fn a_valid_visa_passes_and_reports_its_type() {
        let result = validate_credit_card("4111 1111 1111 1111");
        assert!(result.is_valid);
        assert_eq!(
            result.metadata.as_ref().unwrap()["card_type"],
            serde_json::json!("Visa")
        );
        assert_eq!(
            result.metadata.as_ref().unwrap()["last_four"],
            serde_json::json!("1111")
        );
    }

    #[test]
    fn known_test_numbers_pass_luhn() {
        assert!(validate_credit_card("5555555555554444").is_valid, "mastercard");
        assert!(validate_credit_card("378282246310005").is_valid, "amex");
        assert!(validate_credit_card("6011111111111117").is_valid, "discover");
        assert!(validate_credit_card("30569309025904").is_valid, "diners");
    }

    #[test]
    fn a_failing_luhn_checksum_is_rejected() {
        let result = validate_credit_card("4111111111111112");
        assert!(!result.is_valid);
        assert!(result
            .error_message
            .as_deref()
            .unwrap_or_default()
            .contains("Invalid card number"));
    }

    #[test]
    fn wrong_lengths_are_rejected_before_the_checksum() {
        assert!(!validate_credit_card("411111111111").is_valid);
        assert!(!validate_credit_card("41111111111111111111").is_valid);
        assert!(!validate_credit_card("").is_valid);
        assert!(!validate_credit_card("not a card").is_valid);
    }

    #[test]
    fn separators_and_spaces_are_ignored() {
        assert!(validate_credit_card("4111-1111-1111-1111").is_valid);
        assert!(validate_credit_card("4111.1111.1111.1111").is_valid);
    }

    #[test]
    fn the_masked_value_never_exposes_the_full_number() {
        let result = validate_credit_card("4111111111111111");
        assert!(result.normalized_value.starts_with("4111"));
        assert!(result.normalized_value.ends_with("1111"));
        assert!(!result.normalized_value.contains("111111111111"));
        assert_eq!(result.normalized_value.matches('*').count(), 8);
    }
}
