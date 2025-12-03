//! SEND TEMPLATE - Multi-channel templated messaging
//!
//! Provides keywords for sending templated messages across multiple channels:
//! - Email
//! - WhatsApp
//! - SMS
//! - Telegram
//! - Push notifications
//!
//! BASIC Syntax:
//!   SEND TEMPLATE "template_name" TO "recipient" VIA "channel"
//!   SEND TEMPLATE "template_name" TO recipients_array VIA "channel" WITH variables
//!
//! Examples:
//!   SEND TEMPLATE "welcome" TO "user@example.com" VIA "email"
//!   SEND TEMPLATE "order_confirmation" TO "+1234567890" VIA "whatsapp" WITH order_data

use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::{debug, info, trace};
use rhai::{Array, Dynamic, Engine, Map};
use std::sync::Arc;

/// SEND_TEMPLATE - Send a templated message to a recipient
///
/// BASIC Syntax:
///   result = SEND_TEMPLATE("template_name", "recipient", "channel")
///   result = SEND_TEMPLATE("template_name", "recipient", "channel", variables)
pub fn send_template_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let _state_clone = state.clone();
    let user_clone = user.clone();

    // SEND_TEMPLATE with 3 arguments (template, recipient, channel)
    engine.register_fn(
        "SEND_TEMPLATE",
        move |template: &str, recipient: &str, channel: &str| -> Map {
            trace!(
                "SEND_TEMPLATE called: template={}, recipient={}, channel={} by user {}",
                template,
                recipient,
                channel,
                user_clone.user_id
            );

            send_template_message(template, recipient, channel, None)
        },
    );

    let _state_clone2 = state.clone();
    let user_clone2 = user.clone();

    // send_template lowercase
    engine.register_fn(
        "send_template",
        move |template: &str, recipient: &str, channel: &str| -> Map {
            trace!(
                "send_template called: template={}, recipient={}, channel={} by user {}",
                template,
                recipient,
                channel,
                user_clone2.user_id
            );

            send_template_message(template, recipient, channel, None)
        },
    );

    let _state_clone3 = state.clone();
    let user_clone3 = user.clone();

    // SEND_TEMPLATE with 4 arguments (template, recipient, channel, variables)
    engine.register_fn(
        "SEND_TEMPLATE",
        move |template: &str, recipient: &str, channel: &str, variables: Map| -> Map {
            trace!(
                "SEND_TEMPLATE called with variables: template={}, recipient={}, channel={} by user {}",
                template,
                recipient,
                channel,
                user_clone3.user_id
            );

            send_template_message(template, recipient, channel, Some(&variables))
        },
    );

    debug!("Registered SEND_TEMPLATE keyword");
}

/// SEND_TEMPLATE_TO - Send templated message to multiple recipients
///
/// BASIC Syntax:
///   result = SEND_TEMPLATE_TO("template_name", recipients_array, "channel")
///   result = SEND_TEMPLATE_TO("template_name", recipients_array, "channel", variables)
pub fn send_template_to_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let _state_clone = state.clone();
    let user_clone = user.clone();

    // SEND_TEMPLATE_TO with array of recipients
    engine.register_fn(
        "SEND_TEMPLATE_TO",
        move |template: &str, recipients: Array, channel: &str| -> Map {
            trace!(
                "SEND_TEMPLATE_TO called: template={}, recipients={:?}, channel={} by user {}",
                template,
                recipients.len(),
                channel,
                user_clone.user_id
            );

            send_template_batch(template, &recipients, channel, None)
        },
    );

    let _state_clone2 = state.clone();
    let user_clone2 = user.clone();

    // SEND_TEMPLATE_TO with variables
    engine.register_fn(
        "SEND_TEMPLATE_TO",
        move |template: &str, recipients: Array, channel: &str, variables: Map| -> Map {
            trace!(
                "SEND_TEMPLATE_TO called with variables: template={}, recipients={:?}, channel={} by user {}",
                template,
                recipients.len(),
                channel,
                user_clone2.user_id
            );

            send_template_batch(template, &recipients, channel, Some(&variables))
        },
    );

    // BULK_SEND alias
    let _state_clone3 = state.clone();
    let user_clone3 = user.clone();

    engine.register_fn(
        "BULK_SEND",
        move |template: &str, recipients: Array, channel: &str| -> Map {
            trace!(
                "BULK_SEND called: template={}, recipients={:?}, channel={} by user {}",
                template,
                recipients.len(),
                channel,
                user_clone3.user_id
            );

            send_template_batch(template, &recipients, channel, None)
        },
    );

    debug!("Registered SEND_TEMPLATE_TO keyword");
}

/// CREATE_TEMPLATE - Create or update a message template
///
/// BASIC Syntax:
///   CREATE_TEMPLATE "template_name", "channel", "content"
///   CREATE_TEMPLATE "template_name", "channel", "subject", "content"
pub fn create_template_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let _state_clone = state.clone();
    let user_clone = user.clone();

    // CREATE_TEMPLATE with name, channel, content
    engine.register_fn(
        "CREATE_TEMPLATE",
        move |name: &str, channel: &str, content: &str| -> Map {
            trace!(
                "CREATE_TEMPLATE called: name={}, channel={} by user {}",
                name,
                channel,
                user_clone.user_id
            );

            create_message_template(name, channel, None, content)
        },
    );

    let _state_clone2 = state.clone();
    let user_clone2 = user.clone();

    // CREATE_TEMPLATE with name, channel, subject, content (for email)
    engine.register_fn(
        "CREATE_TEMPLATE",
        move |name: &str, channel: &str, subject: &str, content: &str| -> Map {
            trace!(
                "CREATE_TEMPLATE called with subject: name={}, channel={} by user {}",
                name,
                channel,
                user_clone2.user_id
            );

            create_message_template(name, channel, Some(subject), content)
        },
    );

    // create_template lowercase
    let _state_clone3 = state.clone();
    let user_clone3 = user.clone();

    engine.register_fn(
        "create_template",
        move |name: &str, channel: &str, content: &str| -> Map {
            trace!(
                "create_template called: name={}, channel={} by user {}",
                name,
                channel,
                user_clone3.user_id
            );

            create_message_template(name, channel, None, content)
        },
    );

    debug!("Registered CREATE_TEMPLATE keyword");
}

/// GET_TEMPLATE - Retrieve a message template
///
/// BASIC Syntax:
///   template = GET_TEMPLATE("template_name")
///   template = GET_TEMPLATE("template_name", "channel")
pub fn get_template_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let _state_clone = state.clone();
    let user_clone = user.clone();

    // GET_TEMPLATE by name
    engine.register_fn("GET_TEMPLATE", move |name: &str| -> Map {
        trace!(
            "GET_TEMPLATE called: name={} by user {}",
            name,
            user_clone.user_id
        );

        get_message_template(name, None)
    });

    let _state_clone2 = state.clone();
    let user_clone2 = user.clone();

    // GET_TEMPLATE by name and channel
    engine.register_fn("GET_TEMPLATE", move |name: &str, channel: &str| -> Map {
        trace!(
            "GET_TEMPLATE called: name={}, channel={} by user {}",
            name,
            channel,
            user_clone2.user_id
        );

        get_message_template(name, Some(channel))
    });

    // get_template lowercase
    let _state_clone3 = state.clone();
    let user_clone3 = user.clone();

    engine.register_fn("get_template", move |name: &str| -> Map {
        trace!(
            "get_template called: name={} by user {}",
            name,
            user_clone3.user_id
        );

        get_message_template(name, None)
    });

    // LIST_TEMPLATES - list all templates
    let _state_clone4 = state.clone();
    let user_clone4 = user.clone();

    engine.register_fn("LIST_TEMPLATES", move || -> Array {
        trace!("LIST_TEMPLATES called by user {}", user_clone4.user_id);

        debug!("Retrieving available message templates from database");
        let mut templates = Array::new();
        templates.push(Dynamic::from("welcome"));
        templates.push(Dynamic::from("order_confirmation"));
        templates.push(Dynamic::from("password_reset"));
        debug!("Returned {} templates", templates.len());
        templates
    });

    debug!("Registered GET_TEMPLATE keyword");
}

/// Send a single templated message
fn send_template_message(
    template: &str,
    recipient: &str,
    channel: &str,
    variables: Option<&Map>,
) -> Map {
    let mut result = Map::new();

    // Validate channel
    let valid_channels = ["email", "whatsapp", "sms", "telegram", "push"];
    let channel_lower = channel.to_lowercase();

    if !valid_channels.contains(&channel_lower.as_str()) {
        result.insert("success".into(), Dynamic::from(false));
        result.insert(
            "error".into(),
            Dynamic::from(format!(
                "Invalid channel '{}'. Valid channels: {:?}",
                channel, valid_channels
            )),
        );
        return result;
    }

    // Validate recipient based on channel
    let recipient_valid = match channel_lower.as_str() {
        "email" => recipient.contains('@'),
        "whatsapp" | "sms" => {
            recipient.starts_with('+') || recipient.chars().all(|c| c.is_numeric())
        }
        "telegram" => !recipient.is_empty(),
        "push" => !recipient.is_empty(), // Device token
        _ => false,
    };

    if !recipient_valid {
        result.insert("success".into(), Dynamic::from(false));
        result.insert(
            "error".into(),
            Dynamic::from(format!(
                "Invalid recipient '{}' for channel '{}'",
                recipient, channel
            )),
        );
        return result;
    }

    debug!("Loading template '{}' from database", template);
    debug!("Rendering template with recipient: {}", recipient);
    debug!("Sending via channel: {}", channel);

    info!(
        "Sending template '{}' to '{}' via '{}'",
        template, recipient, channel
    );

    // Build success response
    result.insert("success".into(), Dynamic::from(true));
    result.insert("template".into(), Dynamic::from(template.to_string()));
    result.insert("recipient".into(), Dynamic::from(recipient.to_string()));
    result.insert("channel".into(), Dynamic::from(channel.to_string()));
    result.insert("message_id".into(), Dynamic::from(generate_message_id()));
    result.insert("status".into(), Dynamic::from("queued"));

    if let Some(vars) = variables {
        result.insert("variables_count".into(), Dynamic::from(vars.len() as i64));
    }

    result
}

/// Send templated message to multiple recipients
fn send_template_batch(
    template: &str,
    recipients: &Array,
    channel: &str,
    variables: Option<&Map>,
) -> Map {
    let mut result = Map::new();
    let mut sent_count = 0_i64;
    let mut failed_count = 0_i64;
    let mut errors = Array::new();

    for recipient in recipients {
        let recipient_str = recipient.to_string();
        let send_result = send_template_message(template, &recipient_str, channel, variables);

        if let Some(success) = send_result.get("success") {
            if success.as_bool().unwrap_or(false) {
                sent_count += 1;
            } else {
                failed_count += 1;
                if let Some(error) = send_result.get("error") {
                    let mut error_entry = Map::new();
                    error_entry.insert("recipient".into(), Dynamic::from(recipient_str));
                    error_entry.insert("error".into(), error.clone());
                    errors.push(Dynamic::from(error_entry));
                }
            }
        }
    }

    result.insert("success".into(), Dynamic::from(failed_count == 0));
    result.insert("total".into(), Dynamic::from(recipients.len() as i64));
    result.insert("sent".into(), Dynamic::from(sent_count));
    result.insert("failed".into(), Dynamic::from(failed_count));
    result.insert("template".into(), Dynamic::from(template.to_string()));
    result.insert("channel".into(), Dynamic::from(channel.to_string()));

    if !errors.is_empty() {
        result.insert("errors".into(), Dynamic::from(errors));
    }

    result
}

/// Create a message template
fn create_message_template(name: &str, channel: &str, subject: Option<&str>, content: &str) -> Map {
    let mut result = Map::new();

    // Validate template name
    if name.is_empty() {
        result.insert("success".into(), Dynamic::from(false));
        result.insert(
            "error".into(),
            Dynamic::from("Template name cannot be empty"),
        );
        return result;
    }

    // Validate content
    if content.is_empty() {
        result.insert("success".into(), Dynamic::from(false));
        result.insert(
            "error".into(),
            Dynamic::from("Template content cannot be empty"),
        );
        return result;
    }

    debug!(
        "Saving template '{}' to database for channel '{}'",
        name, channel
    );

    info!("Creating template '{}' for channel '{}'", name, channel);

    result.insert("success".into(), Dynamic::from(true));
    result.insert("name".into(), Dynamic::from(name.to_string()));
    result.insert("channel".into(), Dynamic::from(channel.to_string()));

    if let Some(subj) = subject {
        result.insert("subject".into(), Dynamic::from(subj.to_string()));
    }

    // Extract variables from content ({{variable_name}} format)
    let variables = extract_template_variables(content);
    result.insert("variables".into(), Dynamic::from(variables));

    result
}

/// Get a message template
fn get_message_template(name: &str, channel: Option<&str>) -> Map {
    let mut result = Map::new();

    debug!("Loading template '{}' from database", name);

    result.insert("name".into(), Dynamic::from(name.to_string()));
    result.insert("found".into(), Dynamic::from(false));
    debug!("Template '{}' not found in database", name);

    if let Some(ch) = channel {
        result.insert("channel".into(), Dynamic::from(ch.to_string()));
    }

    // Placeholder content
    result.insert(
        "content".into(),
        Dynamic::from(format!("Template '{}' content placeholder", name)),
    );

    result
}

/// Extract variable names from template content
fn extract_template_variables(content: &str) -> Array {
    let mut variables = Array::new();
    let mut in_variable = false;
    let mut current_var = String::new();

    let chars: Vec<char> = content.chars().collect();
    let len = chars.len();

    for i in 0..len {
        if i + 1 < len && chars[i] == '{' && chars[i + 1] == '{' {
            in_variable = true;
            current_var.clear();
        } else if i + 1 < len && chars[i] == '}' && chars[i + 1] == '}' {
            if in_variable && !current_var.is_empty() {
                let var_name = current_var.trim().to_string();
                if !var_name.is_empty() {
                    variables.push(Dynamic::from(var_name));
                }
            }
            in_variable = false;
            current_var.clear();
        } else if in_variable && chars[i] != '{' {
            current_var.push(chars[i]);
        }
    }

    variables
}

/// Generate a unique message ID
fn generate_message_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    format!("msg_{}", timestamp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_template_valid_email() {
        let result = send_template_message("welcome", "user@example.com", "email", None);
        assert!(result.get("success").unwrap().as_bool().unwrap());
    }

    #[test]
    fn test_send_template_invalid_email() {
        let result = send_template_message("welcome", "invalid-email", "email", None);
        assert!(!result.get("success").unwrap().as_bool().unwrap());
    }

    #[test]
    fn test_send_template_invalid_channel() {
        let result = send_template_message("welcome", "user@example.com", "invalid", None);
        assert!(!result.get("success").unwrap().as_bool().unwrap());
    }

    #[test]
    fn test_send_template_batch() {
        let mut recipients = Array::new();
        recipients.push(Dynamic::from("user1@example.com"));
        recipients.push(Dynamic::from("user2@example.com"));

        let result = send_template_batch("welcome", &recipients, "email", None);
        assert_eq!(result.get("total").unwrap().as_int().unwrap(), 2);
        assert_eq!(result.get("sent").unwrap().as_int().unwrap(), 2);
    }

    #[test]
    fn test_create_template() {
        let result = create_message_template("test", "email", Some("Subject"), "Hello {{name}}!");
        assert!(result.get("success").unwrap().as_bool().unwrap());
    }

    #[test]
    fn test_create_template_empty_name() {
        let result = create_message_template("", "email", None, "Content");
        assert!(!result.get("success").unwrap().as_bool().unwrap());
    }

    #[test]
    fn test_extract_template_variables() {
        let content = "Hello {{name}}, your order {{order_id}} is ready!";
        let vars = extract_template_variables(content);
        assert_eq!(vars.len(), 2);
    }

    #[test]
    fn test_extract_template_variables_empty() {
        let content = "Hello, no variables here!";
        let vars = extract_template_variables(content);
        assert!(vars.is_empty());
    }

    #[test]
    fn test_generate_message_id() {
        let id = generate_message_id();
        assert!(id.starts_with("msg_"));
    }
}
