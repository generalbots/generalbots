











use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::debug;
use rhai::{Array, Engine};
use std::collections::HashSet;
use std::sync::Arc;


pub fn unique_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {

    engine.register_fn("UNIQUE", |arr: Array| -> Array { unique_array(arr) });


    engine.register_fn("unique", |arr: Array| -> Array { unique_array(arr) });


    engine.register_fn("DISTINCT", |arr: Array| -> Array { unique_array(arr) });

    engine.register_fn("distinct", |arr: Array| -> Array { unique_array(arr) });

    debug!("Registered UNIQUE keyword");
}



fn unique_array(arr: Array) -> Array {
    let mut seen: HashSet<String> = HashSet::new();
    let mut result = Array::new();

    for item in arr {


        let key = item.to_string();

        if !seen.contains(&key) {
            seen.insert(key);
            result.push(item);
        }
    }

    result
}
