use log::info;

pub fn log_fraud_event(event_type: &str, entity_type: &str, score: i32, action: &str) {
    info!(
        "[FRAUD] {} | {}:{} | score={} | action={}",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
        entity_type,
        event_type,
        score,
        action
    );
}
