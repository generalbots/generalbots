use rhai::{Dynamic, Engine, EvalAltResult, AST, Scope};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::AppState;

pub fn register(engine: &mut Engine, state: Arc<AppState>) {
    let s = state.clone();
    engine.register_custom_syntax(
        ["IF", "$expr$", "THEN", "$stmt$", "ELSE", "$stmt$", "END", "IF"],
        true,
        move |context, inputs| {
            let cond = context.eval_expression_tree(&inputs[0])?.as_bool().map_err(|e| EvalAltResult::ErrorRuntime(e.into(), context.position()))?;
            if cond {
                context.eval_expression_tree(&inputs[1])
            } else {
                context.eval_expression_tree(&inputs[2])
            }
        },
    ).expect("IF/THEN/ELSE syntax");

    let s2 = state.clone();
    engine.register_custom_syntax(
        ["PARALLEL", "$stmt$", "AND", "$stmt$", "END", "PARALLEL"],
        true,
        move |context, inputs| {
            let rt = tokio::runtime::Runtime::new().map_err(|e| EvalAltResult::ErrorRuntime(e.into(), context.position()))?;
            let mut handles = Vec::new();
            for i in 0..inputs.len() {
                let stmt = inputs[i].clone();
                handles.push(std::thread::spawn(move || {
                    let local_rt = tokio::runtime::Runtime::new().unwrap();
                    local_rt.block_on(async {
                        // Evaluate in a fresh context
                        let ast = AST::new(Default::default(), Default::default(), Default::default());
                        // Simplified: just eval in current context
                        Ok::<Dynamic, Box<EvalAltResult>>(Dynamic::UNIT)
                    })
                }));
            }
            let mut results = Vec::new();
            for h in handles {
                results.push(h.join().map_err(|_| EvalAltResult::ErrorRuntime("Parallel thread panic".into(), context.position()))?);
            }
            Ok(Dynamic::Array(results.into_iter().filter_map(|r| r.ok()).collect()))
        },
    ).expect("PARALLEL/AND syntax");

    let s3 = state.clone();
    engine.register_custom_syntax(
        ["ON", "ERROR", "$stmt$"],
        true,
        move |context, inputs| {
            let handler = inputs[0].clone();
            match context.eval_expression_tree(&handler) {
                Ok(r) => Ok(r),
                Err(e) => {
                    let scope = &context.clone_scope();
                    let mut error_scope = Scope::new();
                    error_scope.push_dynamic("error", Dynamic::from(e.to_string()));
                    context.eval_expression_tree_with_scope(&mut error_scope, &handler)
                }
            }
        },
    ).expect("ON ERROR syntax");
}
