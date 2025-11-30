use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use chrono::{Local, Utc};
use log::debug;
use rhai::Engine;
use std::sync::Arc;

pub fn now_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("NOW", || -> String {
        Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
    });

    engine.register_fn("now", || -> String {
        Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
    });

    engine.register_fn("NOW_UTC", || -> String {
        Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
    });

    debug!("Registered NOW keyword");
}

pub fn today_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("TODAY", || -> String {
        Local::now().format("%Y-%m-%d").to_string()
    });

    engine.register_fn("today", || -> String {
        Local::now().format("%Y-%m-%d").to_string()
    });

    debug!("Registered TODAY keyword");
}

pub fn time_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("TIME", || -> String {
        Local::now().format("%H:%M:%S").to_string()
    });

    engine.register_fn("time", || -> String {
        Local::now().format("%H:%M:%S").to_string()
    });

    engine.register_fn("TIMESTAMP", || -> i64 { Utc::now().timestamp() });

    engine.register_fn("timestamp", || -> i64 { Utc::now().timestamp() });

    debug!("Registered TIME keyword");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_now_format() {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        assert!(now.len() == 19);
        assert!(now.contains('-'));
        assert!(now.contains(':'));
    }

    #[test]
    fn test_today_format() {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        assert!(today.len() == 10);
        assert!(today.contains('-'));
    }

    #[test]
    fn test_time_format() {
        let time = chrono::Local::now().format("%H:%M:%S").to_string();
        assert!(time.len() == 8);
        assert!(time.contains(':'));
    }

    #[test]
    fn test_timestamp() {
        let ts = chrono::Utc::now().timestamp();
        assert!(ts > 1700000000);
    }
}
