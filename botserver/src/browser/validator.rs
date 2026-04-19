pub struct TestValidator {}

impl Default for TestValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl TestValidator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn validate_selectors(&self, _script: &str) -> Vec<String> {
        // Mock implementation
        vec![]
    }

    pub fn check_flaky_conditions(&self, _script: &str) -> Vec<String> {
        vec![]
    }
}
