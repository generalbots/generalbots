//! Biometric signature capture types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A single stroke point in a captured signature.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignatureStroke {
    /// X coordinate in canvas pixels.
    pub x: f32,
    /// Y coordinate in canvas pixels.
    pub y: f32,
    /// Pressure (0.0 — 1.0).
    pub pressure: f32,
    /// Milliseconds since the stroke started.
    pub t_ms: u32,
}

/// Captured signature before the cryptographic binding is applied.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignatureCapture {
    /// Server-assigned capture ID.
    pub id: Uuid,
    /// Document being signed (e.g. contract ID).
    pub document_id: String,
    /// Width of the capture canvas.
    pub width: u32,
    /// Height of the capture canvas.
    pub height: u32,
    /// All strokes recorded in time order.
    pub strokes: Vec<SignatureStroke>,
    /// Capture timestamp.
    pub captured_at: DateTime<Utc>,
}

impl SignatureCapture {
    /// Total number of recorded points.
    pub fn point_count(&self) -> usize {
        self.strokes.len()
    }

    /// Total duration in milliseconds across all strokes.
    pub fn duration_ms(&self) -> u32 {
        self.strokes
            .iter()
            .map(|s| s.t_ms)
            .max()
            .unwrap_or(0)
    }
}

/// A biometric signature with a hash binding it to the document bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BiometricSignature {
    /// Original capture ID.
    pub capture_id: Uuid,
    /// SHA-256 of the signed document.
    pub document_sha256: String,
    /// HMAC-SHA256 over (capture_id || document_sha256) using the signer's
    /// secret key.
    pub hmac: String,
    /// Signer user ID (Zitadel ID).
    pub signer_id: String,
    /// When the binding was created.
    pub signed_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stroke(t: u32) -> SignatureStroke {
        SignatureStroke {
            x: 10.0,
            y: 20.0,
            pressure: 0.5,
            t_ms: t,
        }
    }

    #[test]
    fn point_count_and_duration() {
        let capture = SignatureCapture {
            id: Uuid::new_v4(),
            document_id: "doc-1".to_string(),
            width: 400,
            height: 200,
            strokes: vec![stroke(0), stroke(50), stroke(150)],
            captured_at: Utc::now(),
        };
        assert_eq!(capture.point_count(), 3);
        assert_eq!(capture.duration_ms(), 150);
    }
}
