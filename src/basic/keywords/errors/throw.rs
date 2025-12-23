//! THROW - Error throwing functionality
//!
//! This module provides the THROW/RAISE keywords for error handling in BASIC scripts.
//! The actual implementation is in the parent mod.rs file.
//!
//! BASIC Syntax:
//!   THROW "Error message"
//!   RAISE "Error message"
//!
//! Examples:
//!   IF balance < 0 THEN
//!     THROW "Insufficient funds"
//!   END IF
//!
//!   ON ERROR GOTO error_handler
//!   THROW "Something went wrong"
//!   EXIT SUB
//!   error_handler:
//!     TALK "Error: " + GET_ERROR_MESSAGE()

// This module serves as a placeholder for future expansion.
// The THROW, RAISE, ERROR, IS_ERROR, ASSERT, and logging functions
// are currently implemented directly in the parent mod.rs file.
//
// Future enhancements could include:
// - Custom error types
// - Error codes
// - Stack trace capture
// - Error context/metadata
// - Retry mechanisms
