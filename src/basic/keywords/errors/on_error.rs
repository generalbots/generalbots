//! ON ERROR RESUME NEXT Implementation
//!
//! Provides VB-style error handling for BASIC scripts.
//! When ON ERROR RESUME NEXT is active, errors are caught and stored
//! rather than halting execution.
//!
//! # Usage
//! ```basic
//! ON ERROR RESUME NEXT
//! result = SOME_RISKY_OPERATION()
//! IF ERROR THEN
//!     TALK "An error occurred: " + ERROR MESSAGE
//! END IF
//! ON ERROR GOTO 0  ' Disable error handling
//! ```

use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::{debug, trace};
use rhai::{Dynamic, Engine, EvalAltResult, Position};
use std::cell::RefCell;
use std::sync::Arc;

thread_local! {
    /// Thread-local flag indicating if ON ERROR RESUME NEXT is active
    static ERROR_RESUME_NEXT: RefCell<bool> = RefCell::new(false);

    /// Thread-local storage for the last error that occurred
    static LAST_ERROR: RefCell<Option<String>> = RefCell::new(None);

    /// Thread-local error number (for compatibility)
    static ERROR_NUMBER: RefCell<i64> = RefCell::new(0);
}

/// Check if ON ERROR RESUME NEXT is currently active
pub fn is_error_resume_next_active() -> bool {
    ERROR_RESUME_NEXT.with(|flag| *flag.borrow())
}

/// Set the ON ERROR RESUME NEXT state
pub fn set_error_resume_next(active: bool) {
    ERROR_RESUME_NEXT.with(|flag| {
        *flag.borrow_mut() = active;
    });
    if !active {
        // Clear error state when disabling
        clear_last_error();
    }
}

/// Store an error message
pub fn set_last_error(message: &str, error_num: i64) {
    LAST_ERROR.with(|err| {
        *err.borrow_mut() = Some(message.to_string());
    });
    ERROR_NUMBER.with(|num| {
        *num.borrow_mut() = error_num;
    });
}

/// Clear the last error
pub fn clear_last_error() {
    LAST_ERROR.with(|err| {
        *err.borrow_mut() = None;
    });
    ERROR_NUMBER.with(|num| {
        *num.borrow_mut() = 0;
    });
}

/// Get the last error message
pub fn get_last_error() -> Option<String> {
    LAST_ERROR.with(|err| err.borrow().clone())
}

/// Get the last error number
pub fn get_error_number() -> i64 {
    ERROR_NUMBER.with(|num| *num.borrow())
}

/// Register ON ERROR keywords with the Rhai engine
pub fn register_on_error_keywords(_state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    // ON ERROR RESUME NEXT - Enable error trapping
    engine
        .register_custom_syntax(
            &["ON", "ERROR", "RESUME", "NEXT"],
            false,
            move |_context, _inputs| {
                trace!("ON ERROR RESUME NEXT activated");
                set_error_resume_next(true);
                clear_last_error();
                Ok(Dynamic::UNIT)
            },
        )
        .expect("Failed to register ON ERROR RESUME NEXT");

    // ON ERROR GOTO 0 - Disable error trapping (standard VB syntax)
    engine
        .register_custom_syntax(
            &["ON", "ERROR", "GOTO", "0"],
            false,
            move |_context, _inputs| {
                trace!("ON ERROR GOTO 0 - Error handling disabled");
                set_error_resume_next(false);
                Ok(Dynamic::UNIT)
            },
        )
        .expect("Failed to register ON ERROR GOTO 0");

    // CLEAR ERROR - Clear the current error state
    engine
        .register_custom_syntax(&["CLEAR", "ERROR"], false, move |_context, _inputs| {
            trace!("CLEAR ERROR executed");
            clear_last_error();
            Ok(Dynamic::UNIT)
        })
        .expect("Failed to register CLEAR ERROR");

    // ERROR - Check if an error occurred (returns true/false)
    // Used as: IF ERROR THEN ...
    engine.register_fn("ERROR", || -> bool { get_last_error().is_some() });

    // ERROR MESSAGE - Get the last error message
    // Used as: msg = ERROR MESSAGE
    engine
        .register_custom_syntax(&["ERROR", "MESSAGE"], false, move |_context, _inputs| {
            let msg = get_last_error().unwrap_or_default();
            Ok(Dynamic::from(msg))
        })
        .expect("Failed to register ERROR MESSAGE");

    // ERR - Get error number (VB compatibility)
    engine.register_fn("ERR", || -> i64 { get_error_number() });

    // ERR.NUMBER - Alias for ERR
    engine.register_fn("ERR_NUMBER", || -> i64 { get_error_number() });

    // ERR.DESCRIPTION - Get error description
    engine.register_fn("ERR_DESCRIPTION", || -> String {
        get_last_error().unwrap_or_default()
    });

    // ERR.CLEAR - Clear the error
    engine.register_fn("ERR_CLEAR", || {
        clear_last_error();
    });

    debug!("Registered ON ERROR keywords");
}

/// Wrapper function to execute code with ON ERROR RESUME NEXT support
/// This should be called around risky operations
pub fn try_execute<F, T>(operation: F) -> Result<T, String>
where
    F: FnOnce() -> Result<T, Box<dyn std::error::Error + Send + Sync>>,
{
    match operation() {
        Ok(result) => {
            // Clear any previous error on success
            if is_error_resume_next_active() {
                clear_last_error();
            }
            Ok(result)
        }
        Err(e) => {
            let error_msg = e.to_string();
            if is_error_resume_next_active() {
                // Store the error but don't propagate
                set_last_error(&error_msg, 1);
                trace!("Error caught by ON ERROR RESUME NEXT: {}", error_msg);
                Err(error_msg)
            } else {
                // No error handling, propagate the error
                Err(error_msg)
            }
        }
    }
}

/// Helper macro to wrap operations with ON ERROR RESUME NEXT support
/// Returns Dynamic::UNIT on error if ON ERROR RESUME NEXT is active
#[macro_export]
macro_rules! with_error_handling {
    ($result:expr) => {
        match $result {
            Ok(val) => {
                $crate::basic::keywords::errors::on_error::clear_last_error();
                Ok(val)
            }
            Err(e) => {
                let error_msg = format!("{}", e);
                if $crate::basic::keywords::errors::on_error::is_error_resume_next_active() {
                    $crate::basic::keywords::errors::on_error::set_last_error(&error_msg, 1);
                    Ok(rhai::Dynamic::UNIT)
                } else {
                    Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        error_msg.into(),
                        rhai::Position::NONE,
                    )))
                }
            }
        }
    };
}

/// Create a result that respects ON ERROR RESUME NEXT
pub fn handle_error<T: Into<Dynamic>>(
    result: Result<T, Box<dyn std::error::Error + Send + Sync>>,
) -> Result<Dynamic, Box<EvalAltResult>> {
    match result {
        Ok(val) => {
            clear_last_error();
            Ok(val.into())
        }
        Err(e) => {
            let error_msg = e.to_string();
            if is_error_resume_next_active() {
                set_last_error(&error_msg, 1);
                trace!("Error suppressed by ON ERROR RESUME NEXT: {}", error_msg);
                Ok(Dynamic::UNIT)
            } else {
                Err(Box::new(EvalAltResult::ErrorRuntime(
                    error_msg.into(),
                    Position::NONE,
                )))
            }
        }
    }
}

/// Handle a string error with ON ERROR RESUME NEXT support
pub fn handle_string_error(error_msg: &str) -> Result<Dynamic, Box<EvalAltResult>> {
    if is_error_resume_next_active() {
        set_last_error(error_msg, 1);
        trace!("Error suppressed by ON ERROR RESUME NEXT: {}", error_msg);
        Ok(Dynamic::UNIT)
    } else {
        Err(Box::new(EvalAltResult::ErrorRuntime(
            error_msg.to_string().into(),
            Position::NONE,
        )))
    }
}
