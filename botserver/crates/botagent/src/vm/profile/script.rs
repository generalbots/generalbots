//! `profile::script` — shell generation for one image profile.
//!
//! Split from `profile.rs` per the AGENTS.md file-size rule. The generated
//! block is appended to the bootstrap and workspace-preparation scripts; a
//! failing artifact aborts it (exit 78) after recording the reason in
//! `/opt/vibe/.profile-error`, so a half-provisioned VM is never mistaken for
//! a ready workspace.

use super::{
    Artifact, ArtifactChannel, VmProfile, ARTIFACT_CACHE_ENV, DEFAULT_ARTIFACT_CACHE,
    OFFICE_MAX_DELTA_BYTES, PROFILE_ERROR_FILE,
};

/// Shell that installs one profile, generated from [`Artifact`] data.
///
/// A failure is fatal: a half-installed profile answers document requests with
/// a missing engine, so the reason lands in [`PROFILE_ERROR_FILE`] and the
/// script exits 78 instead of pretending the workspace is ready.
pub(crate) fn install_block(profile: VmProfile) -> String {
    if profile == VmProfile::Base {
        return String::new();
    }
    let mut script = String::new();
    script.push_str(&format!(
        "\n# ── profile '{profile}' (#2062) ────────────────────────────────────────\n\
         # Baked into the prefabricated image by scripts/build_vm_image.sh\n\
         # --profile {profile}; a failure below is fatal, never a silent downgrade.\n\
         # Budget: <= {} MB over today's base image.\n\
         gb_profile_fail() {{\n  \
           printf 'vm-profile: artifact %s failed on profile {profile} (log: /tmp/agent-profile.log)\\n' \"$1\" >&2\n  \
           printf 'vm-profile: artifact %s failed on profile {profile}\\n' \"$1\" > {PROFILE_ERROR_FILE}\n  \
           exit 78\n}}\n",
        OFFICE_MAX_DELTA_BYTES / 1_000_000
    ));
    for artifact in profile.artifacts() {
        script.push_str(&artifact_step(artifact));
    }
    script.push_str(&html_to_pdf_wrapper());
    script
}

fn artifact_step(artifact: &Artifact) -> String {
    if artifact.channel == ArtifactChannel::Static {
        return static_step(artifact);
    }
    let install = match artifact.channel {
        ArtifactChannel::Apt => format!(
            "apt-get -o DPkg::Lock::Timeout=300 install -y -q --no-install-recommends {}",
            artifact.packages.join(" ")
        ),
        ArtifactChannel::Pip => format!(
            "python3 -m pip install --break-system-packages --quiet --disable-pip-version-check {}",
            artifact.packages.join(" ")
        ),
        ArtifactChannel::Static => String::new(),
    };
    format!(
        "\n# {id} ({channel:?}, {mb} MB)\n\
         if ! {probe} >/dev/null 2>&1; then\n  \
           {install} >>/tmp/agent-profile.log 2>&1 || gb_profile_fail {id}\n\
         fi\n",
        id = artifact.id,
        channel = artifact.channel,
        mb = artifact.installed_bytes / 1_000_000,
        probe = artifact.probe,
        install = install,
    )
}

/// Static builds come from the artifact cache (the prefabricated image ships
/// them); the distro package is the online fallback only.
fn static_step(artifact: &Artifact) -> String {
    let cache_file = artifact.cache_file.unwrap_or(artifact.id);
    let fallback = artifact.packages.join(" ");
    format!(
        "\n# {id} ({channel:?}, {mb} MB): static build from the cache, distro package only as\n\
         # the online fallback (it drags ~230 MB of dependencies).\n\
         if ! {probe} >/dev/null 2>&1; then\n  \
           if [ -f \"${{{ARTIFACT_CACHE_ENV}:-{DEFAULT_ARTIFACT_CACHE}}}/{cache_file}\" ]; then\n    \
             mkdir -p /opt/chromium && tar -xf \
             \"${{{ARTIFACT_CACHE_ENV}:-{DEFAULT_ARTIFACT_CACHE}}}/{cache_file}\" -C /opt/chromium \
             >>/tmp/agent-profile.log 2>&1 || gb_profile_fail {id}\n  \
           else\n    \
             apt-get -o DPkg::Lock::Timeout=300 install -y -q --no-install-recommends {fallback} \
             >>/tmp/agent-profile.log 2>&1 || gb_profile_fail {id}\n  \
           fi\n\
         fi\n",
        id = artifact.id,
        channel = artifact.channel,
        mb = artifact.installed_bytes / 1_000_000,
        probe = artifact.probe,
    )
}

/// Schemes a caller may hand to `html2pdf` as an input URI (#2845).
///
/// `file:` is deliberately absent: a local HTML file is supplied as a *path*
/// (resolved and sanitized by the generated wrapper), never as a `file://`
/// URL, otherwise `html2pdf file:///etc/shadow …` becomes an arbitrary-file
/// read. Anything not in this list is refused before it reaches Chromium.
const ALLOWED_INPUT_SCHEMES: &[&str] = &["http", "https"];

/// Chromium flags shared by the generated wrapper and its tests (#2845).
///
/// There is intentionally no `--no-sandbox`: the Incus agent VM is the
/// containment boundary, so the browser keeps its own sandbox instead of
/// running unconfined. `MAP * ~NOTFOUND` blackholes every DNS lookup, so
/// sanitized HTML cannot reach the network.
const CHROME_PDF_FLAGS: &[&str] = &[
    "--headless=new",
    "--disable-gpu",
    "--disable-background-networking",
    "--host-resolver-rules=MAP * ~NOTFOUND",
];

/// True when `uri` uses an allowlisted scheme and carries a host (#2845).
///
/// Rejects `file:`, `data:`, `javascript:`, `about:`, bare paths and malformed
/// authorities. Scheme comparison is case-insensitive; userinfo and IPv6
/// literals are tolerated.
fn is_allowed_input_uri(uri: &str) -> bool {
    let Some((scheme, rest)) = uri.split_once(':') else {
        return false;
    };
    if !ALLOWED_INPUT_SCHEMES
        .iter()
        .any(|allowed| scheme.eq_ignore_ascii_case(allowed))
    {
        return false;
    }
    let Some(authority) = rest.strip_prefix("//") else {
        return false;
    };
    let host_port = match authority.rsplit_once('@') {
        Some((_, host_port)) => host_port,
        None => authority,
    };
    let raw_host = host_port.split(['/', '?', '#']).next().unwrap_or("");
    let host = match raw_host.strip_prefix('[') {
        Some(bracketed) => bracketed
            .split_once(']')
            .map(|(host, _)| host)
            .unwrap_or(raw_host),
        None => raw_host,
    };
    !host.is_empty()
        && !host.contains(char::is_whitespace)
        && host
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | ':'))
}

/// Full argument vector for one PDF render, in order (#2845).
///
/// `output_pdf` and `input_uri` are supplied by the caller; when generating
/// the shell wrapper they are the literal strings `"$output"` and `"$uri"`.
/// Keeping the vector in Rust makes the "no `--no-sandbox`, network disabled"
/// properties unit-testable instead of buried in a here-doc.
fn chromium_pdf_args(input_uri: &str, output_pdf: &str) -> Vec<String> {
    let mut args: Vec<String> = CHROME_PDF_FLAGS
        .iter()
        .map(|flag| (*flag).to_string())
        .collect();
    args.push(format!("--print-to-pdf={output_pdf}"));
    args.push(input_uri.to_string());
    args
}

/// Quote only arguments containing whitespace (the resolver rule does),
/// leaving `"$uri"` / `"$output"` expansions intact for the shell.
fn shell_words(args: &[String]) -> String {
    args.iter()
        .map(|arg| {
            if arg.contains(char::is_whitespace) {
                format!("'{arg}'")
            } else {
                arg.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Python sanitizer the generated wrapper runs on every local HTML input
/// before Chromium opens it (#2845). It drops script/iframe/object/embed and
/// other active elements, event handlers and absolute resource URLs, then
/// installs a restrictive CSP as defense in depth.
const HTML_SANITIZER_PY: &str = r##"
import re
import sys
from html import escape
from html.parser import HTMLParser

DROP_WITH_BODY = {"script", "iframe", "object", "form"}
DROP_VOID = {"embed", "link", "base", "meta"}
URL_ATTRS = {"src", "href", "action", "formaction", "data", "poster", "srcset", "xlink:href"}


class Sanitizer(HTMLParser):
    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.out = []
        self.skip = 0
        self.raw = 0

    def _emit(self, tag, attrs):
        if tag == "style":
            self.raw += 1
        safe = []
        for key, value in attrs:
            key_l = key.lower()
            if key_l.startswith("on"):
                continue
            if key_l in URL_ATTRS:
                value = (value or "").strip()
                if (not value or value.startswith("#")
                        or re.match(r"^[A-Za-z][A-Za-z0-9+.-]*:", value)
                        or value.startswith("//")):
                    continue
            safe.append((key, value))
        rendered = "".join(
            ' {}="{}"'.format(k, escape(v or "", quote=True))
            for k, v in safe
        )
        self.out.append("<{}{}>".format(tag, rendered))

    def handle_starttag(self, tag, attrs):
        tag = tag.lower()
        if tag in DROP_VOID:
            return
        if tag in DROP_WITH_BODY:
            self.skip += 1
            return
        if self.skip:
            return
        self._emit(tag, attrs)

    def handle_startendtag(self, tag, attrs):
        tag = tag.lower()
        if tag in DROP_VOID or tag in DROP_WITH_BODY or tag == "style" or self.skip:
            return
        self._emit(tag, attrs)

    def handle_endtag(self, tag):
        tag = tag.lower()
        if tag in DROP_WITH_BODY:
            if self.skip:
                self.skip -= 1
            return
        if tag in DROP_VOID or self.skip:
            return
        if tag == "style" and self.raw:
            self.raw -= 1
        self.out.append("</{}>".format(tag))

    def handle_data(self, data):
        if not self.skip:
            self.out.append(data if self.raw else escape(data, quote=False))


with open(sys.argv[1], encoding="utf-8", errors="replace") as handle:
    document = handle.read()
parser = Sanitizer()
parser.feed(document)
body = "".join(parser.out)
csp = (
    '<meta http-equiv="Content-Security-Policy" '
    'content="default-src \'none\'; style-src \'unsafe-inline\'; '
    'img-src data:; font-src data:; base-uri \'none\'; '
    'form-action \'none\'">'
)
if re.search(r"</head\s*>", body, re.I):
    body = re.sub(r"(</head\s*>)", lambda match: csp + match.group(1), body, count=1, flags=re.I)
else:
    body = csp + body
with open(sys.argv[2], "w", encoding="utf-8") as handle:
    handle.write("<!DOCTYPE html>" + body)
"##;

/// `html2pdf <input.html> <output.pdf>` — the hardened WeasyPrint replacement:
/// one Chromium invocation, no cold start, no network, no local-file escape.
fn html_to_pdf_wrapper() -> String {
    // Emit the shell allowlist from the Rust one so the two cannot drift.
    let allowed_cases = ALLOWED_INPUT_SCHEMES
        .iter()
        .filter(|scheme| is_allowed_input_uri(&format!("{scheme}://example.invalid/")))
        .flat_map(|scheme| {
            [
                format!("{scheme}://*"),
                format!("{}://*", scheme.to_ascii_uppercase()),
            ]
        })
        .collect::<Vec<_>>()
        .join("|");
    let allowlist = ALLOWED_INPUT_SCHEMES.join(", ");
    let exec = shell_words(&chromium_pdf_args("\"$uri\"", "\"$output\""));

    r##"
# html2pdf — HTML->PDF through a hardened, offline headless Chromium (#2845).
# The Incus VM is the containment boundary (no sandbox flag is passed); DNS is
# blackholed and the input HTML is sanitized before Chromium ever opens it.
cat > /usr/local/bin/html2pdf <<'EOS'
#!/bin/sh
# usage: html2pdf INPUT.html|https://host/page OUTPUT.pdf
set -eu
chrome=${GB_CHROME_BIN:-}
if [ -z "$chrome" ]; then
  if [ -x /opt/chromium/chrome ]; then chrome=/opt/chromium/chrome; else chrome=$(command -v chromium) || chrome=; fi
fi
[ -n "$chrome" ] || { echo 'html2pdf: no chromium in this VM' >&2; exit 78; }
[ "$#" -eq 2 ] || { echo 'html2pdf: usage: html2pdf INPUT OUTPUT.pdf' >&2; exit 64; }

input=$1
output=$2

# --- input URI allowlist (#2845) -------------------------------------------
# Caller schemes allowed: @@ALLOWLIST@@. Local paths have no scheme and are
# resolved below; file:, data:, javascript: and every other scheme are refused.
from_url=0
case "$input" in
  @@ALLOWED_CASES@@) uri=$input; from_url=1 ;;
  [a-zA-Z]*:*) echo 'html2pdf: unsupported input URI scheme' >&2; exit 64 ;;
esac

# --- local path sanitization ------------------------------------------------
if [ "$from_url" -eq 0 ]; then
  case "$input" in
    /*) path=$input ;;
    *)  path=$PWD/$input ;;
  esac
  real=$(realpath -e -- "$path") || { echo 'html2pdf: input path does not exist' >&2; exit 66; }
  case "$real" in
    /tmp/*|/var/tmp/*|/opt/vibe/*|/home/*) ;;
    *) echo 'html2pdf: input outside allowed roots' >&2; exit 66 ;;
  esac
  [ -f "$real" ] || { echo 'html2pdf: input is not a regular file' >&2; exit 66; }
  case "$real" in
    *.html|*.htm|*.xhtml) ;;
    *) echo 'html2pdf: input must be .html/.htm/.xhtml' >&2; exit 66 ;;
  esac
  sanitized=$(mktemp /tmp/html2pdf.XXXXXX.html)
  trap 'rm -f "$sanitized"' EXIT HUP INT TERM
  python3 - "$real" "$sanitized" <<'PY'
@@SANITIZER@@
PY
  uri=file://$sanitized
fi

# --- hardened Chromium (#2845) ---------------------------------------------
# The VM is the sandbox: no sandbox flag, no network, one-shot print.
exec @@EXEC@@
EOS
chmod +x /usr/local/bin/html2pdf
"##
        .replace("@@ALLOWLIST@@", &allowlist)
        .replace("@@ALLOWED_CASES@@", &allowed_cases)
        .replace("@@SANITIZER@@", HTML_SANITIZER_PY.trim())
        .replace("@@EXEC@@", &exec)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_uri_allowlist_accepts_only_http_and_https_with_a_host() {
        for allowed in [
            "http://example.invalid/page.html",
            "https://example.invalid:8443/page.html",
            "HTTPS://Example.INVALID/a",
            "https://user@example.invalid/a",
            "http://[::1]:8080/a",
        ] {
            assert!(is_allowed_input_uri(allowed), "{allowed} must be allowed");
        }
        for rejected in [
            "file:///etc/passwd",
            "file://localhost/etc/passwd",
            "javascript:alert(1)",
            "data:text/html,<script>alert(1)</script>",
            "about:blank",
            "ftp://example.invalid/a",
            "//example.invalid/a",
            "example.invalid/a",
            "https://",
            "https:///a",
            "https://bad host/a",
            "http://",
            "",
        ] {
            assert!(
                !is_allowed_input_uri(rejected),
                "{rejected} must be rejected"
            );
        }
    }

    #[test]
    fn pdf_arguments_are_sandboxed_and_offline() {
        let args = chromium_pdf_args("https://example.invalid/a.html", "/tmp/out.pdf");
        assert_eq!(args.first().map(String::as_str), Some("--headless=new"));
        assert!(!args.iter().any(|arg| arg.contains("--no-sandbox")));
        assert!(args
            .iter()
            .any(|arg| arg == "--host-resolver-rules=MAP * ~NOTFOUND"));
        assert!(args
            .iter()
            .any(|arg| arg == "--disable-background-networking"));
        assert!(args.iter().any(|arg| arg == "--print-to-pdf=/tmp/out.pdf"));
        assert_eq!(
            args.last().map(String::as_str),
            Some("https://example.invalid/a.html")
        );
    }

    #[test]
    fn generated_wrapper_is_hardened_and_sanitizes_html() {
        let wrapper = html_to_pdf_wrapper();
        assert!(!wrapper.contains("--no-sandbox"));
        assert!(wrapper.contains("--host-resolver-rules=MAP * ~NOTFOUND"));
        assert!(wrapper.contains("http://*|HTTP://*|https://*|HTTPS://*"));
        assert!(wrapper.contains("unsupported input URI scheme"));
        assert!(wrapper.contains("realpath -e"));
        assert!(wrapper.contains("Content-Security-Policy"));
        assert!(wrapper.contains("DROP_WITH_BODY = {"));
        assert!(!wrapper.contains("@@"));
    }
}
