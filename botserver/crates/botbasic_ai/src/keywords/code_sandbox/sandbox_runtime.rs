use super::*;

impl From<&str> for SandboxRuntime {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "lxc" => Self::LXC,
            "docker" => Self::Docker,
            "firecracker" => Self::Firecracker,
            // Unknown configuration must never silently become host execution.
            _ => Self::Unknown,
        }
    }
}
