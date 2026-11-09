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
    #[serde(default)]
    pub sha256_win: String,
}

impl ComponentDownload {
    /// Select the (url, filename, checksum) triple for the requested platform.
    /// Kept as a pure function so the arch/OS matrix is testable on any host.
    fn variant(&self, windows: bool, aarch64: bool) -> (&str, &str, &str) {
        if windows {
            (
                self.url_win.as_deref().unwrap_or(&self.url),
                self.filename_win.as_deref().unwrap_or(&self.filename),
                if self.url_win.is_some() { &self.sha256_win } else { &self.sha256 },
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

    /// Checksum pinned for a specific asset URL (#2001).
    ///
    /// The `llm` component downloads a different archive on a Vulkan host than
    /// on a plain CPU host, so the hash must be resolved from the URL in use,
    /// never from the component id alone.
    pub(crate) fn sha256_for_url(&self, url: &str) -> Option<&str> {
        if self.url == url {
            Some(&self.sha256)
        } else if self.url_win.as_deref() == Some(url) {
            Some(&self.sha256_win)
        } else if self.url_arm64.as_deref() == Some(url) {
            Some(&self.sha256_arm64)
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
            sha256_win: "win".to_string(),
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
        assert_eq!(c.variant(true, true).2, "win");
    }

    #[test]
    fn windows_without_a_pinned_hash_never_uses_the_linux_hash() {
        let mut c = component();
        c.sha256_win.clear();
        assert_eq!(c.variant(true, false).2, "");
    }

    #[test]
    fn sha256_for_url_resolves_each_variant() {
        let c = component();
        assert_eq!(c.sha256_for_url("https://example.com/llama-x86.zip"), Some("x86"));
        assert_eq!(c.sha256_for_url("https://example.com/llama-win.zip"), Some("win"));
        assert_eq!(
            c.sha256_for_url("https://example.com/llama-arm64.tar.gz"),
            Some("arm")
        );
        assert_eq!(c.sha256_for_url("https://example.com/unknown.zip"), None);
    }
}
