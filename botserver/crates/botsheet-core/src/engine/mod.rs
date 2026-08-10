//! New typed formula engine (#781, #782, #783, #784, #785).
//!
//! This is the replacement foundation the parity plan describes: typed values,
//! a real lexer + Pratt parser producing an AST with `$`-anchored references,
//! and an evaluator that keeps the legacy 170-function library reachable while
//! giving nested calls, precedence, `&` and `^` real behaviour.

pub mod ast;
pub mod cross_sheet;
pub mod eval;
pub mod formats;
pub mod lexer;
pub mod references;
pub mod value;

pub use ast::{parse, Expr};
pub use eval::{eval_expr, eval_expr_in, evaluate_typed, evaluate_typed_in};
pub use references::Reference;
pub use value::{format_number, CellValue};
