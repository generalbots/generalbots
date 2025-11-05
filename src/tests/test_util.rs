//! Common test utilities for the botserver project

use std::sync::Once;

static INIT: Once = Once::new();

/// Setup function to be called at the beginning of each test module
pub fn setup() {
    INIT.call_once(|| {
        // Initialize any test configuration here
    });
}

/// Simple assertion macro for better test error messages
#[macro_export]
macro_rules! assert_ok {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(err) => panic!("Expected Ok, got Err: {:?}", err),
        }
    };
}

/// Simple assertion macro for error cases
#[macro_export]
macro_rules! assert_err {
    ($expr:expr) => {
        match $expr {
            Ok(val) => panic!("Expected Err, got Ok: {:?}", val),
            Err(err) => err,
        }
    };
}

/// Mock structures and common test data can be added here
