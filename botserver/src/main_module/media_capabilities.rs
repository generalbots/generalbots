//! Media/BotModels capability reporting shared by `/api/product` and the
//! first-run provisioner.
//!
//! Only one pythonbox asset is published today
//! (`botmodels-pythonbox-linux-x86_64.tar.gz`). There is no aarch64 Linux or
//! Windows archive, so the desktop edition must say so instead of failing the
//! whole first run when the optional service cannot be installed.

/// Why BotModels (vision/STT/TTS) is unavailable on this host, or `None` when
/// the published Linux x86_64 pythonbox can run here.
pub fn botmodels_unavailable_reason(platform: &str, arch: &str) -> Option<String> {
    match (platform, arch) {
        ("linux", "x86_64") | ("linux", "x86") => None,
        ("windows", _) => Some(
            "BotModels is unavailable on Windows: no Windows pythonbox is published \
             (only botmodels-pythonbox-linux-x86_64). Vision, speech-to-text and \
             text-to-speech stay disabled until a Windows pythonbox exists."
                .to_string(),
        ),
        ("linux", "aarch64") | ("linux", "arm64") => Some(
            "BotModels is unavailable on Linux arm64: no aarch64 pythonbox is published \
             (only botmodels-pythonbox-linux-x86_64). Vision, speech-to-text and \
             text-to-speech stay disabled on this host."
                .to_string(),
        ),
        _ => Some(format!(
            "BotModels is unavailable on {platform}/{arch}: no pythonbox is published for \
             this platform (only botmodels-pythonbox-linux-x86_64)."
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
            "pythonbox": "linux-x86_64",
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
    fn windows_reports_media_unavailable_with_a_clear_reason() {
        let reason = botmodels_unavailable_reason("windows", "x86_64")
            .expect("Windows has no pythonbox");
        assert!(reason.contains("Windows pythonbox"));
        let caps = media_capabilities_json("windows", "x86_64");
        assert_eq!(caps["botmodels"]["available"], false);
        assert_eq!(caps["vision"], false);
        assert_eq!(caps["stt"], false);
        assert_eq!(caps["tts"], false);
        assert_eq!(caps["botmodels"]["reason"], reason);
    }

    #[test]
    fn linux_arm64_reports_media_unavailable() {
        let reason = botmodels_unavailable_reason("linux", "aarch64")
            .expect("no aarch64 pythonbox");
        assert!(reason.contains("Linux arm64"));
        assert_eq!(
            media_capabilities_json("linux", "aarch64")["botmodels"]["available"],
            false
        );
    }
}
