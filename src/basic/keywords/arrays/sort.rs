



use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::debug;
use rhai::{Array, Dynamic, Engine};
use std::sync::Arc;




pub fn sort_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {

    engine.register_fn("SORT", |arr: Array| -> Array { sort_array(arr, false) });

    engine.register_fn("sort", |arr: Array| -> Array { sort_array(arr, false) });


    engine.register_fn("SORT", |arr: Array, direction: &str| -> Array {
        let desc =
            direction.eq_ignore_ascii_case("DESC") || direction.eq_ignore_ascii_case("DESCENDING");
        sort_array(arr, desc)
    });

    engine.register_fn("sort", |arr: Array, direction: &str| -> Array {
        let desc =
            direction.eq_ignore_ascii_case("DESC") || direction.eq_ignore_ascii_case("DESCENDING");
        sort_array(arr, desc)
    });


    engine.register_fn("SORT_ASC", |arr: Array| -> Array { sort_array(arr, false) });

    engine.register_fn("sort_asc", |arr: Array| -> Array { sort_array(arr, false) });


    engine.register_fn("SORT_DESC", |arr: Array| -> Array { sort_array(arr, true) });

    engine.register_fn("sort_desc", |arr: Array| -> Array { sort_array(arr, true) });

    debug!("Registered SORT keyword");
}


fn sort_array(arr: Array, descending: bool) -> Array {
    let mut sorted = arr.clone();

    sorted.sort_by(|a, b| {
        let cmp = compare_dynamic(a, b);
        if descending {
            cmp.reverse()
        } else {
            cmp
        }
    });

    sorted
}


fn compare_dynamic(a: &Dynamic, b: &Dynamic) -> std::cmp::Ordering {

    if let (Some(a_num), Some(b_num)) = (to_f64(a), to_f64(b)) {
        return a_num
            .partial_cmp(&b_num)
            .unwrap_or(std::cmp::Ordering::Equal);
    }


    a.to_string().cmp(&b.to_string())
}


fn to_f64(value: &Dynamic) -> Option<f64> {
    if value.is_int() {
        value.as_int().ok().map(|i| i as f64)
    } else if value.is_float() {
        value.as_float().ok()
    } else if value.is_string() {
        value
            .clone()
            .into_string()
            .ok()
            .and_then(|s| s.parse().ok())
    } else {
        None
    }
}
