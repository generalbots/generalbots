//! Managed public OAuth clients for the desktop edition.
//!
//! The public desktop build may ship first-party Google and Microsoft OAuth
//! *client ids*. These are public identifiers (never secrets): they identify
//! an "installed app" / public-client registration that uses a loopback
//! redirect and PKCE. Keeping them out of source lets maintainers rotate a
//! registration without a code change and lets self-hosters replace them.
//!
//! Resolution order for every provider:
//!
//! 1. Runtime environment (`GB_OAUTH_GOOGLE_CLIENT_ID`,
//!    `GB_OAUTH_MICROSOFT_CLIENT_ID`, ...).
//! 2. Compile-time environment with the same names, injected by the release
//!    workflow (`option_env!`). This is how a packaged desktop build carries
//!    the managed id without storing it in the repository.
//! 3. `None`, which makes the caller fall back to a user-provided app
//!    registration.
//!
//! No client *secret* is ever read or embedded here. Public clients complete
//! the authorization-code exchange with PKCE alone.

use serde::{Deserialize, Serialize};

/// Canonical runtime/build variable for the managed Google client id.
pub const GOOGLE_CLIENT_ID_ENV: &str = "GB_OAUTH_GOOGLE_CLIENT_ID";
/// Canonical runtime/build variable for the managed Microsoft client id.
pub const MICROSOFT_CLIENT_ID_ENV: &str = "GB_OAUTH_MICROSOFT_CLIENT_ID";
/// Optional Microsoft Entra tenant for the managed app (`common` by default).
pub const MICROSOFT_TENANT_ID_ENV: &str = "GB_OAUTH_MICROSOFT_TENANT_ID";

/// The managed provider family a deployment can inject.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ManagedProvider {
    Google,
    Microsoft,
}

impl ManagedProvider {
    /// Maps an integration/provider slug onto the managed client family.
    ///
    /// Gmail, Drive, Calendar, Photos and the other Google APIs all use the
    /// same installed-app registration. Outlook, OneDrive, SharePoint and
    /// Microsoft 365 use the same public Entra application.
    #[must_use]
    pub fn from_slug(slug: &str) -> Option<Self> {
        match slug.trim().to_ascii_lowercase().as_str() {
            "google"
            | "gmail"
            | "drive"
            | "gdrive"
            | "google_drive"
            | "calendar"
            | "google_calendar"
            | "google_photos"
            | "google_forms"
            | "google_cloud"
            | "google_workspace_admin"
            | "blogger"
            | "youtube_analytics" => Some(Self::Google),
            "microsoft" | "m365" | "office365" | "outlook" | "outlook_calendar" | "onedrive"
            | "sharepoint" => Some(Self::Microsoft),
            _ => None,
        }
    }

    /// The managed public client id, if this build/runtime has one.
    #[must_use]
    pub fn client_id(self) -> Option<String> {
        match self {
            Self::Google => first_non_empty(&[
                env_value(GOOGLE_CLIENT_ID_ENV),
                env_value("GB_MANAGED_GOOGLE_CLIENT_ID"),
                env_value("GOOGLE_OAUTH_CLIENT_ID"),
                compile_time_value(option_env!("GB_OAUTH_GOOGLE_CLIENT_ID")),
            ]),
            Self::Microsoft => first_non_empty(&[
                env_value(MICROSOFT_CLIENT_ID_ENV),
                env_value("GB_MANAGED_MICROSOFT_CLIENT_ID"),
                env_value("MICROSOFT_OAUTH_CLIENT_ID"),
                compile_time_value(option_env!("GB_OAUTH_MICROSOFT_CLIENT_ID")),
            ]),
        }
    }

    /// Microsoft Entra tenant used by the managed public client. Google has no
    /// tenant concept; `common` keeps callers uniform.
    #[must_use]
    pub fn tenant_id(self) -> String {
        match self {
            Self::Microsoft => first_non_empty(&[
                env_value(MICROSOFT_TENANT_ID_ENV),
                env_value("GB_MANAGED_MICROSOFT_TENANT_ID"),
                env_value("MICROSOFT_OAUTH_TENANT_ID"),
                compile_time_value(option_env!("GB_OAUTH_MICROSOFT_TENANT_ID")),
            ])
            .unwrap_or_else(|| "common".to_string()),
            Self::Google => "common".to_string(),
        }
    }

    /// True when this deployment/build can offer the managed client.
    #[must_use]
    pub fn is_configured(self) -> bool {
        self.client_id().is_some()
    }
}

/// Booleans-only snapshot exposed to the wizard and Settings. Client ids are
/// public, but the UI only needs to know whether a managed app is available;
/// leaving the value out keeps configuration out of the page and telemetry.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagedOAuthStatus {
    #[serde(default)]
    pub google: bool,
    #[serde(default)]
    pub microsoft: bool,
}

impl ManagedOAuthStatus {
    /// Reads the current environment/build.
    #[must_use]
    pub fn current() -> Self {
        Self {
            google: ManagedProvider::Google.is_configured(),
            microsoft: ManagedProvider::Microsoft.is_configured(),
        }
    }

    /// True when a managed client is configured for `slug`.
    #[must_use]
    pub fn is_configured_for(&self, slug: &str) -> bool {
        match ManagedProvider::from_slug(slug) {
            Some(ManagedProvider::Google) => self.google,
            Some(ManagedProvider::Microsoft) => self.microsoft,
            None => false,
        }
    }
}

/// Convenience lookup used by the integration and mail flows.
#[must_use]
pub fn client_id_for(slug: &str) -> Option<String> {
    ManagedProvider::from_slug(slug)?.client_id()
}

fn env_value(key: &str) -> Option<String> {
    std::env::var(key).ok().and_then(non_empty)
}

fn compile_time_value(value: Option<&'static str>) -> Option<String> {
    value.and_then(non_empty)
}

fn non_empty(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn first_non_empty(values: &[Option<String>]) -> Option<String> {
    values.iter().find_map(Clone::clone)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_slugs_map_to_the_right_family() {
        assert_eq!(
            ManagedProvider::from_slug("gmail"),
            Some(ManagedProvider::Google)
        );
        assert_eq!(
            ManagedProvider::from_slug("drive"),
            Some(ManagedProvider::Google)
        );
        assert_eq!(
            ManagedProvider::from_slug("onedrive"),
            Some(ManagedProvider::Microsoft)
        );
        assert_eq!(
            ManagedProvider::from_slug("outlook_calendar"),
            Some(ManagedProvider::Microsoft)
        );
        assert_eq!(ManagedProvider::from_slug("hubspot"), None);
    }

    #[test]
    fn status_reports_only_configured_families() {
        let status = ManagedOAuthStatus {
            google: true,
            microsoft: false,
        };
        assert!(status.is_configured_for("gmail"));
        assert!(!status.is_configured_for("onedrive"));
        assert!(!status.is_configured_for("dropbox"));
    }
}
