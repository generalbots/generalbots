use crate::core::shared::models::UserSession;
use crate::core::shared::state::AppState;
use log::debug;
use rhai::{Dynamic, Engine};
use std::sync::Arc;

pub fn isnull_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("ISNULL", |value: Dynamic| -> bool { value.is_unit() });

    engine.register_fn("isnull", |value: Dynamic| -> bool { value.is_unit() });

    engine.register_fn("IsNull", |value: Dynamic| -> bool { value.is_unit() });

    debug!("Registered ISNULL keyword");
}

#[cfg(test)]
mod tests {
    use rhai::Dynamic;

    #[test]
    fn test_isnull_unit() {
        let value = Dynamic::UNIT;
        assert!(value.is_unit());
    }

    #[test]
    fn test_isnull_not_unit() {
        let value = Dynamic::from("test");
        assert!(!value.is_unit());
    }

    #[test]
    fn test_isnull_number() {
        let value = Dynamic::from(42);
        assert!(!value.is_unit());
    }

    #[test]
    fn test_isnull_empty_string() {
        let value = Dynamic::from("");
        assert!(!value.is_unit());
    }

    #[test]
    fn test_isnull_bool() {
        let value = Dynamic::from(false);
        assert!(!value.is_unit());
    }
}
