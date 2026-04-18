use super::types::DriveMonitor;

impl DriveMonitor {
    pub fn normalize_config_value(value: &str) -> String {
        let trimmed = value.trim();
        if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("none") {
            String::new()
        } else {
            trimmed.to_string()
        }
    }
}
