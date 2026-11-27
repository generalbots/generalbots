//! HTTP download helpers for the agent runtime.
//!
//! Uses reqwest blocking with `rustls-tls-native-roots` so the Windows
//! certificate store is trusted (government TLS-inspecting proxies).
//! The client is built once per module load and reused.
//!
//! Downloads are limited to the release hosts (github.com and GitHub's
//! asset CDN).  The redirect policy is `none` so a redirect cannot move
//! the request to an internal address.

use super::DownloadError;
use std::io::Write;
use std::path::Path;

const UA: &str = "GeneralBots-Desktop";

/// Hosts from which the agent runtime may download.
const ALLOWED_HOSTS: &[&str] = &[
    "github.com",
    "objects.githubusercontent.com",
    "release-assets.githubusercontent.com",
];

fn host_allowed(url: &str) -> bool {
    if let Ok(parsed) = reqwest::Url::parse(url) {
        if parsed.scheme() != "https" {
            return false;
        }
        if let Some(host) = parsed.host_str() {
            return ALLOWED_HOSTS.iter().any(|h| host.eq_ignore_ascii_case(h));
        }
    }
    false
}

fn build_client() -> Result<reqwest::blocking::Client, DownloadError> {
    reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(600))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent(UA)
        .build()
        .map_err(|e| classify_reqwest_error(&e))
}

/// Map a reqwest error to our error type, detecting certificate failures.
fn classify_reqwest_error(e: &reqwest::Error) -> DownloadError {
    let msg = e.to_string();
    let lower = msg.to_lowercase();
    if lower.contains("certificate")
        || lower.contains("cert")
        || lower.contains("tls")
        || lower.contains("ssl")
        || lower.contains("handshake")
    {
        DownloadError::CertificateNotTrusted(
            "the server certificate was not trusted; \
             a TLS-inspecting proxy may require your IT department \
             to add its certificate to the Windows certificate store"
                .into(),
        )
    } else {
        DownloadError::Network(msg)
    }
}

pub(super) fn download_with_resume(
    url: &str,
    dest: &Path,
    expected_size: u64,
    on_progress: &dyn Fn(u64, u64),
    cancel: &dyn Fn() -> bool,
) -> Result<u64, DownloadError> {
    if !host_allowed(url) {
        return Err(DownloadError::Network(format!("host not allowed: {url}")));
    }

    let client = build_client()?;

    // If the file on disk is already larger than expected, delete it.
    let existing_len = dest.metadata().map(|m| m.len()).unwrap_or(0);
    if existing_len > expected_size && expected_size > 0 {
        let _ = std::fs::remove_file(dest);
    }

    let existing = dest.metadata().map(|m| m.len()).unwrap_or(0);

    let mut request = client.get(url);
    if existing > 0 {
        request = request.header("Range", format!("bytes={existing}-"));
    }

    let response = request
        .send()
        .map_err(|e| classify_reqwest_error(&e))?;
    let status = response.status().as_u16();

    if status == 416 {
        // Range not satisfiable: the file is already complete.
        on_progress(existing, existing);
        return Ok(existing);
    }
    if status == 301 || status == 302 || status == 307 || status == 308 {
        // Redirect to a different host.  Follow it only if the target is
        // also an allowed host.
        if let Some(location) = response.headers().get("location") {
            let loc = location.to_str().unwrap_or("");
            if host_allowed(loc) {
                // Retry with the new URL from the start (no resume on redirect).
                let _ = std::fs::remove_file(dest);
                return download_with_resume(loc, dest, expected_size, on_progress, cancel);
            }
        }
        return Err(DownloadError::Network(format!("redirect to untrusted host (HTTP {status})")));
    }
    if !(200..300).contains(&status) {
        return Err(DownloadError::Network(format!("HTTP {status}")));
    }

    // If the server answered 200 (not 206) to a Range request, it is
    // sending the full file.  Truncate the partial file and write from
    // the start so we do not append the full body to the existing data.
    let resume_offset = if status == 206 {
        // Verify the server is resuming from where we asked.
        let valid_resume = response
            .headers()
            .get("content-range")
            .and_then(|v| v.to_str().ok())
            .is_some_and(|cr| {
                cr.starts_with(&format!("bytes {existing}-"))
            });
        if !valid_resume {
            // Server resumed from a different offset; restart from zero.
            let _ = std::fs::remove_file(dest);
            0u64
        } else {
            existing
        }
    } else {
        // 200: full response.  Truncate any partial file.
        let _ = std::fs::remove_file(dest);
        0u64
    };

    let total = response
        .content_length()
        .map(|cl| cl + resume_offset)
        .unwrap_or(0);

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dest)?;
    let mut downloaded = resume_offset;
    let mut reader = response;
    let mut buf = vec![0u8; 64 * 1024];

    loop {
        if cancel() {
            return Err(DownloadError::Cancelled);
        }
        let n = std::io::Read::read(&mut reader, &mut buf)
            .map_err(|e| DownloadError::Network(e.to_string()))?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n])?;
        downloaded += n as u64;
        on_progress(downloaded, total);
    }
    file.flush()?;
    Ok(downloaded)
}

pub(super) fn fetch_bytes(url: &str) -> Result<Vec<u8>, DownloadError> {
    if !host_allowed(url) {
        return Err(DownloadError::Network(format!("host not allowed: {url}")));
    }
    let client = build_client()?;
    let response = client
        .get(url)
        .send()
        .map_err(|e| classify_reqwest_error(&e))?;
    let status = response.status().as_u16();
    // Follow one redirect to an allowed host (GitHub redirects release
    // assets to its CDN).
    if matches!(status, 301 | 302 | 307 | 308) {
        if let Some(location) = response.headers().get("location") {
            let loc = location.to_str().unwrap_or("");
            if host_allowed(loc) {
                return fetch_bytes(loc);
            }
        }
        return Err(DownloadError::Network(format!("redirect to untrusted host (HTTP {status})")));
    }
    if !response.status().is_success() {
        return Err(DownloadError::Network(format!("HTTP {status}")));
    }
    response
        .bytes()
        .map(|b| b.to_vec())
        .map_err(|e| classify_reqwest_error(&e))
}
