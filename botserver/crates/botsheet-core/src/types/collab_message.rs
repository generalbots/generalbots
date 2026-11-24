//! WebSocket collaboration envelope shared by the sheet collaboration server.
//!
//! #2447 — the browser (`collab.js::send`) sends `doc_id` and an epoch-ms
//! `timestamp`; historical clients sent `sheet_id` and an RFC3339 string.
//! Both shapes must deserialize or the server silently drops every frame.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollabMessage {
    pub msg_type: String,
    // Both are overwritten from the verified WS path, so accept either key
    // and default an omitted value instead of dropping the whole frame.
    #[serde(default, alias = "doc_id")]
    pub sheet_id: String,
    pub user_id: String,
    pub user_name: String,
    pub user_color: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub col: Option<u32>,
    /// The web client sends both `content` and `value`; serde's `alias`
    /// rejects that as a duplicate field, so keep the legacy key separate and
    /// let the server fold it into `value` after parsing (#2447).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worksheet_index: Option<usize>,
    /// Server-assigned monotonic sequence for the sheet session (#791).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seq: Option<u64>,
    #[serde(default = "Utc::now", deserialize_with = "deserialize_ws_timestamp")]
    pub timestamp: DateTime<Utc>,
}

/// Deserialize a WebSocket timestamp sent as RFC3339 text or epoch millis.
fn deserialize_ws_timestamp<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error as _;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum WsTimestamp {
        Rfc3339(String),
        EpochMillis(i64),
    }

    match WsTimestamp::deserialize(deserializer) {
        Ok(WsTimestamp::Rfc3339(value)) => DateTime::parse_from_rfc3339(&value)
            .map(|parsed| parsed.with_timezone(&Utc))
            .map_err(D::Error::custom),
        Ok(WsTimestamp::EpochMillis(ms)) => DateTime::from_timestamp_millis(ms)
            .ok_or_else(|| D::Error::custom("invalid epoch millisecond timestamp")),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::CollabMessage;

    /// The exact frame shape emitted by `collab.js::send` must not be dropped.
    #[test]
    fn browser_edit_frame_deserializes() {
        let frame = r##"{"msg_type":"edit","user_id":"anon-1","user_name":"Guest",
            "user_color":"#F7DC6F","doc_id":"sheet-1","timestamp":1795524348900,
            "row":0,"col":1,"content":"beta","value":"beta","length":4}"##;
        let msg: CollabMessage = match serde_json::from_str(frame) {
            Ok(msg) => msg,
            Err(error) => panic!("browser frame must deserialize: {error}"),
        };
        assert_eq!(msg.msg_type, "edit");
        assert_eq!(msg.sheet_id, "sheet-1");
        assert_eq!(msg.row, Some(0));
        assert_eq!(msg.col, Some(1));
        assert_eq!(msg.value.as_deref(), Some("beta"));
        assert_eq!(msg.timestamp.timestamp_millis(), 1795524348900);
    }

    #[test]
    fn legacy_rfc3339_frame_deserializes() {
        let frame = r##"{"msg_type":"hello","sheet_id":"sheet-1","user_id":"u",
            "user_name":"n","user_color":"#fff","timestamp":"2026-11-24T12:00:00Z"}"##;
        let msg: CollabMessage = match serde_json::from_str(frame) {
            Ok(msg) => msg,
            Err(error) => panic!("legacy frame must deserialize: {error}"),
        };
        assert_eq!(msg.sheet_id, "sheet-1");
        assert_eq!(msg.timestamp.to_rfc3339(), "2026-11-24T12:00:00+00:00");
    }

    #[test]
    fn missing_timestamp_and_sheet_id_default() {
        let frame = r##"{"msg_type":"ping","user_id":"u","user_name":"n","user_color":"#fff"}"##;
        let msg: CollabMessage = match serde_json::from_str(frame) {
            Ok(msg) => msg,
            Err(error) => panic!("minimal frame must deserialize: {error}"),
        };
        assert!(msg.sheet_id.is_empty());
        assert!(msg.timestamp.timestamp() > 0);
    }
}
