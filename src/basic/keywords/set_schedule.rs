use crate::shared::models::TriggerKind;
use diesel::prelude::*;
use log::trace;
use serde_json::{json, Value};
use uuid::Uuid;

/// Parses natural language schedule expressions into cron format.
/// Uses a fast rule-based parser - no LLM or external dependencies needed.
///
/// # Supported Patterns
///
/// ## Time Intervals
/// - "every minute" -> "* * * * *"
/// - "every 5 minutes" -> "*/5 * * * *"
/// - "every hour" -> "0 * * * *"
/// - "every 2 hours" -> "0 */2 * * *"
/// - "every day" / "daily" -> "0 0 * * *"
/// - "every week" / "weekly" -> "0 0 * * 0"
/// - "every month" / "monthly" -> "0 0 1 * *"
///
/// ## Specific Times
/// - "at 9am" -> "0 9 * * *"
/// - "at 9:30am" -> "30 9 * * *"
/// - "at 14:00" -> "0 14 * * *"
/// - "at midnight" -> "0 0 * * *"
/// - "at noon" -> "0 12 * * *"
///
/// ## Day-specific
/// - "every monday" -> "0 0 * * 1"
/// - "every monday at 9am" -> "0 9 * * 1"
/// - "weekdays" / "every weekday" -> "0 0 * * 1-5"
/// - "weekends" -> "0 0 * * 0,6"
/// - "weekdays at 8am" -> "0 8 * * 1-5"
///
/// ## Combined
/// - "every day at 9am" -> "0 9 * * *"
/// - "every hour from 9 to 17" -> "0 9-17 * * *"
/// - "every 30 minutes during business hours" -> "*/30 9-17 * * 1-5"
///
/// ## Raw Cron (fallback)
/// - Any 5-part cron expression is passed through: "0 */2 * * *"
pub fn parse_natural_schedule(input: &str) -> Result<String, String> {
    let input = input.trim().to_lowercase();

    // If it looks like a cron expression (5 space-separated parts), pass through
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.len() == 5 && is_cron_expression(&parts) {
        return Ok(input);
    }

    // Parse natural language
    parse_natural_language(&input)
}

fn is_cron_expression(parts: &[&str]) -> bool {
    // Check if all parts look like valid cron fields
    parts.iter().all(|part| {
        part.chars()
            .all(|c| c.is_ascii_digit() || c == '*' || c == '/' || c == '-' || c == ',')
    })
}

fn parse_natural_language(input: &str) -> Result<String, String> {
    // Normalize input
    let input = input
        .replace("every ", "every_")
        .replace(" at ", "_at_")
        .replace(" from ", "_from_")
        .replace(" to ", "_to_")
        .replace(" during ", "_during_");

    let input = input.trim();

    // Simple interval patterns
    if let Some(cron) = parse_simple_interval(input) {
        return Ok(cron);
    }

    // Time-specific patterns
    if let Some(cron) = parse_at_time(input) {
        return Ok(cron);
    }

    // Day-specific patterns
    if let Some(cron) = parse_day_pattern(input) {
        return Ok(cron);
    }

    // Combined patterns
    if let Some(cron) = parse_combined_pattern(input) {
        return Ok(cron);
    }

    // Business hours patterns
    if let Some(cron) = parse_business_hours(input) {
        return Ok(cron);
    }

    Err(format!(
        "Could not parse schedule '{}'. Use patterns like 'every hour', 'every 5 minutes', \
         'at 9am', 'every monday at 9am', 'weekdays at 8am', or raw cron '0 * * * *'",
        input.replace('_', " ")
    ))
}

fn parse_simple_interval(input: &str) -> Option<String> {
    // every_minute
    if input == "every_minute" || input == "every_1_minute" {
        return Some("* * * * *".to_string());
    }

    // every_N_minutes
    if let Some(rest) = input.strip_prefix("every_") {
        if let Some(num_str) = rest.strip_suffix("_minutes") {
            if let Ok(n) = num_str.parse::<u32>() {
                if n > 0 && n <= 59 {
                    return Some(format!("*/{} * * * *", n));
                }
            }
        }

        // every_hour
        if rest == "hour" || rest == "1_hour" {
            return Some("0 * * * *".to_string());
        }

        // every_N_hours
        if let Some(num_str) = rest.strip_suffix("_hours") {
            if let Ok(n) = num_str.parse::<u32>() {
                if n > 0 && n <= 23 {
                    return Some(format!("0 */{} * * *", n));
                }
            }
        }

        // every_day / daily
        if rest == "day" {
            return Some("0 0 * * *".to_string());
        }

        // every_week / weekly
        if rest == "week" {
            return Some("0 0 * * 0".to_string());
        }

        // every_month / monthly
        if rest == "month" {
            return Some("0 0 1 * *".to_string());
        }

        // every_year / yearly
        if rest == "year" {
            return Some("0 0 1 1 *".to_string());
        }
    }

    // Aliases
    match input {
        "daily" => Some("0 0 * * *".to_string()),
        "weekly" => Some("0 0 * * 0".to_string()),
        "monthly" => Some("0 0 1 * *".to_string()),
        "yearly" | "annually" => Some("0 0 1 1 *".to_string()),
        "hourly" => Some("0 * * * *".to_string()),
        _ => None,
    }
}

fn parse_at_time(input: &str) -> Option<String> {
    // Handle "_at_TIME" patterns
    let time_str = if input.starts_with("_at_") {
        &input[4..]
    } else if input.starts_with("at_") {
        &input[3..]
    } else {
        return None;
    };

    parse_time_to_cron(time_str, "*", "*")
}

fn parse_time_to_cron(time_str: &str, hour_default: &str, dow: &str) -> Option<String> {
    // midnight
    if time_str == "midnight" {
        return Some(format!("0 0 * * {}", dow));
    }

    // noon
    if time_str == "noon" {
        return Some(format!("0 12 * * {}", dow));
    }

    // Parse time like "9am", "9:30am", "14:00", "9:30pm"
    let (hour, minute) = parse_time_value(time_str)?;

    Some(format!("{} {} * * {}", minute, hour, dow))
}

fn parse_time_value(time_str: &str) -> Option<(u32, u32)> {
    let time_str = time_str.trim();

    // Check for am/pm
    let (time_part, is_pm) = if time_str.ends_with("am") {
        (&time_str[..time_str.len() - 2], false)
    } else if time_str.ends_with("pm") {
        (&time_str[..time_str.len() - 2], true)
    } else {
        (time_str, false)
    };

    // Parse hour:minute or just hour
    let (hour, minute) = if time_part.contains(':') {
        let parts: Vec<&str> = time_part.split(':').collect();
        if parts.len() != 2 {
            return None;
        }
        let h: u32 = parts[0].parse().ok()?;
        let m: u32 = parts[1].parse().ok()?;
        (h, m)
    } else {
        let h: u32 = time_part.parse().ok()?;
        (h, 0)
    };

    // Validate
    if minute > 59 {
        return None;
    }

    // Convert to 24-hour if needed
    let hour = if is_pm && hour < 12 {
        hour + 12
    } else if !is_pm && hour == 12 && time_str.ends_with("am") {
        0
    } else {
        hour
    };

    if hour > 23 {
        return None;
    }

    Some((hour, minute))
}

fn parse_day_pattern(input: &str) -> Option<String> {
    let dow = get_day_of_week(input)?;

    // Check for "_at_TIME" suffix
    if let Some(at_pos) = input.find("_at_") {
        let time_str = &input[at_pos + 4..];
        return parse_time_to_cron(time_str, "0", &dow.to_string());
    }

    // Just the day, default to midnight
    Some(format!("0 0 * * {}", dow))
}

fn get_day_of_week(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();

    // Handle "every_DAYNAME" patterns
    let day_part = input_lower.strip_prefix("every_").unwrap_or(&input_lower);

    // Remove any "_at_..." suffix for day matching
    let day_part = if let Some(at_pos) = day_part.find("_at_") {
        &day_part[..at_pos]
    } else {
        day_part
    };

    match day_part {
        "sunday" | "sun" => Some("0".to_string()),
        "monday" | "mon" => Some("1".to_string()),
        "tuesday" | "tue" | "tues" => Some("2".to_string()),
        "wednesday" | "wed" => Some("3".to_string()),
        "thursday" | "thu" | "thurs" => Some("4".to_string()),
        "friday" | "fri" => Some("5".to_string()),
        "saturday" | "sat" => Some("6".to_string()),
        "weekday" | "weekdays" => Some("1-5".to_string()),
        "weekend" | "weekends" => Some("0,6".to_string()),
        _ => None,
    }
}

fn parse_combined_pattern(input: &str) -> Option<String> {
    // every_day_at_TIME
    if input.starts_with("every_day_at_") {
        let time_str = &input[13..];
        return parse_time_to_cron(time_str, "0", "*");
    }

    // every_weekday_at_TIME
    if input.starts_with("every_weekday_at_") || input.starts_with("weekdays_at_") {
        let time_str = if input.starts_with("every_weekday_at_") {
            &input[17..]
        } else {
            &input[12..]
        };
        return parse_time_to_cron(time_str, "0", "1-5");
    }

    // every_weekend_at_TIME / weekends_at_TIME
    if input.starts_with("every_weekend_at_") || input.starts_with("weekends_at_") {
        let time_str = if input.starts_with("every_weekend_at_") {
            &input[17..]
        } else {
            &input[12..]
        };
        return parse_time_to_cron(time_str, "0", "0,6");
    }

    // every_hour_from_X_to_Y (e.g., "every_hour_from_9_to_17")
    if input.starts_with("every_hour_from_") {
        let rest = &input[16..];
        if let Some(to_pos) = rest.find("_to_") {
            let start: u32 = rest[..to_pos].parse().ok()?;
            let end: u32 = rest[to_pos + 4..].parse().ok()?;
            if start <= 23 && end <= 23 {
                return Some(format!("0 {}-{} * * *", start, end));
            }
        }
    }

    None
}

fn parse_business_hours(input: &str) -> Option<String> {
    // business_hours or during_business_hours
    if input.contains("business_hours") || input.contains("business hours") {
        // Default business hours: 9-17, weekdays

        // Check for interval prefix
        if input.starts_with("every_") {
            // every_N_minutes_during_business_hours
            if let Some(rest) = input.strip_prefix("every_") {
                if let Some(minutes_pos) = rest.find("_minutes") {
                    let num_str = &rest[..minutes_pos];
                    if let Ok(n) = num_str.parse::<u32>() {
                        if n > 0 && n <= 59 {
                            return Some(format!("*/{} 9-17 * * 1-5", n));
                        }
                    }
                }

                // every_hour_during_business_hours
                if rest.starts_with("hour") {
                    return Some("0 9-17 * * 1-5".to_string());
                }
            }
        }

        // Just "business hours" or "during business hours"
        return Some("0 9-17 * * 1-5".to_string());
    }

    None
}

pub fn execute_set_schedule(
    conn: &mut diesel::PgConnection,
    cron_or_natural: &str,
    script_name: &str,
    bot_uuid: Uuid,
) -> Result<Value, Box<dyn std::error::Error>> {
    // Parse natural language to cron if needed
    let cron = parse_natural_schedule(cron_or_natural)?;

    trace!(
        "Scheduling SET SCHEDULE cron: {} (from: '{}'), script: {}, bot_id: {:?}",
        cron,
        cron_or_natural,
        script_name,
        bot_uuid
    );

    use crate::shared::models::bots::dsl::bots;
    let bot_exists: bool = diesel::select(diesel::dsl::exists(
        bots.filter(crate::shared::models::bots::dsl::id.eq(bot_uuid)),
    ))
    .get_result(conn)?;

    if !bot_exists {
        return Err(format!("Bot with id {} does not exist", bot_uuid).into());
    }

    use crate::shared::models::system_automations::dsl::*;

    let new_automation = (
        bot_id.eq(bot_uuid),
        kind.eq(TriggerKind::Scheduled as i32),
        schedule.eq(&cron),
        param.eq(script_name),
        is_active.eq(true),
    );

    let update_result = diesel::update(system_automations)
        .filter(bot_id.eq(bot_uuid))
        .filter(kind.eq(TriggerKind::Scheduled as i32))
        .filter(param.eq(script_name))
        .set((
            schedule.eq(&cron),
            is_active.eq(true),
            last_triggered.eq(None::<chrono::DateTime<chrono::Utc>>),
        ))
        .execute(&mut *conn)?;

    let result = if update_result == 0 {
        diesel::insert_into(system_automations)
            .values(&new_automation)
            .execute(&mut *conn)?
    } else {
        update_result
    };

    Ok(json!({
        "command": "set_schedule",
        "schedule": cron,
        "original_input": cron_or_natural,
        "script": script_name,
        "bot_id": bot_uuid.to_string(),
        "rows_affected": result
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_every_minute() {
        assert_eq!(parse_natural_schedule("every minute").unwrap(), "* * * * *");
    }

    #[test]
    fn test_every_n_minutes() {
        assert_eq!(
            parse_natural_schedule("every 5 minutes").unwrap(),
            "*/5 * * * *"
        );
        assert_eq!(
            parse_natural_schedule("every 15 minutes").unwrap(),
            "*/15 * * * *"
        );
        assert_eq!(
            parse_natural_schedule("every 30 minutes").unwrap(),
            "*/30 * * * *"
        );
    }

    #[test]
    fn test_every_hour() {
        assert_eq!(parse_natural_schedule("every hour").unwrap(), "0 * * * *");
        assert_eq!(parse_natural_schedule("hourly").unwrap(), "0 * * * *");
    }

    #[test]
    fn test_every_n_hours() {
        assert_eq!(
            parse_natural_schedule("every 2 hours").unwrap(),
            "0 */2 * * *"
        );
        assert_eq!(
            parse_natural_schedule("every 6 hours").unwrap(),
            "0 */6 * * *"
        );
    }

    #[test]
    fn test_every_day() {
        assert_eq!(parse_natural_schedule("every day").unwrap(), "0 0 * * *");
        assert_eq!(parse_natural_schedule("daily").unwrap(), "0 0 * * *");
    }

    #[test]
    fn test_every_week() {
        assert_eq!(parse_natural_schedule("every week").unwrap(), "0 0 * * 0");
        assert_eq!(parse_natural_schedule("weekly").unwrap(), "0 0 * * 0");
    }

    #[test]
    fn test_every_month() {
        assert_eq!(parse_natural_schedule("every month").unwrap(), "0 0 1 * *");
        assert_eq!(parse_natural_schedule("monthly").unwrap(), "0 0 1 * *");
    }

    #[test]
    fn test_at_time() {
        assert_eq!(parse_natural_schedule("at 9am").unwrap(), "0 9 * * *");
        assert_eq!(parse_natural_schedule("at 9:30am").unwrap(), "30 9 * * *");
        assert_eq!(parse_natural_schedule("at 2pm").unwrap(), "0 14 * * *");
        assert_eq!(parse_natural_schedule("at 14:00").unwrap(), "0 14 * * *");
        assert_eq!(parse_natural_schedule("at midnight").unwrap(), "0 0 * * *");
        assert_eq!(parse_natural_schedule("at noon").unwrap(), "0 12 * * *");
    }

    #[test]
    fn test_day_of_week() {
        assert_eq!(parse_natural_schedule("every monday").unwrap(), "0 0 * * 1");
        assert_eq!(parse_natural_schedule("every friday").unwrap(), "0 0 * * 5");
        assert_eq!(parse_natural_schedule("every sunday").unwrap(), "0 0 * * 0");
    }

    #[test]
    fn test_day_with_time() {
        assert_eq!(
            parse_natural_schedule("every monday at 9am").unwrap(),
            "0 9 * * 1"
        );
        assert_eq!(
            parse_natural_schedule("every friday at 5pm").unwrap(),
            "0 17 * * 5"
        );
    }

    #[test]
    fn test_weekdays() {
        assert_eq!(parse_natural_schedule("weekdays").unwrap(), "0 0 * * 1-5");
        assert_eq!(
            parse_natural_schedule("every weekday").unwrap(),
            "0 0 * * 1-5"
        );
        assert_eq!(
            parse_natural_schedule("weekdays at 8am").unwrap(),
            "0 8 * * 1-5"
        );
    }

    #[test]
    fn test_weekends() {
        assert_eq!(parse_natural_schedule("weekends").unwrap(), "0 0 * * 0,6");
        assert_eq!(
            parse_natural_schedule("every weekend").unwrap(),
            "0 0 * * 0,6"
        );
    }

    #[test]
    fn test_combined() {
        assert_eq!(
            parse_natural_schedule("every day at 9am").unwrap(),
            "0 9 * * *"
        );
        assert_eq!(
            parse_natural_schedule("every day at 6:30pm").unwrap(),
            "30 18 * * *"
        );
    }

    #[test]
    fn test_hour_range() {
        assert_eq!(
            parse_natural_schedule("every hour from 9 to 17").unwrap(),
            "0 9-17 * * *"
        );
    }

    #[test]
    fn test_business_hours() {
        assert_eq!(
            parse_natural_schedule("business hours").unwrap(),
            "0 9-17 * * 1-5"
        );
        assert_eq!(
            parse_natural_schedule("every 30 minutes during business hours").unwrap(),
            "*/30 9-17 * * 1-5"
        );
        assert_eq!(
            parse_natural_schedule("every hour during business hours").unwrap(),
            "0 9-17 * * 1-5"
        );
    }

    #[test]
    fn test_raw_cron_passthrough() {
        assert_eq!(parse_natural_schedule("0 * * * *").unwrap(), "0 * * * *");
        assert_eq!(
            parse_natural_schedule("*/5 * * * *").unwrap(),
            "*/5 * * * *"
        );
        assert_eq!(
            parse_natural_schedule("0 9-17 * * 1-5").unwrap(),
            "0 9-17 * * 1-5"
        );
    }

    #[test]
    fn test_invalid_input() {
        assert!(parse_natural_schedule("potato salad").is_err());
        assert!(parse_natural_schedule("every 100 minutes").is_err()); // > 59
    }
}
