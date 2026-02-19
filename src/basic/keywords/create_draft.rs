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

                let fut = execute_create_draft(&state_clone, &to, &subject, &reply_text);
                let result =
                    tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(fut))
                        .map_err(|e| format!("Draft creation error: {e}"))?;
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
