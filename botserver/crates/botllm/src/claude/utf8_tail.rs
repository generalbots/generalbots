//! Decodes a byte stream whose reads can split a UTF-8 character.
//!
//! A network read may end in the middle of a multi-byte character (any accented
//! letter). Decoding each read on its own turned both halves into U+FFFD, so
//! long answers and generated programs lost letters at random places.

/// Takes the decodable text from `pending`, keeping an incomplete trailing
/// character for the next read. Invalid bytes become U+FFFD.
pub(super) fn take_text(pending: &mut Vec<u8>) -> String {
    let keep_from = match std::str::from_utf8(pending) {
        Ok(_) => pending.len(),
        Err(error) if error.error_len().is_none() => error.valid_up_to(),
        Err(_) => pending.len(),
    };
    let text = String::from_utf8_lossy(&pending[..keep_from]).into_owned();
    pending.drain(..keep_from);
    text
}

#[cfg(test)]
mod tests {
    use super::take_text;

    #[test]
    fn a_character_split_across_reads_is_decoded_once() {
        let bytes = "ação".as_bytes();
        let mut pending = bytes[..2].to_vec();
        let first = take_text(&mut pending);
        assert_eq!(first, "a");
        assert_eq!(pending.len(), 1);
        pending.extend_from_slice(&bytes[2..]);
        assert_eq!(first + take_text(&mut pending).as_str(), "ação");
        assert!(pending.is_empty());
    }

    #[test]
    fn invalid_bytes_do_not_stall_the_stream() {
        let mut pending = vec![b'a', 0xff, b'b'];
        assert_eq!(take_text(&mut pending), "a\u{fffd}b");
        assert!(pending.is_empty());
    }
}
