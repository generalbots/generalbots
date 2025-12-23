















use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::{debug, trace};
use rhai::{Dynamic, Engine, EvalAltResult, Position};
use std::cell::RefCell;
use std::sync::Arc;

thread_local! {

    static ERROR_RESUME_NEXT: RefCell<bool> = RefCell::new(false);


    static LAST_ERROR: RefCell<Option<String>> = RefCell::new(None);


    static ERROR_NUMBER: RefCell<i64> = RefCell::new(0);
}


pub fn is_error_resume_next_active() -> bool {
    ERROR_RESUME_NEXT.with(|flag| *flag.borrow())
}


pub fn set_error_resume_next(active: bool) {
    ERROR_RESUME_NEXT.with(|flag| {
        *flag.borrow_mut() = active;
    });
    if !active {

        clear_last_error();
    }
}


pub fn set_last_error(message: &str, error_num: i64) {
    LAST_ERROR.with(|err| {
        *err.borrow_mut() = Some(message.to_string());
    });
    ERROR_NUMBER.with(|num| {
        *num.borrow_mut() = error_num;
    });
}


pub fn clear_last_error() {
    LAST_ERROR.with(|err| {
        *err.borrow_mut() = None;
    });
    ERROR_NUMBER.with(|num| {
        *num.borrow_mut() = 0;
    });
}


pub fn get_last_error() -> Option<String> {
    LAST_ERROR.with(|err| err.borrow().clone())
}


pub fn get_error_number() -> i64 {
    ERROR_NUMBER.with(|num| *num.borrow())
}


pub fn register_on_error_keywords(_state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {

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


    engine
        .register_custom_syntax(&["CLEAR", "ERROR"], false, move |_context, _inputs| {
            trace!("CLEAR ERROR executed");
            clear_last_error();
            Ok(Dynamic::UNIT)
        })
        .expect("Failed to register CLEAR ERROR");



    engine.register_fn("ERROR", || -> bool { get_last_error().is_some() });



    engine
        .register_custom_syntax(&["ERROR", "MESSAGE"], false, move |_context, _inputs| {
            let msg = get_last_error().unwrap_or_default();
            Ok(Dynamic::from(msg))
        })
        .expect("Failed to register ERROR MESSAGE");


    engine.register_fn("ERR", || -> i64 { get_error_number() });


    engine.register_fn("ERR_NUMBER", || -> i64 { get_error_number() });


    engine.register_fn("ERR_DESCRIPTION", || -> String {
        get_last_error().unwrap_or_default()
    });


    engine.register_fn("ERR_CLEAR", || {
        clear_last_error();
    });

    debug!("Registered ON ERROR keywords");
}



pub fn try_execute<F, T>(operation: F) -> Result<T, String>
where
    F: FnOnce() -> Result<T, Box<dyn std::error::Error + Send + Sync>>,
{
    match operation() {
        Ok(result) => {

            if is_error_resume_next_active() {
                clear_last_error();
            }
            Ok(result)
        }
        Err(e) => {
            let error_msg = e.to_string();
            if is_error_resume_next_active() {

                set_last_error(&error_msg, 1);
                trace!("Error caught by ON ERROR RESUME NEXT: {}", error_msg);
                Err(error_msg)
            } else {

                Err(error_msg)
            }
        }
    }
}



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
