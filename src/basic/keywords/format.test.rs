#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::test_util;
    #[test]
    fn test_currency_formatting() {
        test_util::setup();
        let formatted = format_currency(1234.56, "R$");
        assert_eq!(formatted, "R$ 1.234.56", "Currency formatting should use periods");
    }
    #[test]
    fn test_numeric_formatting_with_locale() {
        test_util::setup();
        let formatted = format_number(1234.56, 2);
        assert_eq!(formatted, "1.234.56", "Number formatting should use periods");
    }
    #[test]
    fn test_text_formatting() {
        test_util::setup();
        let formatted = format_text("hello", "HELLO");
        assert_eq!(formatted, "Result: helloHELLO", "Text formatting should concatenate");
    }
}
