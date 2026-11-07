use super::*;

impl ComponentEntry {
    /// Select the URL for the requested platform. Windows wins over arm64 so a
    /// Windows arm64 host still gets the Windows asset; on Linux/macOS arm64 an
    /// explicit arm64 variant is preferred and x86_64 remains the fallback.
    fn select_url(&self, windows: bool, aarch64: bool) -> &str {
        if windows {
            self.url_win.as_deref().unwrap_or(&self.url)
        } else if aarch64 {
            self.url_arm64.as_deref().unwrap_or(&self.url)
        } else {
            &self.url
        }
    }

    pub(super) fn effective_url(&self) -> &str {
        self.select_url(cfg!(target_os = "windows"), cfg!(target_arch = "aarch64"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry() -> ComponentEntry {
        ComponentEntry {
            url: "https://example.com/linux-x86_64".to_string(),
            url_win: Some("https://example.com/windows-x86_64".to_string()),
            url_arm64: Some("https://example.com/linux-aarch64".to_string()),
        }
    }

    #[test]
    fn aarch64_selects_the_arm64_url() {
        let e = entry();
        assert_eq!(e.select_url(false, true), "https://example.com/linux-aarch64");
    }

    #[test]
    fn aarch64_without_a_variant_falls_back_to_x86_64() {
        let mut e = entry();
        e.url_arm64 = None;
        assert_eq!(e.select_url(false, true), "https://example.com/linux-x86_64");
    }

    #[test]
    fn windows_keeps_precedence_over_aarch64() {
        let e = entry();
        assert_eq!(e.select_url(true, true), "https://example.com/windows-x86_64");
    }
}
