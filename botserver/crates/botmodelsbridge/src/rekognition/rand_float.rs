

pub(super) fn rand_float() -> f32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|e| {
            log::warn!("System clock precedes UNIX_EPOCH: {e}");
            std::time::Duration::ZERO
        })
        .subsec_nanos();
    (nanos % 1000) as f32 / 1000.0
}
