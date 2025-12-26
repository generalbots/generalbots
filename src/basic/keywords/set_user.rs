use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::{error, trace};
use rhai::{Dynamic, Engine};
use std::sync::Arc;
use uuid::Uuid;

pub fn set_user_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user;

    engine
        .register_custom_syntax(["SET", "USER", "$expr$"], true, move |context, inputs| {
            let user_id_str = context.eval_expression_tree(&inputs[0])?.to_string();

            match Uuid::parse_str(user_id_str.as_str()) {
                Ok(user_id) => {
                    let state_for_spawn = Arc::clone(&state_clone);
                    let user_clone_spawn = user_clone.clone();
                    let mut session_manager =
                        futures::executor::block_on(state_for_spawn.session_manager.lock());

                    if let Err(e) = session_manager.update_user_id(user_clone_spawn.id, user_id) {
                        error!("Failed to update user ID in session: {e}");
                    } else {
                        trace!(
                            "Updated session {} to user ID: {user_id}",
                            user_clone_spawn.id
                        );
                    }
                }
                Err(e) => {
                    trace!("Invalid user ID format: {e}");
                }
            }

            Ok(Dynamic::UNIT)
        })
        .ok();
}
