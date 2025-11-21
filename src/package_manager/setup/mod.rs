pub mod directory_setup;
pub mod email_setup;

pub use directory_setup::{
    generate_directory_config, DefaultOrganization, DefaultUser, DirectoryConfig, DirectorySetup,
};
pub use email_setup::{generate_email_config, EmailConfig, EmailDomain, EmailSetup};
