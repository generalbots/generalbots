use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use chrono::Utc;
use diesel::prelude::*;
use log::{error, trace};
use rhai::{Dynamic, Engine};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

pub fn send_mail_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine
        .register_custom_syntax(
            &[
                "SEND_MAIL",
                "$expr$",
                ",",
                "$expr$",
                ",",
                "$expr$",
                ",",
                "$expr$",
            ],
            false,
            move |context, inputs| {
                let to = context.eval_expression_tree(&inputs[0])?.to_string();
                let subject = context.eval_expression_tree(&inputs[1])?.to_string();
                let body = context.eval_expression_tree(&inputs[2])?.to_string();
                let attachments_input = context.eval_expression_tree(&inputs[3])?;

                // Parse attachments array
                let mut attachments = Vec::new();
                if attachments_input.is_array() {
                    let arr = attachments_input.cast::<rhai::Array>();
                    for item in arr.iter() {
                        attachments.push(item.to_string());
                    }
                } else if !attachments_input.to_string().is_empty() {
                    attachments.push(attachments_input.to_string());
                }

                trace!(
                    "SEND_MAIL: to={}, subject={}, attachments={:?} for user={}",
                    to,
                    subject,
                    attachments,
                    user_clone.user_id
                );

                let state_for_task = Arc::clone(&state_clone);
                let user_for_task = user_clone.clone();

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();

                    let send_err = if let Ok(rt) = rt {
                        let result = rt.block_on(async move {
                            execute_send_mail(
                                &state_for_task,
                                &user_for_task,
                                &to,
                                &subject,
                                &body,
                                attachments,
                            )
                            .await
                        });
                        tx.send(result).err()
                    } else {
                        tx.send(Err("Failed to build tokio runtime".to_string()))
                            .err()
                    };

                    if send_err.is_some() {
                        error!("Failed to send SEND_MAIL result from thread");
                    }
                });

                match rx.recv_timeout(std::time::Duration::from_secs(30)) {
                    Ok(Ok(message_id)) => Ok(Dynamic::from(message_id)),
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("SEND_MAIL failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            "SEND_MAIL timed out".into(),
                            rhai::Position::NONE,
                        )))
                    }
                    Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("SEND_MAIL thread failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .unwrap();

    // Register SEND_TEMPLATE for bulk templated emails
    let state_clone2 = Arc::clone(&state);
    let user_clone2 = user.clone();

    engine
        .register_custom_syntax(
            &["SEND_TEMPLATE", "$expr$", ",", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let recipients_input = context.eval_expression_tree(&inputs[0])?;
                let template = context.eval_expression_tree(&inputs[1])?.to_string();
                let variables = context.eval_expression_tree(&inputs[2])?;

                // Parse recipients
                let mut recipients = Vec::new();
                if recipients_input.is_array() {
                    let arr = recipients_input.cast::<rhai::Array>();
                    for item in arr.iter() {
                        recipients.push(item.to_string());
                    }
                } else {
                    recipients.push(recipients_input.to_string());
                }

                // Convert variables to JSON
                let vars_json = if variables.is_map() {
                    // Convert Rhai map to JSON
                    json!(variables.to_string())
                } else {
                    json!({})
                };

                trace!(
                    "SEND_TEMPLATE: recipients={:?}, template={} for user={}",
                    recipients,
                    template,
                    user_clone2.user_id
                );

                let state_for_task = Arc::clone(&state_clone2);
                let user_for_task = user_clone2.clone();

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();

                    let send_err = if let Ok(rt) = rt {
                        let result = rt.block_on(async move {
                            execute_send_template(
                                &state_for_task,
                                &user_for_task,
                                recipients,
                                &template,
                                vars_json,
                            )
                            .await
                        });
                        tx.send(result).err()
                    } else {
                        tx.send(Err("Failed to build tokio runtime".to_string()))
                            .err()
                    };

                    if send_err.is_some() {
                        error!("Failed to send SEND_TEMPLATE result from thread");
                    }
                });

                match rx.recv_timeout(std::time::Duration::from_secs(60)) {
                    Ok(Ok(count)) => Ok(Dynamic::from(count)),
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("SEND_TEMPLATE failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                    Err(_) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        "SEND_TEMPLATE timed out".into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .unwrap();
}

async fn execute_send_mail(
    state: &AppState,
    user: &UserSession,
    to: &str,
    subject: &str,
    body: &str,
    attachments: Vec<String>,
) -> Result<String, String> {
    let message_id = Uuid::new_v4().to_string();

    // Track email in communication history
    track_email(state, user, &message_id, to, subject, "sent").await?;

    // Send the actual email if email feature is enabled
    #[cfg(feature = "email")]
    {
        let email_request = crate::email::EmailRequest {
            to: to.to_string(),
            subject: subject.to_string(),
            body: body.to_string(),
            cc: None,
            bcc: None,
            attachments: if attachments.is_empty() {
                None
            } else {
                Some(attachments.clone())
            },
            reply_to: None,
            headers: None,
        };

        if let Some(config) = &state.config {
            if let Ok(_) = crate::email::send_email(&config.email, &email_request).await {
                trace!("Email sent successfully: {}", message_id);
                return Ok(format!("Email sent: {}", message_id));
            }
        }
    }

    // Fallback: store as draft if email sending fails
    save_email_draft(state, user, to, subject, body, attachments).await?;

    Ok(format!("Email saved as draft: {}", message_id))
}

async fn execute_send_template(
    state: &AppState,
    user: &UserSession,
    recipients: Vec<String>,
    template_name: &str,
    variables: serde_json::Value,
) -> Result<i32, String> {
    let template_content = load_template(state, template_name).await?;

    let mut sent_count = 0;

    for recipient in recipients {
        // Personalize template for each recipient
        let personalized_content =
            apply_template_variables(&template_content, &variables, &recipient)?;

        // Extract subject from template or use default
        let subject = extract_template_subject(&personalized_content)
            .unwrap_or_else(|| format!("Message from {}", user.user_id));

        // Send email
        if let Ok(_) = execute_send_mail(
            state,
            user,
            &recipient,
            &subject,
            &personalized_content,
            vec![],
        )
        .await
        {
            sent_count += 1;
        }

        // Add small delay to avoid rate limiting
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    trace!("Sent {} templated emails", sent_count);
    Ok(sent_count)
}

async fn track_email(
    state: &AppState,
    user: &UserSession,
    message_id: &str,
    to: &str,
    subject: &str,
    status: &str,
) -> Result<(), String> {
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {}", e))?;

    let log_id = Uuid::new_v4().to_string();
    let user_id_str = user.user_id.to_string();
    let bot_id_str = user.bot_id.to_string();
    let now = Utc::now();

    let query = diesel::sql_query(
        "INSERT INTO communication_logs (id, user_id, bot_id, message_id, recipient, subject, status, timestamp, type)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'email')"
    )
    .bind::<diesel::sql_types::Text, _>(&log_id)
    .bind::<diesel::sql_types::Text, _>(&user_id_str)
    .bind::<diesel::sql_types::Text, _>(&bot_id_str)
    .bind::<diesel::sql_types::Text, _>(message_id)
    .bind::<diesel::sql_types::Text, _>(to)
    .bind::<diesel::sql_types::Text, _>(subject)
    .bind::<diesel::sql_types::Text, _>(status)
    .bind::<diesel::sql_types::Timestamptz, _>(&now);

    query.execute(&mut *conn).map_err(|e| {
        error!("Failed to track email: {}", e);
        format!("Failed to track email: {}", e)
    })?;

    Ok(())
}

async fn save_email_draft(
    state: &AppState,
    user: &UserSession,
    to: &str,
    subject: &str,
    body: &str,
    attachments: Vec<String>,
) -> Result<(), String> {
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {}", e))?;

    let draft_id = Uuid::new_v4().to_string();
    let user_id_str = user.user_id.to_string();
    let bot_id_str = user.bot_id.to_string();
    let now = Utc::now();

    let query = diesel::sql_query(
        "INSERT INTO email_drafts (id, user_id, bot_id, to_address, subject, body, attachments, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
    )
    .bind::<diesel::sql_types::Text, _>(&draft_id)
    .bind::<diesel::sql_types::Text, _>(&user_id_str)
    .bind::<diesel::sql_types::Text, _>(&bot_id_str)
    .bind::<diesel::sql_types::Text, _>(to)
    .bind::<diesel::sql_types::Text, _>(subject)
    .bind::<diesel::sql_types::Text, _>(body);

    let attachments_json = json!(attachments);
    let query = query
        .bind::<diesel::sql_types::Jsonb, _>(&attachments_json)
        .bind::<diesel::sql_types::Timestamptz, _>(&now);

    query.execute(&mut *conn).map_err(|e| {
        error!("Failed to save draft: {}", e);
        format!("Failed to save draft: {}", e)
    })?;

    trace!("Email saved as draft: {}", draft_id);
    Ok(())
}

async fn load_template(state: &AppState, template_name: &str) -> Result<String, String> {
    // Try loading from database first
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {}", e))?;

    let query =
        diesel::sql_query("SELECT content FROM email_templates WHERE name = $1 AND active = true")
            .bind::<diesel::sql_types::Text, _>(template_name);

    #[derive(QueryableByName)]
    struct TemplateRecord {
        #[diesel(sql_type = diesel::sql_types::Text)]
        content: String,
    }

    let result: Result<Vec<TemplateRecord>, _> = query.load(&mut *conn);

    match result {
        Ok(records) if !records.is_empty() => Ok(records[0].content.clone()),
        _ => {
            // Fallback to file system
            let template_path = format!(".gbdrive/templates/{}.html", template_name);
            std::fs::read_to_string(&template_path)
                .map_err(|e| format!("Template not found: {}", e))
        }
    }
}

fn apply_template_variables(
    template: &str,
    variables: &serde_json::Value,
    recipient: &str,
) -> Result<String, String> {
    let mut content = template.to_string();

    // Replace {{recipient}} variable
    content = content.replace("{{recipient}}", recipient);

    // Replace other variables from the JSON object
    if let Some(obj) = variables.as_object() {
        for (key, value) in obj {
            let placeholder = format!("{{{{{}}}}}", key);
            let replacement = value.as_str().unwrap_or(&value.to_string());
            content = content.replace(&placeholder, replacement);
        }
    }

    Ok(content)
}

fn extract_template_subject(content: &str) -> Option<String> {
    // Look for subject line in template (e.g., "Subject: ...")
    for line in content.lines() {
        if line.starts_with("Subject:") {
            return Some(line.trim_start_matches("Subject:").trim().to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_template_variables() {
        let template = "Hello {{name}}, your order {{order_id}} is ready!";
        let vars = json!({
            "name": "John",
            "order_id": "12345"
        });

        let result = apply_template_variables(template, &vars, "john@example.com").unwrap();
        assert!(result.contains("John"));
        assert!(result.contains("12345"));
    }

    #[test]
    fn test_extract_template_subject() {
        let content = "Subject: Welcome to our service\n\nHello there!";
        let subject = extract_template_subject(content);
        assert_eq!(subject, Some("Welcome to our service".to_string()));
    }
}
