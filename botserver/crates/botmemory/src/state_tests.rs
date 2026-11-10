//! Tests for the memory service configuration knobs.
//!
//! The service itself needs a pool and an injected LLM, so only the pure
//! clamping rule that guards the sampling percentage is unit-tested here; the
//! extraction hook it feeds is covered in `extract`.

use super::*;

#[test]
fn the_default_sampling_percentage_is_twenty() {
    assert_eq!(DEFAULT_SAMPLE_PCT, 20);
}

#[test]
fn sampling_percentages_above_one_hundred_are_clamped() {
    let cases = [
        (0u8, 0u8),
        (1, 1),
        (20, 20),
        (99, 99),
        (100, 100),
        (101, 100),
        (255, 100),
    ];
    for (input, expected) in cases {
        assert_eq!(clamp_sample_pct(input), expected, "clamp {input}");
    }
}
