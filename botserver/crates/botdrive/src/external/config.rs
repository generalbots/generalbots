//! OAuth client configuration for the external-drive flows.
//!
//! A deployment can register its own app (confidential or public) through the
//! provider-specific variables; those always win:
//!
//! * `EXTERNAL_DRIVE_ONEDRIVE_CLIENT_ID` / `..._CLIENT_SECRET`, plus
//!   `EXTERNAL_DRIVE_ONEDRIVE_TENANT_ID` (defaults to `common`, i.e. any
//!   consumer tenant — required for multi-tenant installs).
//! * `EXTERNAL_DRIVE_GOOGLE_CLIENT_ID` / `..._CLIENT_SECRET`.
//!
//! When no deployment app is registered, the desktop build falls back to the
//! managed public clients from [`botlib::managed_oauth`]
//! (`GB_OAUTH_GOOGLE_CLIENT_ID` / `GB_OAUTH_MICROSOFT_CLIENT_ID`). Those are
//! installed-app registrations: no client secret is stored or sent, and the
//! flow uses PKCE with the loopback callback.
//!
//! When neither is present the provider is reported as *not configured*
//! rather than failing at connect time: the UI shows the button as unavailable
//! and every other feature keeps working.

use crate::external::types::Provider;
use botlib::managed_oauth::ManagedProvider;

/// Path both providers redirect back to. It must be registered verbatim with
/// each provider, because OAuth compares it byte for byte.
pub const CALLBACK_PATH: &str = "/api/external-drives/callback";

/// The OAuth application identity for one provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderClient {
    /// OAuth application (client) id.
    pub client_id: String,
    /// OAuth application secret. `None` for a managed/public client, which
    /// uses PKCE instead of confidential-client authentication.
    pub client_secret: Option<String>,
    /// Microsoft Entra tenant. Unused for Google; `common` there.
    pub tenant_id: String,
    /// True when the id came from the shipped managed registration (not from
    /// an operator-provided `EXTERNAL_DRIVE_*` variable).
    pub managed: bool,
}

impl ProviderClient {
    /// Public clients authenticate the code exchange with PKCE, not a secret.
    #[must_use]
    pub fn uses_pkce(&self) -> bool {
        self.client_secret.is_none()
    }
}

/// Resolve the configured client for a provider, or `None` when neither the
/// deployment nor the build has registered one.
pub fn client_for(provider: Provider) -> Option<ProviderClient> {
    let (id_key, secret_key) = match provider {
        Provider::OneDrive => (
            "EXTERNAL_DRIVE_ONEDRIVE_CLIENT_ID",
            "EXTERNAL_DRIVE_ONEDRIVE_CLIENT_SECRET",
        ),
        Provider::GoogleDrive => (
            "EXTERNAL_DRIVE_GOOGLE_CLIENT_ID",
            "EXTERNAL_DRIVE_GOOGLE_CLIENT_SECRET",
        ),
    };

    if let Some(client_id) = env_non_empty(id_key) {
        return Some(ProviderClient {
            client_id,
            client_secret: env_non_empty(secret_key),
            tenant_id: match provider {
                Provider::OneDrive => env_non_empty("EXTERNAL_DRIVE_ONEDRIVE_TENANT_ID")
                    .unwrap_or_else(|| "common".to_string()),
                Provider::GoogleDrive => "common".to_string(),
            },
            managed: false,
        });
    }

    let managed_provider = match provider {
        Provider::OneDrive => ManagedProvider::Microsoft,
        Provider::GoogleDrive => ManagedProvider::Google,
    };
    let client_id = managed_provider.client_id()?;
    Some(ProviderClient {
        client_id,
        client_secret: None,
        tenant_id: managed_provider.tenant_id(),
        managed: true,
    })
}

/// Absolute redirect URI both providers redirect back to.
///
/// `EXTERNAL_DRIVE_REDIRECT_BASE` wins so a deployment behind a proxy can
/// pin one value; otherwise `PUBLIC_BASE_URL` is used; otherwise the base is
/// derived from the request the connect call arrived on.
pub fn redirect_uri(base: Option<&str>) -> String {
    let base = base
        .map(str::trim)
        .filter(|b| !b.is_empty())
        .map(str::to_string)
        .or_else(|| env_non_empty("PUBLIC_BASE_URL"))
        .unwrap_or_default();
    format!("{}{CALLBACK_PATH}", base.trim_end_matches('/'))
}

/// The public origin (`scheme://host`) of the request that started the flow.
/// Falls back to the plain-text form so the caller can answer with a clear
/// configuration error rather than a broken redirect.
pub fn base_from_headers(
    headers: &axum::http::HeaderMap,
    forwarded_host: Option<&str>,
    forwarded_proto: Option<&str>,
) -> Option<String> {
    let host = forwarded_host
        .map(str::to_string)
        .or_else(|| header_str(headers, "host"))?;
    let scheme = forwarded_proto
        .map(str::to_string)
        .or_else(|| header_str(headers, "x-forwarded-proto"))
        .unwrap_or_else(|| "https".to_string());
    Some(format!("{}://{}", scheme, host))
}

fn header_str(headers: &axum::http::HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn env_non_empty(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redirect_uri_joins_base_and_callback_without_double_slash() {
        assert_eq!(
            redirect_uri(Some("https://drive.example.org/")),
            "https://drive.example.org/api/external-drives/callback"
        );
        assert_eq!(
            redirect_uri(Some("https://drive.example.org")),
            "https://drive.example.org/api/external-drives/callback"
        );
    }

    #[test]
    fn missing_base_yields_a_relative_callback() {
        // No EXTERNAL_DRIVE_REDIRECT_BASE in the test environment: the URI is
        // still well-formed, it just lacks an origin.
        assert!(redirect_uri(None).ends_with(CALLBACK_PATH));
    }

    #[test]
    fn client_is_absent_when_unconfigured() {
        // The test environment registers no OAuth apps, so every provider must
        // report "not configured" rather than a half-filled client.
        assert!(client_for(Provider::OneDrive).is_none());
        assert!(client_for(Provider::GoogleDrive).is_none());
    }
}
