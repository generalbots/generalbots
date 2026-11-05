//! Mid-run steering (#1701).
//!
//! A message sent while a turn is streaming used to wait for the turn
//! boundary. The stream loop now drains the steering queue between chunks,
//! aborts the provider stream, and continues with the same conversation plus
//! this instruction, so the correction is honored inside the running step.
//! The helpers here are dependency-free so the wording and the bound are
//! covered by executed tests.

/// A turn can be corrected a few times; beyond this the queue waits for the
/// next turn so a loop cannot rewrite the run forever.
pub const MAX_STEERS_PER_RUN: u8 = 3;

/// The instruction appended to the conversation when the user interrupts.
#[must_use]
pub fn instruction(steer: &str) -> String {
    format!(
        "[Steering] The user interrupted the run: {}. Adjust the remaining work and continue; do not repeat completed parts.",
        steer.trim()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instruction_carries_the_user_text() {
        let text = instruction("skip step 3 and add a summary at the end");
        assert!(text.contains("skip step 3 and add a summary at the end"));
        assert!(text.contains("do not repeat completed parts"));
    }

    #[test]
    fn instruction_trims_whitespace_and_is_safe_for_empty_input() {
        assert!(!instruction("   ").contains("interrupted the run:  ."));
        assert!(instruction("").starts_with("[Steering]"));
    }

    #[test]
    fn steering_is_bounded_per_run() {
        assert!((1..=5).contains(&MAX_STEERS_PER_RUN));
    }
}
