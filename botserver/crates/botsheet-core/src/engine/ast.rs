//! Formula AST and Pratt parser (#782, #783).
//!
//! The parser builds a typed syntax tree from the lexer stream using the Pratt
//! (precedence-climbing) method, which gives correct operator precedence and
//! left associativity: `2^3^2 = 512`, `-2^2 = -4`, `=A1&B1` concatenation, and
//! nested function calls all fall out of the binding powers below. `$` anchors
//! survive into [`Reference`] so fill and paste can translate references
//! correctly.

use std::fmt;

use super::lexer::{Token, TokenKind};
use super::value::CellValue;

/// A parsed formula expression.
#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    /// A literal value.
    Literal(CellValue),
    /// A cell reference (`A1`, `$A$1`, `Sheet2!B3`).
    Reference(Reference),
    /// A range reference (`A1:B3`).
    Range(Reference, Reference),
    /// A named range / identifier that is not a function call.
    Name(String),
    /// A function call with its argument list (argument separator preserved
    /// for legacy functions that interpret the raw argument text).
    Call {
        name: String,
        args: Vec<Expr>,
        /// Raw argument text (everything between the parens), for legacy
        /// functions that parse it themselves.
        raw: String,
    },
    /// A unary operator application.
    Unary {
        op: String,
        expr: Box<Expr>,
    },
    /// A binary operator application.
    Binary {
        op: String,
        left: Box<Expr>,
        right: Box<Expr>,
    },
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Literal(v) => write!(f, "{v}"),
            Expr::Reference(r) => write!(f, "{r}"),
            Expr::Range(a, b) => write!(f, "{a}:{b}"),
            Expr::Name(n) => f.write_str(n),
            Expr::Call { name, raw, .. } => write!(f, "{name}({raw})"),
            Expr::Unary { op, expr } => write!(f, "{op}{expr}"),
            Expr::Binary { op, left, right } => write!(f, "{left} {op} {right}"),
        }
    }
}

/// A cell reference with its absolute anchors preserved (#783).
///
/// The struct lives in [`super::references`]; this re-import keeps the parser
/// and evaluator APIs stable.
pub use super::references::Reference;

/// Binding powers for Pratt parsing (higher binds tighter).
fn infix_power(op: &str) -> Option<(u8, u8)> {
    Some(match op {
        "&" => (3, 4),
        "=" | "<>" | "<" | ">" | "<=" | ">=" => (5, 6),
        "+" | "-" => (7, 8),
        "*" | "/" => (9, 10),
        // (11, 10): the right operand binds one power below the operator, so
        // `2^3^2` parses right-to-left as 2^(3^2) = 512.
        "^" => (11, 10),
        _ => return None,
    })
}

fn prefix_power(op: &str) -> Option<u8> {
    Some(match op {
        // Binds looser than `^` but tighter than `*`: `-2^2 = -(2^2) = -4`.
        "+" | "-" => 10,
        _ => return None,
    })
}

/// A cursor over the token stream.
struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> &Token {
        &self.tokens[self.pos.min(self.tokens.len() - 1)]
    }

    fn next(&mut self) -> &Token {
        let t = &self.tokens[self.pos.min(self.tokens.len() - 1)];
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        t
    }

    fn at_end(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Eof)
    }

    fn parse_expr(&mut self, min_power: u8) -> Result<Expr, String> {
        let mut left = self.parse_prefix()?;
        loop {
            let tok = self.peek().clone();
            if self.at_end() {
                break;
            }
            if let TokenKind::Op(ref op) = tok.kind {
                if op == "%" {
                    // Postfix percent: divides the value so far by 100.
                    let _ = self.next();
                    left = Expr::Unary {
                        op: "%".to_string(),
                        expr: Box::new(left),
                    };
                    continue;
                }
                if let Some((l, r)) = infix_power(op) {
                    if l < min_power {
                        break;
                    }
                    let _ = self.next();
                    if op == "^" {
                        // Right-associative.
                        let right = self.parse_expr(r)?;
                        left = Expr::Binary {
                            op: op.clone(),
                            left: Box::new(left),
                            right: Box::new(right),
                        };
                    } else {
                        let right = self.parse_expr(r)?;
                        left = Expr::Binary {
                            op: op.clone(),
                            left: Box::new(left),
                            right: Box::new(right),
                        };
                    }
                    continue;
                }
            }
            break;
        }
        Ok(left)
    }

    fn parse_prefix(&mut self) -> Result<Expr, String> {
        let tok = self.next().clone();
        match tok.kind {
            TokenKind::Number(n) => Ok(Expr::Literal(CellValue::Number(n))),
            TokenKind::Str(s) => Ok(Expr::Literal(CellValue::Text(s))),
            TokenKind::Bool(b) => Ok(Expr::Literal(CellValue::Bool(b))),
            TokenKind::Reference(r) => {
                let first = Reference::parse(&r).ok_or_else(|| format!("bad reference {r}"))?;
                // Range?
                if matches!(self.peek().kind, TokenKind::Colon) {
                    let _ = self.next();
                    let end_tok = self.next().clone();
                    if let TokenKind::Reference(r2) = end_tok.kind {
                        let second =
                            Reference::parse(&r2).ok_or_else(|| format!("bad reference {r2}"))?;
                        Ok(Expr::Range(first, second))
                    } else {
                        Err(format!("expected reference after ':' got {end_tok:?}"))
                    }
                } else {
                    Ok(Expr::Reference(first))
                }
            }
            TokenKind::SheetRef(s) => {
                let first = Reference::parse(&s)
                    .ok_or_else(|| format!("bad sheet reference {s}"))?;
                if matches!(self.peek().kind, TokenKind::Colon) {
                    let _ = self.next();
                    let end_tok = self.next().clone();
                    let raw = match end_tok.kind {
                        TokenKind::Reference(r2) => r2,
                        TokenKind::SheetRef(s2) => s2,
                        other => return Err(format!("expected reference got {other}")),
                    };
                    let second =
                        Reference::parse(&raw).ok_or_else(|| format!("bad reference {raw}"))?;
                    Ok(Expr::Range(first, second))
                } else {
                    Ok(Expr::Reference(first))
                }
            }
            TokenKind::Ident(name) => {
                let upper = name.to_ascii_uppercase();
                if upper == "TRUE" {
                    return Ok(Expr::Literal(CellValue::Bool(true)));
                }
                if upper == "FALSE" {
                    return Ok(Expr::Literal(CellValue::Bool(false)));
                }
                // Function call?
                if matches!(self.peek().kind, TokenKind::LParen) {
                    let _ = self.next();
                    let arg_start = self.pos;
                    let mut args: Vec<Expr> = Vec::new();
                    let mut depth = 0usize;
                    let arg_end;
                    loop {
                        if self.at_end() {
                            return Err(format!("unterminated call to {name}"));
                        }
                        match self.peek().kind {
                            TokenKind::RParen if depth == 0 => {
                                arg_end = self.pos;
                                let _ = self.next();
                                break;
                            }
                            TokenKind::LParen => {
                                depth += 1;
                                let _ = self.next();
                            }
                            TokenKind::RParen => {
                                depth = depth.saturating_sub(1);
                                let _ = self.next();
                            }
                            TokenKind::Comma if depth == 0 => {
                                let _ = self.next();
                            }
                            _ => {
                                args.push(self.parse_expr(0)?);
                            }
                        }
                    }
                    let raw = raw_args(self.tokens, arg_start, arg_end);
                    return Ok(Expr::Call {
                        name: upper,
                        args,
                        raw,
                    });
                }
                Ok(Expr::Name(name))
            }
            TokenKind::Op(ref op) if prefix_power(op).is_some() => {
                let power = prefix_power(op).unwrap_or(13);
                let inner = self.parse_expr(power)?;
                Ok(Expr::Unary {
                    op: op.clone(),
                    expr: Box::new(inner),
                })
            }
            TokenKind::Op(ref op) => Err(format!("cannot start expression with operator '{op}'")),
            TokenKind::LParen => {
                let inner = self.parse_expr(0)?;
                if !matches!(self.peek().kind, TokenKind::RParen) {
                    return Err("expected ')'".to_string());
                }
                let _ = self.next();
                Ok(inner)
            }
            TokenKind::Comma | TokenKind::Colon => Err(format!("unexpected {tok:?}")),
            TokenKind::RParen => Err("unbalanced ')'".to_string()),
            TokenKind::Eof => Err("unexpected end of formula".to_string()),
        }
    }
}

fn raw_args(tokens: &[Token], start: usize, end: usize) -> String {
    let mut s = String::new();
    for i in start..end.min(tokens.len()) {
        match &tokens[i].kind {
            TokenKind::Eof => break,
            kind => s.push_str(&kind.to_string()),
        }
    }
    s
}

/// Parses a formula body into an expression tree.
pub fn parse(input: &str) -> Result<Expr, String> {
    let tokens = super::lexer::lex(input).map_err(|e| e.to_string())?;
    let mut parser = Parser {
        tokens: &tokens,
        pos: 0,
    };
    let expr = parser.parse_expr(0)?;
    if !parser.at_end() {
        return Err(format!("unexpected token {}", parser.peek().kind));
    }
    Ok(expr)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_ok(input: &str) -> Expr {
        parse(input).expect("parse ok")
    }

    #[test]
    fn precedence_mult_before_add() {
        let e = parse_ok("1+2*3");
        match e {
            Expr::Binary { op, right, .. } => {
                assert_eq!(op, "+");
                assert!(matches!(*right, Expr::Binary { op: ref o, .. } if o == "*"));
            }
            other => panic!("expected binary +, got {other:?}"),
        }
    }

    #[test]
    fn exponent_is_right_associative() {
        // 2^3^2 must parse as 2^(3^2).
        let e = parse_ok("2^3^2");
        match e {
            Expr::Binary { op, right, .. } => {
                assert_eq!(op, "^");
                assert!(matches!(*right, Expr::Binary { op: ref o, .. } if o == "^"));
            }
            other => panic!("expected binary ^, got {other:?}"),
        }
    }

    #[test]
    fn unary_minus_binds_loosely() {
        // -2^2 == -(2^2)
        let e = parse_ok("-2^2");
        assert!(matches!(e, Expr::Unary { .. }));
    }

    #[test]
    fn nested_function_call() {
        let e = parse_ok("INDEX(A1:A3,MATCH(20,A1:A3,0))");
        match e {
            Expr::Call { name, args, .. } => {
                assert_eq!(name, "INDEX");
                assert_eq!(args.len(), 2);
                assert!(matches!(args[1], Expr::Call { ref name, .. } if name == "MATCH"));
            }
            other => panic!("expected call, got {other:?}"),
        }
    }

    #[test]
    fn string_concat() {
        let e = parse_ok("\"Total: \"&A1");
        match e {
            Expr::Binary { op, left, right } => {
                assert_eq!(op, "&");
                assert!(matches!(*left, Expr::Literal(CellValue::Text(_))));
                assert!(matches!(*right, Expr::Reference(_)));
            }
            other => panic!("expected concat, got {other:?}"),
        }
    }

    #[test]
    fn absolute_reference_roundtrip() {
        let r = Reference::parse("$A$1").expect("parse");
        assert!(r.col_absolute && r.row_absolute);
        assert_eq!(r.to_string(), "$A$1");
        assert_eq!(r.translate(2, 3).to_string(), "$A$1");
    }

    #[test]
    fn relative_reference_translates() {
        let r = Reference::parse("A1").expect("parse");
        assert_eq!(r.translate(2, 3).to_string(), "D3");
        let mixed = Reference::parse("$A1").expect("parse");
        assert_eq!(mixed.translate(2, 3).to_string(), "$A3");
    }

    #[test]
    fn sheet_qualified_reference() {
        let e = parse_ok("Sheet2!A1");
        match e {
            Expr::Reference(r) => {
                assert_eq!(r.sheet.as_deref(), Some("Sheet2"));
                assert_eq!(r.to_string(), "Sheet2!A1");
            }
            other => panic!("expected reference, got {other:?}"),
        }
    }

    #[test]
    fn rejects_unbalanced_parens() {
        assert!(parse("(1+2").is_err());
        assert!(parse("1+2)").is_err());
    }

    #[test]
    fn postfix_percent() {
        match parse_ok("50%") {
            Expr::Unary { op, .. } => assert_eq!(op, "%"),
            other => panic!("expected unary %, got {other:?}"),
        }
        // Percent binds tighter than multiplication: 50% * 2.
        match parse_ok("50%*2") {
            Expr::Binary { op, left, .. } => {
                assert_eq!(op, "*");
                assert!(matches!(*left, Expr::Unary { .. }));
            }
            other => panic!("expected binary *, got {other:?}"),
        }
    }
}