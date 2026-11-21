//! The only Tauri command exposed by the mobile thin client.
//!
//! No filesystem, download, sync, process-inspection or host-information
//! commands exist here: every suite operation goes through the remote API.

use tauri::Url;

/// Accepts only absolute https links to a real host, with no credentials or
/// fragment. This is a pure validator — no local file or scheme is reachable.
fn https_link(raw: &str) -> Result<Url, String> {
    let url = Url::parse(raw.trim()).map_err(|_| "invalid link".to_string())?;
    if url.scheme() != "https" || url.host_str().unwrap_or_default().is_empty() {
        return Err("only https links can be opened".to_string());
    }
    if !url.username().is_empty() || url.password().is_some() || url.fragment().is_some() {
        return Err("unsupported link".to_string());
    }
    Ok(url)
}

#[tauri::command]
pub fn open_external_url(url: String) -> Result<(), String> {
    let link = https_link(&url)?;
    opener::open(link.as_str()).map_err(|error| format!("could not open link: {error}"))
}

#[cfg(test)]
mod tests {
    use super::https_link;

    #[test]
    fn accepts_https_links() {
        assert!(https_link("https://example.com/page?q=1").is_ok());
    }

    #[test]
    fn rejects_everything_else() {
        assert!(https_link("http://example.com").is_err());
        assert!(https_link("file:///etc/passwd").is_err());
        assert!(https_link("javascript:alert(1)").is_err());
        assert!(https_link("https://user@example.com").is_err());
        assert!(https_link("https://example.com/#frag").is_err());
        assert!(https_link("not a url").is_err());
    }
}
