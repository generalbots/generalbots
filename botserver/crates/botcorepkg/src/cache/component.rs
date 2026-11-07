use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ComponentDownload {
    pub name: String,
    pub url: String,
    pub filename: String,
    #[serde(default)]
    pub url_win: Option<String>,
    #[serde(default)]
    pub filename_win: Option<String>,
    #[serde(default)]
    pub url_arm64: Option<String>,
    #[serde(default)]
    pub filename_arm64: Option<String>,
    #[serde(default)]
    pub sha256: String,
    #[serde(default)]
    pub sha256_arm64: String,
}

impl ComponentDownload {
    /// Select the (url, filename, checksum) triple for the requested platform.
    /// Kept as a pure function so the arch/OS matrix is testable on any host.
    fn variant(&self, windows: bool, aarch64: bool) -> (&str, &str, &str) {
        if windows {
            (
                self.url_win.as_deref().unwrap_or(&self.url),
                self.filename_win.as_deref().unwrap_or(&self.filename),
                &self.sha256,
            )
        } else if aarch64 && self.url_arm64.is_some() {
            (
                self.url_arm64.as_deref().unwrap_or(&self.url),
                self.filename_arm64.as_deref().unwrap_or(&self.filename),
                &self.sha256_arm64,
            )
        } else {
            (&self.url, &self.filename, &self.sha256)
        }
    }

    pub fn effective_url(&self) -> &str {
        self.variant(cfg!(target_os = "windows"), cfg!(target_arch = "aarch64"))
            .0
    }

    pub fn effective_filename(&self) -> &str {
        self.variant(cfg!(target_os = "windows"), cfg!(target_arch = "aarch64"))
            .1
    }

    pub fn effective_sha256(&self) -> &str {
        self.variant(cfg!(target_os = "windows"), cfg!(target_arch = "aarch64"))
            .2
    }

    pub(super) fn filename_for_url(&self, url: &str) -> Option<&str> {
        if self.url == url {
            Some(&self.filename)
        } else if self.url_win.as_deref() == Some(url) {
            Some(self.filename_win.as_deref().unwrap_or(&self.filename))
        } else if self.url_arm64.as_deref() == Some(url) {
            Some(self.filename_arm64.as_deref().unwrap_or(&self.filename))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn component() -> ComponentDownload {
        ComponentDownload {
            name: "llm".to_string(),
            url: "https://example.com/llama-x86.zip".to_string(),
            filename: "llama-x86.zip".to_string(),
            url_win: Some("https://example.com/llama-win.zip".to_string()),
            filename_win: Some("llama-win.zip".to_string()),
            url_arm64: Some("https://example.com/llama-arm64.tar.gz".to_string()),
            filename_arm64: Some("llama-arm64.tar.gz".to_string()),
            sha256: "x86".to_string(),
            sha256_arm64: "arm".to_string(),
        }
    }

    #[test]
    fn arm64_variant_is_selected_with_its_own_checksum() {
        let c = component();
        assert_eq!(
            c.variant(false, true),
            (
                "https://example.com/llama-arm64.tar.gz",
                "llama-arm64.tar.gz",
                "arm"
            )
        );
    }

    #[test]
    fn x86_is_unchanged_when_no_arm64_variant_exists() {
        let mut c = component();
        c.url_arm64 = None;
        c.filename_arm64 = None;
        c.sha256_arm64 = String::new();
        assert_eq!(
            c.variant(false, true),
            ("https://example.com/llama-x86.zip", "llama-x86.zip", "x86")
        );
    }

    #[test]
    fn windows_uses_the_win_variant() {
        let c = component();
        assert_eq!(c.variant(true, true).1, "llama-win.zip");
    }
}
