use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::trace;
use rhai::Dynamic;
use rhai::Engine;

pub fn print_keyword(_state: &AppState, _user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(["PRINT", "$expr$"], true, |context, inputs| {
            let value = context.eval_expression_tree(&inputs[0])?;
            trace!("PRINT: {value}");
            Ok(Dynamic::UNIT)
        })
        .ok();
}
