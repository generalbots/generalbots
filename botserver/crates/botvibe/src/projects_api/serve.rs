//! `projects_api::serve` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

/// Resolve the default gateway IPv4 address from `/proc/net/route`.
/// The vibe-http proxy device binds on the host; a botserver running inside
/// an Incus container must reach it via the gateway rather than 127.0.0.1.
/// Returns `None` when no default route exists (e.g. running directly on the
/// host). Parsing the proc file avoids spawning any external command.
pub(crate) fn default_gateway_ip() -> Option<String> {
    let raw = std::fs::read_to_string("/proc/net/route").ok()?;
    for line in raw.lines().skip(1) {
        let mut fields = line.split_whitespace();
        let _iface = fields.next()?;
        let dest = fields.next()?;
        let gw = fields.next()?;
        if dest == "00000000" && gw.len() == 8 {
            // /proc/net/route stores the gateway little-endian: the first two
            // hex digits are the least-significant byte of the address.
            let bytes = gw.as_bytes();
            let mut octets = [0u8; 4];
            for (i, chunk) in bytes.chunks(2).enumerate() {
                octets[i] = u8::from_str_radix(
                    &String::from_utf8_lossy(chunk),
                    16,
                )
                .ok()?;
            }
            return Some(format!(
                "{}.{}.{}.{}",
                octets[3], octets[2], octets[1], octets[0]
            ));
        }
    }
    None
}

#[derive(Debug, Deserialize)]
pub(crate) struct ServeQuery {
    pub(crate) token: Option<String>,
}

pub(crate) fn serve_mime_for(path: &str) -> &'static str {
    let ext = path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "json" | "map" => "application/json; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "md" => "text/markdown; charset=utf-8",
        "txt" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

/// Percent-encode a path for use as a query-parameter value. Keeps only `/` and
/// unreserved characters readable; encodes `?`, `&`, `=`, `+`, `%` etc. so the
/// whole original URL (path + query) round-trips through `Query<VmPreviewQuery>`
/// without being split at the first `&`.
pub(crate) fn urlencode_path(path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    for b in path.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
                out.push(b as char)
            }
            _ => {
                out.push('%');
                out.push_str(&format!("{b:02X}"));
            }
        }
    }
    out
}

/// Rewrite relative `src`/`href` asset URLs in an HTML document so sub-resources
/// carry the same `?token=` used for the iframe itself (embedded Browser iframes
/// cannot set an Authorization header). Absolute, root-relative, scheme-relative,
/// fragment, and data: URLs are left untouched.
pub(crate) fn serve_inject_token(html: &str, token: &str) -> String {
    if token.is_empty() || html.contains("?token=") {
        return html.to_string();
    }
    let mut out = String::with_capacity(html.len() + 64);
    let mut rest = html;
    while !rest.is_empty() {
        // Find the next `src="|href="|src='|href='` marker.
        let candidates = ["src=\"", "href=\"", "src='", "href='"]
            .into_iter()
            .filter_map(|m| rest.find(m).map(|p| (p, m)))
            .min_by_key(|(p, _)| *p);
        match candidates {
            Some((pos, marker)) => {
                out.push_str(&rest[..pos]);
                out.push_str(marker);
                let quote = &marker[marker.len() - 1..];
                let value_start = pos + marker.len();
                let value_end = rest[value_start..]
                    .find(quote)
                    .map(|p| value_start + p)
                    .unwrap_or(rest.len());
                let url = &rest[value_start..value_end];
                if !url.is_empty()
                    && !url.starts_with("http://")
                    && !url.starts_with("https://")
                    && !url.starts_with("//")
                    && !url.starts_with('/')
                    && !url.starts_with('#')
                    && !url.starts_with("data:")
                    && !url.starts_with("blob:")
                {
                    let sep = if url.contains('?') { "&" } else { "?" };
                    out.push_str(url);
                    out.push_str(sep);
                    out.push_str("token=");
                    out.push_str(token);
                } else {
                    out.push_str(url);
                }
                out.push_str(quote);
                rest = &rest[value_end + 1..];
            }
            None => {
                out.push_str(rest);
                rest = "";
            }
        }
    }
    out
}
