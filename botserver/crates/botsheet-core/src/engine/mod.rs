//! New typed formula engine (#781, #782, #783, #784, #785).
//!
//! This is the replacement foundation the parity plan describes: typed values,
//! a real lexer + Pratt parser producing an AST with `$`-anchored references,
//! and an evaluator that keeps the legacy 170-function library reachable while
//! giving nested calls, precedence, `&` and `^` real behaviour.

pub mod ast;
pub mod cross_sheet;
pub mod date_serial;
pub mod eval;
pub mod formula_compat;
pub mod format_render;
pub mod formats;
pub mod lexer;
pub mod multi_criteria;
pub mod references;
pub mod value;
pub mod xlsx_codec;

pub use ast::{parse, Expr};
pub use date_serial::{date_to_serial, serial_to_date_parts, EPOCH_DIFF_DAYS};
pub use eval::{eval_expr, eval_expr_in, evaluate_typed, evaluate_typed_in};
pub use formula_compat::{
    canonical_function_name, canonicalize_dispatch_formula, normalize_formula,
};
pub use references::Reference;
pub use value::{canonical_error_literal, format_number, CANONICAL_ERROR_CODES, CellValue};
pub use xlsx_codec::{
    date_to_workbook_serial, from_xlsx_formula, is_canonical_xlsx_formula, to_xlsx_formula,
    workbook_serial_to_date,
};
