//! Vault installation: credential lookup, local init, seeding and unsealing.
//!
//! #1732 — this directory replaces the former flat `installer_vault` file and
//! its numeric-suffix twin, a split that existed only by file name. The file
//! layout now mirrors `botcore::package_manager::installer` step by step
//! (`credentials`, `initialize_local`, `seed_defaults`, `seeds_exist`,
//! `unseal`), so both trees name the same Vault step the same way.

pub mod container_env;
mod credentials;
mod initialize_local;
mod seed_defaults;
mod seeds_exist;
mod unseal;

#[cfg(unix)]
pub use credentials::fetch_vault_credentials_impl;
pub use container_env::{generate_env_from_vault, initialize_vault};
pub use credentials::fetch_vault_credentials;
pub use initialize_local::initialize_vault_local;
pub use seed_defaults::seed_vault_defaults;
pub use seeds_exist::vault_seeds_exist;
pub use unseal::{ensure_env_file_exists, recover_existing_vault, unseal_vault};
