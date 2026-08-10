//! Formula lexer (#782).
//!
//! Splits a formula body into tokens: numbers, string literals, identifiers,
//! cell and range references (with `$` anchors preserved — #783), operators and
//! punctuation. String literals and sheet-qualified names are handled so that a
//! later Pratt parser can consume the stream positionally.

use std::fmt;

/// A single lexical token with the span it occupied in the source.
#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    /// Character offset where this token starts in the formula body.
    pub start: usize,
    /// Character offset one past the last byte of the token.
    pub end: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    /// A floating-point literal.
    Number(f64),
    /// A double-quoted string literal with quotes removed.
    Str(String),
    /// A boolean literal `TRUE` / `FALSE`.
    Bool(bool),
    /// A cell reference such as `A1`, `$A$1` or a range `A1:B3`.
    Reference(String),
    /// A sheet-qualified name such as `Sheet2!A1` (kept whole).
    SheetRef(String),
    /// A bare identifier: function name, named range or variable.
    Ident(String),
    /// An operator: `+ - * / ^ & = <> < <= > >= %`.
    Op(String),
    /// `(` or `)`.
    LParen,
    RParen,
    /// `,` argument separator.
    Comma,
    /// `:` — used only inside references, but kept for errors.
    Colon,
    /// End of input.
    Eof,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Number(n) => write!(f, "{n}"),
            TokenKind::Str(s) => write!(f, "\"{s}\""),
            TokenKind::Bool(b) => write!(f, "{b}"),
            TokenKind::Reference(r) | TokenKind::SheetRef(r) | TokenKind::Ident(r) => {
                f.write_str(r)
            }
            TokenKind::Op(o) => f.write_str(o),
            TokenKind::LParen => f.write_str("("),
            TokenKind::RParen => f.write_str(")"),
            TokenKind::Comma => f.write_str(","),
            TokenKind::Colon => f.write_str(":"),
            TokenKind::Eof => f.write_str("<eof>"),
        }
    }
}

/// Result of lexing; the error carries the offset where lexing stopped.
pub type LexResult<T> = Result<T, LexError>;

#[derive(Clone, Debug, PartialEq)]
pub struct LexError {
    pub offset: usize,
    pub message: String,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "lex error at {}: {}", self.offset, self.message)
    }
}

/// Tokenizes a formula body (the text after the leading `=`).
pub fn lex(input: &str) -> LexResult<Vec<Token>> {
    let bytes = input.as_bytes();
    let mut tokens = Vec::new();
    let mut i = 0usize;
    let n = bytes.len();

    while i < n {
        let c = bytes[i];
        if c.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        // String literal.
        if c == b'"' {
            let start = i;
            i += 1;
            let mut s = String::new();
            let mut closed = false;
            while i < n {
                if bytes[i] == b'"' {
                    if i + 1 < n && bytes[i + 1] == b'"' {
                        s.push('"');
                        i += 2;
                        continue;
                    }
                    i += 1;
                    closed = true;
                    break;
                }
                let ch = input[i..].chars().next().unwrap_or('\u{fffd}');
                s.push(ch);
                i += ch.len_utf8();
                continue;
            }
            if !closed {
                return Err(LexError {
                    offset: start,
                    message: "unterminated string literal".to_string(),
                });
            }
            tokens.push(Token {
                kind: TokenKind::Str(s),
                start,
                end: i,
            });
            continue;
        }
        // Number.
        if c.is_ascii_digit() || (c == b'.' && i + 1 < n && bytes[i + 1].is_ascii_digit()) {
            let start = i;
            while i < n && (bytes[i].is_ascii_digit() || bytes[i] == b'.') {
                i += 1;
            }
            let text = &input[start..i];
            match text.parse::<f64>() {
                Ok(num) => tokens.push(Token {
                    kind: TokenKind::Number(num),
                    start,
                    end: i,
                }),
                Err(_) => {
                    return Err(LexError {
                        offset: start,
                        message: format!("invalid number \"{text}\""),
                    })
                }
            }
            continue;
        }
        // Multi-char operators first, then single char.
        let op3 = &input[i..];
        let matched: Option<&str> = ["<>", "<=", ">=", "<>"]
            .iter()
            .find(|op| op3.starts_with(**op))
            .copied();
        if let Some(op) = matched {
            let end = i + op.len();
            tokens.push(Token {
                kind: TokenKind::Op(op.to_string()),
                start: i,
                end,
            });
            i = end;
            continue;
        }
        let ch = c as char;
        if "+-*/^&=<>%".contains(ch) {
            tokens.push(Token {
                kind: TokenKind::Op(ch.to_string()),
                start: i,
                end: i + 1,
            });
            i += 1;
            continue;
        }
        match ch {
            '(' => {
                tokens.push(Token {
                    kind: TokenKind::LParen,
                    start: i,
                    end: i + 1,
                });
                i += 1;
                continue;
            }
            ')' => {
                tokens.push(Token {
                    kind: TokenKind::RParen,
                    start: i,
                    end: i + 1,
                });
                i += 1;
                continue;
            }
            ',' => {
                tokens.push(Token {
                    kind: TokenKind::Comma,
                    start: i,
                    end: i + 1,
                });
                i += 1;
                continue;
            }
            ':' => {
                tokens.push(Token {
                    kind: TokenKind::Colon,
                    start: i,
                    end: i + 1,
                });
                i += 1;
                continue;
            }
            _ => {}
        }
        // Identifier or reference: letters, digits, `$`, `_` and `!`.
        if c.is_ascii_alphabetic() || c == b'$' || c == b'_' {
            let start = i;
            while i < n
                && (bytes[i].is_ascii_alphanumeric()
                    || bytes[i] == b'_'
                    || bytes[i] == b'$'
                    || bytes[i] == b'!')
            {
                i += 1;
            }
            let text = &input[start..i];
            let kind = classify_word(text);
            tokens.push(Token {
                kind,
                start,
                end: i,
            });
            continue;
        }
        return Err(LexError {
            offset: i,
            message: format!(
                "unexpected character '{}'",
                input[i..].chars().next().unwrap_or('\u{fffd}')
            ),
        });
    }

    tokens.push(Token {
        kind: TokenKind::Eof,
        start: n,
        end: n,
    });

    Ok(tokens)
}

fn classify_word(word: &str) -> TokenKind {
    if word.contains('!') {
        return TokenKind::SheetRef(word.to_string());
    }
    let upper = word.to_ascii_uppercase();
    if upper == "TRUE" || upper == "FALSE" {
        return TokenKind::Bool(upper == "TRUE");
    }
    if looks_like_reference(word) {
        TokenKind::Reference(word.to_string())
    } else {
        TokenKind::Ident(word.to_string())
    }
}

fn looks_like_reference(word: &str) -> bool {
    // A reference is: optional `$` column anchor, one or more letters,
    // optional `$` row anchor, then one or more digits. Everything else
    // (function names, bare numbers, `Sheet!Ref` handled by caller) is not
    // a reference.
    let bytes = word.as_bytes();
    let mut i = 0usize;
    if i < bytes.len() && bytes[i] == b'$' {
        i += 1;
    }
    let mut seen_letters = false;
    while i < bytes.len() && bytes[i].is_ascii_alphabetic() {
        i += 1;
        seen_letters = true;
    }
    if !seen_letters {
        return false;
    }
    if i < bytes.len() && bytes[i] == b'$' {
        i += 1;
    }
    let mut seen_digits = false;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
        seen_digits = true;
    }
    seen_digits && i == bytes.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexes_mixed_formula() {
        let tokens = lex("A1+B2*3").expect("lex");
        let kinds: Vec<TokenKind> = tokens.into_iter().map(|t| t.kind).collect();
        assert_eq!(kinds.len(), 6);
        assert_eq!(kinds[0], TokenKind::Reference("A1".to_string()));
        assert_eq!(kinds[1], TokenKind::Op("+".to_string()));
        assert_eq!(kinds[2], TokenKind::Reference("B2".to_string()));
        assert_eq!(kinds[3], TokenKind::Op("*".to_string()));
        assert_eq!(kinds[4], TokenKind::Number(3.0));
    }

    #[test]
    fn lexes_absolute_references() {
        let kinds: Vec<TokenKind> =
            lex("$A$1+1").expect("lex").into_iter().map(|t| t.kind).collect();
        assert_eq!(kinds[0], TokenKind::Reference("$A$1".to_string()));
    }

    #[test]
    fn lexes_string_literals() {
        let kinds: Vec<TokenKind> = lex("\"Total: \"&A1")
            .expect("lex")
            .into_iter()
            .map(|t| t.kind)
            .collect();
        assert_eq!(kinds[0], TokenKind::Str("Total: ".to_string()));
        assert_eq!(kinds[1], TokenKind::Op("&".to_string()));
    }

    #[test]
    fn lexes_sheet_reference() {
        let kinds: Vec<TokenKind> = lex("Sheet2!A1")
            .expect("lex")
            .into_iter()
            .map(|t| t.kind)
            .collect();
        assert_eq!(kinds[0], TokenKind::SheetRef("Sheet2!A1".to_string()));
    }

    #[test]
    fn lexes_booleans() {
        let kinds: Vec<TokenKind> = lex("TRUE+FALSE")
            .expect("lex")
            .into_iter()
            .map(|t| t.kind)
            .collect();
        assert_eq!(kinds[0], TokenKind::Bool(true));
        assert_eq!(kinds[2], TokenKind::Bool(false));
    }

    #[test]
    fn reports_unterminated_string() {
        let err = lex("\"oops").expect_err("must error");
        assert!(err.message.contains("unterminated"));
    }

    #[test]
    fn keeps_function_names_as_identifiers() {
        let kinds: Vec<TokenKind> = lex("SUM(A1:A3)")
            .expect("lex")
            .into_iter()
            .map(|t| t.kind)
            .collect();
        assert_eq!(kinds[0], TokenKind::Ident("SUM".to_string()));
    }

    #[test]
    fn decodes_utf8_inside_strings() {
        let tokens = lex("=\"café\"+\"Σ\"&A1").expect("lex");
        let kinds: Vec<&TokenKind> = tokens.iter().map(|t| &t.kind).collect();
        let strs: Vec<&str> = kinds
            .iter()
            .filter_map(|k| match k {
                TokenKind::Str(s) => Some(s.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(strs, vec!["café", "Σ"]);
    }
}