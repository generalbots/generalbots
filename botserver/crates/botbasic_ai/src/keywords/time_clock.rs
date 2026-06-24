use botbasic_types::BasicRuntime;
use botbasic_types::UserSession;
use log::trace;
use rhai::{Dynamic, Engine};
use serde_json::{json, Value};
use std::sync::Arc;

/// HR time clock BASIC keywords for issue #623.
///
/// Provides: CLOCK IN, CLOCK OUT, GET BANCO HORAS, FERIAS BALANCE, BANCO HORAS REPORT.
pub fn register_time_clock_keywords(
    state: Arc<dyn BasicRuntime>,
    user: UserSession,
    engine: &mut Engine,
) {
    register_clock_in(state.clone(), user.clone(), engine);
    register_clock_out(state.clone(), user.clone(), engine);
    register_get_banco_horas(state.clone(), user.clone(), engine);
    register_ferias_balance(state.clone(), user.clone(), engine);
    register_banco_horas_report(state, user, engine);
}

fn register_clock_in(
    state: Arc<dyn BasicRuntime>,
    _user: UserSession,
    engine: &mut Engine,
) {
    let _state_clone = state;

    engine
        .register_custom_syntax(["CLOCK", "IN"], false, move |_context, _inputs| {
            trace!("CLOCK IN");
            let result = json!({
                "kind": "clock_entry",
                "action": "in",
                "entry_type": "clock_in",
                "timestamp": "now",
            });
            Ok(serde_json_to_dynamic(&result))
        })
        .expect("valid CLOCK IN syntax");
}

fn register_clock_out(
    state: Arc<dyn BasicRuntime>,
    _user: UserSession,
    engine: &mut Engine,
) {
    let _state_clone = state;

    engine
        .register_custom_syntax(["CLOCK", "OUT"], false, move |_context, _inputs| {
            trace!("CLOCK OUT");
            let result = json!({
                "kind": "clock_entry",
                "action": "out",
                "entry_type": "clock_out",
                "timestamp": "now",
            });
            Ok(serde_json_to_dynamic(&result))
        })
        .expect("valid CLOCK OUT syntax");
}

fn register_get_banco_horas(
    state: Arc<dyn BasicRuntime>,
    _user: UserSession,
    engine: &mut Engine,
) {
    let _state_clone = state;

    engine
        .register_custom_syntax(
            ["GET", "BANCO", "HORAS", "$expr$"],
            false,
            move |context, inputs| {
                let person_id = context.eval_expression_tree(&inputs[0])?.to_string();
                trace!("GET BANCO HORAS: {person_id}");
                let result = json!({
                    "kind": "banco_horas",
                    "person_id": person_id,
                    "balance_hours": 0.0,
                    "pending_overtime": 0.0,
                    "pending_debit": 0.0,
                });
                Ok(serde_json_to_dynamic(&result))
            },
        )
        .expect("valid GET BANCO HORAS syntax");
}

fn register_ferias_balance(
    state: Arc<dyn BasicRuntime>,
    _user: UserSession,
    engine: &mut Engine,
) {
    let _state_clone = state;

    engine
        .register_custom_syntax(
            ["FERIAS", "BALANCE", "$expr$"],
            false,
            move |context, inputs| {
                let person_id = context.eval_expression_tree(&inputs[0])?.to_string();
                trace!("FERIAS BALANCE: {person_id}");
                let result = json!({
                    "kind": "ferias_balance",
                    "person_id": person_id,
                    "days_available": 30,
                    "days_taken": 0,
                    "days_remaining": 30,
                    "next_period_start": null,
                });
                Ok(serde_json_to_dynamic(&result))
            },
        )
        .expect("valid FERIAS BALANCE syntax");
}

fn register_banco_horas_report(
    state: Arc<dyn BasicRuntime>,
    _user: UserSession,
    engine: &mut Engine,
) {
    let _state_clone = state;

    engine
        .register_custom_syntax(
            ["BANCO", "HORAS", "REPORT", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let person_id = context.eval_expression_tree(&inputs[0])?.to_string();
                let period = context.eval_expression_tree(&inputs[1])?.to_string();
                trace!("BANCO HORAS REPORT: {person_id} period={period}");
                let result = json!({
                    "kind": "banco_horas_report",
                    "person_id": person_id,
                    "period": period,
                    "total_hours_worked": 0.0,
                    "total_overtime": 0.0,
                    "total_night_shift": 0.0,
                    "total_absences": 0,
                });
                Ok(serde_json_to_dynamic(&result))
            },
        )
        .expect("valid BANCO HORAS REPORT syntax");
}

fn serde_json_to_dynamic(v: &Value) -> Dynamic {
    Dynamic::from(v.to_string())
}
