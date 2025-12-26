use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use diesel::prelude::*;
use log::{debug, trace};
use rhai::{Dynamic, Engine};
use std::sync::Arc;

pub fn delete_post_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user;

    engine
        .register_custom_syntax(
            ["DELETE", "POST", "$expr$"],
            false,
            move |context, inputs| {
                let post_id = context.eval_expression_tree(&inputs[0])?.to_string();
                let post_id = post_id.trim_matches('"');

                trace!("DELETE POST: {}", post_id);

                let mut conn = state_clone
                    .conn
                    .get()
                    .map_err(|e| format!("DB error: {}", e))?;

                let result =
                    delete_social_post(&mut conn, user_clone.bot_id, post_id).map_err(|e| {
                        Box::new(rhai::EvalAltResult::ErrorRuntime(
                            format!("DELETE POST failed: {}", e).into(),
                            rhai::Position::NONE,
                        ))
                    })?;

                Ok(Dynamic::from(result))
            },
        )
        .unwrap();

    debug!("Registered DELETE POST keyword");
}

fn delete_social_post(
    conn: &mut diesel::PgConnection,
    bot_id: uuid::Uuid,
    post_id: &str,
) -> Result<bool, String> {
    let result = diesel::sql_query("DELETE FROM social_posts WHERE bot_id = $1 AND id = $2")
        .bind::<diesel::sql_types::Uuid, _>(bot_id)
        .bind::<diesel::sql_types::Text, _>(post_id)
        .execute(conn)
        .map_err(|e| format!("Failed to delete post: {}", e))?;

    Ok(result > 0)
}
