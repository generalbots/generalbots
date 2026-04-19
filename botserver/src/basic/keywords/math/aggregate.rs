use crate::core::shared::models::UserSession;
use crate::core::shared::state::AppState;
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

#[cfg(test)]
mod tests {
    use rhai::Dynamic;

    #[test]
    fn test_sum() {
        let arr: Vec<Dynamic> = vec![
            Dynamic::from(10_i64),
            Dynamic::from(20_i64),
            Dynamic::from(30_i64),
        ];
        let sum: f64 = arr
            .iter()
            .filter_map(|v| v.as_int().ok().map(|i| i as f64))
            .sum();
        assert!((sum - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_sum_floats() {
        let arr: Vec<Dynamic> = vec![
            Dynamic::from(1.5_f64),
            Dynamic::from(2.5_f64),
            Dynamic::from(3.0_f64),
        ];
        let sum: f64 = arr.iter().filter_map(|v| v.as_float().ok()).sum();
        assert!((sum - 7.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_avg() {
        let arr: Vec<f64> = vec![10.0, 20.0, 30.0];
        let sum: f64 = arr.iter().sum();
        let avg = sum / arr.len() as f64;
        assert!((avg - 20.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_avg_single() {
        let arr: Vec<f64> = vec![42.0];
        let sum: f64 = arr.iter().sum();
        let avg = sum / arr.len() as f64;
        assert!((avg - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_empty_array_sum() {
        let arr: Vec<f64> = vec![];
        let result: f64 = arr.iter().sum();
        assert!(result.abs() < f64::EPSILON);
    }

    #[test]
    fn test_empty_array_avg() {
        let arr: Vec<f64> = vec![];
        let result = if arr.is_empty() {
            0.0
        } else {
            arr.iter().sum::<f64>() / arr.len() as f64
        };
        assert!(result.abs() < f64::EPSILON);
    }

    #[test]
    fn test_mixed_types() {
        let arr: Vec<Dynamic> = vec![
            Dynamic::from(10_i64),
            Dynamic::from(20.5_f64),
            Dynamic::from(30_i64),
        ];
        let sum: f64 = arr
            .iter()
            .filter_map(|v| {
                v.as_float()
                    .ok()
                    .or_else(|| v.as_int().ok().map(|i| i as f64))
            })
            .sum();
        assert!((sum - 60.5).abs() < f64::EPSILON);
    }
}
