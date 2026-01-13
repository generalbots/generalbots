#![recursion_limit = "512"]

pub mod auto_task;
pub mod basic;
pub mod billing;
pub mod canvas;
pub mod channels;
pub mod contacts;
pub mod core;
pub mod dashboards;
pub mod embedded_ui;
pub mod maintenance;
pub mod multimodal;
pub mod player;
pub mod people;
pub mod products;
pub mod search;
pub mod security;
pub mod tickets;
pub mod attendant;

pub mod analytics;
pub mod designer;
pub mod docs;
pub mod learn;
pub mod paper;
pub mod research;
pub mod sheet;
pub mod slides;
pub mod social;
pub mod sources;
pub mod video;

pub use core::shared;

#[derive(Debug, Clone)]
pub enum BootstrapProgress {
    StartingBootstrap,
    InstallingComponent(String),
    StartingComponent(String),
    UploadingTemplates,
    ConnectingDatabase,
    StartingLLM,
    BootstrapComplete,
    BootstrapError(String),
}

pub use core::automation;
pub use core::bootstrap;
pub use core::bot;
pub use core::config;
pub use core::package_manager;
pub use core::session;

pub use security::{get_secure_port, SecurityConfig, SecurityManager};

#[cfg(feature = "attendance")]
pub mod attendance;

#[cfg(feature = "calendar")]
pub mod calendar;

#[cfg(feature = "compliance")]
pub mod compliance;

#[cfg(feature = "console")]
pub mod console;

#[cfg(feature = "directory")]
pub mod directory;

#[cfg(feature = "drive")]
pub mod drive;
#[cfg(feature = "drive")]
pub use drive::drive_monitor::DriveMonitor;

#[cfg(feature = "email")]
pub mod email;

#[cfg(feature = "instagram")]
pub mod instagram;

#[cfg(feature = "llm")]
pub mod llm;
#[cfg(feature = "llm")]
pub use llm::cache::{CacheConfig, CachedLLMProvider, CachedResponse, LocalEmbeddingService};
#[cfg(feature = "llm")]
pub use llm::DynamicLLMProvider;

#[cfg(feature = "meet")]
pub mod meet;

pub mod monitoring;

pub mod project;

pub mod workspaces;

pub mod botmodels;

pub mod legal;

pub mod settings;

#[cfg(feature = "msteams")]
pub mod msteams;

#[cfg(feature = "nvidia")]
pub mod nvidia;

#[cfg(feature = "tasks")]
pub mod tasks;
#[cfg(feature = "tasks")]
pub use tasks::TaskEngine;

#[cfg(feature = "vectordb")]
#[path = "vector-db/mod.rs"]
pub mod vector_db;

#[cfg(feature = "weba")]
pub mod weba;

#[cfg(feature = "whatsapp")]
pub mod whatsapp;

#[cfg(feature = "telegram")]
pub mod telegram;

#[cfg(test)]
mod tests {
    use super::*;

    // Test configuration types from bottest/harness.rs

    #[derive(Debug, Clone)]
    pub struct TestConfig {
        pub postgres: bool,
        pub minio: bool,
        pub redis: bool,
        pub mock_zitadel: bool,
        pub mock_llm: bool,
        pub run_migrations: bool,
    }

    impl Default for TestConfig {
        fn default() -> Self {
            Self {
                postgres: true,
                minio: false,
                redis: false,
                mock_zitadel: true,
                mock_llm: true,
                run_migrations: true,
            }
        }
    }

    impl TestConfig {
        pub const fn minimal() -> Self {
            Self {
                postgres: false,
                minio: false,
                redis: false,
                mock_zitadel: false,
                mock_llm: false,
                run_migrations: false,
            }
        }

        pub const fn full() -> Self {
            Self {
                postgres: false,
                minio: false,
                redis: false,
                mock_zitadel: true,
                mock_llm: true,
                run_migrations: false,
            }
        }

        pub const fn database_only() -> Self {
            Self {
                postgres: true,
                minio: false,
                redis: false,
                mock_zitadel: false,
                mock_llm: false,
                run_migrations: true,
            }
        }
    }

    // Port allocation types from bottest/ports.rs

    #[derive(Debug)]
    pub struct TestPorts {
        pub postgres: u16,
        pub minio: u16,
        pub redis: u16,
        pub botserver: u16,
        pub mock_zitadel: u16,
        pub mock_llm: u16,
    }

    impl TestPorts {
        pub fn allocate_starting_from(base: u16) -> Self {
            Self {
                postgres: base,
                minio: base + 1,
                redis: base + 2,
                botserver: base + 3,
                mock_zitadel: base + 4,
                mock_llm: base + 5,
            }
        }
    }

    // Default ports from bottest/harness.rs

    pub struct DefaultPorts;

    impl DefaultPorts {
        pub const POSTGRES: u16 = 5432;
        pub const MINIO: u16 = 9000;
        pub const REDIS: u16 = 6379;
        pub const ZITADEL: u16 = 8080;
        pub const BOTSERVER: u16 = 4242;
    }

    // Expectation types from bottest/mocks/mod.rs

    #[derive(Debug, Clone)]
    pub struct Expectation {
        pub name: String,
        pub expected_calls: Option<usize>,
        pub actual_calls: usize,
        pub matched: bool,
    }

    impl Expectation {
        pub fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                expected_calls: None,
                actual_calls: 0,
                matched: false,
            }
        }

        pub const fn times(mut self, n: usize) -> Self {
            self.expected_calls = Some(n);
            self
        }

        pub fn record_call(&mut self) {
            self.actual_calls += 1;
            self.matched = true;
        }

        pub fn verify(&self) -> Result<(), String> {
            if let Some(expected) = self.expected_calls {
                if self.actual_calls != expected {
                    return Err(format!(
                        "Expectation '{}' expected {} calls but got {}",
                        self.name, expected, self.actual_calls
                    ));
                }
            }
            Ok(())
        }
    }

    // Tests

    #[test]
    fn test_library_loads() {
        let version = env!("CARGO_PKG_VERSION");
        assert!(!version.is_empty());
    }

    #[test]
    fn test_config_default() {
        let config = TestConfig::default();
        assert!(config.postgres);
        assert!(!config.minio);
        assert!(!config.redis);
        assert!(config.mock_zitadel);
        assert!(config.mock_llm);
        assert!(config.run_migrations);
    }

    #[test]
    fn test_config_full() {
        let config = TestConfig::full();
        assert!(!config.postgres);
        assert!(!config.minio);
        assert!(!config.redis);
        assert!(config.mock_zitadel);
        assert!(config.mock_llm);
        assert!(!config.run_migrations);
    }

    #[test]
    fn test_config_minimal() {
        let config = TestConfig::minimal();
        assert!(!config.postgres);
        assert!(!config.minio);
        assert!(!config.redis);
        assert!(!config.mock_zitadel);
        assert!(!config.mock_llm);
        assert!(!config.run_migrations);
    }

    #[test]
    fn test_config_database_only() {
        let config = TestConfig::database_only();
        assert!(config.postgres);
        assert!(!config.minio);
        assert!(!config.redis);
        assert!(!config.mock_zitadel);
        assert!(!config.mock_llm);
        assert!(config.run_migrations);
    }

    #[test]
    fn test_port_allocation() {
        let ports = TestPorts::allocate_starting_from(15000);
        assert_eq!(ports.postgres, 15000);
        assert_eq!(ports.minio, 15001);
        assert_eq!(ports.redis, 15002);
        assert_eq!(ports.botserver, 15003);
        assert_eq!(ports.mock_zitadel, 15004);
        assert_eq!(ports.mock_llm, 15005);
    }

    #[test]
    fn test_default_ports() {
        assert_eq!(DefaultPorts::POSTGRES, 5432);
        assert_eq!(DefaultPorts::MINIO, 9000);
        assert_eq!(DefaultPorts::REDIS, 6379);
        assert_eq!(DefaultPorts::ZITADEL, 8080);
        assert_eq!(DefaultPorts::BOTSERVER, 4242);
    }

    #[test]
    fn test_expectation_basic() {
        let mut exp = Expectation::new("test");
        assert_eq!(exp.actual_calls, 0);
        assert!(!exp.matched);

        exp.record_call();
        assert_eq!(exp.actual_calls, 1);
        assert!(exp.matched);
    }

    #[test]
    fn test_expectation_times() {
        let mut exp = Expectation::new("test").times(2);
        exp.record_call();
        exp.record_call();

        assert!(exp.verify().is_ok());
    }

    #[test]
    fn test_expectation_times_fail() {
        let mut exp = Expectation::new("test").times(2);
        exp.record_call();

        assert!(exp.verify().is_err());
    }

    #[test]
    fn test_bootstrap_progress_variants() {
        let progress = BootstrapProgress::StartingBootstrap;
        assert!(matches!(progress, BootstrapProgress::StartingBootstrap));

        let progress = BootstrapProgress::InstallingComponent("postgres".to_string());
        assert!(matches!(
            progress,
            BootstrapProgress::InstallingComponent(_)
        ));

        let progress = BootstrapProgress::StartingComponent("redis".to_string());
        assert!(matches!(progress, BootstrapProgress::StartingComponent(_)));

        let progress = BootstrapProgress::BootstrapComplete;
        assert!(matches!(progress, BootstrapProgress::BootstrapComplete));

        let progress = BootstrapProgress::BootstrapError("test error".to_string());
        assert!(matches!(progress, BootstrapProgress::BootstrapError(_)));
    }

    #[test]
    fn test_ports_are_unique() {
        let ports = TestPorts::allocate_starting_from(20000);
        let all_ports = vec![
            ports.postgres,
            ports.minio,
            ports.redis,
            ports.botserver,
            ports.mock_zitadel,
            ports.mock_llm,
        ];

        let mut seen = std::collections::HashSet::new();
        for port in &all_ports {
            assert!(seen.insert(*port), "Duplicate port found: {}", port);
        }
    }

    #[test]
    fn test_expectation_multiple_calls() {
        let mut exp = Expectation::new("multi-call").times(5);

        for _ in 0..5 {
            exp.record_call();
        }

        assert_eq!(exp.actual_calls, 5);
        assert!(exp.verify().is_ok());
    }

    #[test]
    fn test_expectation_no_expected_calls() {
        let mut exp = Expectation::new("any-calls");

        exp.record_call();
        exp.record_call();
        exp.record_call();

        // Should pass verification since no expected call count was set
        assert!(exp.verify().is_ok());
    }
}
