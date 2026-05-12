use botbasic_types::UserSession;
use botbasic_types::BasicRuntime;
use log::{error, trace};
use rhai::{Dynamic, Engine};
use std::sync::Arc;
use uuid::Uuid;

pub fn set_user_keyword(state: &Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(state);
    let user_clone = user;

    engine
        .register_custom_syntax(["SET", "USER", "$expr$"], true, move |context, inputs| {
            let user_id_str = context.eval_expression_tree(&inputs[0])?.to_string();

            match Uuid::parse_str(user_id_str.as_str()) {
                Ok(user_id) => {
                    let state_for_spawn = Arc::clone(&state_clone);
                    let user_clone_spawn = user_clone.clone();
                    let (tx, rx) = std::sync::mpsc::channel();
                    std::thread::spawn(move || {
                        let rt = tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build();
        let result = if let Ok(_rt) = rt {
                state_for_spawn.update_session_user(user_clone_spawn.id, user_id)
                        } else {
                            Err("Failed to create runtime".into())
                        };
                        let _ = tx.send(result);
                    });
                    if let Ok(Ok(())) = rx.recv() {
                        trace!(
                            "Updated session {} to user ID: {user_id}",
                            user_clone_spawn.id
                        );
                    } else {
                        error!("Failed to update user ID in session");
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
