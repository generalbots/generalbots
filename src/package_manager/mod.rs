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
