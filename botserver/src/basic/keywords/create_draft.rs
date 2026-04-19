use crate::core::shared::models::UserSession;
use crate::core::shared::state::AppState;
use rhai::Dynamic;
use rhai::Engine;

pub fn create_draft_keyword(state: &AppState, _user: UserSession, engine: &mut Engine) {
    let state_clone = state.clone();
    engine
        .register_custom_syntax(
            ["CREATE_DRAFT", "$expr$", ",", "$expr$", ",", "$expr$"],
            true,
            move |context, inputs| {
                let to = context.eval_expression_tree(&inputs[0])?.to_string();
                let subject = context.eval_expression_tree(&inputs[1])?.to_string();
                let reply_text = context.eval_expression_tree(&inputs[2])?.to_string();

                let state_clone2 = state_clone.clone();
                let to_owned = to.clone();
                let subject_owned = subject.clone();
                let reply_text_owned = reply_text.clone();
                let (tx, rx) = std::sync::mpsc::channel();
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build();
                    let result = match rt {
                        Ok(rt) => rt.block_on(async {
                            execute_create_draft(&state_clone2, &to_owned, &subject_owned, &reply_text_owned).await
                        }),
                        Err(e) => Err(format!("Runtime creation failed: {e}")),
                    };
                    let _ = tx.send(result);
                });
                let result = rx.recv().unwrap_or(Err("Failed to receive result".to_string()))
                        .map_err(|e| format!("Draft creation failed: {e}"))?;
                Ok(Dynamic::from(result))
            },
        )
        .ok();
}

async fn execute_create_draft(
    state: &AppState,
    to: &str,
    subject: &str,
    reply_text: &str,
) -> Result<String, String> {
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

            Ok::<_, String>(format!("Draft saved with ID: {draft_id}"))
        })
        .await
        .map_err(|e| e.to_string())?
}
