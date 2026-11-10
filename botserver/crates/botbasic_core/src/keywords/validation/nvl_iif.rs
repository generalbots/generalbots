use botbasic_types::UserSession;
use botbasic_types::BasicRuntime;
use log::debug;
use rhai::{Dynamic, Engine};
use std::sync::Arc;

/// `NVL`/`COALESCE`/`IFNULL` semantics: a unit value or an empty string is
/// replaced by the default; everything else (including `0`, `false` and empty
/// collections) is returned untouched.
pub(crate) fn coalesce_value(value: Dynamic, default: Dynamic) -> Dynamic {
    if value.is_unit() || value.to_string().is_empty() {
        default
    } else {
        value
    }
}

/// `CHOOSE` semantics: 1-based positional pick, anything else is unit.
pub(crate) fn choose_value(index: i64, first: Dynamic, second: Dynamic) -> Dynamic {
    match index {
        1 => first,
        2 => second,
        _ => Dynamic::UNIT,
    }
}

/// `SWITCH_FUNC` semantics: string equality against the first candidate.
pub(crate) fn switch_value(
    expr: Dynamic,
    candidate: Dynamic,
    matched: Dynamic,
    default: Dynamic,
) -> Dynamic {
    if expr.to_string() == candidate.to_string() {
        matched
    } else {
        default
    }
}



pub fn nvl_keyword(_state: &Arc<dyn BasicRuntime>, _user: UserSession, engine: &mut Engine) {

    engine.register_fn("NVL", |value: Dynamic, default: Dynamic| -> Dynamic {
        coalesce_value(value, default)
    });

    engine.register_fn("nvl", |value: Dynamic, default: Dynamic| -> Dynamic {
        coalesce_value(value, default)
    });


    engine.register_fn("COALESCE", |value: Dynamic, default: Dynamic| -> Dynamic {
        coalesce_value(value, default)
    });

    engine.register_fn("coalesce", |value: Dynamic, default: Dynamic| -> Dynamic {
        coalesce_value(value, default)
    });


    engine.register_fn("IFNULL", |value: Dynamic, default: Dynamic| -> Dynamic {
        coalesce_value(value, default)
    });

    engine.register_fn("ifnull", |value: Dynamic, default: Dynamic| -> Dynamic {
        coalesce_value(value, default)
    });

    debug!("Registered NVL/COALESCE/IFNULL keywords");
}



pub fn iif_keyword(_state: &Arc<dyn BasicRuntime>, _user: UserSession, engine: &mut Engine) {

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
            choose_value(index, val1, val2)
        },
    );

    engine.register_fn(
        "choose",
        |index: i64, val1: Dynamic, val2: Dynamic| -> Dynamic {
            choose_value(index, val1, val2)
        },
    );




    engine.register_fn(
        "SWITCH_FUNC",
        |expr: Dynamic, val1: Dynamic, result1: Dynamic, default: Dynamic| -> Dynamic {
            switch_value(expr, val1, result1, default)
        },
    );

    debug!("Registered IIF/IF_FUNC/CHOOSE/SWITCH_FUNC keywords");
}

#[cfg(test)]
mod nvl_iif_tests {
    use super::*;

    #[test]
    fn coalesce_replaces_unit_and_empty_strings() {
        assert_eq!(coalesce_value(Dynamic::UNIT, Dynamic::from("d")).to_string(), "d");
        assert_eq!(
            coalesce_value(Dynamic::from(""), Dynamic::from("d")).to_string(),
            "d"
        );
    }

    #[test]
    fn coalesce_keeps_falsy_but_present_values() {
        assert_eq!(coalesce_value(Dynamic::from(0_i64), Dynamic::from(9_i64)).as_int().unwrap(), 0);
        assert!(!coalesce_value(Dynamic::from(false), Dynamic::from(true))
            .as_bool()
            .unwrap());
        assert_eq!(
            coalesce_value(Dynamic::from("x"), Dynamic::from("d")).to_string(),
            "x"
        );
    }

    #[test]
    fn choose_is_one_based_and_bounds_checked() {
        assert_eq!(choose_value(1, Dynamic::from("a"), Dynamic::from("b")).to_string(), "a");
        assert_eq!(choose_value(2, Dynamic::from("a"), Dynamic::from("b")).to_string(), "b");
        assert!(choose_value(0, Dynamic::from("a"), Dynamic::from("b")).is_unit());
        assert!(choose_value(3, Dynamic::from("a"), Dynamic::from("b")).is_unit());
        assert!(choose_value(-1, Dynamic::from("a"), Dynamic::from("b")).is_unit());
    }

    #[test]
    fn switch_compares_string_forms() {
        assert_eq!(
            switch_value(Dynamic::from(1_i64), Dynamic::from("1"), Dynamic::from("yes"), Dynamic::from("no"))
                .to_string(),
            "yes"
        );
        assert_eq!(
            switch_value(Dynamic::from("a"), Dynamic::from("b"), Dynamic::from("yes"), Dynamic::from("no"))
                .to_string(),
            "no"
        );
    }

    #[test]
    fn keywords_are_registered_in_both_casings() {
        let mut engine = Engine::new();
        engine.register_fn("NVL", |value: Dynamic, default: Dynamic| -> Dynamic {
            coalesce_value(value, default)
        });
        engine.register_fn("nvl", |value: Dynamic, default: Dynamic| -> Dynamic {
            coalesce_value(value, default)
        });
        engine.register_fn("IIF", |condition: bool, a: Dynamic, b: Dynamic| -> Dynamic {
            if condition { a } else { b }
        });
        engine.register_fn("iif", |condition: bool, a: Dynamic, b: Dynamic| -> Dynamic {
            if condition { a } else { b }
        });

        assert_eq!(engine.eval::<String>(r#"NVL("", "fallback")"#).unwrap(), "fallback");
        assert_eq!(engine.eval::<i64>("nvl(7, 9)").unwrap(), 7);
        assert_eq!(engine.eval::<String>(r#"IIF(true, "y", "n")"#).unwrap(), "y");
        assert_eq!(engine.eval::<String>(r#"iif(false, "y", "n")"#).unwrap(), "n");
    }
}
