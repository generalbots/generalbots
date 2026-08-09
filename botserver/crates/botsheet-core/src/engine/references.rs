//! Cell reference model (#783).
//!
//! A [`Reference`] carries its `$` anchors so fill, copy and paste can
//! translate only the relative parts. Sheet qualifiers parse and render
//! (`Sheet2!A1`); the single-worksheet evaluator reports a typed `#REF!` when
//! it cannot satisfy a cross-sheet reference.

use std::fmt;

/// A cell reference with its absolute anchors preserved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reference {
    /// Optional sheet qualifier such as `Sheet2`.
    pub sheet: Option<String>,
    pub col: u32,
    pub row: u32,
    pub col_absolute: bool,
    pub row_absolute: bool,
}

impl Reference {
    /// Builds a reference from a raw token such as `A1`, `$A$1` or `Sheet2!B3`.
    pub fn parse(raw: &str) -> Option<Reference> {
        let (sheet, cell_part) = match raw.split_once('!') {
            Some((s, c)) => (Some(s.to_string()), c),
            None => (None, raw),
        };
        let bytes = cell_part.as_bytes();
        let mut i = 0usize;
        let n = bytes.len();
        let mut col_absolute = false;
        if i < n && bytes[i] == b'$' {
            col_absolute = true;
            i += 1;
        }
        let col_start = i;
        while i < n && bytes[i].is_ascii_alphabetic() {
            i += 1;
        }
        let col_name = &cell_part[col_start..i];
        if col_name.is_empty() {
            return None;
        }
        let mut row_absolute = false;
        if i < n && bytes[i] == b'$' {
            row_absolute = true;
            i += 1;
        }
        let row_start = i;
        while i < n && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i != n || row_start == i {
            return None;
        }
        let col = col_name_to_index(col_name)?;
        let row = cell_part[row_start..i].parse::<u32>().ok()?.checked_sub(1)?;
        Some(Reference {
            sheet,
            col,
            row,
            col_absolute,
            row_absolute,
        })
    }

    /// Translates a relative reference by `(dr, dc)`; absolute anchors are kept.
    pub fn translate(&self, dr: i64, dc: i64) -> Reference {
        Reference {
            sheet: self.sheet.clone(),
            col: if self.col_absolute {
                self.col
            } else {
                (i64::from(self.col) + dc).max(0) as u32
            },
            row: if self.row_absolute {
                self.row
            } else {
                (i64::from(self.row) + dr).max(0) as u32
            },
            col_absolute: self.col_absolute,
            row_absolute: self.row_absolute,
        }
    }
}

fn col_name_to_index(name: &str) -> Option<u32> {
    if name.is_empty() || name.len() > 3 {
        return None;
    }
    let mut col: u32 = 0;
    for ch in name.chars() {
        let upper = ch.to_ascii_uppercase();
        if !upper.is_ascii_uppercase() {
            return None;
        }
        col = col.checked_mul(26)?.checked_add(upper as u32 - 'A' as u32 + 1)?;
    }
    col.checked_sub(1)
}

impl fmt::Display for Reference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref s) = self.sheet {
            write!(f, "{s}!")?;
        }
        if self.col_absolute {
            f.write_str("$")?;
        }
        f.write_str(&col_name(self.col))?;
        if self.row_absolute {
            f.write_str("$")?;
        }
        write!(f, "{}", self.row + 1)
    }
}

fn col_name(idx: u32) -> String {
    let mut s = String::new();
    let mut n = idx + 1;
    while n > 0 {
        let r = (n - 1) % 26;
        s.insert(0, (b'A' + r as u8) as char);
        n = (n - 1) / 26;
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let r = Reference::parse("Sheet2!A1").expect("parse");
        assert_eq!(r.sheet.as_deref(), Some("Sheet2"));
        assert_eq!(r.to_string(), "Sheet2!A1");
    }

    #[test]
    fn rejects_invalid() {
        assert!(Reference::parse("").is_none());
        assert!(Reference::parse("A").is_none());
        assert!(Reference::parse("1").is_none());
    }
}