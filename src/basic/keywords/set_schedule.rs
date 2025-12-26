use crate::shared::models::TriggerKind;
use diesel::prelude::*;
use log::trace;
use serde_json::{json, Value};
use uuid::Uuid;

pub fn parse_natural_schedule(input: &str) -> Result<String, String> {
    let input = input.trim().to_lowercase();

    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.len() == 5 && is_cron_expression(&parts) {
        return Ok(input);
    }

    parse_natural_language(&input)
}

fn is_cron_expression(parts: &[&str]) -> bool {
    parts.iter().all(|part| {
        part.chars()
            .all(|c| c.is_ascii_digit() || c == '*' || c == '/' || c == '-' || c == ',')
    })
}

fn parse_natural_language(input: &str) -> Result<String, String> {
    let input = input
        .replace("every ", "every_")
        .replace(" at ", "_at_")
        .replace(" from ", "_from_")
        .replace(" to ", "_to_")
        .replace(" during ", "_during_");

    let input = input.trim();

    if let Some(cron) = parse_simple_interval(input) {
        return Ok(cron);
    }

    if let Some(cron) = parse_at_time(input) {
        return Ok(cron);
    }

    if let Some(cron) = parse_day_pattern(input) {
        return Ok(cron);
    }

    if let Some(cron) = parse_combined_pattern(input) {
        return Ok(cron);
    }

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
    if input == "every_minute" || input == "every_1_minute" {
        return Some("* * * * *".to_string());
    }

    if let Some(rest) = input.strip_prefix("every_") {
        if let Some(num_str) = rest.strip_suffix("_minutes") {
            if let Ok(n) = num_str.parse::<u32>() {
                if n > 0 && n <= 59 {
                    return Some(format!("*/{} * * * *", n));
                }
            }
        }

        if rest == "hour" || rest == "1_hour" {
            return Some("0 * * * *".to_string());
        }

        if let Some(num_str) = rest.strip_suffix("_hours") {
            if let Ok(n) = num_str.parse::<u32>() {
                if n > 0 && n <= 23 {
                    return Some(format!("0 */{} * * *", n));
                }
            }
        }

        if rest == "day" {
            return Some("0 0 * * *".to_string());
        }

        if rest == "week" {
            return Some("0 0 * * 0".to_string());
        }

        if rest == "month" {
            return Some("0 0 1 * *".to_string());
        }

        if rest == "year" {
            return Some("0 0 1 1 *".to_string());
        }
    }

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
    let time_str = if let Some(rest) = input.strip_prefix("_at_") {
        rest
    } else if let Some(rest) = input.strip_prefix("at_") {
        rest
    } else {
        return None;
    };

    parse_time_to_cron(time_str, "*", "*")
}

fn parse_time_to_cron(time_str: &str, _hour_default: &str, dow: &str) -> Option<String> {
    if time_str == "midnight" {
        return Some(format!("0 0 * * {}", dow));
    }

    if time_str == "noon" {
        return Some(format!("0 12 * * {}", dow));
    }

    let (hour, minute) = parse_time_value(time_str)?;

    Some(format!("{} {} * * {}", minute, hour, dow))
}

fn parse_time_value(time_str: &str) -> Option<(u32, u32)> {
    let time_str = time_str.trim();

    let (time_part, is_pm) = if let Some(rest) = time_str.strip_suffix("am") {
        (rest, false)
    } else if let Some(rest) = time_str.strip_suffix("pm") {
        (rest, true)
    } else {
        (time_str, false)
    };

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

    if minute > 59 {
        return None;
    }

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

    if let Some(at_pos) = input.find("_at_") {
        let time_str = &input[at_pos + 4..];
        return parse_time_to_cron(time_str, "0", &dow);
    }

    Some(format!("0 0 * * {}", dow))
}

fn get_day_of_week(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();

    let day_part = input_lower.strip_prefix("every_").unwrap_or(&input_lower);

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
    if let Some(time_str) = input.strip_prefix("every_day_at_") {
        return parse_time_to_cron(time_str, "0", "*");
    }

    if let Some(time_str) = input
        .strip_prefix("every_weekday_at_")
        .or_else(|| input.strip_prefix("weekdays_at_"))
    {
        return parse_time_to_cron(time_str, "0", "1-5");
    }

    if let Some(time_str) = input
        .strip_prefix("every_weekend_at_")
        .or_else(|| input.strip_prefix("weekends_at_"))
    {
        return parse_time_to_cron(time_str, "0", "0,6");
    }

    if let Some(rest) = input.strip_prefix("every_hour_from_") {
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
    if input.contains("business_hours") || input.contains("business hours") {
        if input.starts_with("every_") {
            if let Some(rest) = input.strip_prefix("every_") {
                if let Some(minutes_pos) = rest.find("_minutes") {
                    let num_str = &rest[..minutes_pos];
                    if let Ok(n) = num_str.parse::<u32>() {
                        if n > 0 && n <= 59 {
                            return Some(format!("*/{} 9-17 * * 1-5", n));
                        }
                    }
                }

                if rest.starts_with("hour") {
                    return Some("0 9-17 * * 1-5".to_string());
                }
            }
        }

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
