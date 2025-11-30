//! HEAR and TALK keywords for conversational I/O
//!
//! HEAR waits for user input with optional type validation:
//! - HEAR variable                     - Basic input, no validation
//! - HEAR variable AS EMAIL            - Validates email format
//! - HEAR variable AS DATE             - Validates and parses date
//! - HEAR variable AS NAME             - Validates name (letters, spaces)
//! - HEAR variable AS INTEGER          - Validates integer number
//! - HEAR variable AS BOOLEAN          - Validates yes/no, true/false
//! - HEAR variable AS HOUR             - Validates time format (HH:MM)
//! - HEAR variable AS MONEY            - Validates currency amount
//! - HEAR variable AS MOBILE           - Validates mobile phone number
//! - HEAR variable AS ZIPCODE          - Validates ZIP/postal code
//! - HEAR variable AS LANGUAGE         - Validates language code
//! - HEAR variable AS CPF              - Validates Brazilian CPF
//! - HEAR variable AS CNPJ             - Validates Brazilian CNPJ
//! - HEAR variable AS QRCODE           - Reads QR code from image
//! - HEAR variable AS LOGIN            - Waits for authentication
//! - HEAR variable AS "Option1", "Option2", "Option3" - Menu selection
//! - HEAR variable AS FILE             - Waits for file upload
//! - HEAR variable AS IMAGE            - Waits for image upload
//! - HEAR variable AS AUDIO            - Waits for audio upload
//! - HEAR variable AS VIDEO            - Waits for video upload
//!
//! All AS variants automatically retry with helpful messages until valid input.

use crate::shared::message_types::MessageType;
use crate::shared::models::{BotResponse, UserSession};
use crate::shared::state::AppState;
use log::{error, info, trace};
use regex::Regex;
use rhai::{Dynamic, Engine, EvalAltResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Input validation types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InputType {
    Any,
    Email,
    Date,
    Name,
    Integer,
    Float,
    Boolean,
    Hour,
    Money,
    Mobile,
    Zipcode,
    Language,
    Cpf,
    Cnpj,
    QrCode,
    Login,
    Menu(Vec<String>),
    File,
    Image,
    Audio,
    Video,
    Document,
    Url,
    Uuid,
    Color,
    CreditCard,
    Password,
}

impl InputType {
    /// Get validation error message for this type
    pub fn error_message(&self) -> String {
        match self {
            InputType::Any => "".to_string(),
            InputType::Email => {
                "Please enter a valid email address (e.g., user@example.com)".to_string()
            }
            InputType::Date => {
                "Please enter a valid date (e.g., 25/12/2024 or 2024-12-25)".to_string()
            }
            InputType::Name => "Please enter a valid name (letters and spaces only)".to_string(),
            InputType::Integer => "Please enter a valid whole number".to_string(),
            InputType::Float => "Please enter a valid number".to_string(),
            InputType::Boolean => "Please answer yes or no".to_string(),
            InputType::Hour => "Please enter a valid time (e.g., 14:30 or 2:30 PM)".to_string(),
            InputType::Money => {
                "Please enter a valid amount (e.g., 100.00 or R$ 100,00)".to_string()
            }
            InputType::Mobile => "Please enter a valid mobile number".to_string(),
            InputType::Zipcode => "Please enter a valid ZIP/postal code".to_string(),
            InputType::Language => {
                "Please enter a valid language code (e.g., en, pt, es)".to_string()
            }
            InputType::Cpf => "Please enter a valid CPF (11 digits)".to_string(),
            InputType::Cnpj => "Please enter a valid CNPJ (14 digits)".to_string(),
            InputType::QrCode => "Please send an image containing a QR code".to_string(),
            InputType::Login => "Please complete the authentication process".to_string(),
            InputType::Menu(options) => format!("Please select one of: {}", options.join(", ")),
            InputType::File => "Please upload a file".to_string(),
            InputType::Image => "Please send an image".to_string(),
            InputType::Audio => "Please send an audio file or voice message".to_string(),
            InputType::Video => "Please send a video".to_string(),
            InputType::Document => "Please send a document (PDF, Word, etc.)".to_string(),
            InputType::Url => "Please enter a valid URL".to_string(),
            InputType::Uuid => "Please enter a valid UUID".to_string(),
            InputType::Color => "Please enter a valid color (e.g., #FF0000 or red)".to_string(),
            InputType::CreditCard => "Please enter a valid credit card number".to_string(),
            InputType::Password => "Please enter a password (minimum 8 characters)".to_string(),
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "EMAIL" => InputType::Email,
            "DATE" => InputType::Date,
            "NAME" => InputType::Name,
            "INTEGER" | "INT" | "NUMBER" => InputType::Integer,
            "FLOAT" | "DECIMAL" | "DOUBLE" => InputType::Float,
            "BOOLEAN" | "BOOL" => InputType::Boolean,
            "HOUR" | "TIME" => InputType::Hour,
            "MONEY" | "CURRENCY" | "AMOUNT" => InputType::Money,
            "MOBILE" | "PHONE" | "TELEPHONE" => InputType::Mobile,
            "ZIPCODE" | "ZIP" | "CEP" | "POSTALCODE" => InputType::Zipcode,
            "LANGUAGE" | "LANG" => InputType::Language,
            "CPF" => InputType::Cpf,
            "CNPJ" => InputType::Cnpj,
            "QRCODE" | "QR" => InputType::QrCode,
            "LOGIN" | "AUTH" => InputType::Login,
            "FILE" => InputType::File,
            "IMAGE" | "PHOTO" | "PICTURE" => InputType::Image,
            "AUDIO" | "VOICE" | "SOUND" => InputType::Audio,
            "VIDEO" => InputType::Video,
            "DOCUMENT" | "DOC" | "PDF" => InputType::Document,
            "URL" | "LINK" => InputType::Url,
            "UUID" | "GUID" => InputType::Uuid,
            "COLOR" | "COLOUR" => InputType::Color,
            "CREDITCARD" | "CARD" => InputType::CreditCard,
            "PASSWORD" | "PASS" | "SECRET" => InputType::Password,
            _ => InputType::Any,
        }
    }
}

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub normalized_value: String,
    pub error_message: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

impl ValidationResult {
    pub fn valid(value: String) -> Self {
        Self {
            is_valid: true,
            normalized_value: value,
            error_message: None,
            metadata: None,
        }
    }

    pub fn valid_with_metadata(value: String, metadata: serde_json::Value) -> Self {
        Self {
            is_valid: true,
            normalized_value: value,
            error_message: None,
            metadata: Some(metadata),
        }
    }

    pub fn invalid(error: String) -> Self {
        Self {
            is_valid: false,
            normalized_value: String::new(),
            error_message: Some(error),
            metadata: None,
        }
    }
}

/// Register all HEAR keyword variants
pub fn hear_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    // Basic HEAR without validation
    register_hear_basic(state.clone(), user.clone(), engine);

    // HEAR with AS type validation
    register_hear_as_type(state.clone(), user.clone(), engine);

    // HEAR with menu options: HEAR var AS "Option1", "Option2", "Option3"
    register_hear_as_menu(state.clone(), user.clone(), engine);
}

/// Basic HEAR variable - no validation
fn register_hear_basic(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let session_id = user.id;
    let state_clone = Arc::clone(&state);

    engine
        .register_custom_syntax(&["HEAR", "$ident$"], true, move |_context, inputs| {
            // Normalize variable name to lowercase for case-insensitive BASIC
            let variable_name = inputs[0]
                .get_string_value()
                .expect("Expected identifier as string")
                .to_lowercase();

            trace!(
                "HEAR command waiting for user input to store in variable: {}",
                variable_name
            );

            let state_for_spawn = Arc::clone(&state_clone);
            let session_id_clone = session_id;
            let var_name_clone = variable_name.clone();

            tokio::spawn(async move {
                trace!(
                    "HEAR: Setting session {} to wait for input for variable '{}'",
                    session_id_clone,
                    var_name_clone
                );

                let mut session_manager = state_for_spawn.session_manager.lock().await;
                session_manager.mark_waiting(session_id_clone);

                // Store wait state in Redis
                if let Some(redis_client) = &state_for_spawn.cache {
                    if let Ok(mut conn) = redis_client.get_multiplexed_async_connection().await {
                        let key = format!("hear:{}:{}", session_id_clone, var_name_clone);
                        let wait_data = serde_json::json!({
                            "variable": var_name_clone,
                            "type": "any",
                            "waiting": true,
                            "retry_count": 0
                        });
                        let _: Result<(), _> = redis::cmd("SET")
                            .arg(&key)
                            .arg(wait_data.to_string())
                            .arg("EX")
                            .arg(3600) // 1 hour expiry
                            .query_async(&mut conn)
                            .await;
                    }
                }
            });

            Err(Box::new(EvalAltResult::ErrorRuntime(
                "Waiting for user input".into(),
                rhai::Position::NONE,
            )))
        })
        .unwrap();
}

/// HEAR variable AS TYPE - with validation
fn register_hear_as_type(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let session_id = user.id;
    let state_clone = Arc::clone(&state);

    engine
        .register_custom_syntax(
            &["HEAR", "$ident$", "AS", "$ident$"],
            true,
            move |_context, inputs| {
                // Normalize variable name to lowercase for case-insensitive BASIC
                let variable_name = inputs[0]
                    .get_string_value()
                    .expect("Expected identifier for variable")
                    .to_lowercase();
                let type_name = inputs[1]
                    .get_string_value()
                    .expect("Expected identifier for type")
                    .to_string();

                let input_type = InputType::from_str(&type_name);

                trace!(
                    "HEAR {} AS {} - waiting for validated input",
                    variable_name,
                    type_name
                );

                let state_for_spawn = Arc::clone(&state_clone);
                let session_id_clone = session_id;
                let var_name_clone = variable_name.clone();
                let type_clone = type_name.clone();

                tokio::spawn(async move {
                    let mut session_manager = state_for_spawn.session_manager.lock().await;
                    session_manager.mark_waiting(session_id_clone);

                    // Store wait state with type in Redis
                    if let Some(redis_client) = &state_for_spawn.cache {
                        if let Ok(mut conn) = redis_client.get_multiplexed_async_connection().await
                        {
                            let key = format!("hear:{}:{}", session_id_clone, var_name_clone);
                            let wait_data = serde_json::json!({
                                "variable": var_name_clone,
                                "type": type_clone.to_lowercase(),
                                "waiting": true,
                                "retry_count": 0,
                                "max_retries": 3
                            });
                            let _: Result<(), _> = redis::cmd("SET")
                                .arg(&key)
                                .arg(wait_data.to_string())
                                .arg("EX")
                                .arg(3600)
                                .query_async(&mut conn)
                                .await;
                        }
                    }
                });

                Err(Box::new(EvalAltResult::ErrorRuntime(
                    "Waiting for user input".into(),
                    rhai::Position::NONE,
                )))
            },
        )
        .unwrap();
}

/// HEAR variable AS "Option1", "Option2", "Option3" - menu selection
fn register_hear_as_menu(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let session_id = user.id;
    let state_clone = Arc::clone(&state);

    // This handles: HEAR var AS "opt1", "opt2", "opt3"
    // We need to handle variable number of string options
    engine
        .register_custom_syntax(
            &["HEAR", "$ident$", "AS", "$expr$"],
            true,
            move |context, inputs| {
                // Normalize variable name to lowercase for case-insensitive BASIC
                let variable_name = inputs[0]
                    .get_string_value()
                    .expect("Expected identifier for variable")
                    .to_lowercase();

                // Evaluate the expression to get options
                let options_expr = context.eval_expression_tree(&inputs[1])?;
                let options_str = options_expr.to_string();

                // Check if it's a type keyword or menu options
                let input_type = InputType::from_str(&options_str);
                if input_type != InputType::Any {
                    // It's a type, handled by register_hear_as_type
                    return Err(Box::new(EvalAltResult::ErrorRuntime(
                        "Use HEAR AS TYPE syntax".into(),
                        rhai::Position::NONE,
                    )));
                }

                // Parse as menu options (comma-separated or array)
                let options: Vec<String> = if options_str.starts_with('[') {
                    // Array format
                    serde_json::from_str(&options_str).unwrap_or_default()
                } else {
                    // Comma-separated or single value
                    options_str
                        .split(',')
                        .map(|s| s.trim().trim_matches('"').to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                };

                if options.is_empty() {
                    return Err(Box::new(EvalAltResult::ErrorRuntime(
                        "Menu requires at least one option".into(),
                        rhai::Position::NONE,
                    )));
                }

                trace!("HEAR {} AS MENU with options: {:?}", variable_name, options);

                let state_for_spawn = Arc::clone(&state_clone);
                let session_id_clone = session_id;
                let var_name_clone = variable_name.clone();
                let options_clone = options.clone();

                tokio::spawn(async move {
                    let mut session_manager = state_for_spawn.session_manager.lock().await;
                    session_manager.mark_waiting(session_id_clone);

                    // Store menu options in Redis
                    if let Some(redis_client) = &state_for_spawn.cache {
                        if let Ok(mut conn) = redis_client.get_multiplexed_async_connection().await
                        {
                            let key = format!("hear:{}:{}", session_id_clone, var_name_clone);
                            let wait_data = serde_json::json!({
                                "variable": var_name_clone,
                                "type": "menu",
                                "options": options_clone,
                                "waiting": true,
                                "retry_count": 0
                            });
                            let _: Result<(), _> = redis::cmd("SET")
                                .arg(&key)
                                .arg(wait_data.to_string())
                                .arg("EX")
                                .arg(3600)
                                .query_async(&mut conn)
                                .await;

                            // Also add suggestions for the menu
                            let suggestions_key =
                                format!("suggestions:{}:{}", session_id_clone, session_id_clone);
                            for opt in &options_clone {
                                let suggestion = serde_json::json!({
                                    "text": opt,
                                    "value": opt
                                });
                                let _: Result<(), _> = redis::cmd("RPUSH")
                                    .arg(&suggestions_key)
                                    .arg(suggestion.to_string())
                                    .query_async(&mut conn)
                                    .await;
                            }
                        }
                    }
                });

                Err(Box::new(EvalAltResult::ErrorRuntime(
                    "Waiting for user input".into(),
                    rhai::Position::NONE,
                )))
            },
        )
        .unwrap();
}

// ============================================================================
// Validation Functions
// ============================================================================

/// Validate input based on type
pub fn validate_input(input: &str, input_type: &InputType) -> ValidationResult {
    let trimmed = input.trim();

    match input_type {
        InputType::Any => ValidationResult::valid(trimmed.to_string()),

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

        // Media types are validated by checking attachment presence
        InputType::QrCode
        | InputType::File
        | InputType::Image
        | InputType::Audio
        | InputType::Video
        | InputType::Document => ValidationResult::valid(trimmed.to_string()),

        InputType::Login => ValidationResult::valid(trimmed.to_string()),
    }
}

fn validate_email(input: &str) -> ValidationResult {
    let email_regex = Regex::new(
        r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$"
    ).unwrap();

    if email_regex.is_match(input) {
        ValidationResult::valid(input.to_lowercase())
    } else {
        ValidationResult::invalid(InputType::Email.error_message())
    }
}

fn validate_date(input: &str) -> ValidationResult {
    // Try multiple date formats
    let formats = [
        "%d/%m/%Y", // 25/12/2024
        "%d-%m-%Y", // 25-12-2024
        "%Y-%m-%d", // 2024-12-25
        "%Y/%m/%d", // 2024/12/25
        "%d.%m.%Y", // 25.12.2024
        "%m/%d/%Y", // 12/25/2024 (US format)
        "%d %b %Y", // 25 Dec 2024
        "%d %B %Y", // 25 December 2024
    ];

    for format in &formats {
        if let Ok(date) = chrono::NaiveDate::parse_from_str(input, format) {
            // Return in ISO format
            return ValidationResult::valid_with_metadata(
                date.format("%Y-%m-%d").to_string(),
                serde_json::json!({
                    "original": input,
                    "parsed_format": format
                }),
            );
        }
    }

    // Try natural language parsing
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
    // Name should contain only letters, spaces, hyphens, and apostrophes
    let name_regex = Regex::new(r"^[\p{L}\s\-']+$").unwrap();

    if input.len() < 2 {
        return ValidationResult::invalid("Name must be at least 2 characters".to_string());
    }

    if input.len() > 100 {
        return ValidationResult::invalid("Name is too long".to_string());
    }

    if name_regex.is_match(input) {
        // Normalize: capitalize first letter of each word
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
    // Remove common formatting
    let cleaned = input
        .replace(",", "")
        .replace(".", "")
        .replace(" ", "")
        .trim()
        .to_string();

    match cleaned.parse::<i64>() {
        Ok(num) => ValidationResult::valid_with_metadata(
            num.to_string(),
            serde_json::json!({ "value": num }),
        ),
        Err(_) => ValidationResult::invalid(InputType::Integer.error_message()),
    }
}

fn validate_float(input: &str) -> ValidationResult {
    // Handle both . and , as decimal separator
    let cleaned = input.replace(" ", "").replace(",", ".").trim().to_string();

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
    // Try 24-hour format
    let time_24_regex = Regex::new(r"^([01]?\d|2[0-3]):([0-5]\d)$").unwrap();
    if let Some(caps) = time_24_regex.captures(input) {
        let hour: u32 = caps[1].parse().unwrap();
        let minute: u32 = caps[2].parse().unwrap();
        return ValidationResult::valid_with_metadata(
            format!("{:02}:{:02}", hour, minute),
            serde_json::json!({ "hour": hour, "minute": minute }),
        );
    }

    // Try 12-hour format with AM/PM
    let time_12_regex =
        Regex::new(r"^(1[0-2]|0?[1-9]):([0-5]\d)\s*(AM|PM|am|pm|a\.m\.|p\.m\.)$").unwrap();
    if let Some(caps) = time_12_regex.captures(input) {
        let mut hour: u32 = caps[1].parse().unwrap();
        let minute: u32 = caps[2].parse().unwrap();
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
    // Remove currency symbols and normalize
    let cleaned = input
        .replace("R$", "")
        .replace("$", "")
        .replace("€", "")
        .replace("£", "")
        .replace("¥", "")
        .replace(" ", "")
        .trim()
        .to_string();

    // Handle Brazilian format (1.234,56) vs US format (1,234.56)
    let normalized = if cleaned.contains(',') && cleaned.contains('.') {
        // Check which is the decimal separator (the last one)
        let last_comma = cleaned.rfind(',').unwrap_or(0);
        let last_dot = cleaned.rfind('.').unwrap_or(0);

        if last_comma > last_dot {
            // Brazilian format: 1.234,56
            cleaned.replace(".", "").replace(",", ".")
        } else {
            // US format: 1,234.56
            cleaned.replace(",", "")
        }
    } else if cleaned.contains(',') {
        // Only comma - likely decimal separator
        cleaned.replace(",", ".")
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
    // Remove all non-digits
    let digits: String = input.chars().filter(|c| c.is_ascii_digit()).collect();

    // Check length (most mobile numbers are 10-15 digits)
    if digits.len() < 10 || digits.len() > 15 {
        return ValidationResult::invalid(InputType::Mobile.error_message());
    }

    // Format for display
    let formatted = if digits.len() == 11 && digits.starts_with('9') {
        // Brazilian mobile: 9XXXX-XXXX
        format!("({}) {}-{}", &digits[0..2], &digits[2..7], &digits[7..11])
    } else if digits.len() == 11 {
        // Brazilian mobile with area code: (XX) 9XXXX-XXXX
        format!("({}) {}-{}", &digits[0..2], &digits[2..7], &digits[7..11])
    } else if digits.len() == 10 {
        // US format: (XXX) XXX-XXXX
        format!("({}) {}-{}", &digits[0..3], &digits[3..6], &digits[6..10])
    } else {
        // International format
        format!("+{}", digits)
    };

    ValidationResult::valid_with_metadata(
        formatted,
        serde_json::json!({ "digits": digits, "formatted": formatted }),
    )
}

fn validate_zipcode(input: &str) -> ValidationResult {
    // Remove all non-alphanumeric
    let cleaned: String = input
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect();

    // Brazilian CEP (8 digits)
    if cleaned.len() == 8 && cleaned.chars().all(|c| c.is_ascii_digit()) {
        let formatted = format!("{}-{}", &cleaned[0..5], &cleaned[5..8]);
        return ValidationResult::valid_with_metadata(
            formatted.clone(),
            serde_json::json!({ "digits": cleaned, "formatted": formatted, "country": "BR" }),
        );
    }

    // US ZIP (5 or 9 digits)
    if (cleaned.len() == 5 || cleaned.len() == 9) && cleaned.chars().all(|c| c.is_ascii_digit()) {
        let formatted = if cleaned.len() == 9 {
            format!("{}-{}", &cleaned[0..5], &cleaned[5..9])
        } else {
            cleaned.clone()
        };
        return ValidationResult::valid_with_metadata(
            formatted.clone(),
            serde_json::json!({ "digits": cleaned, "formatted": formatted, "country": "US" }),
        );
    }

    // UK postcode (alphanumeric, 5-7 chars)
    let uk_regex = Regex::new(r"^[A-Z]{1,2}\d[A-Z\d]?\s?\d[A-Z]{2}$").unwrap();
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

    // Common language codes and names
    let languages = [
        ("en", "english", "inglês", "ingles"),
        ("pt", "portuguese", "português", "portugues"),
        ("es", "spanish", "espanhol", "español"),
        ("fr", "french", "francês", "frances"),
        ("de", "german", "alemão", "alemao"),
        ("it", "italian", "italiano"),
        ("ja", "japanese", "japonês", "japones"),
        ("zh", "chinese", "chinês", "chines"),
        ("ko", "korean", "coreano"),
        ("ru", "russian", "russo"),
        ("ar", "arabic", "árabe", "arabe"),
        ("hi", "hindi"),
        ("nl", "dutch", "holandês", "holandes"),
        ("pl", "polish", "polonês", "polones"),
        ("tr", "turkish", "turco"),
    ];

    for (code, variants @ ..) in &languages {
        if lower == *code || variants.iter().any(|v| lower == *v) {
            return ValidationResult::valid_with_metadata(
                code.to_string(),
                serde_json::json!({ "code": code, "input": input }),
            );
        }
    }

    // Check if it's a valid ISO 639-1 code (2 letters)
    if lower.len() == 2 && lower.chars().all(|c| c.is_ascii_lowercase()) {
        return ValidationResult::valid(lower);
    }

    ValidationResult::invalid(InputType::Language.error_message())
}

fn validate_cpf(input: &str) -> ValidationResult {
    // Remove non-digits
    let digits: String = input.chars().filter(|c| c.is_ascii_digit()).collect();

    if digits.len() != 11 {
        return ValidationResult::invalid(InputType::Cpf.error_message());
    }

    // Check for known invalid patterns (all same digit)
    if digits.chars().all(|c| c == digits.chars().next().unwrap()) {
        return ValidationResult::invalid("Invalid CPF".to_string());
    }

    // Validate check digits
    let digits_vec: Vec<u32> = digits.chars().map(|c| c.to_digit(10).unwrap()).collect();

    // First check digit
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

    // Second check digit
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

    // Format: XXX.XXX.XXX-XX
    let formatted = format!(
        "{}.{}.{}-{}",
        &digits[0..3],
        &digits[3..6],
        &digits[6..9],
        &digits[9..11]
    );

    ValidationResult::valid_with_metadata(
        formatted.clone(),
        serde_json::json!({ "digits": digits, "formatted": formatted }),
    )
}

fn validate_cnpj(input: &str) -> ValidationResult {
    // Remove non-digits
    let digits: String = input.chars().filter(|c| c.is_ascii_digit()).collect();

    if digits.len() != 14 {
        return ValidationResult::invalid(InputType::Cnpj.error_message());
    }

    // Validate check digits
    let digits_vec: Vec<u32> = digits.chars().map(|c| c.to_digit(10).unwrap()).collect();

    // First check digit
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

    // Second check digit
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

    // Format: XX.XXX.XXX/XXXX-XX
    let formatted = format!(
        "{}.{}.{}/{}-{}",
        &digits[0..2],
        &digits[2..5],
        &digits[5..8],
        &digits[8..12],
        &digits[12..14]
    );

    ValidationResult::valid_with_metadata(
        formatted.clone(),
        serde_json::json!({ "digits": digits, "formatted": formatted }),
    )
}

fn validate_url(input: &str) -> ValidationResult {
    // Add protocol if missing
    let url_str = if !input.starts_with("http://") && !input.starts_with("https://") {
        format!("https://{}", input)
    } else {
        input.to_string()
    };

    let url_regex = Regex::new(
        r"^https?://[a-zA-Z0-9][-a-zA-Z0-9]*(\.[a-zA-Z0-9][-a-zA-Z0-9]*)+(/[-a-zA-Z0-9()@:%_\+.~#?&/=]*)?$"
    ).unwrap();

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

    // Named colors
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
                hex.to_string(),
                serde_json::json!({ "name": name, "hex": hex }),
            );
        }
    }

    // Hex color
    let hex_regex = Regex::new(r"^#?([A-Fa-f0-9]{6}|[A-Fa-f0-9]{3})$").unwrap();
    if let Some(caps) = hex_regex.captures(&lower) {
        let hex = caps[1].to_uppercase();
        let full_hex = if hex.len() == 3 {
            hex.chars()
                .map(|c| format!("{}{}", c, c))
                .collect::<String>()
        } else {
            hex
        };
        return ValidationResult::valid(format!("#{}", full_hex));
    }

    // RGB format
    let rgb_regex =
        Regex::new(r"^rgb\s*\(\s*(\d{1,3})\s*,\s*(\d{1,3})\s*,\s*(\d{1,3})\s*\)$").unwrap();
    if let Some(caps) = rgb_regex.captures(&lower) {
        let r: u8 = caps[1].parse().unwrap_or(0);
        let g: u8 = caps[2].parse().unwrap_or(0);
        let b: u8 = caps[3].parse().unwrap_or(0);
        return ValidationResult::valid(format!("#{:02X}{:02X}{:02X}", r, g, b));
    }

    ValidationResult::invalid(InputType::Color.error_message())
}

fn validate_credit_card(input: &str) -> ValidationResult {
    // Remove spaces and dashes
    let digits: String = input.chars().filter(|c| c.is_ascii_digit()).collect();

    if digits.len() < 13 || digits.len() > 19 {
        return ValidationResult::invalid(InputType::CreditCard.error_message());
    }

    // Luhn algorithm
    let mut sum = 0;
    let mut double = false;

    for c in digits.chars().rev() {
        let mut digit = c.to_digit(10).unwrap();
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

    // Detect card type
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

    // Mask middle digits
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

    // Don't return the actual password, just confirmation
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

    // Try exact match first
    for (i, opt) in options.iter().enumerate() {
        if opt.to_lowercase() == lower_input {
            return ValidationResult::valid_with_metadata(
                opt.clone(),
                serde_json::json!({ "index": i, "value": opt }),
            );
        }
    }

    // Try numeric selection (1, 2, 3...)
    if let Ok(num) = lower_input.parse::<usize>() {
        if num >= 1 && num <= options.len() {
            let selected = &options[num - 1];
            return ValidationResult::valid_with_metadata(
                selected.clone(),
                serde_json::json!({ "index": num - 1, "value": selected }),
            );
        }
    }

    // Try partial match
    let matches: Vec<&String> = options
        .iter()
        .filter(|opt| opt.to_lowercase().contains(&lower_input))
        .collect();

    if matches.len() == 1 {
        let idx = options.iter().position(|o| o == *matches[0]).unwrap();
        return ValidationResult::valid_with_metadata(
            matches[0].clone(),
            serde_json::json!({ "index": idx, "value": matches[0] }),
        );
    }

    ValidationResult::invalid(format!("Please select one of: {}", options.join(", ")))
}

// ============================================================================
// TALK Keyword
// ============================================================================

pub async fn execute_talk(
    state: Arc<AppState>,
    user_session: UserSession,
    message: String,
) -> Result<BotResponse, Box<dyn std::error::Error + Send + Sync>> {
    let mut suggestions = Vec::new();

    // Load suggestions from Redis
    if let Some(redis_client) = &state.cache {
        if let Ok(mut conn) = redis_client.get_multiplexed_async_connection().await {
            let redis_key = format!("suggestions:{}:{}", user_session.user_id, user_session.id);

            let suggestions_json: Result<Vec<String>, _> = redis::cmd("LRANGE")
                .arg(redis_key.as_str())
                .arg(0)
                .arg(-1)
                .query_async(&mut conn)
                .await;

            if let Ok(suggestions_list) = suggestions_json {
                suggestions = suggestions_list
                    .into_iter()
                    .filter_map(|s| serde_json::from_str(&s).ok())
                    .collect();
            }
        }
    }

    let response = BotResponse {
        bot_id: user_session.bot_id.to_string(),
        user_id: user_session.user_id.to_string(),
        session_id: user_session.id.to_string(),
        channel: "web".to_string(),
        content: message,
        message_type: MessageType::USER,
        stream_token: None,
        is_complete: true,
        suggestions,
        context_name: None,
        context_length: 0,
        context_max_length: 0,
    };

    let user_id = user_session.id.to_string();
    let response_clone = response.clone();

    // Send via web adapter
    let web_adapter = Arc::clone(&state.web_adapter);
    tokio::spawn(async move {
        if let Err(e) = web_adapter
            .send_message_to_session(&user_id, response_clone)
            .await
        {
            error!("Failed to send TALK message via web adapter: {}", e);
        } else {
            trace!("TALK message sent via web adapter");
        }
    });

    Ok(response)
}

pub fn talk_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine
        .register_custom_syntax(&["TALK", "$expr$"], true, move |context, inputs| {
            let message = context.eval_expression_tree(&inputs[0])?.to_string();
            let state_for_talk = Arc::clone(&state_clone);
            let user_for_talk = user_clone.clone();

            tokio::spawn(async move {
                if let Err(e) = execute_talk(state_for_talk, user_for_talk, message).await {
                    error!("Error executing TALK command: {}", e);
                }
            });

            Ok(Dynamic::UNIT)
        })
        .unwrap();
}

// ============================================================================
// Input Processing (called when user sends message)
// ============================================================================

/// Process user input with validation
pub async fn process_hear_input(
    state: &AppState,
    session_id: Uuid,
    variable_name: &str,
    input: &str,
    attachments: Option<Vec<crate::shared::models::Attachment>>,
) -> Result<(String, Option<serde_json::Value>), String> {
    // Get wait state from Redis
    let wait_data = if let Some(redis_client) = &state.cache {
        if let Ok(mut conn) = redis_client.get_multiplexed_async_connection().await {
            let key = format!("hear:{}:{}", session_id, variable_name);

            let data: Result<String, _> = redis::cmd("GET").arg(&key).query_async(&mut conn).await;

            match data {
                Ok(json_str) => serde_json::from_str::<serde_json::Value>(&json_str).ok(),
                Err(_) => None,
            }
        } else {
            None
        }
    } else {
        None
    };

    let input_type = wait_data
        .as_ref()
        .and_then(|d| d.get("type"))
        .and_then(|t| t.as_str())
        .unwrap_or("any");

    let options = wait_data
        .as_ref()
        .and_then(|d| d.get("options"))
        .and_then(|o| o.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect::<Vec<_>>()
        });

    // Determine the validation type
    let validation_type = if let Some(opts) = options {
        InputType::Menu(opts)
    } else {
        InputType::from_str(input_type)
    };

    // Handle media types with attachments
    match validation_type {
        InputType::Image | InputType::QrCode => {
            if let Some(atts) = &attachments {
                if let Some(img) = atts.iter().find(|a| a.content_type.starts_with("image/")) {
                    if validation_type == InputType::QrCode {
                        // Call botmodels to read QR code
                        return process_qrcode(state, &img.url).await;
                    }
                    return Ok((
                        img.url.clone(),
                        Some(serde_json::json!({ "attachment": img })),
                    ));
                }
            }
            return Err(validation_type.error_message());
        }
        InputType::Audio => {
            if let Some(atts) = &attachments {
                if let Some(audio) = atts.iter().find(|a| a.content_type.starts_with("audio/")) {
                    // Call botmodels for speech-to-text
                    return process_audio_to_text(state, &audio.url).await;
                }
            }
            return Err(validation_type.error_message());
        }
        InputType::Video => {
            if let Some(atts) = &attachments {
                if let Some(video) = atts.iter().find(|a| a.content_type.starts_with("video/")) {
                    // Call botmodels for video description
                    return process_video_description(state, &video.url).await;
                }
            }
            return Err(validation_type.error_message());
        }
        InputType::File | InputType::Document => {
            if let Some(atts) = &attachments {
                if let Some(doc) = atts.first() {
                    return Ok((
                        doc.url.clone(),
                        Some(serde_json::json!({ "attachment": doc })),
                    ));
                }
            }
            return Err(validation_type.error_message());
        }
        _ => {}
    }

    // Validate text input
    let result = validate_input(input, &validation_type);

    if result.is_valid {
        // Clear the wait state
        if let Some(redis_client) = &state.cache {
            if let Ok(mut conn) = redis_client.get_multiplexed_async_connection().await {
                let key = format!("hear:{}:{}", session_id, variable_name);
                let _: Result<(), _> = redis::cmd("DEL").arg(&key).query_async(&mut conn).await;
            }
        }

        Ok((result.normalized_value, result.metadata))
    } else {
        Err(result
            .error_message
            .unwrap_or_else(|| validation_type.error_message()))
    }
}

/// Process QR code from image using botmodels
async fn process_qrcode(
    state: &AppState,
    image_url: &str,
) -> Result<(String, Option<serde_json::Value>), String> {
    // Call botmodels vision service
    let botmodels_url = state
        .config
        .get("botmodels-url")
        .unwrap_or_else(|| "http://localhost:8001".to_string());

    let client = reqwest::Client::new();

    // Download image
    let image_data = client
        .get(image_url)
        .send()
        .await
        .map_err(|e| format!("Failed to download image: {}", e))?
        .bytes()
        .await
        .map_err(|e| format!("Failed to read image: {}", e))?;

    // Send to botmodels for QR code reading
    let response = client
        .post(format!("{}/api/v1/vision/qrcode", botmodels_url))
        .header("Content-Type", "application/octet-stream")
        .body(image_data.to_vec())
        .send()
        .await
        .map_err(|e| format!("Failed to call botmodels: {}", e))?;

    if response.status().is_success() {
        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        if let Some(qr_data) = result.get("data").and_then(|d| d.as_str()) {
            Ok((
                qr_data.to_string(),
                Some(serde_json::json!({
                    "type": "qrcode",
                    "raw": result
                })),
            ))
        } else {
            Err("No QR code found in image".to_string())
        }
    } else {
        Err("Failed to read QR code".to_string())
    }
}

/// Process audio to text using botmodels
async fn process_audio_to_text(
    state: &AppState,
    audio_url: &str,
) -> Result<(String, Option<serde_json::Value>), String> {
    let botmodels_url = state
        .config
        .get("botmodels-url")
        .unwrap_or_else(|| "http://localhost:8001".to_string());

    let client = reqwest::Client::new();

    // Download audio
    let audio_data = client
        .get(audio_url)
        .send()
        .await
        .map_err(|e| format!("Failed to download audio: {}", e))?
        .bytes()
        .await
        .map_err(|e| format!("Failed to read audio: {}", e))?;

    // Send to botmodels for speech-to-text
    let response = client
        .post(format!("{}/api/v1/speech/to-text", botmodels_url))
        .header("Content-Type", "application/octet-stream")
        .body(audio_data.to_vec())
        .send()
        .await
        .map_err(|e| format!("Failed to call botmodels: {}", e))?;

    if response.status().is_success() {
        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        if let Some(text) = result.get("text").and_then(|t| t.as_str()) {
            Ok((
                text.to_string(),
                Some(serde_json::json!({
                    "type": "audio_transcription",
                    "language": result.get("language"),
                    "confidence": result.get("confidence")
                })),
            ))
        } else {
            Err("Could not transcribe audio".to_string())
        }
    } else {
        Err("Failed to process audio".to_string())
    }
}

/// Process video description using botmodels
async fn process_video_description(
    state: &AppState,
    video_url: &str,
) -> Result<(String, Option<serde_json::Value>), String> {
    let botmodels_url = state
        .config
        .get("botmodels-url")
        .unwrap_or_else(|| "http://localhost:8001".to_string());

    let client = reqwest::Client::new();

    // Download video
    let video_data = client
        .get(video_url)
        .send()
        .await
        .map_err(|e| format!("Failed to download video: {}", e))?
        .bytes()
        .await
        .map_err(|e| format!("Failed to read video: {}", e))?;

    // Send to botmodels for video description
    let response = client
        .post(format!("{}/api/v1/vision/describe-video", botmodels_url))
        .header("Content-Type", "application/octet-stream")
        .body(video_data.to_vec())
        .send()
        .await
        .map_err(|e| format!("Failed to call botmodels: {}", e))?;

    if response.status().is_success() {
        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        if let Some(description) = result.get("description").and_then(|d| d.as_str()) {
            Ok((
                description.to_string(),
                Some(serde_json::json!({
                    "type": "video_description",
                    "frame_count": result.get("frame_count"),
                    "url": video_url
                })),
            ))
        } else {
            Err("Could not describe video".to_string())
        }
    } else {
        Err("Failed to process video".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_email() {
        assert!(validate_email("test@example.com").is_valid);
        assert!(validate_email("user.name+tag@domain.co.uk").is_valid);
        assert!(!validate_email("invalid").is_valid);
        assert!(!validate_email("@nodomain.com").is_valid);
    }

    #[test]
    fn test_validate_date() {
        assert!(validate_date("25/12/2024").is_valid);
        assert!(validate_date("2024-12-25").is_valid);
        assert!(validate_date("today").is_valid);
        assert!(validate_date("tomorrow").is_valid);
        assert!(!validate_date("invalid").is_valid);
    }

    #[test]
    fn test_validate_cpf() {
        assert!(validate_cpf("529.982.247-25").is_valid);
        assert!(validate_cpf("52998224725").is_valid);
        assert!(!validate_cpf("111.111.111-11").is_valid);
        assert!(!validate_cpf("123").is_valid);
    }

    #[test]
    fn test_validate_money() {
        let result = validate_money("R$ 1.234,56");
        assert!(result.is_valid);
        assert_eq!(result.normalized_value, "1234.56");

        let result = validate_money("$1,234.56");
        assert!(result.is_valid);
        assert_eq!(result.normalized_value, "1234.56");
    }

    #[test]
    fn test_validate_boolean() {
        assert!(validate_boolean("yes").is_valid);
        assert!(validate_boolean("sim").is_valid);
        assert!(validate_boolean("no").is_valid);
        assert!(validate_boolean("não").is_valid);
        assert!(!validate_boolean("maybe").is_valid);
    }

    #[test]
    fn test_validate_menu() {
        let options = vec![
            "Apple".to_string(),
            "Banana".to_string(),
            "Cherry".to_string(),
        ];

        assert!(validate_menu("Apple", &options).is_valid);
        assert!(validate_menu("1", &options).is_valid);
        assert!(validate_menu("ban", &options).is_valid); // Partial match
        assert!(!validate_menu("Orange", &options).is_valid);
    }

    #[test]
    fn test_validate_credit_card() {
        // Valid Visa test number
        assert!(validate_credit_card("4111 1111 1111 1111").is_valid);
        // Invalid (fails Luhn)
        assert!(!validate_credit_card("1234567890123456").is_valid);
    }
}
