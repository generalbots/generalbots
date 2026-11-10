use super::*;

impl QuietHours {
    pub fn is_active(&self) -> bool {
        let now = Utc::now();
        let hour = now.format("%H").to_string().parse::<u8>().unwrap_or(0);
        self.is_active_at(hour)
    }

    /// Whether `hour` (0-23, local server time) falls inside the window.
    /// A window whose start is after its end wraps midnight (`22..6`); an
    /// inverted-or-equal window is treated as wrapping, so `0..0` means "all
    /// day" rather than "never" — the caller that configures a full-day
    /// silence expects silence.
    pub fn is_active_at(&self, hour: u8) -> bool {
        if self.start_hour < self.end_hour {
            hour >= self.start_hour && hour < self.end_hour
        } else {
            hour >= self.start_hour || hour < self.end_hour
        }
    }
}
