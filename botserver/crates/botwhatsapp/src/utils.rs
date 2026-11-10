pub fn format_phone_number(phone: &str) -> String {
    let cleaned: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();

    // An input without a single digit used to come back as "55" (the country
    // code prefixed to nothing), which then addressed a Graph API call to a
    // nonexistent recipient. Keep it empty so the caller can reject it.
    if cleaned.is_empty() {
        return String::new();
    }

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
                || is_numbered_marker(trimmed)
        })
        .count();

    list_lines >= 2
}

/// A numbered marker is `<digits>.` followed by a space or the end of the line.
///
/// The check used to stop at the dot, so "1.5 is a number" counted as a list
/// item and two decimal lines made the whole reply render as a list.
fn is_numbered_marker(line: &str) -> bool {
    let digits = line.chars().take_while(|c| c.is_ascii_digit()).count();
    if digits == 0 {
        return false;
    }

    match line[digits..].strip_prefix('.') {
        Some(rest) => rest.is_empty() || rest.starts_with(' '),
        None => false,
    }
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

/// Verifies a Meta webhook delivery.
///
/// Meta signs the **raw request body** with the app secret:
/// `X-Hub-Signature-256: sha256=<hex(hmac_sha256(app_secret, body))>`. The
/// signature must be computed over the bytes exactly as received — a
/// re-serialized `serde_json` value differs in key order and whitespace and
/// never matches.
///
/// This used to be a stub returning `true`, so any caller could post a forged
/// delivery. The digest is compared in constant time: the hex form is public
/// through the header, and a byte-wise `==` leaks how many leading characters
/// were guessed.
pub fn verify_webhook_signature(body: &str, signature: &str, app_secret: &str) -> bool {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    if app_secret.is_empty() {
        return false;
    }

    let provided = match signature.trim().strip_prefix("sha256=") {
        Some(hex_digest) if !hex_digest.is_empty() => hex_digest,
        _ => return false,
    };

    let mut mac = match Hmac::<Sha256>::new_from_slice(app_secret.as_bytes()) {
        Ok(mac) => mac,
        Err(e) => {
            log::error!("WhatsApp webhook HMAC key error: {e}");
            return false;
        }
    };
    mac.update(body.as_bytes());
    let computed = hex::encode(mac.finalize().into_bytes());

    // Meta always sends lowercase hex; an uppercase digest is not the same
    // string and is rejected rather than normalized.
    constant_time_eq(computed.as_bytes(), provided.as_bytes())
}

/// Byte-wise equality that does not short-circuit on the first difference.
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (left, right) in a.iter().zip(b.iter()) {
        diff |= left ^ right;
    }
    diff == 0
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
#[path = "utils_tests.rs"]
mod utils_tests;
