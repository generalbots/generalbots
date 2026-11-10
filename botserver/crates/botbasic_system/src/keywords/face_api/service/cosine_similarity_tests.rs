//! Contract tests for the face-match similarity score (#2064).
//!
//! `cosine_similarity` decides whether two embeddings describe the same face.
//! Its edge behaviour (mismatched lengths, empty or zero vectors) must be a
//! score of `0.0` — a `NaN` would silently pass any `>= threshold` check.

use super::cosine_similarity;

fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 1e-6,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn identical_vectors_score_one() {
    assert_close(cosine_similarity(&[1.0, 2.0, 3.0], &[1.0, 2.0, 3.0]), 1.0);
    assert_close(cosine_similarity(&[0.25], &[0.25]), 1.0);
}

#[test]
fn orthogonal_vectors_score_zero() {
    assert_close(cosine_similarity(&[1.0, 0.0], &[0.0, 1.0]), 0.0);
}

#[test]
fn opposite_vectors_are_clamped_to_zero() {
    // A negative similarity is not a weaker match: it is clamped so callers
    // never compare against a negative threshold.
    assert_close(cosine_similarity(&[1.0, 1.0], &[-1.0, -1.0]), 0.0);
}

#[test]
fn a_partial_overlap_matches_the_analytic_value() {
    assert_close(
        cosine_similarity(&[1.0, 0.0], &[1.0, 1.0]),
        std::f32::consts::FRAC_1_SQRT_2,
    );
}

#[test]
fn scaling_a_vector_does_not_change_the_score() {
    let base = [0.3, -0.7, 1.4];
    let scaled = base.map(|value| value * 12.0);
    assert_close(cosine_similarity(&base, &scaled), 1.0);
}

#[test]
fn mismatched_empty_and_zero_vectors_score_zero() {
    assert_close(cosine_similarity(&[1.0, 2.0], &[1.0]), 0.0);
    assert_close(cosine_similarity(&[], &[]), 0.0);
    assert_close(cosine_similarity(&[0.0, 0.0], &[1.0, 1.0]), 0.0);
    assert_close(cosine_similarity(&[1.0, 1.0], &[0.0, 0.0]), 0.0);
}

#[test]
fn the_score_is_never_nan() {
    let cases = [
        ([0.0, 0.0], [0.0, 0.0]),
        ([1.0, 2.0], [0.0, 0.0]),
        ([0.5, -0.5], [-0.5, 0.5]),
    ];
    for (left, right) in cases {
        let score = cosine_similarity(&left, &right);
        assert!(!score.is_nan(), "{left:?} vs {right:?} produced NaN");
        assert!((0.0..=1.0).contains(&score), "score {score} is out of range");
    }
}
