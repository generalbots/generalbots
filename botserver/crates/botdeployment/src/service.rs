//! Cross-platform service controller abstraction. Each platform provides a
//! `ServiceBackend` implementation that can start, stop and check the
//! status of background services (PostgreSQL, Redis, MinIO, etc.) without
//! relying on Linux-only tooling like `pg_ctl`, `initdb` or `systemctl`.
//!
//! Concrete backends (Windows Service Control Manager, launchd, etc.) are
//! stubbed with the structural surface area they need; the actual native
//! call sites live in feature-gated submodules so the build still
//! succeeds when only one OS is the compilation target.

use crate::installer::TargetOs;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceState {
    Stopped,
    Running,
    Unknown,
}

impl fmt::Display for ServiceState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServiceState::Stopped => write!(f, "stopped"),
            ServiceState::Running => write!(f, "running"),
            ServiceState::Unknown => write!(f, "unknown"),
        }
    }
}

pub trait ServiceBackend: Send + Sync {
    fn start(&self, service: &str) -> Result<(), String>;
    fn stop(&self, service: &str) -> Result<(), String>;
    fn status(&self, service: &str) -> Result<ServiceState, String>;
    fn platform(&self) -> TargetOs;
}

pub struct LinuxBackend;

impl ServiceBackend for LinuxBackend {
    fn start(&self, service: &str) -> Result<(), String> {
        Err(format!("linux backend for {service} not wired in this build; use SafeCommand"))
    }
    fn stop(&self, service: &str) -> Result<(), String> {
        Err(format!("linux backend for {service} not wired in this build"))
    }
    fn status(&self, service: &str) -> Result<ServiceState, String> {
        Err(format!("linux backend for {service} status not wired"))
    }
    fn platform(&self) -> TargetOs {
        TargetOs::Linux
    }
}

pub struct MacosBackend;

impl ServiceBackend for MacosBackend {
    fn start(&self, service: &str) -> Result<(), String> {
        Err(format!("macos backend for {service} not wired"))
    }
    fn stop(&self, service: &str) -> Result<(), String> {
        Err(format!("macos backend for {service} not wired"))
    }
    fn status(&self, _service: &str) -> Result<ServiceState, String> {
        Ok(ServiceState::Unknown)
    }
    fn platform(&self) -> TargetOs {
        TargetOs::Macos
    }
}

pub struct WindowsBackend;

impl ServiceBackend for WindowsBackend {
    fn start(&self, service: &str) -> Result<(), String> {
        Err(format!("windows backend for {service} not wired"))
    }
    fn stop(&self, service: &str) -> Result<(), String> {
        Err(format!("windows backend for {service} not wired"))
    }
    fn status(&self, _service: &str) -> Result<ServiceState, String> {
        Ok(ServiceState::Unknown)
    }
    fn platform(&self) -> TargetOs {
        TargetOs::Windows
    }
}

pub fn backend_for(target: TargetOs) -> Box<dyn ServiceBackend> {
    match target {
        TargetOs::Linux => Box::new(LinuxBackend),
        TargetOs::Macos => Box::new(MacosBackend),
        TargetOs::Windows => Box::new(WindowsBackend),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHandle {
    pub name: String,
    pub state: ServiceState,
    pub platform: TargetOs,
}

pub fn enumerate(b: &dyn ServiceBackend, services: &[&str]) -> Vec<ServiceHandle> {
    services
        .iter()
        .map(|name| ServiceHandle {
            name: (*name).to_string(),
            state: b.status(name).unwrap_or(ServiceState::Unknown),
            platform: b.platform(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_for_returns_matching_platform() {
        assert_eq!(backend_for(TargetOs::Linux).platform(), TargetOs::Linux);
        assert_eq!(backend_for(TargetOs::Windows).platform(), TargetOs::Windows);
        assert_eq!(backend_for(TargetOs::Macos).platform(), TargetOs::Macos);
    }

    #[test]
    fn enumerate_returns_handles() {
        let b = backend_for(TargetOs::Windows);
        let handles = enumerate(b.as_ref(), &["postgres", "redis"]);
        assert_eq!(handles.len(), 2);
        assert_eq!(handles[0].name, "postgres");
        assert_eq!(handles[1].name, "redis");
    }

    #[test]
    fn service_state_display() {
        assert_eq!(ServiceState::Running.to_string(), "running");
        assert_eq!(ServiceState::Stopped.to_string(), "stopped");
        assert_eq!(ServiceState::Unknown.to_string(), "unknown");
    }
}
