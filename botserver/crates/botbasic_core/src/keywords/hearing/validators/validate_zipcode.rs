use super::*;
use crate::re_cache::cached_re;

pub(super) fn validate_zipcode(input: &str) -> ValidationResult {
    let cleaned: String = input
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect();

    if cleaned.len() == 8 && cleaned.chars().all(|c| c.is_ascii_digit()) {
        let formatted = format!("{}-{}", &cleaned[0..5], &cleaned[5..8]);
        return ValidationResult::valid_with_metadata(
            formatted.clone(),
            serde_json::json!({ "digits": cleaned, "formatted": formatted, "country": "BR" }),
        );
    }

    if (cleaned.len() == 5 || cleaned.len() == 9) && cleaned.chars().all(|c| c.is_ascii_digit()) {
        let formatted = if cleaned.len() == 9 {
            // ZIP+4 keeps all nine digits: the old slice stopped at index 8,
            // so "12345-6789" came back as "12345-678".
            format!("{}-{}", &cleaned[0..5], &cleaned[5..9])
        } else {
            cleaned.clone()
        };
        return ValidationResult::valid_with_metadata(
            formatted.clone(),
            serde_json::json!({ "digits": cleaned, "formatted": formatted, "country": "US" }),
        );
    }

    let Some(uk_regex) = cached_re!(r"^[A-Z]{1,2}\d[A-Z\d]?\s?\d[A-Z]{2}$") else {
        return ValidationResult::invalid("internal validation error".to_string());
    };
    if uk_regex.is_match(&cleaned.to_uppercase()) {
        return ValidationResult::valid_with_metadata(
            cleaned.to_uppercase(),
            serde_json::json!({ "formatted": cleaned.to_uppercase(), "country": "UK" }),
        );
    }

    ValidationResult::invalid(InputType::Zipcode.error_message())
}
pub(super) fn validate_language(input: &str) -> ValidationResult {
    let lower = input.to_lowercase().trim().to_string();

    let languages = [
        ("en", "english", "inglês", "ingles"),
        ("pt", "portuguese", "português", "portugues"),
        ("es", "spanish", "espanhol", "español"),
        ("fr", "french", "francês", "frances"),
        ("de", "german", "alemão", "alemao"),
        ("it", "italian", "italiano", ""),
        ("ja", "japanese", "japonês", "japones"),
        ("zh", "chinese", "chinês", "chines"),
        ("ko", "korean", "coreano", ""),
        ("ru", "russian", "russo", ""),
        ("ar", "arabic", "árabe", "arabe"),
        ("hi", "hindi", "", ""),
        ("nl", "dutch", "holandês", "holandes"),
        ("pl", "polish", "polonês", "polones"),
        ("tr", "turkish", "turco", ""),
    ];

    for entry in &languages {
        let code = entry.0;
        let variants = [entry.1, entry.2, entry.3];
        if lower.as_str() == code
            || variants
                .iter()
                .any(|v| !v.is_empty() && lower.as_str() == *v)
        {
            return ValidationResult::valid_with_metadata(
                code.to_string(),
                serde_json::json!({ "code": code, "input": input }),
            );
        }
    }

    if lower.len() == 2 && lower.chars().all(|c| c.is_ascii_lowercase()) {
        return ValidationResult::valid(lower);
    }

    ValidationResult::invalid(InputType::Language.error_message())
}
pub(super) fn validate_cpf(input: &str) -> ValidationResult {
    let digits: String = input.chars().filter(|c| c.is_ascii_digit()).collect();

    if digits.len() != 11 {
        return ValidationResult::invalid(InputType::Cpf.error_message());
    }

    if let Some(first_char) = digits.chars().next() {
        if digits.chars().all(|c| c == first_char) {
            return ValidationResult::invalid("Invalid CPF".to_string());
        }
    }

    let digits_vec: Vec<u32> = digits.chars().filter_map(|c| c.to_digit(10)).collect();

    let sum1: u32 = digits_vec[0..9]
        .iter()
        .enumerate()
        .map(|(i, &d)| d * (10 - i as u32))
        .sum();
    let check1 = (sum1 * 10) % 11;
    let check1 = if check1 == 10 { 0 } else { check1 };

    if check1 != digits_vec[9] {
        return ValidationResult::invalid("Invalid CPF".to_string());
    }

    let sum2: u32 = digits_vec[0..10]
        .iter()
        .enumerate()
        .map(|(i, &d)| d * (11 - i as u32))
        .sum();
    let check2 = (sum2 * 10) % 11;
    let check2 = if check2 == 10 { 0 } else { check2 };

    if check2 != digits_vec[10] {
        return ValidationResult::invalid("Invalid CPF".to_string());
    }

    let formatted = format!(
        "{}.{}.{}-{}",
        &digits[0..3], &digits[3..6], &digits[6..9], &digits[9..11]
    );

    ValidationResult::valid_with_metadata(
        formatted.clone(),
        serde_json::json!({ "digits": digits, "formatted": formatted }),
    )
}
pub(super) fn validate_cnpj(input: &str) -> ValidationResult {
    let digits: String = input.chars().filter(|c| c.is_ascii_digit()).collect();

    if digits.len() != 14 {
        return ValidationResult::invalid(InputType::Cnpj.error_message());
    }

    let digits_vec: Vec<u32> = digits.chars().filter_map(|c| c.to_digit(10)).collect();

    let weights1 = [5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2];
    let sum1: u32 = digits_vec[0..12]
        .iter()
        .zip(weights1.iter())
        .map(|(&d, &w)| d * w)
        .sum();
    let check1 = sum1 % 11;
    let check1 = if check1 < 2 { 0 } else { 11 - check1 };

    if check1 != digits_vec[12] {
        return ValidationResult::invalid("Invalid CNPJ".to_string());
    }

    let weights2 = [6, 5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2];
    let sum2: u32 = digits_vec[0..13]
        .iter()
        .zip(weights2.iter())
        .map(|(&d, &w)| d * w)
        .sum();
    let check2 = sum2 % 11;
    let check2 = if check2 < 2 { 0 } else { 11 - check2 };

    if check2 != digits_vec[13] {
        return ValidationResult::invalid("Invalid CNPJ".to_string());
    }

    let formatted = format!(
        "{}.{}.{}/{}-{}",
        &digits[0..2], &digits[2..5], &digits[5..8], &digits[8..12], &digits[12..14]
    );

    ValidationResult::valid_with_metadata(
        formatted.clone(),
        serde_json::json!({ "digits": digits, "formatted": formatted }),
    )
}
pub(super) fn validate_url(input: &str) -> ValidationResult {
    let url_str = if !input.starts_with("http://") && !input.starts_with("https://") {
        format!("https://{input}")
    } else {
        input.to_string()
    };

    let Some(url_regex) = cached_re!(r"^https?://[a-zA-Z0-9][-a-zA-Z0-9]*(\.[a-zA-Z0-9][-a-zA-Z0-9]*)+(/[-a-zA-Z0-9()@:%_\+.~#?&/=]*)?$") else {
        return ValidationResult::invalid("internal validation error".to_string());
    };

    if url_regex.is_match(&url_str) {
        ValidationResult::valid(url_str)
    } else {
        ValidationResult::invalid(InputType::Url.error_message())
    }
}
pub(super) fn validate_uuid(input: &str) -> ValidationResult {
    match Uuid::parse_str(input.trim()) {
        Ok(uuid) => ValidationResult::valid(uuid.to_string()),
        Err(_) => ValidationResult::invalid(InputType::Uuid.error_message()),
    }
}
pub(super) fn validate_color(input: &str) -> ValidationResult {
    let lower = input.to_lowercase().trim().to_string();

    let named_colors = [
        ("red", "#FF0000"),
        ("green", "#00FF00"),
        ("blue", "#0000FF"),
        ("white", "#FFFFFF"),
        ("black", "#000000"),
        ("yellow", "#FFFF00"),
        ("orange", "#FFA500"),
        ("purple", "#800080"),
        ("pink", "#FFC0CB"),
        ("gray", "#808080"),
        ("grey", "#808080"),
        ("brown", "#A52A2A"),
        ("cyan", "#00FFFF"),
        ("magenta", "#FF00FF"),
    ];

    for (name, hex) in &named_colors {
        if lower == *name {
            return ValidationResult::valid_with_metadata(
                (*hex).to_owned(),
                serde_json::json!({ "name": name, "hex": hex }),
            );
        }
    }

    let Some(hex_regex) = cached_re!(r"^#?([A-Fa-f0-9]{6}|[A-Fa-f0-9]{3})$") else {
        return ValidationResult::invalid("internal validation error".to_string());
    };
    if let Some(caps) = hex_regex.captures(&lower) {
        let hex = caps[1].to_uppercase();
        let full_hex = if hex.len() == 3 {
            let mut result = String::with_capacity(6);
            for c in hex.chars() {
                result.push(c);
                result.push(c);
            }
            result
        } else {
            hex
        };
        return ValidationResult::valid(format!("#{}", full_hex));
    }

    if let Some(caps) = cached_re!(r"^rgb\s*\(\s*(\d{1,3})\s*,\s*(\d{1,3})\s*,\s*(\d{1,3})\s*\)$")
        .and_then(|re| re.captures(&lower))
    {
        let r: u8 = caps[1].parse().unwrap_or(0);
        let g: u8 = caps[2].parse().unwrap_or(0);
        let b: u8 = caps[3].parse().unwrap_or(0);
        return ValidationResult::valid(format!("#{:02X}{:02X}{:02X}", r, g, b));
    }

    ValidationResult::invalid(InputType::Color.error_message())
}

#[cfg(test)]
mod identity_validator_tests {
    use super::*;

    #[test]
    fn brazilian_ceps_are_formatted() {
        let result = validate_zipcode("01310100");
        assert!(result.is_valid);
        assert_eq!(result.normalized_value, "01310-100");
        assert_eq!(
            result.metadata.as_ref().unwrap()["country"],
            serde_json::json!("BR")
        );
        assert_eq!(validate_zipcode("01310-100").normalized_value, "01310-100");
    }

    #[test]
    fn us_zips_keep_every_digit() {
        assert_eq!(validate_zipcode("12345").normalized_value, "12345");
        // ZIP+4 used to come back as "12345-678", losing the last digit.
        let plus_four = validate_zipcode("12345-6789");
        assert!(plus_four.is_valid);
        assert_eq!(plus_four.normalized_value, "12345-6789");
        assert_eq!(
            plus_four.metadata.as_ref().unwrap()["country"],
            serde_json::json!("US")
        );
    }

    #[test]
    fn uk_postcodes_are_uppercased() {
        let result = validate_zipcode("sw1a 1aa");
        assert!(result.is_valid);
        assert_eq!(result.normalized_value, "SW1A1AA");
    }

    #[test]
    fn junk_is_rejected_as_a_zipcode() {
        assert!(!validate_zipcode("").is_valid);
        assert!(!validate_zipcode("abc").is_valid);
        assert!(!validate_zipcode("1234").is_valid);
    }

    #[test]
    fn valid_cpfs_pass_and_are_formatted() {
        let result = validate_cpf("529.982.247-25");
        assert!(result.is_valid);
        assert_eq!(result.normalized_value, "529.982.247-25");
        assert!(validate_cpf("52998224725").is_valid);
    }

    #[test]
    fn invalid_cpfs_are_rejected() {
        assert!(!validate_cpf("111.111.111-11").is_valid, "repeated digits");
        assert!(!validate_cpf("529.982.247-26").is_valid, "bad check digit");
        assert!(!validate_cpf("123").is_valid, "wrong length");
        assert!(!validate_cpf("").is_valid);
    }

    #[test]
    fn valid_cnpjs_pass_and_are_formatted() {
        let result = validate_cnpj("11.222.333/0001-81");
        assert!(result.is_valid);
        assert_eq!(result.normalized_value, "11.222.333/0001-81");
        assert!(validate_cnpj("11222333000181").is_valid);
    }

    #[test]
    fn invalid_cnpjs_are_rejected() {
        assert!(!validate_cnpj("11.222.333/0001-82").is_valid, "bad check digit");
        assert!(!validate_cnpj("123").is_valid, "wrong length");
        assert!(!validate_cnpj("").is_valid);
    }

    #[test]
    fn languages_match_names_and_codes() {
        assert_eq!(validate_language("Portuguese").normalized_value, "pt");
        assert_eq!(validate_language("português").normalized_value, "pt");
        assert_eq!(validate_language("EN").normalized_value, "en");
        assert_eq!(validate_language("espanhol").normalized_value, "es");
        assert!(!validate_language("klingon").is_valid);
        assert!(!validate_language("").is_valid);
    }

    #[test]
    fn urls_gain_a_scheme_when_missing() {
        assert_eq!(validate_url("example.com").normalized_value, "https://example.com");
        assert_eq!(
            validate_url("https://example.com/path?q=1").normalized_value,
            "https://example.com/path?q=1"
        );
        assert!(!validate_url("not a url").is_valid);
        assert!(!validate_url("").is_valid);
    }

    #[test]
    fn named_and_hex_colors_normalize_to_hex() {
        assert_eq!(validate_color("red").normalized_value, "#FF0000");
        assert_eq!(validate_color("#ff0000").normalized_value, "#FF0000");
        assert_eq!(validate_color("#f00").normalized_value, "#FF0000");
        assert_eq!(validate_color("rgb(255, 0, 0)").normalized_value, "#FF0000");
        assert!(!validate_color("not-a-color").is_valid);
    }

    #[test]
    fn uuids_are_validated_and_canonicalized() {
        assert_eq!(
            validate_uuid("550E8400-E29B-41D4-A716-446655440000").normalized_value,
            "550e8400-e29b-41d4-a716-446655440000"
        );
        assert!(!validate_uuid("not-a-uuid").is_valid);
    }
}
