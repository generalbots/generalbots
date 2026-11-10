//! Constant-time equality for secret comparisons (issue #2125).
//!
//! Comparing an expected signature with a supplied one using `==` short
//! circuits on the first differing byte, which leaks the matching prefix
//! through timing. Every HMAC/signature equality check in the workspace is
//! expected to go through this module so the shape stays uniform.

/// Byte-wise equality that does not short-circuit on the first difference.
///
/// Lengths are compared first: a length mismatch is not secret (the expected
/// signature length is public) and returning early there keeps the loop
/// bounded to the shorter input.
pub fn eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (left, right) in a.iter().zip(b.iter()) {
        diff |= left ^ right;
    }
    // `black_box` keeps the optimizer from turning the accumulated OR back
    // into an early-exit comparison.
    std::hint::black_box(diff) == 0
}

/// [`eq`] for `str` inputs, compared as raw UTF-8 bytes.
pub fn eq_str(a: &str, b: &str) -> bool {
    eq(a.as_bytes(), b.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::{eq, eq_str};

    #[test]
    fn equal_inputs_match() {
        assert!(eq(b"", b""));
        assert!(eq(b"abc", b"abc"));
        assert!(eq_str("abc123", "abc123"));
    }

    #[test]
    fn one_byte_difference_fails() {
        assert!(!eq(b"abc", b"abd"));
        assert!(!eq(b"abc", b"xbc"));
        assert!(!eq_str("abc", "abC"));
    }

    #[test]
    fn length_difference_fails() {
        assert!(!eq(b"abc", b"abcd"));
        assert!(!eq(b"abcd", b"abc"));
        assert!(!eq_str("abc", ""));
        assert!(!eq_str("", "abc"));
    }
}
