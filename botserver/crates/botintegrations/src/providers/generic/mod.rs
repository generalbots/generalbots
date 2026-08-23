//! Declarative REST provider engine (#939 wave: token/key/basic providers).
//!
//! Each provider is a static [`ProviderSpec`]: an API origin, a credential
//! style and a table of [`ActionSpec`] entries (method, path template,
//! parameters, risk). The engine turns a Vault envelope plus validated
//! parameters into live calls through the shared `rest_client`, so a new
//! provider costs only data - no bespoke request plumbing.
//!
//! Security contract (same as the bespoke adapters):
//! - credentials load strictly from Vault immediately before invocation;
//! - every outcome passes through [`crate::providers::redact_credentials`];
//! - error strings are static sentinels; provider details stay in logs.

use std::future::Future;
use std::pin::Pin;

use serde_json::Value;

use crate::providers::rest_client::{self, RestRequest, MAX_RESPONSE_BYTES};
use crate::providers::{ActionOutcome, LlmSafeAction, LlmSafeParam, ProviderAdapter};

pub mod helpers;
pub mod developer;
pub mod devops;
pub mod observability;
pub mod startups;
pub mod productivity;
pub mod small_business;
pub mod finance;
pub mod social_messaging;
pub mod lifestyle;

use base64::Engine as _;

/// Maximum number of list items surfaced from one outcome; larger arrays
/// are cut and flagged so chat payloads stay bounded.
const MAX_LIST_ITEMS: usize = 25;

/// Risk classification mirroring the integration catalog profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Risk {
    Low,
    Medium,
    High,
}

/// Parameter types advertised on the chat surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamKind {
    Str,
    Integer,
    Json,
}

impl ParamKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Str => "string",
            Self::Integer => "integer",
            Self::Json => "json",
        }
    }
}

/// One declared action parameter (chat metadata and validation).
pub struct ParamSpec {
    pub name: &'static str,
    pub kind: ParamKind,
    pub required: bool,
}

/// Credential styles supported by the generic engine.
pub enum AuthStyle {
    /// `Authorization: Bearer <field>`.
    Bearer {
        token_field: &'static str,
    },
    /// `<header>: <field>` (for example `X-Api-Key`).
    ApiKeyHeader {
        header: &'static str,
        field: &'static str,
    },
    /// HTTP Basic with a templated username (`{email}/token`) and one
    /// password field. Template placeholders reference envelope fields.
    BasicTemplate {
        user_template: &'static str,
        password_field: &'static str,
    },
    /// HTTP Basic joining two fields with a separator; a missing second
    /// field renders as empty (Greenhouse `key:` style).
    BasicJoin {
        first_field: &'static str,
        separator: char,
        second_field: Option<&'static str>,
    },
    /// Credentials appended as query parameters (Trello `key`/`token`).
    QueryPairs {
        pairs: &'static [(&'static str, &'static str)],
    },
    /// Credential injected as a top-level JSON body field on every call;
    /// all declared parameters flatten into the same body (Canny style).
    BodyField {
        field: &'static str,
    },
    /// Multiple credential-backed headers in one style (Algolia application
    /// id + key).
    ApiKeyHeaders {
        pairs: &'static [(&'static str, &'static str)],
    },
}

/// Origin resolution for providers whose host depends on the credential.
pub enum Origin {
    Static(&'static str),
    /// Zendesk: `https://{subdomain}.api.zendesk.com/api/v2`.
    ZendeskSubdomain,
    /// Mailchimp: data center suffix of the API key selects the host.
    MailchimpDataCenter,
    /// Credential-supplied base URL (`{value}` placeholder), restricted to
    /// https so a hostile envelope cannot downgrade transport.
    FromField {
        field: &'static str,
        pattern: &'static str,
    },
}

/// One executable provider action.
pub struct ActionSpec {
    pub key: &'static str,
    pub method: &'static str,
    pub path: &'static str,
    pub summary: &'static str,
    /// Path placeholders substituted from params (URL-encoded).
    pub path_params: &'static [&'static str],
    /// Extra query parameters: `(request param name, params object key)`.
    pub query: &'static [(&'static str, &'static str)],
    /// Parameter carrying a JSON request body.
    pub body_param: Option<&'static str>,
    /// Optional top-level wrapper key for the JSON body (Zendesk `ticket`).
    pub body_wrapper: Option<&'static str>,
    pub risk: Risk,
    pub params: &'static [ParamSpec],
}

impl ActionSpec {
    fn is_read(&self) -> bool {
        self.method == "GET"
    }
}

/// Static description of one generic REST provider.
pub struct ProviderSpec {
    pub slug: &'static str,
    pub origin: Origin,
    pub auth: AuthStyle,
    /// Keys mirrored exactly by [`ProviderSpec::action_keys`] (trait needs a
    /// plain slice; the test asserts both stay in sync).
    pub actions: &'static [ActionSpec],
    pub action_keys: &'static [&'static str],
}

fn invalid(message: String) -> String {
    rest_client::invalid(message)
}

/// Reads a non-empty trimmed string field from the credential envelope.
pub(crate) fn cred_str<'a>(
    credentials: &'a Value,
    field: &str,
) -> Result<&'a str, String> {
    credentials
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .ok_or_else(|| invalid(format!("credential key {field} is missing or empty")))
}

fn basic_header(user: &str, password: &str) -> String {
    let raw = format!("{user}:{password}");
    format!("Basic {}", base64::engine::general_purpose::STANDARD.encode(raw))
}

/// Resolves the base URL for the provider, applying credential-derived
/// origins where the official API requires them.
fn resolve_origin(spec: &ProviderSpec, credentials: &Value) -> Result<String, String> {
    match spec.origin {
        Origin::Static(origin) => Ok(origin.to_string()),
        Origin::ZendeskSubdomain => {
            let subdomain = cred_str(credentials, "subdomain")?;
            if !subdomain
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-')
            {
                return Err(invalid(
                    "credential key subdomain contains invalid characters".to_string(),
                ));
            }
            Ok(format!(
                "https://{subdomain}.api.zendesk.com/api/v2"
            ))
        }
        Origin::MailchimpDataCenter => {
            let key = cred_str(credentials, "api_key")?;
            let dc = key.rsplit('-').next().unwrap_or_default();
            if dc.len() > 16 || !dc.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
                return Err(invalid(
                    "credential key api_key has no usable data center suffix".to_string(),
                ));
            }
            Ok(format!("https://{dc}.api.mailchimp.com/3.0"))
        }
        Origin::FromField { field, pattern } => {
            let value = cred_str(credentials, field)?;
            let origin = pattern.replace("{value}", value);
            if !origin.starts_with("https://") {
                return Err(invalid(
                    "credential key base_url must be an https URL".to_string(),
                ));
            }
            Ok(origin)
        }
    }
}

fn auth_headers_and_query(
    spec: &ProviderSpec,
    credentials: &Value,
) -> Result<(Vec<(&'static str, String)>, Vec<(String, String)>), String> {
    let mut headers = Vec::new();
    let mut query = Vec::new();
    match &spec.auth {
        AuthStyle::Bearer { token_field } => {
            let token = cred_str(credentials, token_field)?;
            headers.push(("authorization", format!("Bearer {token}")));
        }
        AuthStyle::ApiKeyHeader { header, field } => {
            let key = cred_str(credentials, field)?;
            headers.push((*header, key.to_string()));
        }
        AuthStyle::BasicTemplate {
            user_template,
            password_field,
        } => {
            let password = cred_str(credentials, password_field)?;
            let mut user = user_template.to_string();
            while let Some(start) = user.find('{') {
                let Some(end) = user[start..].find('}') else {
                    return Err(invalid("malformed credential template".to_string()));
                };
                let field = &user[start + 1..start + end];
                let value = cred_str(credentials, field)?;
                user.replace_range(start..start + end + 1, value);
            }
            headers.push(("authorization", basic_header(&user, password)));
        }
        AuthStyle::BasicJoin {
            first_field,
            separator,
            second_field,
        } => {
            let first = cred_str(credentials, first_field)?;
            let second = second_field
                .map(|field| cred_str(credentials, field))
                .transpose()?
                .unwrap_or_default();
            if second.contains(*separator) {
                return Err(invalid("credential field contains the separator".to_string()));
            }
            headers.push((
                "authorization",
                basic_header(first, &second),
            ));
        }
        AuthStyle::QueryPairs { pairs } => {
            for (name, field) in *pairs {
                let value = cred_str(credentials, field)?;
                query.push(((*name).to_string(), value.to_string()));
            }
        }
        AuthStyle::BodyField { .. } => {
            // Credential material is injected into the JSON body by
            // `flat_body` at invocation time; nothing enters headers.
        }
        AuthStyle::ApiKeyHeaders { pairs } => {
            for (header, field) in *pairs {
                let value = cred_str(credentials, field)?;
                headers.push((*header, value.to_string()));
            }
        }
    }
    headers.push(("user-agent", "generalbots-botintegrations".to_string()));
    Ok((headers, query))
}

fn require_param_str(params: &Value, name: &str) -> Result<String, String> {
    let text = match params.get(name) {
        Some(Value::String(text)) => text.trim().to_string(),
        Some(Value::Number(number)) => number.to_string(),
        _ => {
            return Err(invalid(format!("parameter {name} is missing")));
        }
    };
    if text.is_empty() {
        return Err(invalid(format!("parameter {name} is empty")));
    }
    Ok(text)
}

fn build_body(action: &ActionSpec, params: &Value) -> Result<Option<Vec<u8>>, String> {
    let Some(body_param) = action.body_param else {
        return Ok(None);
    };
    let payload = match params.get(body_param).and_then(Value::as_object) {
        Some(map) => Value::Object(map.clone()),
        _ => {
            return Err(invalid(format!(
                "parameter {body_param} must be a JSON object"
            )))
        }
    };
    let payload = match action.body_wrapper {
        Some(wrapper) => {
            let mut outer = serde_json::Map::new();
            outer.insert(wrapper.to_string(), payload);
            Value::Object(outer)
        }
        None => payload,
    };
    Ok(Some(payload.to_string().into_bytes()))
}

/// Builds the flat credential-signed body used by `AuthStyle::BodyField`
/// providers: every declared parameter present in `params` becomes a
/// top-level key alongside the credential field.
fn flat_body(
    credentials: &Value,
    field: &str,
    action: &ActionSpec,
    params: &Value,
) -> Result<Vec<u8>, String> {
    let mut body = serde_json::Map::new();
    body.insert(
        field.to_string(),
        Value::String(cred_str(credentials, field)?.to_string()),
    );
    for param in action.params {
        if let Some(value) = params.get(param.name) {
            match value {
                Value::Null => {}
                Value::String(text) if text.trim().is_empty() => {}
                other => {
                    body.insert((*param.name).to_string(), other.clone());
                }
            }
        }
    }
    Ok(Value::Object(body).to_string().into_bytes())
}

fn shape_outcome(action: &ActionSpec, status: u16, body: &[u8]) -> ActionOutcome {
    let parsed: Option<Value> = serde_json::from_slice(body).ok();
    let (data, truncated) = match parsed {
        Some(Value::Array(items)) if items.len() > MAX_LIST_ITEMS => (
            Value::Array(items[..MAX_LIST_ITEMS].to_vec()),
            true,
        ),
        Some(Value::Object(mut map)) => {
            let mut truncated = false;
            for (_key, value) in map.iter_mut() {
                if let Value::Array(items) = value {
                    if items.len() > MAX_LIST_ITEMS {
                        *value = Value::Array(items[..MAX_LIST_ITEMS].to_vec());
                        truncated = true;
                    }
                }
            }
            (Value::Object(map), truncated)
        }
        Some(other) => (other, false),
        None => (Value::Null, false),
    };
    let count_suffix = match (&data, action.is_read()) {
        (Value::Array(items), true) => format!(" {} items", items.len()),
        _ => String::new(),
    };
    ActionOutcome {
        summary: format!("{}{} (status {status})", action.summary, count_suffix),
        data,
        truncated,
    }
}

/// Adapter executing one static provider specification.
pub struct GenericAdapter {
    spec: &'static ProviderSpec,
}

impl GenericAdapter {
    pub const fn new(spec: &'static ProviderSpec) -> Self {
        Self { spec }
    }

    fn find_action(&self, action: &str) -> Option<&'static ActionSpec> {
        self.spec.actions.iter().find(|entry| entry.key == action)
    }
}

impl ProviderAdapter for GenericAdapter {
    fn provider(&self) -> &'static str {
        self.spec.slug
    }

    fn implemented_actions(&self) -> &'static [&'static str] {
        self.spec.action_keys
    }

    fn safe_action_catalog(&self) -> Vec<LlmSafeAction> {
        self.spec
            .actions
            .iter()
            .map(|action| LlmSafeAction {
                name: action.key.to_string(),
                summary: action.summary.to_string(),
                params: action
                    .params
                    .iter()
                    .map(|param| LlmSafeParam {
                        name: param.name.to_string(),
                        kind: param.kind.as_str().to_string(),
                        required: param.required,
                    })
                    .collect(),
                risk: match action.risk {
                    Risk::Low => "low",
                    Risk::Medium => "medium",
                    Risk::High => "high",
                }
                .to_string(),
                requires_approval: !action.is_read(),
            })
            .collect()
    }

    fn invoke<'a>(
        &'a self,
        action: &'a str,
        credentials: &'a Value,
        params: &'a Value,
    ) -> Pin<Box<dyn Future<Output = Result<ActionOutcome, String>> + Send + 'a>> {
        Box::pin(async move {
            let Some(spec) = self.find_action(action) else {
                return Err(crate::providers::ERR_ACTION_NOT_AVAILABLE.to_string());
            };
            let origin = resolve_origin(self.spec, credentials)?;
            let (mut headers, auth_query) = auth_headers_and_query(self.spec, credentials)?;

            let mut query = auth_query;
            for (name, field) in spec.query {
                if let Some(value) = params.get(*field) {
                    if let Some(text) = value.as_str() {
                        if !text.trim().is_empty() {
                            query.push(((*name).to_string(), text.trim().to_string()));
                        }
                    }
                }
            }
            let url = build_url_from_parts(&origin, spec, params, query)?;
            let credential_body = match &self.spec.auth {
                AuthStyle::BodyField { field } => {
                    Some(flat_body(credentials, field, spec, params)?)
                }
                _ => None,
            };
            if credential_body.is_some() || (spec.method != "GET" && build_body(spec, params)?.is_some()) {
                let body = match credential_body {
                    Some(body) => body,
                    None => build_body(spec, params)?.unwrap_or_default(),
                };
                headers.push(("content-type", "application/json".to_string()));
                let response = rest_client::send(RestRequest {
                    method: reqwest::Method::from_bytes(spec.method.as_bytes())
                        .map_err(|_| invalid("unsupported method".to_string()))?,
                    url,
                    headers,
                    body: Some(body),
                    response_cap: MAX_RESPONSE_BYTES,
                })
                .await?;
                response.require_success(spec.summary)?;
                return Ok(shape_outcome(spec, response.status, &response.body));
            }
            let response = rest_client::send(RestRequest {
                method: reqwest::Method::from_bytes(spec.method.as_bytes())
                    .map_err(|_| invalid("unsupported method".to_string()))?,
                url,
                headers,
                body: None,
                response_cap: MAX_RESPONSE_BYTES,
            })
            .await?;
            response.require_success(spec.summary)?;
            Ok(shape_outcome(spec, response.status, &response.body))
        })
    }
}

fn build_url_from_parts(
    origin: &str,
    action: &ActionSpec,
    params: &Value,
    query: Vec<(String, String)>,
) -> Result<String, String> {
    let mut path = action.path.to_string();
    for placeholder in action.path_params {
        let value = require_param_str(params, placeholder)?;
        let encoded = urlencoding::encode(&value).into_owned();
        path = path.replace(&format!("{{{placeholder}}}"), &encoded);
    }
    let mut url = format!("{origin}{path}");
    if !query.is_empty() {
        let rendered: Vec<String> = query
            .into_iter()
            .map(|(name, value)| {
                format!(
                    "{}={}",
                    urlencoding::encode(&name),
                    urlencoding::encode(&value)
                )
            })
            .collect();
        let glue = if url.contains('?') { "&" } else { "?" };
        url = format!("{url}{glue}{}", rendered.join("&"));
    }
    Ok(url)
}

#[cfg(test)]
mod tests;
