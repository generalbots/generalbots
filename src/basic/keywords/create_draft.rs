use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use rhai::Dynamic;
use rhai::Engine;

pub fn create_draft_keyword(_state: &AppState, _user: UserSession, engine: &mut Engine) {
    let state_clone = _state.clone();
    engine
        .register_custom_syntax(
            &["CREATE_DRAFT", "$expr$", ",", "$expr$", ",", "$expr$"],
            true,
            move |context, inputs| {
                let to = context.eval_expression_tree(&inputs[0])?.to_string();
                let subject = context.eval_expression_tree(&inputs[1])?.to_string();
                let reply_text = context.eval_expression_tree(&inputs[2])?.to_string();

                let fut = execute_create_draft(&state_clone, &to, &subject, &reply_text);
                let result =
                    tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(fut))
                        .map_err(|e| format!("Draft creation error: {}", e))?;
                Ok(Dynamic::from(result))
            },
        )
        .unwrap();
}

async fn execute_create_draft(
    state: &AppState,
    to: &str,
    subject: &str,
    reply_text: &str,
) -> Result<String, String> {
    #[cfg(feature = "email")]
    {
        use crate::email::{fetch_latest_sent_to, save_email_draft, SaveDraftRequest};

        let config = state.config.as_ref().ok_or("No email config")?;

        // Fetch any previous emails to this recipient for threading
        let previous_email = fetch_latest_sent_to(&config.email, to)
            .await
            .unwrap_or_default();

        let email_body = if !previous_email.is_empty() {
            // Create a threaded reply
            let email_separator = "<br><hr><br>";
            let formatted_reply = reply_text.replace("FIX", "Fixed");
            let formatted_old = previous_email.replace("\n", "<br>");
            format!("{}{}{}", formatted_reply, email_separator, formatted_old)
        } else {
            reply_text.to_string()
        };

        let draft_request = SaveDraftRequest {
            to: to.to_string(),
            subject: subject.to_string(),
            cc: None,
            body: email_body,
        };

        save_email_draft(&config.email, &draft_request)
            .await
            .map(|_| "Draft saved successfully".to_string())
            .map_err(|e| e.to_string())
    }

    #[cfg(not(feature = "email"))]
    {
        // Store draft in database when email feature is disabled
        use chrono::Utc;
        use diesel::prelude::*;
        use uuid::Uuid;

        let draft_id = Uuid::new_v4();
        let conn = state.conn.clone();
        let to = to.to_string();
        let subject = subject.to_string();
        let reply_text = reply_text.to_string();

        tokio::task::spawn_blocking(move || {
            let mut db_conn = conn.get().map_err(|e| e.to_string())?;

            diesel::sql_query(
                "INSERT INTO email_drafts (id, recipient, subject, body, created_at)
                 VALUES ($1, $2, $3, $4, $5)",
            )
            .bind::<diesel::sql_types::Uuid, _>(&draft_id)
            .bind::<diesel::sql_types::Text, _>(&to)
            .bind::<diesel::sql_types::Text, _>(&subject)
            .bind::<diesel::sql_types::Text, _>(&reply_text)
            .bind::<diesel::sql_types::Timestamptz, _>(&Utc::now())
            .execute(&mut db_conn)
            .map_err(|e| e.to_string())?;

            Ok::<_, String>(format!("Draft saved with ID: {}", draft_id))
        })
        .await
        .map_err(|e| e.to_string())?
    }
}
