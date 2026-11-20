//! Excel date serial conversion for both workbook date systems (#2572).
//!
//! XLSX stores dates as numbers. The 1900 system carries Lotus's phantom leap
//! day (serial 60) and is therefore 1462 days ahead of the 1904 system for the
//! same calendar date. Import/export must use the workbook's `dateSystem` or a
//! saved date shifts by that amount on the next open.

use chrono::{Datelike, Duration, NaiveDate};

use crate::types::DateSystem;

/// Difference between the two serial numbering schemes at the same date.
pub const EPOCH_DIFF_DAYS: i64 = 1462;

const EPOCH_1900: (i32, u32, u32) = (1900, 1, 1);
const EPOCH_1904: (i32, u32, u32) = (1904, 1, 1);
const MAX_SERIAL: i64 = 3_000_000;

fn epoch(date_system: DateSystem) -> Option<NaiveDate> {
    let (year, month, day) = match date_system {
        DateSystem::Excel1900 => EPOCH_1900,
        DateSystem::Excel1904 => EPOCH_1904,
    };
    NaiveDate::from_ymd_opt(year, month, day)
}

/// Converts a calendar date into the workbook's serial number.
/// The phantom 1900-02-29 has no serial in this direction.
pub fn date_to_serial(
    year: i32,
    month: u32,
    day: u32,
    date_system: DateSystem,
) -> Option<i64> {
    let date = NaiveDate::from_ymd_opt(year, month, day)?;
    let days = date.signed_duration_since(epoch(date_system)?).num_days();
    match date_system {
        // Serial 1 is 1900-01-01; from 1900-03-01 onwards one phantom day is
        // added, so 1900-02-28 = 59 and 1900-03-01 = 61.
        DateSystem::Excel1900 => Some(days + 1 + i64::from(days >= 59)),
        DateSystem::Excel1904 => Some(days),
    }
}

/// Converts a serial (fractional part = time of day) into calendar parts.
/// Serial 60 in the 1900 system is the phantom 1900-02-29.
pub fn serial_to_date_parts(
    serial: f64,
    date_system: DateSystem,
) -> Option<(i32, u32, u32)> {
    if !serial.is_finite() || serial < 0.0 {
        return None;
    }
    let whole = serial.floor() as i64;
    if whole > MAX_SERIAL {
        return None;
    }
    let date = match date_system {
        DateSystem::Excel1900 if whole == 60 => return Some((1900, 2, 29)),
        DateSystem::Excel1900 => {
            let adjusted = if whole < 60 { whole - 1 } else { whole - 2 };
            epoch(DateSystem::Excel1900)?.checked_add_signed(Duration::days(adjusted))?
        }
        DateSystem::Excel1904 => epoch(DateSystem::Excel1904)?
            .checked_add_signed(Duration::days(whole))?,
    };
    Some((date.year(), date.month(), date.day()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn excel_1900_reference_serials() {
        assert_eq!(date_to_serial(1900, 1, 1, DateSystem::Excel1900), Some(1));
        assert_eq!(date_to_serial(1900, 2, 28, DateSystem::Excel1900), Some(59));
        assert_eq!(date_to_serial(1900, 3, 1, DateSystem::Excel1900), Some(61));
        assert_eq!(date_to_serial(2000, 1, 1, DateSystem::Excel1900), Some(36526));
        assert_eq!(serial_to_date_parts(1.0, DateSystem::Excel1900), Some((1900, 1, 1)));
        assert_eq!(serial_to_date_parts(59.0, DateSystem::Excel1900), Some((1900, 2, 28)));
        assert_eq!(serial_to_date_parts(61.0, DateSystem::Excel1900), Some((1900, 3, 1)));
        assert_eq!(serial_to_date_parts(36526.5, DateSystem::Excel1900), Some((2000, 1, 1)));
    }

    #[test]
    fn excel_1904_reference_serials() {
        assert_eq!(date_to_serial(1904, 1, 1, DateSystem::Excel1904), Some(0));
        assert_eq!(date_to_serial(1904, 1, 2, DateSystem::Excel1904), Some(1));
        assert_eq!(serial_to_date_parts(0.0, DateSystem::Excel1904), Some((1904, 1, 1)));
        assert_eq!(serial_to_date_parts(1462.0, DateSystem::Excel1904), Some((1908, 1, 1)));
        assert_eq!(
            date_to_serial(1904, 1, 1, DateSystem::Excel1900),
            Some(EPOCH_DIFF_DAYS)
        );
    }

    #[test]
    fn phantom_leap_day_is_explicit() {
        assert_eq!(serial_to_date_parts(60.0, DateSystem::Excel1900), Some((1900, 2, 29)));
        assert_eq!(date_to_serial(1900, 2, 29, DateSystem::Excel1900), None);
    }

    #[test]
    fn real_dates_round_trip_in_both_systems() {
        let dates = [(1901, 5, 17), (1970, 1, 1), (2026, 11, 20), (2100, 3, 1)];
        for date_system in [DateSystem::Excel1900, DateSystem::Excel1904] {
            for (year, month, day) in dates {
                let serial = date_to_serial(year, month, day, date_system);
                assert!(serial.is_some(), "no serial for {year}-{month}-{day}");
                if let Some(serial) = serial {
                    assert_eq!(
                        serial_to_date_parts(serial as f64, date_system),
                        Some((year, month, day))
                    );
                }
            }
        }
    }

    #[test]
    fn invalid_serials_are_rejected() {
        assert_eq!(serial_to_date_parts(f64::NAN, DateSystem::Excel1900), None);
        assert_eq!(serial_to_date_parts(-1.0, DateSystem::Excel1900), None);
        assert_eq!(serial_to_date_parts(4.0e9, DateSystem::Excel1904), None);
    }
}
