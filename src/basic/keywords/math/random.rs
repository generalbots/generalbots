use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::debug;
use rand::Rng;
use rhai::Engine;
use std::sync::Arc;

pub fn random_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("RANDOM", || -> f64 {
        let mut rng = rand::rng();
        rng.random::<f64>()
    });

    engine.register_fn("RANDOM", |max: i64| -> i64 {
        let mut rng = rand::rng();
        if max <= 0 {
            0
        } else {
            rng.random_range(0..max)
        }
    });

    engine.register_fn("RANDOM", |min: i64, max: i64| -> i64 {
        let mut rng = rand::rng();
        if min >= max {
            min
        } else {
            rng.random_range(min..=max)
        }
    });

    engine.register_fn("random", || -> f64 {
        let mut rng = rand::rng();
        rng.random::<f64>()
    });

    engine.register_fn("random", |max: i64| -> i64 {
        let mut rng = rand::rng();
        if max <= 0 {
            0
        } else {
            rng.random_range(0..max)
        }
    });

    engine.register_fn("RND", || -> f64 {
        let mut rng = rand::rng();
        rng.random::<f64>()
    });

    engine.register_fn("rnd", || -> f64 {
        let mut rng = rand::rng();
        rng.random::<f64>()
    });

    debug!("Registered RANDOM keyword");
}

pub fn mod_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("MOD", |a: i64, b: i64| -> i64 {
        if b == 0 {
            0
        } else {
            a % b
        }
    });

    engine.register_fn("MOD", |a: f64, b: f64| -> f64 {
        if b == 0.0 {
            0.0
        } else {
            a % b
        }
    });

    engine.register_fn("mod", |a: i64, b: i64| -> i64 {
        if b == 0 {
            0
        } else {
            a % b
        }
    });

    debug!("Registered MOD keyword");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_mod() {
        assert_eq!(17 % 5, 2);
        assert_eq!(10 % 3, 1);
        assert_eq!(0 % 5, 0);
    }
}
