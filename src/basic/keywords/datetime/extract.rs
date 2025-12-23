use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use chrono::{Datelike, NaiveDate, NaiveDateTime, Timelike};
use log::debug;
use rhai::Engine;
use std::sync::Arc;

fn parse_date(date_str: &str) -> Option<NaiveDate> {
    let trimmed = date_str.trim();
    NaiveDate::parse_from_str(trimmed, "%Y-%m-%d")
        .ok()
        .or_else(|| NaiveDate::parse_from_str(trimmed, "%d/%m/%Y").ok())
        .or_else(|| NaiveDate::parse_from_str(trimmed, "%m/%d/%Y").ok())
        .or_else(|| NaiveDate::parse_from_str(trimmed, "%Y/%m/%d").ok())
        .or_else(|| parse_datetime(trimmed).map(|dt| dt.date()))
}

fn parse_datetime(datetime_str: &str) -> Option<NaiveDateTime> {
    let trimmed = datetime_str.trim();
    NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%d %H:%M:%S")
        .ok()
        .or_else(|| NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%dT%H:%M:%S").ok())
        .or_else(|| NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%d %H:%M").ok())
        .or_else(|| parse_date(trimmed).map(|d| d.and_hms_opt(0, 0, 0).unwrap()))
}

pub fn year_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("YEAR", |date_str: &str| -> i64 {
        parse_date(date_str).map(|d| d.year() as i64).unwrap_or(0)
    });
    engine.register_fn("year", |date_str: &str| -> i64 {
        parse_date(date_str).map(|d| d.year() as i64).unwrap_or(0)
    });

    debug!("Registered YEAR keyword");
}

pub fn month_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("MONTH", |date_str: &str| -> i64 {
        parse_date(date_str).map(|d| d.month() as i64).unwrap_or(0)
    });
    engine.register_fn("month", |date_str: &str| -> i64 {
        parse_date(date_str).map(|d| d.month() as i64).unwrap_or(0)
    });

    debug!("Registered MONTH keyword");
}

pub fn day_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("DAY", |date_str: &str| -> i64 {
        parse_date(date_str).map(|d| d.day() as i64).unwrap_or(0)
    });
    engine.register_fn("day", |date_str: &str| -> i64 {
        parse_date(date_str).map(|d| d.day() as i64).unwrap_or(0)
    });

    debug!("Registered DAY keyword");
}

pub fn hour_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("HOUR", |datetime_str: &str| -> i64 {
        parse_datetime(datetime_str)
            .map(|d| d.hour() as i64)
            .unwrap_or(0)
    });
    engine.register_fn("hour", |datetime_str: &str| -> i64 {
        parse_datetime(datetime_str)
            .map(|d| d.hour() as i64)
            .unwrap_or(0)
    });

    debug!("Registered HOUR keyword");
}

pub fn minute_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("MINUTE", |datetime_str: &str| -> i64 {
        parse_datetime(datetime_str)
            .map(|d| d.minute() as i64)
            .unwrap_or(0)
    });
    engine.register_fn("minute", |datetime_str: &str| -> i64 {
        parse_datetime(datetime_str)
            .map(|d| d.minute() as i64)
            .unwrap_or(0)
    });

    debug!("Registered MINUTE keyword");
}

pub fn second_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("SECOND", |datetime_str: &str| -> i64 {
        parse_datetime(datetime_str)
            .map(|d| d.second() as i64)
            .unwrap_or(0)
    });
    engine.register_fn("second", |datetime_str: &str| -> i64 {
        parse_datetime(datetime_str)
            .map(|d| d.second() as i64)
            .unwrap_or(0)
    });

    debug!("Registered SECOND keyword");
}

pub fn weekday_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("WEEKDAY", |date_str: &str| -> i64 {
        parse_date(date_str)
            .map(|d| d.weekday().num_days_from_sunday() as i64 + 1)
            .unwrap_or(0)
    });
    engine.register_fn("weekday", |date_str: &str| -> i64 {
        parse_date(date_str)
            .map(|d| d.weekday().num_days_from_sunday() as i64 + 1)
            .unwrap_or(0)
    });

    debug!("Registered WEEKDAY keyword");
}

pub fn format_date_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("FORMAT_DATE", |date_str: &str, format: &str| -> String {
        format_date_impl(date_str, format)
    });
    engine.register_fn("format_date", |date_str: &str, format: &str| -> String {
        format_date_impl(date_str, format)
    });

    debug!("Registered FORMAT_DATE keyword");
}

pub fn isdate_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("ISDATE", |value: &str| -> bool {
        parse_date(value).is_some()
    });
    engine.register_fn("isdate", |value: &str| -> bool {
        parse_date(value).is_some()
    });
    engine.register_fn("IS_DATE", |value: &str| -> bool {
        parse_date(value).is_some()
    });

    debug!("Registered ISDATE keyword");
}

pub fn format_date_impl(date_str: &str, format: &str) -> String {
    if let Some(datetime) = parse_datetime(date_str) {
        let chrono_format = format
            .replace("YYYY", "%Y")
            .replace("yyyy", "%Y")
            .replace("YY", "%y")
            .replace("yy", "%y")
            .replace("MMMM", "%B")
            .replace("MMM", "%b")
            .replace("MM", "%m")
            .replace("DD", "%d")
            .replace("dd", "%d")
            .replace("HH", "%H")
            .replace("hh", "%I")
            .replace("mm", "%M")
            .replace("ss", "%S")
            .replace("AM/PM", "%p")
            .replace("am/pm", "%P");

        datetime.format(&chrono_format).to_string()
    } else {
        date_str.to_string()
    }
}
