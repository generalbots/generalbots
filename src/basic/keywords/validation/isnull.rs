use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::debug;
use rhai::{Dynamic, Engine};
use std::sync::Arc;

pub fn isnull_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    // ISNULL - Check if value is null/unit
    engine.register_fn("ISNULL", |value: Dynamic| -> bool { value.is_unit() });

    engine.register_fn("isnull", |value: Dynamic| -> bool { value.is_unit() });

    // IsNull - case variation
    engine.register_fn("IsNull", |value: Dynamic| -> bool { value.is_unit() });

    debug!("Registered ISNULL keyword");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_isnull_unit() {
        use rhai::Dynamic;
        let value = Dynamic::UNIT;
        assert!(value.is_unit());
    }

    #[test]
    fn test_isnull_not_unit() {
        use rhai::Dynamic;
        let value = Dynamic::from("test");
        assert!(!value.is_unit());
    }

    #[test]
    fn test_isnull_number() {
        use rhai::Dynamic;
        let value = Dynamic::from(42);
        assert!(!value.is_unit());
    }
}
