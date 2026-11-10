//! Minimal JWT payload decoding for the browser-task API.
//!
//! #2066 — the payload was decoded with no look at the header, so a forged
//! unsecured token (`{"alg":"none"}`) could name any user id. An unsecured
//! token can never have been minted by the server, so it is refused outright.

pub(crate) fn jwt_payload(token: &str) -> Option<serde_json::Value> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    if !jwt_header_is_secured(parts[0]) {
        return None;
    }
    let decoded = base64_url_decode(parts[1]).ok()?;
    serde_json::from_slice::<serde_json::Value>(&decoded).ok()
}

/// #2066 — the payload was decoded with no look at the header, so a forged
/// unsecured token (`{"alg":"none"}`) could name any user id. An unsecured
/// token can never have been minted by the server, so it is refused outright.
pub(crate) fn jwt_header_is_secured(header_segment: &str) -> bool {
    let Some(decoded) = base64_url_decode(header_segment.trim_end_matches('=')).ok() else {
        return false;
    };
    let Ok(header) = serde_json::from_slice::<serde_json::Value>(&decoded) else {
        return false;
    };
    match header.get("alg").and_then(|v| v.as_str()) {
        Some(alg) => !alg.trim().eq_ignore_ascii_case("none"),
        None => false,
    }
}

pub(crate) fn base64_url_decode(input: &str) -> Result<Vec<u8>, String> {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = Vec::new();
    let mut buf: u32 = 0;
    let mut bits = 0u32;
    for ch in input.bytes() {
        if ch == b'=' {
            break;
        }
        let val = ALPHABET
            .iter()
            .position(|c| *c == ch)
            .ok_or_else(|| "invalid base64url character".to_string())? as u32;
        buf = (buf << 6) | val;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((buf >> bits) & 0xFF) as u8);
        }
    }
    Ok(out)
}
