use std::sync::Arc;
use botcore::shared::state::AppState;
use rhai::Engine;
use botcore::shared::UserSession;

pub fn register_set_answer_mode_keyword(
    state: Arc<AppState>,
    user_session: UserSession,
    engine: &mut Engine,
) {
    let state_clone = state.clone();
    let session_id = user_session.id;
    let result = engine.register_custom_syntax(
        ["SET", "ANSWER", "MODE", "$expr$"],
        true,
        move |context, inputs| {
            let mode_str = context.eval_expression_tree(&inputs[0])?.to_string();
            let mode = crate::core::bot::answer_mode::AnswerMode::from_str(&mode_str);

            let (tx, rx) = std::sync::mpsc::channel();
            let state_for_thread = state_clone.clone();

            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build();
                let result = if let Ok(rt) = rt {
                    rt.block_on(async {
                        crate::core::bot::answer_mode::store_answer_mode(
                            &state_for_thread,
                            &session_id,
                            &mode,
                        )
                        .await
                    })
                } else {
                    Err("Failed to create tokio runtime".to_string())
                };
                let _ = tx.send(result);
            });

            match rx.recv() {
                Ok(Ok(())) => {
                    let msg = format!("Answer mode set to '{}'", mode.as_str());
                    log::info!("SET ANSWER MODE: {} (session={})", msg, session_id);
                    Ok(rhai::Dynamic::from(msg))
                }
                Ok(Err(e)) => Err(format!("SET ANSWER MODE failed: {}", e).into()),
                Err(e) => Err(format!("Channel error: {}", e).into()),
            }
        },
    );
    if let Err(e) = result {
        log::error!("Failed to register SET ANSWER MODE syntax: {}", e);
    }
}
