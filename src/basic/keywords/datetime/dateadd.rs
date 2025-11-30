use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime};
use log::debug;
use rhai::Engine;
use std::sync::Arc;

pub fn dateadd_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn(
        "DATEADD",
        |date_str: &str, amount: i64, unit: &str| -> String {
            dateadd_impl(date_str, amount, unit)
        },
    );

    engine.register_fn(
        "dateadd",
        |date_str: &str, amount: i64, unit: &str| -> String {
            dateadd_impl(date_str, amount, unit)
        },
    );

    debug!("Registered DATEADD keyword");
}

pub fn datediff_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("DATEDIFF", |date1: &str, date2: &str, unit: &str| -> i64 {
        datediff_impl(date1, date2, unit)
    });

    engine.register_fn("datediff", |date1: &str, date2: &str, unit: &str| -> i64 {
        datediff_impl(date1, date2, unit)
    });

    debug!("Registered DATEDIFF keyword");
}

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
        .or_else(|| parse_date(trimmed).and_then(|d| d.and_hms_opt(0, 0, 0)))
}

fn dateadd_impl(date_str: &str, amount: i64, unit: &str) -> String {
    let unit_lower = unit.to_lowercase();

    if let Some(datetime) = parse_datetime(date_str) {
        let result = match unit_lower.as_str() {
            "day" | "days" | "d" => datetime + Duration::days(amount),
            "week" | "weeks" | "w" => datetime + Duration::weeks(amount),
            "hour" | "hours" | "h" => datetime + Duration::hours(amount),
            "minute" | "minutes" | "min" | "m" => datetime + Duration::minutes(amount),
            "second" | "seconds" | "sec" | "s" => datetime + Duration::seconds(amount),
            "month" | "months" => {
                if amount >= 0 {
                    datetime
                        .date()
                        .checked_add_months(chrono::Months::new(amount as u32))
                        .map(|d| d.and_time(datetime.time()))
                        .unwrap_or(datetime)
                } else {
                    datetime
                        .date()
                        .checked_sub_months(chrono::Months::new((-amount) as u32))
                        .map(|d| d.and_time(datetime.time()))
                        .unwrap_or(datetime)
                }
            }
            "year" | "years" | "y" => {
                let years = amount as i32;
                NaiveDate::from_ymd_opt(
                    datetime.year() + years,
                    datetime.month(),
                    datetime.day().min(28),
                )
                .map(|d| d.and_time(datetime.time()))
                .unwrap_or(datetime)
            }
            _ => datetime,
        };

        if date_str.contains(':') {
            result.format("%Y-%m-%d %H:%M:%S").to_string()
        } else {
            result.format("%Y-%m-%d").to_string()
        }
    } else {
        date_str.to_string()
    }
}

fn datediff_impl(date1: &str, date2: &str, unit: &str) -> i64 {
    let unit_lower = unit.to_lowercase();

    if let (Some(dt1), Some(dt2)) = (parse_datetime(date1), parse_datetime(date2)) {
        let duration = dt2.signed_duration_since(dt1);

        match unit_lower.as_str() {
            "day" | "days" | "d" => duration.num_days(),
            "week" | "weeks" | "w" => duration.num_weeks(),
            "hour" | "hours" | "h" => duration.num_hours(),
            "minute" | "minutes" | "min" | "m" => duration.num_minutes(),
            "second" | "seconds" | "sec" | "s" => duration.num_seconds(),
            "month" | "months" => {
                let months1 = dt1.year() * 12 + dt1.month() as i32;
                let months2 = dt2.year() * 12 + dt2.month() as i32;
                (months2 - months1) as i64
            }
            "year" | "years" | "y" => (dt2.year() - dt1.year()) as i64,
            _ => duration.num_days(),
        }
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dateadd_days() {
        assert_eq!(dateadd_impl("2025-01-15", 5, "day"), "2025-01-20");
        assert_eq!(dateadd_impl("2025-01-15", -10, "day"), "2025-01-05");
    }

    #[test]
    fn test_dateadd_months() {
        assert_eq!(dateadd_impl("2025-01-15", 1, "month"), "2025-02-15");
        assert_eq!(dateadd_impl("2025-01-15", -1, "month"), "2024-12-15");
    }

    #[test]
    fn test_dateadd_years() {
        assert_eq!(dateadd_impl("2025-01-15", 1, "year"), "2026-01-15");
    }

    #[test]
    fn test_datediff_days() {
        assert_eq!(datediff_impl("2025-01-01", "2025-01-15", "day"), 14);
        assert_eq!(datediff_impl("2025-01-15", "2025-01-01", "day"), -14);
    }

    #[test]
    fn test_datediff_months() {
        assert_eq!(datediff_impl("2025-01-01", "2025-03-01", "month"), 2);
    }

    #[test]
    fn test_datediff_years() {
        assert_eq!(datediff_impl("2024-01-01", "2025-01-01", "year"), 1);
    }

    #[test]
    fn test_parse_date() {
        assert!(parse_date("2025-01-15").is_some());
        assert!(parse_date("15/01/2025").is_some());
        assert!(parse_date("invalid").is_none());
    }
}
