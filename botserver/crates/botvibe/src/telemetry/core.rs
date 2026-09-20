//! `telemetry::core` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

pub(crate) const MAX_EVENTS: usize = 50000;

impl Default for VibeTelemetry {
    fn default() -> Self {
        Self::new()
    }
}
