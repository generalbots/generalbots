//! WebSocket auth token parsing (split from [`super::websocket`]).

use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct WsAuthQuery {
    #[serde(default)]
    pub token: String,
}

/// Decodes the JWT payload and extracts `(user_id, display_name)`.
pub fn extract_user_from_token(token: &str) -> Option<(String, String)> {
    if token.is_empty() {
        return None;
    }
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    let payload_b64 = parts[1].replace('-', "+").replace('_', "/");
    let padding = (4 - payload_b64.len() % 4) % 4;
    let padded = format!("{}{}", payload_b64, "=".repeat(padding));
    if let Ok(bytes) = base64_decode(&padded) {
        if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&bytes) {
            let sub = v.get("sub").and_then(|x| x.as_str())
                .or_else(|| v.get("user_id").and_then(|x| x.as_str()))
                .or_else(|| v.get("email").and_then(|x| x.as_str()));
            let name = v.get("name").and_then(|x| x.as_str())
                .or_else(|| v.get("display_name").and_then(|x| x.as_str()))
                .or_else(|| v.get("preferred_username").and_then(|x| x.as_str()))
                .or_else(|| v.get("email").and_then(|x| x.as_str()));
            if let (Some(s), Some(n)) = (sub, name) {
                return Some((s.to_string(), n.to_string()));
            }
            if let Some(s) = sub {
                return Some((s.to_string(), s.to_string()));
            }
        }
    }
    None
}

fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    use std::str;
    let chars: Vec<u8> = input.bytes().collect();
    let mut out = Vec::with_capacity(chars.len() * 3 / 4);
    let mut buf: u32 = 0;
    let mut bits: u32 = 0;
    for &c in &chars {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' => break,
            _ => return Err(format!("invalid base64 char: {}", c as char)),
        };
        buf = (buf << 6) | v as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
            buf &= (1u32 << bits) - 1;
        }
    }
    if str::from_utf8(&out).is_ok() {
        Ok(out)
    } else {
        Err("decode failed".to_string())
    }
}
