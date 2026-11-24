//! Credential redaction for strings that may reach logs or error messages.
//!
//! ALM/Forgejo personal access tokens are embedded in clone URLs as
//! `scheme://token@host/path` and sent in `Authorization: token ...` headers.
//! These helpers strip the secret before such text is logged or returned in an
//! error, so neither the raw token nor any prefix of it can reach application
//! logs (#2860).

/// Redact the userinfo of every `scheme://userinfo@host/path` occurrence in
/// `text`, producing `scheme://***@host/path`.
///
/// This works on arbitrary text (git stderr, error strings) without knowing
/// the token: after each `://` it masks everything up to the first `@` that
/// precedes a path/query/fragment delimiter or whitespace. URLs without
/// userinfo and SSH-style remotes (`git@host:path`) pass through unchanged.
pub fn redact_url_userinfo(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut idx = 0usize;

    while idx < text.len() {
        let bytes = text.as_bytes();
        if bytes[idx] == b':'
            && bytes.get(idx + 1) == Some(&b'/')
            && bytes.get(idx + 2) == Some(&b'/')
        {
            out.push_str("://");
            idx += 3;
            let authority_start = idx;
            while idx < text.len()
                && !matches!(
                    bytes[idx],
                    b'/' | b'?' | b'#' | b'@' | b' ' | b'\n' | b'\r' | b'\t' | b'"' | b'\''
                )
            {
                idx += 1;
            }
            if idx < text.len() && bytes[idx] == b'@' {
                out.push_str("***@");
                idx += 1;
            } else {
                out.push_str(&text[authority_start..idx]);
            }
            continue;
        }

        match text[idx..].chars().next() {
            Some(ch) => {
                out.push(ch);
                idx += ch.len_utf8();
            }
            None => break,
        }
    }

    out
}

/// Remove an exact secret from arbitrary text, replacing it with `<redacted>`.
///
/// Covers values that are not formatted as URL userinfo, such as an
/// `Authorization: token ...` header echoed by an HTTP client on error.
pub fn redact_literal(text: &str, secret: &str) -> String {
    if secret.is_empty() {
        return text.to_string();
    }
    text.replace(secret, "<redacted>")
}

/// Redact an ALM/Forgejo token from any text: first the exact value (header
/// echoes), then any `scheme://token@host/...` clone URL it was embedded in.
pub fn redact_alm_secret(text: &str, token: &str) -> String {
    redact_url_userinfo(&redact_literal(text, token))
}

#[cfg(test)]
mod tests {
    use super::{redact_alm_secret, redact_literal, redact_url_userinfo};

    const SENTINEL: &str = "gb2860SENTINELtoken9f3a7b1c";

    #[test]
    fn url_userinfo_masks_token_and_every_prefix() {
        let url = format!("https://{SENTINEL}@alm.internal/org/repo.git");
        let redacted = redact_url_userinfo(&url);
        assert_eq!(redacted, "https://***@alm.internal/org/repo.git");
        assert!(!redacted.contains(SENTINEL));
        // Prefixes shorter than ~8 chars are common substrings of any URL
        // (`g` appears in `git`/`org`), so assert on identifying prefixes.
        for n in 8..=SENTINEL.len() {
            assert!(
                !redacted.contains(&SENTINEL[..n]),
                "prefix of len {n} leaked"
            );
        }
    }

    #[test]
    fn url_userinfo_masks_token_inside_git_stderr() {
        let stderr = format!(
            "fatal: unable to access 'http://{SENTINEL}@alm.internal/org/repo.git/': refused"
        );
        let redacted = redact_url_userinfo(&stderr);
        assert!(!redacted.contains(SENTINEL));
        assert!(redacted.contains("http://***@alm.internal/org/repo.git/"));
    }

    #[test]
    fn url_userinfo_preserves_plain_urls_and_ssh_remotes() {
        let plain = "https://alm.internal/org/repo.git";
        assert_eq!(redact_url_userinfo(plain), plain);
        let ssh = "git@alm.internal:org/repo.git";
        assert_eq!(redact_url_userinfo(ssh), ssh);
    }

    #[test]
    fn url_userinfo_handles_multibyte_without_panicking() {
        let mixed = format!("http://{SENTINEL}@alm.internal/café/repo.git");
        let redacted = redact_url_userinfo(&mixed);
        assert!(!redacted.contains(SENTINEL));
        assert!(redacted.contains("/café/"));
    }

    #[test]
    fn literal_removes_authorization_header_value() {
        let header = format!("Authorization: token {SENTINEL}");
        let redacted = redact_literal(&header, SENTINEL);
        assert_eq!(redacted, "Authorization: token <redacted>");
        assert!(!redacted.contains(SENTINEL));
        assert!(!redacted.contains(&SENTINEL[..8]));
    }

    #[test]
    fn alm_secret_removes_literal_and_url_forms() {
        let header = format!("Authorization: token {SENTINEL}");
        let header_redacted = redact_alm_secret(&header, SENTINEL);
        assert!(!header_redacted.contains(SENTINEL));
        assert!(header_redacted.contains("<redacted>"));

        let url = format!("http://{SENTINEL}@alm.internal/org/repo.git");
        let url_redacted = redact_alm_secret(&url, SENTINEL);
        assert_eq!(url_redacted, "http://***@alm.internal/org/repo.git");
        assert!(!url_redacted.contains(SENTINEL));
        assert!(!url_redacted.contains(&SENTINEL[..4]));
    }

    #[test]
    fn empty_secret_is_a_no_op() {
        assert_eq!(redact_literal("plain text", ""), "plain text");
        assert_eq!(redact_alm_secret("plain text", ""), "plain text");
    }
}
