//! Media/BotModels capability reporting shared by `/api/product` and the
//! first-run provisioner.
//!
//! The published pythonbox releases carry `botmodels-pythonbox-linux-x86_64`
//! and `botmodels-pythonbox-windows-x86_64`. There is no aarch64 archive, so
//! arm64 hosts must report the gap instead of failing the whole first run when
//! the optional service cannot be installed.

/// The published pythonbox asset for a supported host, or `None`.
fn published_pythonbox(platform: &str, arch: &str) -> Option<&'static str> {
    match (platform, arch) {
        ("linux", "x86_64") | ("linux", "x86") => Some("linux-x86_64"),
        ("windows", "x86_64") | ("windows", "x86") => Some("windows-x86_64"),
        _ => None,
    }
}

/// Why BotModels (vision/STT/TTS) is unavailable on this host, or `None` when
/// a published pythonbox can run here.
pub fn botmodels_unavailable_reason(platform: &str, arch: &str) -> Option<String> {
    if published_pythonbox(platform, arch).is_some() {
        return None;
    }
    match (platform, arch) {
        ("linux", "aarch64") | ("linux", "arm64") => Some(
            "BotModels is unavailable on Linux arm64: no aarch64 pythonbox is published \
             (only botmodels-pythonbox-linux-x86_64 and \
             botmodels-pythonbox-windows-x86_64). Vision, speech-to-text and \
             text-to-speech stay disabled on this host."
                .to_string(),
        ),
        _ => Some(format!(
            "BotModels is unavailable on {platform}/{arch}: no pythonbox is published for \
             this platform (only Linux x86_64 and Windows x86_64 archives)."
        )),
    }
}

/// The `media_capabilities` object returned by `/api/product`.
pub fn media_capabilities_json(platform: &str, arch: &str) -> serde_json::Value {
    let reason = botmodels_unavailable_reason(platform, arch);
    let available = reason.is_none();
    serde_json::json!({
        "botmodels": {
            "available": available,
            "platform": platform,
            "arch": arch,
            "pythonbox": published_pythonbox(platform, arch),
            "reason": reason,
        },
        "vision": available,
        "stt": available,
        "tts": available,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linux_x86_64_can_run_the_published_pythonbox() {
        assert_eq!(botmodels_unavailable_reason("linux", "x86_64"), None);
        let caps = media_capabilities_json("linux", "x86_64");
        assert_eq!(caps["botmodels"]["available"], true);
        assert_eq!(caps["vision"], true);
        assert!(caps["botmodels"]["reason"].is_null());
    }

    #[test]
    fn windows_x86_64_uses_the_windows_pythonbox() {
        assert_eq!(botmodels_unavailable_reason("windows", "x86_64"), None);
        let caps = media_capabilities_json("windows", "x86_64");
        assert_eq!(caps["botmodels"]["available"], true);
        assert_eq!(caps["botmodels"]["pythonbox"], "windows-x86_64");
        assert_eq!(caps["vision"], true);
        assert_eq!(caps["stt"], true);
        assert_eq!(caps["tts"], true);
        assert!(caps["botmodels"]["reason"].is_null());
    }

    #[test]
    fn linux_arm64_reports_media_unavailable() {
        let reason =
            botmodels_unavailable_reason("linux", "aarch64").expect("no aarch64 pythonbox");
        assert!(reason.contains("Linux arm64"));
        assert_eq!(
            media_capabilities_json("linux", "aarch64")["botmodels"]["available"],
            false
        );
    }

    #[test]
    fn windows_arm64_reports_the_platform_gap() {
        let reason =
            botmodels_unavailable_reason("windows", "aarch64").expect("no Windows arm64 pythonbox");
        assert!(reason.contains("windows/aarch64"));
        assert_eq!(
            media_capabilities_json("windows", "aarch64")["botmodels"]["available"],
            false
        );
    }
}
