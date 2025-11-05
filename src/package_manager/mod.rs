pub mod component;
pub mod installer;
pub mod os;

pub use installer::PackageManager;
pub mod cli;
pub mod facade;

#[derive(Debug, Clone, PartialEq)]
pub enum InstallMode {
    Local,
    Container,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OsType {
    Linux,
    MacOS,
    Windows,
}

pub struct ComponentInfo {
    pub name: &'static str,
    pub termination_command: &'static str,
}

pub fn get_all_components() -> Vec<ComponentInfo> {
    vec![
        ComponentInfo {
            name: "tables",
            termination_command: "postgres",
        },
        ComponentInfo {
            name: "cache",
            termination_command: "redis-server",
        },
        ComponentInfo {
            name: "drive",
            termination_command: "minio",
        },
        ComponentInfo {
            name: "llm",
            termination_command: "llama-server",
        },
    ]
}
