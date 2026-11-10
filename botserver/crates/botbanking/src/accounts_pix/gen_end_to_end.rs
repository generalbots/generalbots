use super::*;

/// PIX end-to-end id: `E` + the transfer timestamp in milliseconds + the 32
/// hexadecimal characters of the transfer id. Split from the clock and the RNG
/// so the shape is unit-testable.
pub(super) fn end_to_end_id(timestamp_millis: i64, id: Uuid) -> String {
    format!("E{timestamp_millis}{}", id.simple())
}

pub(super) fn gen_end_to_end() -> String {
    end_to_end_id(Utc::now().timestamp_millis(), Uuid::new_v4())
}
