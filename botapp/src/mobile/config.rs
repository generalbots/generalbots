//! Remote origin resolver for the mobile thin client.
//!
//! The origin is the only server the WebView may talk to. It is supplied at
//! build time (Android/iOS have no useful process environment) or, for a
//! desktop thin-client dev run, at run time. Missing or invalid configuration
//! is a hard error — never silently fall back to a public server.

use tauri::Url;

/// Injected by CI with `GB_MOBILE_SERVER_URL=<origin>` at build time.
pub const BUILD_ORIGIN: Option<&str> = option_env!("GB_MOBILE_SERVER_URL");
const RUNTIME_ORIGIN_VAR: &str = "GB_MOBILE_SERVER_URL";
/// Optional session token carried in the suite URL, exactly like the desktop
/// handoff (`suite_url_with_token`); never minted or guessed here. It is read
/// only at run time so no credential is ever baked into a distributed binary.
const RUNTIME_TOKEN_VAR: &str = "GB_MOBILE_SESSION_TOKEN";
const DEBUG_ORIGIN: &str = "http://127.0.0.1:8080";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    MissingOrigin,
    InvalidUrl,
    EmptyHost,
    UserInfoNotAllowed,
    FragmentNotAllowed,
    InsecureScheme,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::MissingOrigin => "GB_MOBILE_SERVER_URL is not configured",
            Self::InvalidUrl => "remote server URL is not a valid URL",
            Self::EmptyHost => "remote server URL has no host",
            Self::UserInfoNotAllowed => "remote server URL must not contain user information",
            Self::FragmentNotAllowed => "remote server URL must not contain a fragment",
            Self::InsecureScheme => "remote server URL must use https",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for ConfigError {}

/// Build-time value first, then the runtime environment, then nothing.
fn configured_value(build: Option<&'static str>, runtime_var: &str) -> Option<String> {
    if let Some(value) = build {
        let value = value.trim();
        if !value.is_empty() {
            return Some(value.to_string());
        }
    }
    std::env::var(runtime_var)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

/// Resolves the configured origin. An absent or invalid value is an error.
pub fn remote_origin() -> Result<String, ConfigError> {
    let raw = configured_value(BUILD_ORIGIN, RUNTIME_ORIGIN_VAR).ok_or(ConfigError::MissingOrigin)?;
    validate_origin(&raw)
}

/// Optional session token; `None` means the suite will show its sign-in page.
pub fn session_token() -> Option<String> {
    configured_value(None, RUNTIME_TOKEN_VAR)
}

/// Normalises a configured URL to `scheme://host[:port]` and enforces the
/// mobile trust policy: https only, no userinfo, no fragment, non-empty host.
/// The loopback botserver is accepted only in debug builds.
pub fn validate_origin(raw: &str) -> Result<String, ConfigError> {
    let url = Url::parse(raw.trim()).map_err(|_| ConfigError::InvalidUrl)?;
    if url.fragment().is_some() {
        return Err(ConfigError::FragmentNotAllowed);
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(ConfigError::UserInfoNotAllowed);
    }
    let host = url.host_str().unwrap_or_default();
    if host.is_empty() {
        return Err(ConfigError::EmptyHost);
    }
    let scheme = url.scheme();
    let mut origin = format!("{scheme}://{host}");
    if let Some(port) = url.port() {
        origin.push(':');
        origin.push_str(&port.to_string());
    }
    if scheme != "https" && !(cfg!(debug_assertions) && origin == DEBUG_ORIGIN) {
        return Err(ConfigError::InsecureScheme);
    }
    Ok(origin)
}

/// The suite entry point, with the optional token appended as `?token=`.
pub fn suite_url(origin: &str, token: Option<&str>) -> Result<Url, ConfigError> {
    let mut url = Url::parse(&format!("{origin}/suite/desktop.html"))
        .map_err(|_| ConfigError::InvalidUrl)?;
    if let Some(token) = token {
        if !token.is_empty() {
            url.query_pairs_mut().append_pair("token", token);
        }
    }
    Ok(url)
}

/// Script installed by Tauri before any page script runs. `serde_json` quotes
/// and escapes the value, so no hostname is ever written into JavaScript.
/// The suite boot-flags file is byte-identical to desktop: all three globals
/// are installed here, where no boot-budget JS-byte ratchet applies.
pub fn initialization_script(origin: &str) -> String {
    let serialized = match serde_json::to_string(origin) {
        Ok(value) => value,
        Err(_) => "\"\"".to_string(),
    };
    format!(
        "window.__GB_MOBILE_SERVER_ORIGIN__ = {serialized};\
         window.__GB_SERVER_ORIGIN__ = {serialized};\
         window.GB_LOGIN_URL = {serialized} + '/suite/login.html';"
    )
}

#[cfg(test)]
mod tests {
    use super::{initialization_script, suite_url, validate_origin, ConfigError, DEBUG_ORIGIN};

    #[test]
    fn accepts_and_normalises_https_origins() {
        assert_eq!(
            validate_origin("https://botserver.example.com"),
            Ok("https://botserver.example.com".to_string())
        );
        assert_eq!(
            validate_origin("https://botserver.example.com:8443/some/path?q=1"),
            Ok("https://botserver.example.com:8443".to_string())
        );
    }

    #[test]
    fn rejects_http_except_the_debug_loopback() {
        assert_eq!(
            validate_origin("http://botserver.example.com"),
            Err(ConfigError::InsecureScheme)
        );
        assert_eq!(
            validate_origin("http://127.0.0.1:3000"),
            Err(ConfigError::InsecureScheme)
        );
        let expected = if cfg!(debug_assertions) {
            Ok(DEBUG_ORIGIN.to_string())
        } else {
            Err(ConfigError::InsecureScheme)
        };
        assert_eq!(validate_origin(DEBUG_ORIGIN), expected);
    }

    #[test]
    fn rejects_userinfo_and_fragments() {
        assert_eq!(
            validate_origin("https://user:pass@botserver.example.com"),
            Err(ConfigError::UserInfoNotAllowed)
        );
        assert_eq!(
            validate_origin("https://user@botserver.example.com"),
            Err(ConfigError::UserInfoNotAllowed)
        );
        assert_eq!(
            validate_origin("https://botserver.example.com/#section"),
            Err(ConfigError::FragmentNotAllowed)
        );
    }

    #[test]
    fn rejects_missing_or_unparseable_hosts() {
        assert_eq!(validate_origin(""), Err(ConfigError::InvalidUrl));
        assert_eq!(validate_origin("not a url"), Err(ConfigError::InvalidUrl));
        assert_eq!(validate_origin("https://"), Err(ConfigError::InvalidUrl));
        assert_eq!(validate_origin("ftp://botserver.example.com"), Err(ConfigError::InsecureScheme));
    }

    #[test]
    fn suite_url_appends_path_and_optional_token() {
        let without = suite_url("https://botserver.example.com", None);
        assert_eq!(
            without.map(|url| url.to_string()),
            Ok("https://botserver.example.com/suite/desktop.html".to_string())
        );
        let with = suite_url("https://botserver.example.com", Some("session-token"));
        assert_eq!(
            with.map(|url| url.to_string()),
            Ok("https://botserver.example.com/suite/desktop.html?token=session-token".to_string())
        );
    }

    #[test]
    fn init_script_installs_all_three_globals() {
        let script = initialization_script("https://botserver.example.com");
        assert_eq!(
            script,
            "window.__GB_MOBILE_SERVER_ORIGIN__ = \"https://botserver.example.com\";\
             window.__GB_SERVER_ORIGIN__ = \"https://botserver.example.com\";\
             window.GB_LOGIN_URL = \"https://botserver.example.com\" + '/suite/login.html';"
        );
    }
}
