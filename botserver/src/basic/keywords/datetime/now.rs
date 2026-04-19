use crate::core::shared::models::UserSession;
use crate::core::shared::state::AppState;
use chrono::{Datelike, Local, Timelike, Utc};
use log::debug;
use rhai::{Dynamic, Engine, Map};
use std::sync::Arc;

fn create_datetime_map(local: chrono::DateTime<Local>) -> Map {
    let mut map = Map::new();

    map.insert("year".into(), Dynamic::from(i64::from(local.year())));
    map.insert("month".into(), Dynamic::from(i64::from(local.month())));
    map.insert("day".into(), Dynamic::from(i64::from(local.day())));

    map.insert("hour".into(), Dynamic::from(i64::from(local.hour())));
    map.insert("minute".into(), Dynamic::from(i64::from(local.minute())));
    map.insert("second".into(), Dynamic::from(i64::from(local.second())));

    map.insert(
        "weekday".into(),
        Dynamic::from(i64::from(local.weekday().num_days_from_sunday()) + 1),
    );

    let weekday_name = match local.weekday().num_days_from_sunday() {
        0 => "Sunday",
        1 => "Monday",
        2 => "Tuesday",
        3 => "Wednesday",
        4 => "Thursday",
        5 => "Friday",
        6 => "Saturday",
        _ => "Unknown",
    };
    map.insert(
        "weekday_name".into(),
        Dynamic::from(weekday_name.to_string()),
    );

    let month_name = match local.month() {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "Unknown",
    };
    map.insert("month_name".into(), Dynamic::from(month_name.to_string()));

    map.insert("timestamp".into(), Dynamic::from(local.timestamp()));

    map.insert(
        "formatted".into(),
        Dynamic::from(local.format("%Y-%m-%d %H:%M:%S").to_string()),
    );
    map.insert(
        "date".into(),
        Dynamic::from(local.format("%Y-%m-%d").to_string()),
    );
    map.insert(
        "time".into(),
        Dynamic::from(local.format("%H:%M:%S").to_string()),
    );
    map.insert(
        "iso".into(),
        Dynamic::from(local.format("%Y-%m-%dT%H:%M:%S%z").to_string()),
    );

    let quarter = ((local.month() - 1) / 3) + 1;
    map.insert("quarter".into(), Dynamic::from(i64::from(quarter)));

    map.insert(
        "day_of_year".into(),
        Dynamic::from(i64::from(local.ordinal())),
    );

    let is_weekend =
        local.weekday().num_days_from_sunday() == 0 || local.weekday().num_days_from_sunday() == 6;
    map.insert("is_weekend".into(), Dynamic::from(is_weekend));

    let is_pm = local.hour() >= 12;
    map.insert("is_pm".into(), Dynamic::from(is_pm));
    map.insert(
        "ampm".into(),
        Dynamic::from(if is_pm { "PM" } else { "AM" }.to_string()),
    );

    let hour12 = if local.hour() == 0 {
        12
    } else if local.hour() > 12 {
        local.hour() - 12
    } else {
        local.hour()
    };
    map.insert("hour12".into(), Dynamic::from(i64::from(hour12)));

    map
}

pub fn now_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("NOW", || -> Map { create_datetime_map(Local::now()) });

    engine.register_fn("now", || -> Map { create_datetime_map(Local::now()) });

    engine.register_fn("NOW_UTC", || -> Map {
        let utc = Utc::now();
        let local = utc.with_timezone(&Local);
        create_datetime_map(local)
    });

    engine.register_fn("NOW_STR", || -> String {
        Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
    });

    engine.register_fn("now_str", || -> String {
        Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
    });

    debug!("Registered NOW keyword with .property access");
}

pub fn today_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("TODAY", || -> Map {
        let now = Local::now();
        let mut map = Map::new();

        map.insert("year".into(), Dynamic::from(i64::from(now.year())));
        map.insert("month".into(), Dynamic::from(i64::from(now.month())));
        map.insert("day".into(), Dynamic::from(i64::from(now.day())));
        map.insert(
            "weekday".into(),
            Dynamic::from(i64::from(now.weekday().num_days_from_sunday()) + 1),
        );
        map.insert(
            "formatted".into(),
            Dynamic::from(now.format("%Y-%m-%d").to_string()),
        );
        map.insert(
            "day_of_year".into(),
            Dynamic::from(i64::from(now.ordinal())),
        );

        let is_weekend =
            now.weekday().num_days_from_sunday() == 0 || now.weekday().num_days_from_sunday() == 6;
        map.insert("is_weekend".into(), Dynamic::from(is_weekend));

        let quarter = ((now.month() - 1) / 3) + 1;
        map.insert("quarter".into(), Dynamic::from(i64::from(quarter)));

        map
    });

    engine.register_fn("today", || -> Map {
        let now = Local::now();
        let mut map = Map::new();

        map.insert("year".into(), Dynamic::from(i64::from(now.year())));
        map.insert("month".into(), Dynamic::from(i64::from(now.month())));
        map.insert("day".into(), Dynamic::from(i64::from(now.day())));
        map.insert(
            "weekday".into(),
            Dynamic::from(i64::from(now.weekday().num_days_from_sunday()) + 1),
        );
        map.insert(
            "formatted".into(),
            Dynamic::from(now.format("%Y-%m-%d").to_string()),
        );

        map
    });

    engine.register_fn("TODAY_STR", || -> String {
        Local::now().format("%Y-%m-%d").to_string()
    });

    engine.register_fn("today_str", || -> String {
        Local::now().format("%Y-%m-%d").to_string()
    });

    debug!("Registered TODAY keyword with .property access");
}

pub fn time_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("TIME", || -> Map {
        let now = Local::now();
        let mut map = Map::new();

        map.insert("hour".into(), Dynamic::from(i64::from(now.hour())));
        map.insert("minute".into(), Dynamic::from(i64::from(now.minute())));
        map.insert("second".into(), Dynamic::from(i64::from(now.second())));
        map.insert(
            "formatted".into(),
            Dynamic::from(now.format("%H:%M:%S").to_string()),
        );

        let is_pm = now.hour() >= 12;
        map.insert("is_pm".into(), Dynamic::from(is_pm));
        map.insert(
            "ampm".into(),
            Dynamic::from(if is_pm { "PM" } else { "AM" }.to_string()),
        );

        let hour12 = if now.hour() == 0 {
            12
        } else if now.hour() > 12 {
            now.hour() - 12
        } else {
            now.hour()
        };
        map.insert("hour12".into(), Dynamic::from(i64::from(hour12)));

        map
    });

    engine.register_fn("time", || -> Map {
        let now = Local::now();
        let mut map = Map::new();

        map.insert("hour".into(), Dynamic::from(i64::from(now.hour())));
        map.insert("minute".into(), Dynamic::from(i64::from(now.minute())));
        map.insert("second".into(), Dynamic::from(i64::from(now.second())));
        map.insert(
            "formatted".into(),
            Dynamic::from(now.format("%H:%M:%S").to_string()),
        );

        map
    });

    engine.register_fn("TIME_STR", || -> String {
        Local::now().format("%H:%M:%S").to_string()
    });

    engine.register_fn("TIMESTAMP", || -> i64 { Utc::now().timestamp() });

    engine.register_fn("timestamp", || -> i64 { Utc::now().timestamp() });

    debug!("Registered TIME keyword with .property access");
}

pub fn timestamp_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("UNIX_TIMESTAMP", || -> i64 { Utc::now().timestamp() });

    engine.register_fn("TIMESTAMP_MS", || -> i64 { Utc::now().timestamp_millis() });

    debug!("Registered TIMESTAMP keyword");
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;

    #[test]
    fn test_create_datetime_map() {
        let now = Local::now();
        let map = create_datetime_map(now);

        assert!(map.contains_key("year"));
        assert!(map.contains_key("month"));
        assert!(map.contains_key("day"));
        assert!(map.contains_key("hour"));
        assert!(map.contains_key("minute"));
        assert!(map.contains_key("second"));
        assert!(map.contains_key("weekday"));
        assert!(map.contains_key("timestamp"));
        assert!(map.contains_key("formatted"));
        assert!(map.contains_key("is_weekend"));
        assert!(map.contains_key("quarter"));
    }

    #[test]
    fn test_year_extraction() {
        let now = Local::now();
        let map = create_datetime_map(now);

        let year = map.get("year").unwrap().as_int().unwrap();
        assert!(year >= 2024);
    }

    #[test]
    fn test_month_range() {
        let now = Local::now();
        let map = create_datetime_map(now);

        let month = map.get("month").unwrap().as_int().unwrap();
        assert!((1..=12).contains(&month));
    }

    #[test]
    fn test_hour12_range() {
        let now = Local::now();
        let map = create_datetime_map(now);

        let hour12 = map.get("hour12").unwrap().as_int().unwrap();
        assert!((1..=12).contains(&hour12));
    }

    #[test]
    fn test_quarter_calculation() {
        let now = Local::now();
        let map = create_datetime_map(now);

        let quarter = map.get("quarter").unwrap().as_int().unwrap();
        assert!((1..=4).contains(&quarter));
    }
}
