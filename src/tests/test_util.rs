use std::sync::Once;
static INIT: Once = Once::new();
pub fn setup() {
    INIT.call_once(|| {
    });
}
#[macro_export]
macro_rules! assert_ok {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(err) => panic!("Expected Ok, got Err: {:?}", err),
        }
    };
}
#[macro_export]
macro_rules! assert_err {
    ($expr:expr) => {
        match $expr {
            Ok(val) => panic!("Expected Err, got Ok: {:?}", val),
            Err(err) => err,
        }
    };
}
