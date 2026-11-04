//! CSP and Permissions-Policy for sandbox resources.
//!
//! Only the MCP Apps keys are accepted: `connectDomains` and
//! `resourceDomains`. There is no `frame` key in the specification, so frame
//! ancestors come from the deployment configuration instead.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiCsp {
    #[serde(default)]
    pub connect_domains: Vec<String>,
    #[serde(default)]
    pub resource_domains: Vec<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default)]
    pub domain: Option<String>,
}

impl UiCsp {
    /// Parses `_meta.ui.csp` (and the permissions list) from a resource's
    /// metadata. Unknown or unsafe sources are dropped, never trusted.
    #[must_use]
    pub fn from_meta(meta: &Value) -> Self {
        let ui = meta.get("ui").unwrap_or(meta);
        let csp = ui.get("csp").cloned().unwrap_or(Value::Null);
        Self {
            connect_domains: domains(csp.get("connectDomains")),
            resource_domains: domains(csp.get("resourceDomains")),
            permissions: permission_names(ui.get("permissions")),
            domain: ui
                .get("domain")
                .and_then(Value::as_str)
                .map(str::to_string),
        }
    }

    #[must_use]
    pub fn resource_csp(&self, app_origin: &str) -> String {
        let resources = sources(&self.resource_domains);
        let connects = sources(&self.connect_domains);
        let ancestors = if app_origin.is_empty() {
            "https: http://localhost:* http://127.0.0.1:*".to_string()
        } else {
            app_origin.to_string()
        };
        let connect_src = if connects.is_empty() {
            "connect-src 'none'".to_string()
        } else {
            format!("connect-src{connects}")
        };
        format!(
            "default-src 'none'; script-src 'unsafe-inline'{resources}; style-src 'unsafe-inline'{resources}; img-src data: blob:{resources}; font-src data:{resources}; {connect_src}; media-src data: blob:{resources}; base-uri 'none'; form-action 'none'; frame-ancestors {ancestors}"
        )
    }

    #[must_use]
    pub fn permissions_policy(&self) -> String {
        let allow = |feature: &str| {
            if self.permissions.iter().any(|value| value == feature) {
                format!("{feature}=(self)")
            } else {
                format!("{feature}=()")
            }
        };
        [
            allow("camera"),
            allow("microphone"),
            allow("geolocation"),
            allow("clipboard-read"),
            allow("clipboard-write"),
        ]
        .join(", ")
    }
}

fn strings(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .filter_map(safe_source)
                .collect()
        })
        .unwrap_or_default()
}

fn domains(value: Option<&Value>) -> Vec<String> {
    strings(value)
}

/// Permission names (`camera`, `microphone`, …) are not URLs: they must not go
/// through the CSP source validator, which would drop every one of them.
fn permission_names(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|name| {
                    !name.is_empty()
                        && name.len() <= 64
                        && name
                            .chars()
                            .all(|character| character.is_ascii_alphanumeric() || character == '-')
                })
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

/// Accepts explicit https/wss origins plus loopback http for development.
/// Wildcards, data: and javascript: are rejected.
fn safe_source(raw: &str) -> Option<String> {
    let value = raw.trim();
    if value.is_empty() || value.len() > 200 {
        return None;
    }
    if value == "self" {
        return Some("'self'".to_string());
    }
    let lower = value.to_ascii_lowercase();
    let allowed_scheme = lower.starts_with("https://")
        || lower.starts_with("wss://")
        || lower.starts_with("http://localhost")
        || lower.starts_with("http://127.0.0.1")
        || lower.starts_with("http://[::1]");
    if !allowed_scheme {
        return None;
    }
    if value.contains('*') || value.contains(' ') || value.contains(';') {
        return None;
    }
    Some(value.to_string())
}

fn sources(domains: &[String]) -> String {
    if domains.is_empty() {
        return String::new();
    }
    domains
        .iter()
        .map(|domain| format!(" {domain}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::UiCsp;
    use serde_json::json;

    #[test]
    fn parses_mcp_apps_keys_and_drops_unsafe_sources() {
        let meta = json!({
            "ui": {
                "csp": {
                    "connectDomains": ["https://api.example.com", "*", "javascript:alert(1)"],
                    "resourceDomains": ["https://cdn.example.com"]
                },
                "permissions": ["camera"]
            }
        });
        let csp = UiCsp::from_meta(&meta);
        assert_eq!(csp.connect_domains, vec!["https://api.example.com"]);
        assert_eq!(csp.resource_domains, vec!["https://cdn.example.com"]);
        let header = csp.resource_csp("https://app.example.com");
        assert!(header.contains("connect-src https://api.example.com"));
        assert!(!header.contains("javascript:"));
        assert!(csp.permissions_policy().contains("camera=(self)"));
        assert!(csp.permissions_policy().contains("microphone=()"));
    }
}
