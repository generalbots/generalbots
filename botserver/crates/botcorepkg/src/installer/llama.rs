use super::*;

pub const LLAMA_CPP_VERSION: &str = "b7345";

pub fn get_llama_cpp_url() -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        #[cfg(target_arch = "x86_64")]
        {
        std::path::Path::new("/usr/local/cuda").exists()
            || std::path::Path::new("/opt/cuda").exists()
            || std::env::var("CUDA_HOME").is_ok();

            if std::path::Path::new("/usr/share/vulkan").exists()
                || std::env::var("VULKAN_SDK").is_ok()
            {
                info!("Detected Vulkan - using Vulkan build");
                return get_component_url("llm_linux_vulkan");
            }

            info!("Using standard Ubuntu x64 build (CPU)");
            get_component_url("llm")
        }

        #[cfg(target_arch = "s390x")]
        {
            info!("Detected s390x architecture");
            return get_component_url("llm_linux_s390x");
        }

        #[cfg(target_arch = "aarch64")]
        {
            info!("Detected ARM64 architecture on Linux");
            // The `llm` component carries a Linux arm64 variant; the resolver
            // selects its url_arm64 on aarch64 and keeps the x86_64 URL for the
            // standard build. No Vulkan detection here (CPU build).
            get_component_url("llm")
        }
    }

    #[cfg(target_os = "macos")]
    {
        #[cfg(target_arch = "aarch64")]
        {
            info!("Detected macOS ARM64 (Apple Silicon)");
            return get_component_url("llm_macos_arm64");
        }

        #[cfg(target_arch = "x86_64")]
        {
            info!("Detected macOS x64 (Intel)");
            return get_component_url("llm_macos_x64");
        }
    }

    #[cfg(target_os = "windows")]
    {
        #[cfg(target_arch = "x86_64")]
        {
            match windows_cuda_major() {
                Some(13) => {
                    info!("Detected CUDA 13.x on Windows");
                    return get_component_url("llm_win_cuda13");
                }
                Some(_) => {
                    info!("Detected CUDA 12.x on Windows");
                    return get_component_url("llm_win_cuda12");
                }
                None => {}
            }

            if std::env::var("VULKAN_SDK").is_ok() {
                info!("Detected Vulkan SDK on Windows");
                return get_component_url("llm_win_vulkan");
            }

            info!("Using standard Windows x64 CPU build");
            return get_component_url("llm_win_cpu_x64");
        }

        #[cfg(target_arch = "aarch64")]
        {
            info!("Detected Windows ARM64");
            return get_component_url("llm_win_cpu_arm64");
        }
    }
}

/// The CUDA major version of the toolkit on this Windows machine (12 or 13),
/// when one is installed.
#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
fn windows_cuda_major() -> Option<u8> {
    std::env::var("CUDA_PATH").ok()?;
    let output = safe_nvcc_version()?;
    let version = String::from_utf8_lossy(&output.stdout);
    if version.contains("13.") {
        Some(13)
    } else if version.contains("12.") {
        Some(12)
    } else {
        None
    }
}

/// The archive of NVIDIA runtime libraries that belongs next to the engine
/// chosen by [`get_llama_cpp_url`]. `None` for builds that need none (CPU,
/// Vulkan, and every platform other than Windows x64 with CUDA).
pub fn get_llama_cpp_runtime_url() -> Option<String> {
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        return match windows_cuda_major()? {
            13 => get_component_url("cudart_win_cuda13"),
            _ => get_component_url("cudart_win_cuda12"),
        };
    }
    #[cfg(not(all(target_os = "windows", target_arch = "x86_64")))]
    {
        None
    }
}

/// The llama.cpp build a release URL points at: `b12483` for
/// `.../releases/download/b12483/llama-b12483-bin-win-cpu-x64.zip`.
pub fn llama_build_tag(url: &str) -> Option<String> {
    let rest = url.split("/releases/download/").nth(1)?;
    let tag = rest.split('/').next()?;
    let digits = tag.strip_prefix('b')?;
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some(tag.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_tag_comes_from_the_release_segment() {
        assert_eq!(
            llama_build_tag("https://github.com/ggml-org/llama.cpp/releases/download/b12483/llama-b12483-bin-win-cuda-13.5-x64.zip").as_deref(),
            Some("b12483"),
        );
        assert_eq!(llama_build_tag("https://example.com/llama.zip"), None);
        assert_eq!(llama_build_tag("https://github.com/o/r/releases/download/v1.0.0/x.zip"), None);
    }

    #[test]
    fn every_pinned_windows_engine_has_a_build_tag_and_a_checksum() {
        let config = get_thirdparty_config();
        for name in ["llm_win_cpu_x64", "llm_win_cuda12", "llm_win_cuda13", "llm_win_vulkan"] {
            let component = config.components.get(name).expect(name);
            assert!(llama_build_tag(&component.url).is_some(), "{name} has no build tag");
            assert_eq!(component.sha256.len(), 64, "{name} has no checksum");
        }
        for name in ["cudart_win_cuda12", "cudart_win_cuda13"] {
            let component = config.components.get(name).expect(name);
            assert_eq!(component.sha256.len(), 64, "{name} has no checksum");
        }
    }
}
