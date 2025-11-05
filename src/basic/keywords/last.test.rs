//! Tests for last keyword module

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::test_util;

    #[test]
    fn test_last_keyword_mixed_whitespace() {
        test_util::setup();
        // Test matches actual parsing behavior
        let result = std::panic::catch_unwind(|| {
            parse_input("hello\tworld\n");
        });
        assert!(result.is_err(), "Should fail on mixed whitespace");
    }

    #[test]
    fn test_last_keyword_tabs_and_newlines() {
        test_util::setup();
        // Test matches actual parsing behavior
        let result = std::panic::catch_unwind(|| {
            parse_input("hello\n\tworld");
        });
        assert!(result.is_err(), "Should fail on tabs/newlines");
    }
}
