use crate::shared::state::AppState;
use crate::shared::models::UserSession;
use log::{debug, error, info};
use rhai::{Dynamic, Engine};
use std::sync::Arc;
use uuid::Uuid;

pub fn set_user_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();
    engine
        .register_custom_syntax(&["SET_USER", "$expr$"], true, move |context, inputs| {
            let user_id_str = context.eval_expression_tree(&inputs[0])?.to_string();

            info!("SET USER command executed with ID: {}", user_id_str);

            match Uuid::parse_str(&user_id_str) {
                Ok(user_id) => {
                    debug!("Successfully parsed user UUID: {}", user_id);

                    let state_for_spawn = Arc::clone(&state_clone);
                    let user_clone_spawn = user_clone.clone();

                    let mut session_manager =
                        futures::executor::block_on(state_for_spawn.session_manager.lock());

                    if let Err(e) = session_manager.update_user_id(user_clone_spawn.id, user_id) {
                        error!("Failed to update user ID in session: {}", e);
                    } else {
                        info!(
                            "Updated session {} to user ID: {}",
                            user_clone_spawn.id, user_id
                        );
                    }
                }
                Err(e) => {
                    debug!("Invalid UUID format for SET USER: {}", e);
                }
            }

            Ok(Dynamic::UNIT)
        })
        .unwrap();
}
