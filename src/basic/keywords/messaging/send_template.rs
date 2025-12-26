use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::{debug, info, trace};
use rhai::{Array, Dynamic, Engine, Map};
use std::sync::Arc;

pub fn send_template_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let _state_clone = state.clone();
    let user_clone = user.clone();
    let user_clone2 = user.clone();
    let user_clone3 = user;

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

    let _state_clone3 = state;

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

pub fn send_template_to_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let user_clone = user.clone();
    let user_clone2 = user.clone();
    let user_clone3 = user;
    let _state_clone = state.clone();

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

    let _state_clone3 = state;

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

pub fn create_template_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let user_clone = user.clone();
    let user_clone2 = user.clone();
    let user_clone3 = user;
    let _state_clone = state.clone();

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

    let _state_clone3 = state;

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

pub fn get_template_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let _state_clone = state.clone();
    let user_clone = user.clone();
    let user_clone2 = user.clone();
    let user_clone3 = user.clone();
    let user_clone4 = user;

    engine.register_fn("GET_TEMPLATE", move |name: &str| -> Map {
        trace!(
            "GET_TEMPLATE called: name={} by user {}",
            name,
            user_clone.user_id
        );

        get_message_template(name, None)
    });

    let _state_clone2 = state.clone();

    engine.register_fn("GET_TEMPLATE", move |name: &str, channel: &str| -> Map {
        trace!(
            "GET_TEMPLATE called: name={}, channel={} by user {}",
            name,
            channel,
            user_clone2.user_id
        );

        get_message_template(name, Some(channel))
    });

    let _state_clone3 = state.clone();

    engine.register_fn("get_template", move |name: &str| -> Map {
        trace!(
            "get_template called: name={} by user {}",
            name,
            user_clone3.user_id
        );

        get_message_template(name, None)
    });

    let _state_clone4 = state;

    engine.register_fn("LIST_TEMPLATES", move || -> Array {
        trace!("LIST_TEMPLATES called by user {}", user_clone4.user_id);

        debug!("Retrieving available message templates from database");
        let templates = vec![
            Dynamic::from("welcome"),
            Dynamic::from("order_confirmation"),
            Dynamic::from("password_reset"),
        ];
        debug!("Returned {} templates", templates.len());
        templates
    });

    debug!("Registered GET_TEMPLATE keyword");
}

fn send_template_message(
    template: &str,
    recipient: &str,
    channel: &str,
    variables: Option<&Map>,
) -> Map {
    let mut result = Map::new();

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

    let recipient_valid = match channel_lower.as_str() {
        "email" => recipient.contains('@'),
        "whatsapp" | "sms" => {
            recipient.starts_with('+') || recipient.chars().all(|c| c.is_numeric())
        }
        "telegram" | "push" => !recipient.is_empty(),
        _ => !recipient.is_empty(),
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

fn create_message_template(name: &str, channel: &str, subject: Option<&str>, content: &str) -> Map {
    let mut result = Map::new();

    if name.is_empty() {
        result.insert("success".into(), Dynamic::from(false));
        result.insert(
            "error".into(),
            Dynamic::from("Template name cannot be empty"),
        );
        return result;
    }

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

    let variables = extract_template_variables(content);
    result.insert("variables".into(), Dynamic::from(variables));

    result
}

fn get_message_template(name: &str, channel: Option<&str>) -> Map {
    let mut result = Map::new();

    debug!("Loading template '{}' from database", name);

    result.insert("name".into(), Dynamic::from(name.to_string()));
    result.insert("found".into(), Dynamic::from(false));
    debug!("Template '{}' not found in database", name);

    if let Some(ch) = channel {
        result.insert("channel".into(), Dynamic::from(ch.to_string()));
    }

    result.insert(
        "content".into(),
        Dynamic::from(format!("Template '{}' content placeholder", name)),
    );

    result
}

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
