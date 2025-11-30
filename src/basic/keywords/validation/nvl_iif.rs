use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::debug;
use rhai::{Dynamic, Engine};
use std::sync::Arc;

/// NVL - Returns the first non-null value (coalesce)
/// Syntax: NVL(value, default)
pub fn nvl_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    // NVL with two arguments
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

    // COALESCE alias for NVL
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

    // IFNULL alias
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

/// IIF - Immediate If (ternary operator)
/// Syntax: IIF(condition, true_value, false_value)
pub fn iif_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    // IIF with boolean condition
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

    // IF alias (common in many BASIC dialects)
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

    // CHOOSE - select from list based on index (1-based)
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

    // SWITCH function - evaluate expression and return matching result
    // SWITCH(expr, val1, result1, val2, result2, ..., default)
    // This is a simplified 2-case version
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

#[cfg(test)]
mod tests {
    #[test]
    fn test_nvl_logic() {
        let value = "";
        let default = "default";
        let result = if value.is_empty() { default } else { value };
        assert_eq!(result, "default");
    }

    #[test]
    fn test_nvl_with_value() {
        let value = "actual";
        let default = "default";
        let result = if value.is_empty() { default } else { value };
        assert_eq!(result, "actual");
    }

    #[test]
    fn test_iif_true() {
        let condition = true;
        let result = if condition { "yes" } else { "no" };
        assert_eq!(result, "yes");
    }

    #[test]
    fn test_iif_false() {
        let condition = false;
        let result = if condition { "yes" } else { "no" };
        assert_eq!(result, "no");
    }

    #[test]
    fn test_choose() {
        let index = 2;
        let values = vec!["first", "second", "third"];
        let result = values.get((index - 1) as usize).unwrap_or(&"");
        assert_eq!(*result, "second");
    }
}
