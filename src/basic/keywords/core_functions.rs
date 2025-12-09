//! Core BASIC Functions - Wrapper module
//!
//! This module serves as a central registration point for all core BASIC functions:
//! - Math functions (ABS, ROUND, INT, MAX, MIN, MOD, RANDOM, etc.)
//! - Date/Time functions (NOW, TODAY, YEAR, MONTH, DAY, etc.)
//! - Validation functions (VAL, STR, ISNULL, ISEMPTY, TYPEOF, etc.)
//! - Array functions (SORT, UNIQUE, CONTAINS, PUSH, POP, etc.)
//! - Error handling functions (THROW, ERROR, IS_ERROR, ASSERT, etc.)

use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::debug;
use rhai::Engine;
use std::sync::Arc;

use super::arrays::register_array_functions;
use super::datetime::register_datetime_functions;
use super::errors::register_error_functions;
use super::math::register_math_functions;
use super::validation::register_validation_functions;

/// Register all core BASIC functions
///
/// This function registers all the standard BASIC functions that are commonly
/// expected in any BASIC implementation:
///
/// ## Math Functions
/// - `ABS(n)` - Absolute value
/// - `ROUND(n)`, `ROUND(n, decimals)` - Round to nearest integer or decimal places
/// - `INT(n)`, `FIX(n)` - Truncate to integer
/// - `FLOOR(n)`, `CEIL(n)` - Floor and ceiling
/// - `MAX(a, b)`, `MIN(a, b)` - Maximum and minimum
/// - `MOD(a, b)` - Modulo operation
/// - `RANDOM()`, `RND()` - Random number generation
/// - `SGN(n)` - Sign of number (-1, 0, 1)
/// - `SQRT(n)`, `SQR(n)` - Square root
/// - `POW(base, exp)` - Power/exponentiation
/// - `LOG(n)`, `LOG10(n)` - Natural and base-10 logarithm
/// - `EXP(n)` - e raised to power n
/// - `SIN(n)`, `COS(n)`, `TAN(n)` - Trigonometric functions
/// - `ASIN(n)`, `ACOS(n)`, `ATAN(n)` - Inverse trigonometric
/// - `PI()` - Pi constant
/// - `SUM(array)`, `AVG(array)` - Aggregation functions
///
/// ## Date/Time Functions
/// - `NOW()` - Current date and time
/// - `TODAY()` - Current date only
/// - `TIME()` - Current time only
/// - `TIMESTAMP()` - Unix timestamp
/// - `YEAR(date)`, `MONTH(date)`, `DAY(date)` - Extract date parts
/// - `HOUR(time)`, `MINUTE(time)`, `SECOND(time)` - Extract time parts
/// - `WEEKDAY(date)` - Day of week (1-7)
/// - `DATEADD(date, amount, unit)` - Add to date
/// - `DATEDIFF(date1, date2, unit)` - Difference between dates
/// - `FORMAT_DATE(date, format)` - Format date as string
/// - `ISDATE(value)` - Check if value is a valid date
///
/// ## Validation Functions
/// - `VAL(string)` - Convert string to number
/// - `STR(number)` - Convert number to string
/// - `CINT(value)` - Convert to integer
/// - `CDBL(value)` - Convert to double
/// - `ISNULL(value)` - Check if null/unit
/// - `ISEMPTY(value)` - Check if empty (string, array, map)
/// - `TYPEOF(value)` - Get type name as string
/// - `ISARRAY(value)` - Check if array
/// - `ISNUMBER(value)` - Check if number
/// - `ISSTRING(value)` - Check if string
/// - `ISBOOL(value)` - Check if boolean
/// - `NVL(value, default)` - Null coalesce
/// - `IIF(condition, true_val, false_val)` - Immediate if
///
/// ## Array Functions
/// - `UBOUND(array)` - Upper bound (length - 1)
/// - `LBOUND(array)` - Lower bound (always 0)
/// - `COUNT(array)` - Array length
/// - `SORT(array)`, `SORT(array, "DESC")` - Sort array
/// - `UNIQUE(array)` - Remove duplicates
/// - `CONTAINS(array, value)` - Check membership
/// - `INDEX_OF(array, value)` - Find index of value
/// - `PUSH(array, value)` - Add to end
/// - `POP(array)` - Remove from end
/// - `SHIFT(array)` - Remove from beginning
/// - `UNSHIFT(array, value)` - Add to beginning
/// - `REVERSE(array)` - Reverse array
/// - `SLICE(array, start, end)` - Extract portion
/// - `JOIN(array, separator)` - Join to string
/// - `SPLIT(string, delimiter)` - Split to array
/// - `CONCAT(array1, array2)` - Combine arrays
/// - `RANGE(start, end)` - Create number range
///
/// ## Error Handling Functions
/// - `THROW(message)`, `RAISE(message)` - Throw error
/// - `ERROR(message)` - Create error object
/// - `IS_ERROR(value)` - Check if error object
/// - `GET_ERROR_MESSAGE(error)` - Get error message
/// - `ASSERT(condition, message)` - Assert condition
/// - `LOG_ERROR(message)` - Log error
/// - `LOG_WARN(message)` - Log warning
/// - `LOG_INFO(message)` - Log info
/// - `LOG_DEBUG(message)` - Log debug
pub fn register_core_functions(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    debug!("Registering core BASIC functions...");

    // Register math functions (ABS, ROUND, INT, MAX, MIN, MOD, RANDOM, etc.)
    register_math_functions(state.clone(), user.clone(), engine);
    debug!("  * Math functions registered");

    // Register date/time functions (NOW, TODAY, YEAR, MONTH, DAY, etc.)
    register_datetime_functions(state.clone(), user.clone(), engine);
    debug!("  * Date/Time functions registered");

    // Register validation functions (VAL, STR, ISNULL, ISEMPTY, TYPEOF, etc.)
    register_validation_functions(state.clone(), user.clone(), engine);
    debug!("  * Validation functions registered");

    // Register array functions (SORT, UNIQUE, CONTAINS, PUSH, POP, etc.)
    register_array_functions(state.clone(), user.clone(), engine);
    debug!("  * Array functions registered");

    // Register error handling functions (THROW, ERROR, IS_ERROR, ASSERT, etc.)
    register_error_functions(state, user, engine);
    debug!("  * Error handling functions registered");

    debug!("All core BASIC functions registered successfully");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_structure() {
        // This test verifies the module compiles correctly
        // Actual function tests are in their respective submodules
        assert!(true);
    }
}
