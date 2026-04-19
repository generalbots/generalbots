use crate::core::shared::models::UserSession;
use crate::core::shared::state::AppState;
use log::debug;
use rhai::{Dynamic, Engine};
use std::sync::Arc;



pub fn nvl_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {

    engine.register_fn("NVL", |value: Dynamic, default: Dynamic| -> Dynamic {
        if value.is_unit() || value.to_string().is_empty() {
            default
        } else {
            value
        }
    });

    engine.register_fn("nvl", |value: Dynamic, default: Dynamic| -> Dynamic {
        if value.is_unit() || value.to_string().is_empty() {
            default
        } else {
            value
        }
    });


    engine.register_fn("COALESCE", |value: Dynamic, default: Dynamic| -> Dynamic {
        if value.is_unit() || value.to_string().is_empty() {
            default
        } else {
            value
        }
    });

    engine.register_fn("coalesce", |value: Dynamic, default: Dynamic| -> Dynamic {
        if value.is_unit() || value.to_string().is_empty() {
            default
        } else {
            value
        }
    });


    engine.register_fn("IFNULL", |value: Dynamic, default: Dynamic| -> Dynamic {
        if value.is_unit() || value.to_string().is_empty() {
            default
        } else {
            value
        }
    });

    engine.register_fn("ifnull", |value: Dynamic, default: Dynamic| -> Dynamic {
        if value.is_unit() || value.to_string().is_empty() {
            default
        } else {
            value
        }
    });

    debug!("Registered NVL/COALESCE/IFNULL keywords");
}



pub fn iif_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {

    engine.register_fn(
        "IIF",
        |condition: bool, true_val: Dynamic, false_val: Dynamic| -> Dynamic {
            if condition {
                true_val
            } else {
                false_val
            }
        },
    );

    engine.register_fn(
        "iif",
        |condition: bool, true_val: Dynamic, false_val: Dynamic| -> Dynamic {
            if condition {
                true_val
            } else {
                false_val
            }
        },
    );


    engine.register_fn(
        "IF_FUNC",
        |condition: bool, true_val: Dynamic, false_val: Dynamic| -> Dynamic {
            if condition {
                true_val
            } else {
                false_val
            }
        },
    );


    engine.register_fn(
        "CHOOSE",
        |index: i64, val1: Dynamic, val2: Dynamic| -> Dynamic {
            match index {
                1 => val1,
                2 => val2,
                _ => Dynamic::UNIT,
            }
        },
    );

    engine.register_fn(
        "choose",
        |index: i64, val1: Dynamic, val2: Dynamic| -> Dynamic {
            match index {
                1 => val1,
                2 => val2,
                _ => Dynamic::UNIT,
            }
        },
    );




    engine.register_fn(
        "SWITCH_FUNC",
        |expr: Dynamic, val1: Dynamic, result1: Dynamic, default: Dynamic| -> Dynamic {
            if expr.to_string() == val1.to_string() {
                result1
            } else {
                default
            }
        },
    );

    debug!("Registered IIF/IF_FUNC/CHOOSE/SWITCH_FUNC keywords");
}
