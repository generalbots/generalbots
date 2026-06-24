// Bootstrap module - orchestration of bot services
pub mod bootstrap_types;
pub mod bootstrap_utils;
pub mod bootstrap_manager;
pub mod instance;
pub mod vault;

// Re-export for backward compatibility
pub use bootstrap_types::{BootstrapManager, BootstrapProgress};
pub use bootstrap_manager::{check_single_instance, release_instance_lock, has_installed_stack, reset_vault_only, get_db_password_from_vault};
