use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BiometricType {
    Face,
    Fingerprint,
    Iris,
    Voice,
    Signature,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiometricTemplate {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub biometric_type: BiometricType,
    pub template_hash: String,
    pub embedding: Vec<f32>,
    pub quality_score: f64,
    pub liveness_score: Option<f64>,
    pub enrolled_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiometricMatch {
    pub matched: bool,
    pub template_id: Option<Uuid>,
    pub similarity: f64,
    pub threshold: f64,
    pub decision: MatchDecision,
    pub liveness_passed: bool,
    pub matched_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MatchDecision {
    Match,
    NoMatch,
    Inconclusive,
    LowQuality,
    LivenessFailed,
    TemplateNotFound,
    Expired,
}

pub struct BiometricMatcher {
    threshold: f64,
}

impl BiometricMatcher {
    pub fn new(threshold: f64) -> Self {
        Self { threshold: (threshold.clamp(0.0, 1.0)) }
    }

    pub fn with_defaults() -> Self {
        Self::new(0.85)
    }

    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }
        let mut dot = 0.0_f64;
        let mut na = 0.0_f64;
        let mut nb = 0.0_f64;
        for (x, y) in a.iter().zip(b.iter()) {
            dot += (*x as f64) * (*y as f64);
            na += (*x as f64).powi(2);
            nb += (*y as f64).powi(2);
        }
        let denom = (na * nb).sqrt();
        if denom == 0.0 { 0.0 } else { dot / denom }
    }

    pub fn match_template(
        &self,
        probe_embedding: &[f32],
        probe_liveness: Option<f64>,
        candidate: &BiometricTemplate,
    ) -> BiometricMatch {
        if !candidate.active {
            return BiometricMatch {
                matched: false,
                template_id: Some(candidate.id),
                similarity: 0.0,
                threshold: self.threshold,
                decision: MatchDecision::TemplateNotFound,
                liveness_passed: false,
                matched_at: Utc::now(),
            };
        }
        if let Some(exp) = candidate.expires_at {
            if Utc::now() > exp {
                return BiometricMatch {
                    matched: false,
                    template_id: Some(candidate.id),
                    similarity: 0.0,
                    threshold: self.threshold,
                    decision: MatchDecision::Expired,
                    liveness_passed: false,
                    matched_at: Utc::now(),
                };
            }
        }
        if candidate.quality_score < 0.5 {
            return BiometricMatch {
                matched: false,
                template_id: Some(candidate.id),
                similarity: 0.0,
                threshold: self.threshold,
                decision: MatchDecision::LowQuality,
                liveness_passed: false,
                matched_at: Utc::now(),
            };
        }
        let liveness = probe_liveness.unwrap_or(1.0);
        if liveness < 0.7 {
            return BiometricMatch {
                matched: false,
                template_id: Some(candidate.id),
                similarity: 0.0,
                threshold: self.threshold,
                decision: MatchDecision::LivenessFailed,
                liveness_passed: false,
                matched_at: Utc::now(),
            };
        }
        let similarity = Self::cosine_similarity(probe_embedding, &candidate.embedding);
        let decision = if similarity >= self.threshold {
            MatchDecision::Match
        } else if similarity >= self.threshold - 0.1 {
            MatchDecision::Inconclusive
        } else {
            MatchDecision::NoMatch
        };
        BiometricMatch {
            matched: decision == MatchDecision::Match,
            template_id: Some(candidate.id),
            similarity,
            threshold: self.threshold,
            decision,
            liveness_passed: true,
            matched_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn template(embedding: Vec<f32>) -> BiometricTemplate {
        BiometricTemplate {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            tenant_id: Uuid::nil(),
            biometric_type: BiometricType::Face,
            template_hash: "hash".into(),
            embedding,
            quality_score: 0.95,
            liveness_score: Some(0.9),
            enrolled_at: Utc::now(),
            expires_at: None,
            active: true,
        }
    }

    #[test]
    fn identical_vectors_match() {
        let m = BiometricMatcher::with_defaults();
        let v = vec![0.1_f32; 128];
        let r = m.match_template(&v, Some(0.9), &template(v.clone()));
        assert_eq!(r.decision, MatchDecision::Match);
    }

    #[test]
    fn orthogonal_vectors_dont_match() {
        let m = BiometricMatcher::with_defaults();
        let mut a = vec![0.0_f32; 128];
        let mut b = vec![0.0_f32; 128];
        a[0] = 1.0;
        b[64] = 1.0;
        let r = m.match_template(&a, Some(0.9), &template(b));
        assert_eq!(r.decision, MatchDecision::NoMatch);
    }
}
