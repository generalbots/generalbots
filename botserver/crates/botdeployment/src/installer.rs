//! Cross-platform installer. Detects the target OS at runtime, selects the
//! right binary version, and produces a download plan that the bootstrap
//! step can execute. No `Command::new` here — actual execution lives in
//! the runtime that wires this with `botcore::security::command_guard`.
//!
//! Mirrors `3rdparty.toml` parsing but adds OS-aware version selection so
//! Windows nodes can bootstrap from a fresh install.

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetOs {
    Linux,
    Macos,
    Windows,
}

impl fmt::Display for TargetOs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TargetOs::Linux => write!(f, "linux"),
            TargetOs::Macos => write!(f, "macos"),
            TargetOs::Windows => write!(f, "windows"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Arch {
    X86_64,
    Aarch64,
}

impl Arch {
    pub fn detect() -> Self {
        match std::env::consts::ARCH {
            "aarch64" | "arm64" => Arch::Aarch64,
            _ => Arch::X86_64,
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            Arch::X86_64 => "x86_64",
            Arch::Aarch64 => "aarch64",
        }
    }
}

impl fmt::Display for Arch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Platform {
    pub os: TargetOs,
    pub arch: Arch,
}

impl Platform {
    pub fn detect() -> Self {
        let os = match std::env::consts::OS {
            "linux" => TargetOs::Linux,
            "macos" => TargetOs::Macos,
            "windows" => TargetOs::Windows,
            _ => TargetOs::Linux,
        };
        Self { os, arch: Arch::detect() }
    }
    pub fn suffix(&self) -> String {
        match self.os {
            TargetOs::Linux => format!("linux-{}", self.arch),
            TargetOs::Macos => format!("macos-{}", self.arch),
            TargetOs::Windows => format!("windows-{}", self.arch),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub name: String,
    pub versions: std::collections::HashMap<String, ComponentVersion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentVersion {
    pub url: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallerManifest {
    pub components: Vec<Component>,
}

impl InstallerManifest {
    pub fn parse_toml(raw: &str) -> Result<Self, String> {
        let value: toml::Value = toml::from_str(raw).map_err(|e| e.to_string())?;
        let mut components = Vec::new();
        if let Some(table) = value.as_table() {
            for (name, entry) in table {
                let versions = match entry.get("versions").and_then(|v| v.as_table()) {
                    Some(t) => t,
                    None => continue,
                };
                let mut parsed = std::collections::HashMap::new();
                for (suffix, v) in versions {
                    let url = v.get("url").and_then(|x| x.as_str()).unwrap_or_default().to_string();
                    let sha = v.get("sha256").and_then(|x| x.as_str()).unwrap_or_default().to_string();
                    parsed.insert(suffix.clone(), ComponentVersion { url, sha256: sha });
                }
                components.push(Component { name: name.clone(), versions: parsed });
            }
        }
        Ok(Self { components })
    }

    pub fn plan_for(&self, platform: Platform) -> Vec<DownloadPlan> {
        let target = platform.suffix();
        self.components
            .iter()
            .filter_map(|c| {
                c.versions.get(&target).cloned().map(|v| DownloadPlan {
                    name: c.name.clone(),
                    platform,
                    url: v.url,
                    sha256: v.sha256,
                })
            })
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadPlan {
    pub name: String,
    pub platform: Platform,
    pub url: String,
    pub sha256: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_suffix_per_os() {
        let p = Platform { os: TargetOs::Linux, arch: Arch::X86_64 };
        assert_eq!(p.suffix(), "linux-x86_64");
        let p = Platform { os: TargetOs::Windows, arch: Arch::X86_64 };
        assert_eq!(p.suffix(), "windows-x86_64");
        let p = Platform { os: TargetOs::Macos, arch: Arch::Aarch64 };
        assert_eq!(p.suffix(), "macos-aarch64");
    }

    #[test]
    fn manifest_plan_filters_to_target_suffix() {
        let raw = r#"
[postgres]
[postgres.versions.linux-x86_64]
url = "https://example/pg-linux.tar.gz"
sha256 = "aaa"
[postgres.versions.windows-x86_64]
url = "https://example/pg-win.zip"
sha256 = "bbb"

[redis]
[redis.versions.linux-x86_64]
url = "https://example/redis-linux.tar.gz"
sha256 = "ccc"
"#;
        let manifest = InstallerManifest::parse_toml(raw).expect("parse");
        let linux = manifest.plan_for(Platform { os: TargetOs::Linux, arch: Arch::X86_64 });
        let windows = manifest.plan_for(Platform { os: TargetOs::Windows, arch: Arch::X86_64 });
        assert_eq!(linux.len(), 2);
        assert_eq!(windows.len(), 1);
        assert_eq!(windows[0].name, "postgres");
    }
}
