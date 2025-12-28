use crate::basic::keywords::use_account::{get_account_credentials, AccountCredentials};
use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use chrono::Utc;
use diesel::prelude::*;
use log::{error, info, trace};
use rhai::{Dynamic, Engine};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

pub fn send_mail_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine
        .register_custom_syntax(
            [
                "SEND", "MAIL", "$expr$", ",", "$expr$", ",", "$expr$", ",", "$expr$",
            ],
            false,
            move |context, inputs| {
                let to = context.eval_expression_tree(&inputs[0])?.to_string();
                let subject = context.eval_expression_tree(&inputs[1])?.to_string();
                let body = context.eval_expression_tree(&inputs[2])?.to_string();
                let attachments_input = context.eval_expression_tree(&inputs[3])?;

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
                    "SEND MAIL: to={}, subject={}, attachments={:?} for user={}",
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
                                None,
                            )
                            .await
                        });
                        tx.send(result).err()
                    } else {
                        tx.send(Err("Failed to build tokio runtime".to_string()))
                            .err()
                    };

                    if send_err.is_some() {
                        error!("Failed to send SEND MAIL result from thread");
                    }
                });

                match rx.recv_timeout(std::time::Duration::from_secs(30)) {
                    Ok(Ok(message_id)) => Ok(Dynamic::from(message_id)),
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("SEND MAIL failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            "SEND MAIL timed out".into(),
                            rhai::Position::NONE,
                        )))
                    }
                    Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("SEND MAIL thread failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .unwrap();

    let state_clone2 = Arc::clone(&state);
    let user_clone2 = user.clone();

    engine
        .register_custom_syntax(
            [
                "SEND", "MAIL", "$expr$", ",", "$expr$", ",", "$expr$", "USING", "$expr$",
            ],
            false,
            move |context, inputs| {
                let to = context.eval_expression_tree(&inputs[0])?.to_string();
                let subject = context.eval_expression_tree(&inputs[1])?.to_string();
                let body = context.eval_expression_tree(&inputs[2])?.to_string();
                let using_account = context.eval_expression_tree(&inputs[3])?.to_string();

                info!(
                    "SEND MAIL USING: to={}, subject={}, using={} for user={}",
                    to, subject, using_account, user_clone2.user_id
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
                            execute_send_mail(
                                &state_for_task,
                                &user_for_task,
                                &to,
                                &subject,
                                &body,
                                vec![],
                                Some(using_account),
                            )
                            .await
                        });
                        tx.send(result).err()
                    } else {
                        tx.send(Err("Failed to build tokio runtime".to_string()))
                            .err()
                    };

                    if send_err.is_some() {
                        error!("Failed to send SEND MAIL USING result from thread");
                    }
                });

                match rx.recv_timeout(std::time::Duration::from_secs(30)) {
                    Ok(Ok(message_id)) => Ok(Dynamic::from(message_id)),
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("SEND MAIL USING failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            "SEND MAIL USING timed out".into(),
                            rhai::Position::NONE,
                        )))
                    }
                    Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("SEND MAIL USING thread failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .unwrap();

    let state_clone2 = Arc::clone(&state);
    let user_clone2 = user;

    engine
        .register_custom_syntax(
            ["SEND_TEMPLATE", "$expr$", ",", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let recipients_input = context.eval_expression_tree(&inputs[0])?;
                let template = context.eval_expression_tree(&inputs[1])?.to_string();
                let variables = context.eval_expression_tree(&inputs[2])?;

                let mut recipients = Vec::new();
                if recipients_input.is_array() {
                    let arr = recipients_input.cast::<rhai::Array>();
                    for item in arr.iter() {
                        recipients.push(item.to_string());
                    }
                } else {
                    recipients.push(recipients_input.to_string());
                }

                let vars_json = if variables.is_map() {
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
    using_account: Option<String>,
) -> Result<String, String> {
    let message_id = Uuid::new_v4().to_string();

    track_email(state, user, &message_id, to, subject, "sent")?;

    if let Some(account_email) = using_account {
        let creds = get_account_credentials(&state.conn, &account_email, user.bot_id)
            .await
            .map_err(|e| format!("Failed to get account credentials: {}", e))?;

        return send_via_connected_account(state, &creds, to, subject, body, attachments).await;
    }

    #[cfg(feature = "email")]
    {
        use crate::email::EmailService;

        let email_service = EmailService::new(Arc::new(state.clone()));

        if email_service
            .send_email(
                &to,
                &subject,
                &body,
                if attachments.is_empty() {
                    None
                } else {
                    Some(attachments.clone())
                },
            )
            .is_ok()
        {
            trace!("Email sent successfully: {}", message_id);
            return Ok(format!("Email sent: {}", message_id));
        }
    }

    save_email_draft(state, user, to, subject, body, attachments)?;

    Ok(format!("Email saved as draft: {}", message_id))
}

async fn send_via_connected_account(
    state: &AppState,
    creds: &AccountCredentials,
    to: &str,
    subject: &str,
    body: &str,
    _attachments: Vec<String>,
) -> Result<String, String> {
    let message_id = Uuid::new_v4().to_string();

    match creds.provider.as_str() {
        "gmail" | "google" => {
            send_via_gmail(state, creds, to, subject, body).await?;
        }
        "outlook" | "microsoft" | "hotmail" => {
            send_via_outlook(state, creds, to, subject, body).await?;
        }
        _ => {
            return Err(format!("Unsupported email provider: {}", creds.provider));
        }
    }

    info!("Email sent via {} account: {}", creds.provider, message_id);
    Ok(format!("Email sent via {}: {}", creds.provider, message_id))
}

async fn send_via_gmail(
    _state: &AppState,
    creds: &AccountCredentials,
    to: &str,
    subject: &str,
    body: &str,
) -> Result<(), String> {
    let client = reqwest::Client::new();

    let raw_message = format!(
        "To: {}\r\nSubject: {}\r\nContent-Type: text/html; charset=utf-8\r\n\r\n{}",
        to, subject, body
    );
    let encoded = base64::Engine::encode(
        &base64::engine::general_purpose::URL_SAFE,
        raw_message.as_bytes(),
    );

    let response = client
        .post("https://gmail.googleapis.com/gmail/v1/users/me/messages/send")
        .bearer_auth(&creds.access_token)
        .json(&serde_json::json!({ "raw": encoded }))
        .send()
        .await
        .map_err(|e| format!("Gmail API request failed: {}", e))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("Gmail API error: {}", error_text));
    }

    Ok(())
}

async fn send_via_outlook(
    _state: &AppState,
    creds: &AccountCredentials,
    to: &str,
    subject: &str,
    body: &str,
) -> Result<(), String> {
    let client = reqwest::Client::new();

    let message = serde_json::json!({
        "message": {
            "subject": subject,
            "body": {
                "contentType": "HTML",
                "content": body
            },
            "toRecipients": [
                {
                    "emailAddress": {
                        "address": to
                    }
                }
            ]
        },
        "saveToSentItems": "true"
    });

    let response = client
        .post("https://graph.microsoft.com/v1.0/me/sendMail")
        .bearer_auth(&creds.access_token)
        .json(&message)
        .send()
        .await
        .map_err(|e| format!("Microsoft Graph request failed: {}", e))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("Microsoft Graph error: {}", error_text));
    }

    Ok(())
}

async fn execute_send_template(
    state: &AppState,
    user: &UserSession,
    recipients: Vec<String>,
    template_name: &str,
    variables: serde_json::Value,
) -> Result<i32, String> {
    let template_content = load_template(state, template_name)?;

    let mut sent_count = 0;

    for recipient in recipients {
        let personalized_content =
            apply_template_variables(&template_content, &variables, &recipient)?;

        let subject = extract_template_subject(&personalized_content)
            .unwrap_or_else(|| format!("Message from {}", user.user_id));

        if execute_send_mail(
            state,
            user,
            &recipient,
            &subject,
            &personalized_content,
            vec![],
            None,
        )
        .await
        .is_ok()
        {
            sent_count += 1;
        }

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    trace!("Sent {} templated emails", sent_count);
    Ok(sent_count)
}

fn track_email(
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

fn save_email_draft(
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

fn load_template(state: &AppState, template_name: &str) -> Result<String, String> {
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

    content = content.replace("{{recipient}}", recipient);

    if let Some(obj) = variables.as_object() {
        for (key, value) in obj {
            let placeholder = format!("{{{{{}}}}}", key);
            let value_string = value.to_string();
            let replacement = value.as_str().unwrap_or(&value_string);
            content = content.replace(&placeholder, replacement);
        }
    }

    Ok(content)
}

fn extract_template_subject(content: &str) -> Option<String> {
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
    use serde_json::json;

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
