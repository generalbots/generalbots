//! Facial liveness challenge / result types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type of challenge issued to the applicant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LivenessChallenge {
    /// "Blink twice"
    Blink,
    /// "Turn head left"
    TurnLeft,
    /// "Turn head right"
    TurnRight,
    /// "Smile"
    Smile,
    /// Random one of the above.
    Random,
}

/// Result of a liveness check.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LivenessResult {
    /// Server-assigned result ID.
    pub id: Uuid,
    /// Confidence score in [0.0, 1.0].
    pub confidence: f32,
    /// Whether the score is above the pass threshold.
    pub passed: bool,
    /// Frame timestamp from the video stream.
    pub captured_at: DateTime<Utc>,
    /// Detected face bounding box (x, y, w, h) in pixels.
    pub face_box: Option<(u32, u32, u32, u32)>,
}

/// Active liveness check session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LivenessCheck {
    /// Server-assigned check ID.
    pub id: Uuid,
    /// KYC case this check belongs to.
    pub case_id: Uuid,
    /// Issued challenge.
    pub challenge: LivenessChallenge,
    /// Expiration of the challenge.
    pub expires_at: DateTime<Utc>,
    /// Whether the check has been answered.
    pub answered: bool,
}

impl LivenessCheck {
    /// Returns true if the challenge has expired.
    pub fn is_expired(&self, now: DateTime<Utc>) -> bool {
        now > self.expires_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn check_expires_correctly() {
        let now = Utc::now();
        let check = LivenessCheck {
            id: Uuid::new_v4(),
            case_id: Uuid::new_v4(),
            challenge: LivenessChallenge::Blink,
            expires_at: now - Duration::seconds(1),
            answered: false,
        };
        assert!(check.is_expired(now));
    }
}
