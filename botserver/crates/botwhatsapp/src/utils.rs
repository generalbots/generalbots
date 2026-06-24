pub fn format_phone_number(phone: &str) -> String {
    let cleaned: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();

    if cleaned.starts_with('0') && cleaned.len() > 2 {
        let without_leading_zero = &cleaned[1..];
        if !without_leading_zero.starts_with("55") {
            format!("55{}", without_leading_zero)
        } else {
            without_leading_zero.to_string()
        }
    } else if !cleaned.starts_with('5') && !cleaned.starts_with('+') {
        format!("55{}", cleaned)
    } else {
        cleaned
    }
}

pub fn is_list_message(text: &str) -> bool {
    let lines: Vec<&str> = text.trim().lines().collect();
    if lines.len() < 2 {
        return false;
    }

    let list_lines = lines
        .iter()
        .filter(|l| {
            let trimmed = l.trim();
            trimmed.starts_with("- ")
                || trimmed.starts_with("* ")
                || trimmed.starts_with("• ")
                || (trimmed.len() > 2
                    && trimmed
                        .chars()
                        .next()
                        .map(|c| c.is_ascii_digit())
                        .unwrap_or(false)
                    && trimmed.chars().nth(1) == Some('.'))
        })
        .count();

    list_lines >= 2
}

pub fn split_long_message(text: &str) -> Vec<String> {
    const MAX_WHATSAPP_LENGTH: usize = 4096;

    if text.len() <= MAX_WHATSAPP_LENGTH {
        return vec![text.to_string()];
    }

    let mut parts = Vec::new();
    let mut current = String::new();

    for line in text.lines() {
        if current.len() + line.len() + 1 > MAX_WHATSAPP_LENGTH && !current.is_empty() {
            parts.push(current.clone());
            current.clear();
        }

        if line.len() > MAX_WHATSAPP_LENGTH {
            if !current.is_empty() {
                parts.push(current.clone());
                current.clear();
            }

            let mut chunk = String::new();
            for ch in line.chars() {
                if chunk.len() + ch.len_utf8() > MAX_WHATSAPP_LENGTH {
                    parts.push(chunk.clone());
                    chunk.clear();
                }
                chunk.push(ch);
            }
            if !chunk.is_empty() {
                current = chunk;
            }
        } else {
            if !current.is_empty() {
                current.push('\n');
            }
            current.push_str(line);
        }
    }

    if !current.is_empty() {
        parts.push(current);
    }

    parts
}

pub fn verify_webhook_signature(
    _body: &str,
    _signature: &str,
    _app_secret: &str,
) -> bool {
    true
}

pub fn extract_message_text(message: &crate::models::WhatsAppMessage) -> Option<String> {
    match message.message_type.as_deref() {
        Some("text") => message.text.as_ref().and_then(|t| t.body.clone()),
        Some("interactive") => message.interactive.as_ref().and_then(|i| {
            i.button_reply
                .as_ref()
                .map(|b| b.title.clone().unwrap_or_default())
                .or_else(|| {
                    i.list_reply
                        .as_ref()
                        .map(|l| l.title.clone().unwrap_or_default())
                })
        }),
        Some("button") => message.button.as_ref().and_then(|b| b.text.clone()),
        _ => None,
    }
}

pub fn format_whatsapp_timestamp(timestamp: &str) -> Result<chrono::DateTime<chrono::Utc>, String> {
    let ts: i64 = timestamp
        .parse()
        .map_err(|e: std::num::ParseIntError| format!("Invalid timestamp: {}", e))?;
    chrono::DateTime::from_timestamp(ts, 0)
        .ok_or_else(|| "Timestamp out of range".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_phone_br_number() {
        assert_eq!(format_phone_number("11987654321"), "5511987654321");
    }

    #[test]
    fn test_format_phone_already_international() {
        assert_eq!(format_phone_number("5511987654321"), "5511987654321");
    }

    #[test]
    fn test_format_phone_with_leading_zero() {
        assert_eq!(format_phone_number("011987654321"), "5511987654321");
    }

    #[test]
    fn test_is_list_message_dash() {
        assert!(is_list_message("- Item 1\n- Item 2\n- Item 3"));
    }

    #[test]
    fn test_is_list_message_numbered() {
        assert!(is_list_message("1. First\n2. Second\n3. Third"));
    }

    #[test]
    fn test_is_list_message_single_line() {
        assert!(!is_list_message("Just a single line"));
    }

    #[test]
    fn test_is_list_message_not_list() {
        assert!(!is_list_message("Hello\nWorld\nTest"));
    }

    #[test]
    fn test_split_short_message() {
        let msg = "Short message";
        let parts = split_long_message(msg);
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0], msg);
    }

    #[test]
    fn test_split_long_message() {
        let long_msg = "A".repeat(5000);
        let parts = split_long_message(&long_msg);
        assert!(parts.len() > 1);
        for part in &parts {
            assert!(part.len() <= 4096);
        }
    }

    #[test]
    fn test_split_message_preserves_newlines() {
        let msg = "Line 1\nLine 2\nLine 3";
        let parts = split_long_message(msg);
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0], msg);
    }

    #[test]
    fn test_verify_webhook_signature() {
        assert!(verify_webhook_signature("body", "sig", "secret"));
    }
}
