#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::test_util;
    #[test]
    fn test_basic_module() {
        test_util::setup();
        assert!(true, "Basic module test");
    }
}
