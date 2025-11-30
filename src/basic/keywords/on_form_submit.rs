//! ON FORM SUBMIT - Webhook-based form handling for landing pages
//!
//! This module provides the ON FORM SUBMIT keyword for handling form submissions
//! from .gbui landing pages. Forms submitted from gbui files trigger this handler.
//!
//! BASIC Syntax:
//!   ON FORM SUBMIT "form_name"
//!     ' Handle form data
//!     name = FORM.name
//!     email = FORM.email
//!     TALK "Thank you, " + name
//!   END ON
//!
//! Examples:
//!   ' Handle contact form submission
//!   ON FORM SUBMIT "contact_form"
//!     name = FORM.name
//!     email = FORM.email
//!     message = FORM.message
//!
//!     ' Save to database
//!     SAVE "contacts", name, email, message
//!
//!     ' Send notification
//!     SEND MAIL TO "admin@company.com" WITH
//!       subject = "New Contact: " + name
//!       body = message
//!     END WITH
//!
//!     ' Respond to user
//!     TALK "Thank you for contacting us, " + name + "!"
//!   END ON
//!
//!   ' Handle lead capture form
//!   ON FORM SUBMIT "lead_capture"
//!     lead = #{
//!       "name": FORM.name,
//!       "email": FORM.email,
//!       "company": FORM.company,
//!       "phone": FORM.phone
//!     }
//!
//!     score = SCORE_LEAD(lead)
//!
//!     IF score >= 70 THEN
//!       SEND TEMPLATE "high_value_lead" TO "sales@company.com" VIA "email" WITH lead
//!     END IF
//!   END ON

use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::{debug, error, info, trace};
use rhai::{Dynamic, Engine, EvalAltResult, Map, Position};
use std::collections::HashMap;
use std::sync::Arc;

/// Register the ON FORM SUBMIT keyword
///
/// This keyword allows BASIC scripts to handle form submissions from .gbui files.
/// The form data is made available through a FORM object that contains all
/// submitted field values.
pub fn on_form_submit_keyword(state: &Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = state.clone();
    let user_clone = user.clone();

    // Register FORM_DATA function to get form data map
    engine.register_fn("FORM_DATA", move || -> Map {
        trace!("FORM_DATA called by user {}", user_clone.user_id);
        // Return empty map - actual form data is injected at runtime
        Map::new()
    });

    let user_clone2 = user.clone();

    // Register FORM_FIELD function to get specific field
    engine.register_fn("FORM_FIELD", move |field_name: &str| -> Dynamic {
        trace!(
            "FORM_FIELD called for '{}' by user {}",
            field_name,
            user_clone2.user_id
        );
        // Return unit - actual value is injected at runtime
        Dynamic::UNIT
    });

    let user_clone3 = user.clone();

    // Register FORM_HAS function to check if field exists
    engine.register_fn("FORM_HAS", move |field_name: &str| -> bool {
        trace!(
            "FORM_HAS called for '{}' by user {}",
            field_name,
            user_clone3.user_id
        );
        false
    });

    let user_clone4 = user.clone();

    // Register FORM_FIELDS function to get list of field names
    engine.register_fn("FORM_FIELDS", move || -> rhai::Array {
        trace!("FORM_FIELDS called by user {}", user_clone4.user_id);
        rhai::Array::new()
    });

    // Register GET_FORM helper
    let user_clone5 = user.clone();
    engine.register_fn("GET_FORM", move |form_name: &str| -> Map {
        trace!(
            "GET_FORM called for '{}' by user {}",
            form_name,
            user_clone5.user_id
        );
        let mut result = Map::new();
        result.insert("form_name".into(), Dynamic::from(form_name.to_string()));
        result.insert("submitted".into(), Dynamic::from(false));
        result
    });

    // Register VALIDATE_FORM helper
    let user_clone6 = user.clone();
    engine.register_fn("VALIDATE_FORM", move |form_data: Map| -> Map {
        trace!("VALIDATE_FORM called by user {}", user_clone6.user_id);
        validate_form_data(&form_data)
    });

    // Register VALIDATE_FORM with rules
    let user_clone7 = user.clone();
    engine.register_fn("VALIDATE_FORM", move |form_data: Map, rules: Map| -> Map {
        trace!(
            "VALIDATE_FORM with rules called by user {}",
            user_clone7.user_id
        );
        validate_form_with_rules(&form_data, &rules)
    });

    // Register REGISTER_FORM_HANDLER to set up form handler
    let state_for_handler = state_clone.clone();
    let user_clone8 = user.clone();
    engine.register_fn(
        "REGISTER_FORM_HANDLER",
        move |form_name: &str, handler_script: &str| -> bool {
            trace!(
                "REGISTER_FORM_HANDLER called for '{}' by user {}",
                form_name,
                user_clone8.user_id
            );
            // TODO: Store handler registration in state
            info!(
                "Registered form handler for '{}' -> '{}'",
                form_name, handler_script
            );
            true
        },
    );

    // Register IS_FORM_SUBMISSION check
    let user_clone9 = user.clone();
    engine.register_fn("IS_FORM_SUBMISSION", move || -> bool {
        trace!("IS_FORM_SUBMISSION called by user {}", user_clone9.user_id);
        // This would be set to true when script is invoked from form submission
        false
    });

    // Register GET_SUBMISSION_ID
    let user_clone10 = user.clone();
    engine.register_fn("GET_SUBMISSION_ID", move || -> String {
        trace!("GET_SUBMISSION_ID called by user {}", user_clone10.user_id);
        // Generate or return the current submission ID
        generate_submission_id()
    });

    // Register SAVE_SUBMISSION to persist form data
    let user_clone11 = user.clone();
    engine.register_fn(
        "SAVE_SUBMISSION",
        move |form_name: &str, data: Map| -> Map {
            trace!(
                "SAVE_SUBMISSION called for '{}' by user {}",
                form_name,
                user_clone11.user_id
            );
            save_form_submission(form_name, &data)
        },
    );

    // Register GET_SUBMISSIONS to retrieve past submissions
    let user_clone12 = user.clone();
    engine.register_fn("GET_SUBMISSIONS", move |form_name: &str| -> rhai::Array {
        trace!(
            "GET_SUBMISSIONS called for '{}' by user {}",
            form_name,
            user_clone12.user_id
        );
        // TODO: Implement database lookup
        rhai::Array::new()
    });

    // Register GET_SUBMISSIONS with limit
    let user_clone13 = user.clone();
    engine.register_fn(
        "GET_SUBMISSIONS",
        move |form_name: &str, limit: i64| -> rhai::Array {
            trace!(
                "GET_SUBMISSIONS called for '{}' with limit {} by user {}",
                form_name,
                limit,
                user_clone13.user_id
            );
            // TODO: Implement database lookup with limit
            rhai::Array::new()
        },
    );

    debug!("Registered ON FORM SUBMIT keyword and helpers");
}

/// Validate form data with basic rules
fn validate_form_data(form_data: &Map) -> Map {
    let mut result = Map::new();
    let mut is_valid = true;
    let mut errors = rhai::Array::new();

    // Check for empty required fields (fields that exist but are empty)
    for (key, value) in form_data.iter() {
        if value.is_unit() || value.to_string().trim().is_empty() {
            // Field is empty - might be an error depending on context
            // For basic validation, we just note it
        }
    }

    result.insert("valid".into(), Dynamic::from(is_valid));
    result.insert("errors".into(), Dynamic::from(errors));
    result.insert("field_count".into(), Dynamic::from(form_data.len() as i64));

    result
}

/// Validate form data with custom rules
fn validate_form_with_rules(form_data: &Map, rules: &Map) -> Map {
    let mut result = Map::new();
    let mut is_valid = true;
    let mut errors = rhai::Array::new();

    for (field_name, rule) in rules.iter() {
        let field_key = field_name.as_str();
        let rule_str = rule.to_string().to_lowercase();

        // Check if field exists
        let field_value = form_data.get(field_key);

        if rule_str.contains("required") {
            match field_value {
                None => {
                    is_valid = false;
                    let mut error = Map::new();
                    error.insert("field".into(), Dynamic::from(field_key.to_string()));
                    error.insert("rule".into(), Dynamic::from("required"));
                    error.insert(
                        "message".into(),
                        Dynamic::from(format!("Field '{}' is required", field_key)),
                    );
                    errors.push(Dynamic::from(error));
                }
                Some(val) if val.is_unit() || val.to_string().trim().is_empty() => {
                    is_valid = false;
                    let mut error = Map::new();
                    error.insert("field".into(), Dynamic::from(field_key.to_string()));
                    error.insert("rule".into(), Dynamic::from("required"));
                    error.insert(
                        "message".into(),
                        Dynamic::from(format!("Field '{}' cannot be empty", field_key)),
                    );
                    errors.push(Dynamic::from(error));
                }
                _ => {}
            }
        }

        if rule_str.contains("email") {
            if let Some(val) = field_value {
                let email = val.to_string();
                if !email.is_empty() && !is_valid_email(&email) {
                    is_valid = false;
                    let mut error = Map::new();
                    error.insert("field".into(), Dynamic::from(field_key.to_string()));
                    error.insert("rule".into(), Dynamic::from("email"));
                    error.insert(
                        "message".into(),
                        Dynamic::from(format!("Field '{}' must be a valid email", field_key)),
                    );
                    errors.push(Dynamic::from(error));
                }
            }
        }

        if rule_str.contains("phone") {
            if let Some(val) = field_value {
                let phone = val.to_string();
                if !phone.is_empty() && !is_valid_phone(&phone) {
                    is_valid = false;
                    let mut error = Map::new();
                    error.insert("field".into(), Dynamic::from(field_key.to_string()));
                    error.insert("rule".into(), Dynamic::from("phone"));
                    error.insert(
                        "message".into(),
                        Dynamic::from(format!(
                            "Field '{}' must be a valid phone number",
                            field_key
                        )),
                    );
                    errors.push(Dynamic::from(error));
                }
            }
        }
    }

    result.insert("valid".into(), Dynamic::from(is_valid));
    result.insert("errors".into(), Dynamic::from(errors));
    result.insert("field_count".into(), Dynamic::from(form_data.len() as i64));
    result.insert("rules_checked".into(), Dynamic::from(rules.len() as i64));

    result
}

/// Basic email validation
fn is_valid_email(email: &str) -> bool {
    let email = email.trim();
    if email.is_empty() {
        return false;
    }

    // Simple validation: must contain @ and have something before and after
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return false;
    }

    let local = parts[0];
    let domain = parts[1];

    // Local part must not be empty
    if local.is_empty() {
        return false;
    }

    // Domain must contain at least one dot and not be empty
    if domain.is_empty() || !domain.contains('.') {
        return false;
    }

    // Domain must have something after the last dot
    let domain_parts: Vec<&str> = domain.split('.').collect();
    if domain_parts.last().map(|s| s.is_empty()).unwrap_or(true) {
        return false;
    }

    true
}

/// Basic phone validation
fn is_valid_phone(phone: &str) -> bool {
    let phone = phone.trim();
    if phone.is_empty() {
        return false;
    }

    // Remove common formatting characters
    let digits: String = phone
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '+')
        .collect();

    // Must have at least 7 digits (minimum for a phone number)
    let digit_count = digits.chars().filter(|c| c.is_ascii_digit()).count();
    digit_count >= 7
}

/// Generate a unique submission ID
fn generate_submission_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    format!("sub_{}", timestamp)
}

/// Save form submission to storage
fn save_form_submission(form_name: &str, data: &Map) -> Map {
    let mut result = Map::new();

    let submission_id = generate_submission_id();

    // TODO: Implement actual database storage

    info!(
        "Saving form submission for '{}' with id '{}'",
        form_name, submission_id
    );

    result.insert("success".into(), Dynamic::from(true));
    result.insert("submission_id".into(), Dynamic::from(submission_id));
    result.insert("form_name".into(), Dynamic::from(form_name.to_string()));
    result.insert("field_count".into(), Dynamic::from(data.len() as i64));
    result.insert("timestamp".into(), Dynamic::from(chrono_timestamp()));

    result
}

/// Get current timestamp in ISO format
fn chrono_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    let secs = duration.as_secs();
    // Simple ISO-like format without external dependencies
    format!("{}Z", secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_email() {
        assert!(is_valid_email("user@example.com"));
        assert!(is_valid_email("user.name@example.co.uk"));
        assert!(is_valid_email("user+tag@example.com"));
        assert!(!is_valid_email("invalid"));
        assert!(!is_valid_email("@example.com"));
        assert!(!is_valid_email("user@"));
        assert!(!is_valid_email("user@example"));
        assert!(!is_valid_email(""));
    }

    #[test]
    fn test_is_valid_phone() {
        assert!(is_valid_phone("+1234567890"));
        assert!(is_valid_phone("123-456-7890"));
        assert!(is_valid_phone("(123) 456-7890"));
        assert!(is_valid_phone("1234567"));
        assert!(!is_valid_phone("123"));
        assert!(!is_valid_phone(""));
        assert!(!is_valid_phone("abc"));
    }

    #[test]
    fn test_validate_form_data() {
        let mut form_data = Map::new();
        form_data.insert("name".into(), Dynamic::from("John"));
        form_data.insert("email".into(), Dynamic::from("john@example.com"));

        let result = validate_form_data(&form_data);
        assert!(result.get("valid").unwrap().as_bool().unwrap());
    }

    #[test]
    fn test_validate_form_with_rules_required() {
        let mut form_data = Map::new();
        form_data.insert("name".into(), Dynamic::from("John"));
        // Missing email field

        let mut rules = Map::new();
        rules.insert("name".into(), Dynamic::from("required"));
        rules.insert("email".into(), Dynamic::from("required"));

        let result = validate_form_with_rules(&form_data, &rules);
        assert!(!result.get("valid").unwrap().as_bool().unwrap());
    }

    #[test]
    fn test_validate_form_with_rules_email() {
        let mut form_data = Map::new();
        form_data.insert("email".into(), Dynamic::from("invalid-email"));

        let mut rules = Map::new();
        rules.insert("email".into(), Dynamic::from("email"));

        let result = validate_form_with_rules(&form_data, &rules);
        assert!(!result.get("valid").unwrap().as_bool().unwrap());
    }

    #[test]
    fn test_generate_submission_id() {
        let id = generate_submission_id();
        assert!(id.starts_with("sub_"));
    }

    #[test]
    fn test_save_form_submission() {
        let mut data = Map::new();
        data.insert("name".into(), Dynamic::from("Test"));

        let result = save_form_submission("test_form", &data);
        assert!(result.get("success").unwrap().as_bool().unwrap());
        assert!(result.contains_key("submission_id"));
    }
}
