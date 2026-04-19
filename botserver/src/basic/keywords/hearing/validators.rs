use super::types::{InputType, ValidationResult};
use regex::Regex;
use uuid::Uuid;

#[must_use]
pub fn validate_input(input: &str, input_type: &InputType) -> ValidationResult {
    let trimmed = input.trim();

    match input_type {
        InputType::Any
        | InputType::QrCode
        | InputType::File
        | InputType::Image
        | InputType::Audio
        | InputType::Video
        | InputType::Document
        | InputType::Login => ValidationResult::valid(trimmed.to_string()),
        InputType::Email => validate_email(trimmed),
        InputType::Date => validate_date(trimmed),
        InputType::Name => validate_name(trimmed),
        InputType::Integer => validate_integer(trimmed),
        InputType::Float => validate_float(trimmed),
        InputType::Boolean => validate_boolean(trimmed),
        InputType::Hour => validate_hour(trimmed),
        InputType::Money => validate_money(trimmed),
        InputType::Mobile => validate_mobile(trimmed),
        InputType::Zipcode => validate_zipcode(trimmed),
        InputType::Language => validate_language(trimmed),
        InputType::Cpf => validate_cpf(trimmed),
        InputType::Cnpj => validate_cnpj(trimmed),
        InputType::Url => validate_url(trimmed),
        InputType::Uuid => validate_uuid(trimmed),
        InputType::Color => validate_color(trimmed),
        InputType::CreditCard => validate_credit_card(trimmed),
        InputType::Password => validate_password(trimmed),
        InputType::Menu(options) => validate_menu(trimmed, options),
    }
}

fn validate_email(input: &str) -> ValidationResult {
    let email_regex = Regex::new(r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$").expect("valid regex");

    if email_regex.is_match(input) {
        ValidationResult::valid(input.to_lowercase())
    } else {
        ValidationResult::invalid(InputType::Email.error_message())
    }
}

fn validate_date(input: &str) -> ValidationResult {
    let formats = [
        "%d/%m/%Y", "%d-%m-%Y", "%Y-%m-%d", "%Y/%m/%d", "%d.%m.%Y", "%m/%d/%Y", "%d %b %Y",
        "%d %B %Y",
    ];

    for format in &formats {
        if let Ok(date) = chrono::NaiveDate::parse_from_str(input, format) {
            return ValidationResult::valid_with_metadata(
                date.format("%Y-%m-%d").to_string(),
                serde_json::json!({
                    "original": input,
                    "parsed_format": *format
                }),
            );
        }
    }

    let lower = input.to_lowercase();
    let today = chrono::Local::now().date_naive();

    if lower == "today" || lower == "hoje" {
        return ValidationResult::valid(today.format("%Y-%m-%d").to_string());
    }
    if lower == "tomorrow" || lower == "amanhã" || lower == "amanha" {
        return ValidationResult::valid(
            (today + chrono::Duration::days(1))
                .format("%Y-%m-%d")
                .to_string(),
        );
    }
    if lower == "yesterday" || lower == "ontem" {
        return ValidationResult::valid(
            (today - chrono::Duration::days(1))
                .format("%Y-%m-%d")
                .to_string(),
        );
    }

    ValidationResult::invalid(InputType::Date.error_message())
}

fn validate_name(input: &str) -> ValidationResult {
    let name_regex = Regex::new(r"^[\p{L}\s\-']+$").expect("valid regex");

    if input.len() < 2 {
        return ValidationResult::invalid("Name must be at least 2 characters".to_string());
    }

    if input.len() > 100 {
        return ValidationResult::invalid("Name is too long".to_string());
    }

    if name_regex.is_match(input) {
        let normalized = input
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ");
        ValidationResult::valid(normalized)
    } else {
        ValidationResult::invalid(InputType::Name.error_message())
    }
}

fn validate_integer(input: &str) -> ValidationResult {
    let cleaned = input.replace([',', '.', ' '], "").trim().to_string();

    match cleaned.parse::<i64>() {
        Ok(num) => ValidationResult::valid_with_metadata(
            num.to_string(),
            serde_json::json!({ "value": num }),
        ),
        Err(_) => ValidationResult::invalid(InputType::Integer.error_message()),
    }
}

fn validate_float(input: &str) -> ValidationResult {
    let cleaned = input.replace(' ', "").replace(',', ".").trim().to_string();

    match cleaned.parse::<f64>() {
        Ok(num) => ValidationResult::valid_with_metadata(
            format!("{:.2}", num),
            serde_json::json!({ "value": num }),
        ),
        Err(_) => ValidationResult::invalid(InputType::Float.error_message()),
    }
}

fn validate_boolean(input: &str) -> ValidationResult {
    let lower = input.to_lowercase();

    let true_values = [
        "yes",
        "y",
        "true",
        "1",
        "sim",
        "s",
        "si",
        "oui",
        "ja",
        "da",
        "ok",
        "yeah",
        "yep",
        "sure",
        "confirm",
        "confirmed",
        "accept",
        "agreed",
        "agree",
    ];

    let false_values = [
        "no", "n", "false", "0", "não", "nao", "non", "nein", "net", "nope", "cancel", "deny",
        "denied", "reject", "declined", "disagree",
    ];

    if true_values.contains(&lower.as_str()) {
        ValidationResult::valid_with_metadata(
            "true".to_string(),
            serde_json::json!({ "value": true }),
        )
    } else if false_values.contains(&lower.as_str()) {
        ValidationResult::valid_with_metadata(
            "false".to_string(),
            serde_json::json!({ "value": false }),
        )
    } else {
        ValidationResult::invalid(InputType::Boolean.error_message())
    }
}

fn validate_hour(input: &str) -> ValidationResult {
    let time_24_regex = Regex::new(r"^([01]?\d|2[0-3]):([0-5]\d)$").expect("valid regex");
    if let Some(caps) = time_24_regex.captures(input) {
        let hour: u32 = caps[1].parse().unwrap_or_default();
        let minute: u32 = caps[2].parse().unwrap_or_default();
        return ValidationResult::valid_with_metadata(
            format!("{:02}:{:02}", hour, minute),
            serde_json::json!({ "hour": hour, "minute": minute }),
        );
    }

    let time_12_regex =
        Regex::new(r"^(1[0-2]|0?[1-9]):([0-5]\d)\s*(AM|PM|am|pm|a\.m\.|p\.m\.)$").expect("valid regex");
    if let Some(caps) = time_12_regex.captures(input) {
        let mut hour: u32 = caps[1].parse().unwrap_or_default();
        let minute: u32 = caps[2].parse().unwrap_or_default();
        let period = caps[3].to_uppercase();

        if period.starts_with('P') && hour != 12 {
            hour += 12;
        } else if period.starts_with('A') && hour == 12 {
            hour = 0;
        }

        return ValidationResult::valid_with_metadata(
            format!("{:02}:{:02}", hour, minute),
            serde_json::json!({ "hour": hour, "minute": minute }),
        );
    }

    ValidationResult::invalid(InputType::Hour.error_message())
}

fn validate_money(input: &str) -> ValidationResult {
    let cleaned = input
        .replace("R$", "")
        .replace(['$', '€', '£', '¥', ' '], "")
        .trim()
        .to_string();

    let normalized = if cleaned.contains(',') && cleaned.contains('.') {
        let last_comma = cleaned.rfind(',').unwrap_or(0);
        let last_dot = cleaned.rfind('.').unwrap_or(0);

        if last_comma > last_dot {
            cleaned.replace('.', "").replace(',', ".")
        } else {
            cleaned.replace(',', "")
        }
    } else if cleaned.contains(',') {
        cleaned.replace(',', ".")
    } else {
        cleaned
    };

    match normalized.parse::<f64>() {
        Ok(amount) if amount >= 0.0 => ValidationResult::valid_with_metadata(
            format!("{:.2}", amount),
            serde_json::json!({ "value": amount }),
        ),
        _ => ValidationResult::invalid(InputType::Money.error_message()),
    }
}

fn validate_mobile(input: &str) -> ValidationResult {
    let digits: String = input.chars().filter(|c| c.is_ascii_digit()).collect();

    if digits.len() < 10 || digits.len() > 15 {
        return ValidationResult::invalid(InputType::Mobile.error_message());
    }

    let formatted = match digits.len() {
        11 => format!("({}) {}-{}", &digits[0..2], &digits[2..7], &digits[7..11]),
        10 => format!("({}) {}-{}", &digits[0..3], &digits[3..6], &digits[6..10]),
        _ => format!("+{digits}"),
    };

    ValidationResult::valid_with_metadata(
        formatted.clone(),
        serde_json::json!({ "digits": digits, "formatted": formatted }),
    )
}

fn validate_zipcode(input: &str) -> ValidationResult {
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
            format!("{}-{}", &cleaned[0..5], &cleaned[5..8])
        } else {
            cleaned.clone()
        };
        return ValidationResult::valid_with_metadata(
            formatted.clone(),
            serde_json::json!({ "digits": cleaned, "formatted": formatted, "country": "US" }),
        );
    }

    let uk_regex = Regex::new(r"^[A-Z]{1,2}\d[A-Z\d]?\s?\d[A-Z]{2}$").expect("valid regex");
    if uk_regex.is_match(&cleaned.to_uppercase()) {
        return ValidationResult::valid_with_metadata(
            cleaned.to_uppercase(),
            serde_json::json!({ "formatted": cleaned.to_uppercase(), "country": "UK" }),
        );
    }

    ValidationResult::invalid(InputType::Zipcode.error_message())
}

fn validate_language(input: &str) -> ValidationResult {
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

fn validate_cpf(input: &str) -> ValidationResult {
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

fn validate_cnpj(input: &str) -> ValidationResult {
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

fn validate_url(input: &str) -> ValidationResult {
    let url_str = if !input.starts_with("http://") && !input.starts_with("https://") {
        format!("https://{input}")
    } else {
        input.to_string()
    };

    let url_regex = Regex::new(r"^https?://[a-zA-Z0-9][-a-zA-Z0-9]*(\.[a-zA-Z0-9][-a-zA-Z0-9]*)+(/[-a-zA-Z0-9()@:%_\+.~#?&/=]*)?$").expect("valid regex");

    if url_regex.is_match(&url_str) {
        ValidationResult::valid(url_str)
    } else {
        ValidationResult::invalid(InputType::Url.error_message())
    }
}

fn validate_uuid(input: &str) -> ValidationResult {
    match Uuid::parse_str(input.trim()) {
        Ok(uuid) => ValidationResult::valid(uuid.to_string()),
        Err(_) => ValidationResult::invalid(InputType::Uuid.error_message()),
    }
}

fn validate_color(input: &str) -> ValidationResult {
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

    let hex_regex = Regex::new(r"^#?([A-Fa-f0-9]{6}|[A-Fa-f0-9]{3})$").expect("valid regex");
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

    let rgb_regex =
        Regex::new(r"^rgb\s*\(\s*(\d{1,3})\s*,\s*(\d{1,3})\s*,\s*(\d{1,3})\s*\)$").expect("valid regex");
    if let Some(caps) = rgb_regex.captures(&lower) {
        let r: u8 = caps[1].parse().unwrap_or(0);
        let g: u8 = caps[2].parse().unwrap_or(0);
        let b: u8 = caps[3].parse().unwrap_or(0);
        return ValidationResult::valid(format!("#{:02X}{:02X}{:02X}", r, g, b));
    }

    ValidationResult::invalid(InputType::Color.error_message())
}

fn validate_credit_card(input: &str) -> ValidationResult {
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

fn validate_password(input: &str) -> ValidationResult {
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

fn validate_menu(input: &str, options: &[String]) -> ValidationResult {
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
