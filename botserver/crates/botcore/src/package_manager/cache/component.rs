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

    pub(super) fn sha256_for_url(&self, url: &str) -> Option<&str> {
        if self.url == url {
            Some(&self.sha256)
        } else if self.url_arm64.as_deref() == Some(url) {
            Some(&self.sha256_arm64)
        } else {
            // No Windows checksum is pinned in 3rdparty.toml yet; never verify
            // a Windows archive against the Linux hash.
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn component() -> ComponentDownload {
        ComponentDownload {
            name: "drive".to_string(),
            url: "https://example.com/minio-linux-amd64".to_string(),
            filename: "minio".to_string(),
            url_win: Some("https://example.com/minio-windows-amd64.exe".to_string()),
            filename_win: Some("minio.exe".to_string()),
            url_arm64: Some("https://example.com/minio-linux-arm64".to_string()),
            filename_arm64: Some("minio-arm64".to_string()),
            sha256: "x86".to_string(),
            sha256_arm64: "arm".to_string(),
        }
    }

    #[test]
    fn arm64_selects_arm64_url_filename_and_checksum() {
        let c = component();
        assert_eq!(
            c.variant(false, true),
            (
                "https://example.com/minio-linux-arm64",
                "minio-arm64",
                "arm"
            )
        );
    }

    #[test]
    fn arm64_without_a_variant_falls_back_to_x86_64() {
        let mut c = component();
        c.url_arm64 = None;
        c.filename_arm64 = None;
        c.sha256_arm64 = String::new();
        assert_eq!(
            c.variant(false, true),
            ("https://example.com/minio-linux-amd64", "minio", "x86")
        );
    }

    #[test]
    fn windows_keeps_precedence_over_arm64() {
        let c = component();
        assert_eq!(
            c.variant(true, true),
            (
                "https://example.com/minio-windows-amd64.exe",
                "minio.exe",
                "x86"
            )
        );
    }

    #[test]
    fn checksums_follow_the_url_in_use() {
        let c = component();
        assert_eq!(c.sha256_for_url("https://example.com/minio-linux-amd64"), Some("x86"));
        assert_eq!(c.sha256_for_url("https://example.com/minio-linux-arm64"), Some("arm"));
        assert_eq!(c.sha256_for_url("https://example.com/minio-windows-amd64.exe"), None);
    }

    #[test]
    fn bundled_names_satisfy_upstream_urls() {
        let c = component();
        assert_eq!(
            c.filename_for_url("https://example.com/minio-windows-amd64.exe"),
            Some("minio.exe")
        );
        assert_eq!(
            c.filename_for_url("https://example.com/minio-linux-arm64"),
            Some("minio-arm64")
        );
    }
}
