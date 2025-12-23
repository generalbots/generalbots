use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::debug;
use rhai::{Array, Engine};
use std::sync::Arc;

pub fn sum_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("SUM", |arr: Array| -> f64 {
        arr.iter()
            .filter_map(|v| {
                v.as_float()
                    .ok()
                    .or_else(|| v.as_int().ok().map(|i| i as f64))
            })
            .sum()
    });

    engine.register_fn("sum", |arr: Array| -> f64 {
        arr.iter()
            .filter_map(|v| {
                v.as_float()
                    .ok()
                    .or_else(|| v.as_int().ok().map(|i| i as f64))
            })
            .sum()
    });

    debug!("Registered SUM keyword");
}

pub fn avg_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("AVG", |arr: Array| -> f64 {
        if arr.is_empty() {
            return 0.0;
        }
        let values: Vec<f64> = arr
            .iter()
            .filter_map(|v| {
                v.as_float()
                    .ok()
                    .or_else(|| v.as_int().ok().map(|i| i as f64))
            })
            .collect();

        if values.is_empty() {
            return 0.0;
        }

        let sum: f64 = values.iter().sum();
        sum / values.len() as f64
    });

    engine.register_fn("avg", |arr: Array| -> f64 {
        if arr.is_empty() {
            return 0.0;
        }
        let values: Vec<f64> = arr
            .iter()
            .filter_map(|v| {
                v.as_float()
                    .ok()
                    .or_else(|| v.as_int().ok().map(|i| i as f64))
            })
            .collect();

        if values.is_empty() {
            return 0.0;
        }

        let sum: f64 = values.iter().sum();
        sum / values.len() as f64
    });

    engine.register_fn("AVERAGE", |arr: Array| -> f64 {
        if arr.is_empty() {
            return 0.0;
        }
        let values: Vec<f64> = arr
            .iter()
            .filter_map(|v| {
                v.as_float()
                    .ok()
                    .or_else(|| v.as_int().ok().map(|i| i as f64))
            })
            .collect();

        if values.is_empty() {
            return 0.0;
        }

        let sum: f64 = values.iter().sum();
        sum / values.len() as f64
    });

    debug!("Registered AVG/AVERAGE keyword");
}
