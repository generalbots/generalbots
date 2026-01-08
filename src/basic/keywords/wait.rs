use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use rhai::{Dynamic, Engine};
use std::thread;
use std::time::Duration;

pub fn wait_keyword(_state: &AppState, _user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(["WAIT", "$expr$"], false, move |context, inputs| {
            let seconds = context.eval_expression_tree(&inputs[0])?;
            let duration_secs = if seconds.is::<i64>() {
                let val = seconds.cast::<i64>() as f64;
                val
            } else if seconds.is::<f64>() {
                seconds.cast::<f64>()
            } else {
                return Err(format!("WAIT expects a number, got: {seconds}").into());
            };
            if duration_secs < 0.0 {
                return Err("WAIT duration cannot be negative".into());
            }
            let capped_duration = duration_secs.min(300.0);
            let duration = Duration::from_secs_f64(capped_duration);
            thread::sleep(duration);
            Ok(Dynamic::from(format!("Waited {capped_duration} seconds")))
        })
        .ok();
}
