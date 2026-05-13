use std::sync::Arc;
use botbasic_types::UserSession;
use botbasic_types::BasicRuntime;
use log::trace;
use rhai::Dynamic;
use rhai::Engine;

pub fn print_keyword(_state: &Arc<dyn BasicRuntime>, _user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(["PRINT", "$expr$"], true, |context, inputs| {
            let value = context.eval_expression_tree(&inputs[0])?;
            trace!("PRINT: {value}");
            Ok(Dynamic::UNIT)
        })
        .ok();
}
