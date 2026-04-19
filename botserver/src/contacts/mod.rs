// Contacts API - Core contact management functionality
pub mod contacts_api;

#[cfg(feature = "calendar")]
pub mod calendar_integration;
pub mod crm;
pub mod crm_ui;
#[cfg(feature = "external_sync")]
pub mod external_sync;
#[cfg(feature = "external_sync")]
pub mod google_client;
#[cfg(feature = "external_sync")]
pub mod microsoft_client;
#[cfg(feature = "external_sync")]
pub mod sync_types;
#[cfg(feature = "tasks")]
pub mod tasks_integration;

// Re-export contacts_api types for backward compatibility
pub use contacts_api::*;
