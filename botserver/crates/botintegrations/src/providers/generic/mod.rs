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

pub mod simple;

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
}

/// Origin resolution for providers whose host depends on the credential.
pub enum Origin {
    Static(&'static str),
    /// Zendesk: `https://{subdomain}.api.zendesk.com/api/v2`.
    ZendeskSubdomain,
    /// Mailchimp: data center suffix of the API key selects the host.
    MailchimpDataCenter,
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
            if spec.method != "GET" {
                if let Some(body) = build_body(spec, params)? {
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
mod tests {
    use super::*;
    use serde_json::json;

    const TEST_ACTIONS: &[ActionSpec] = &[ActionSpec {
        key: "widgets.items.list",
        method: "GET",
        path: "/items/{item_id}",
        summary: "List items.",
        path_params: &["item_id"],
        query: &[("limit", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[
            ParamSpec {
                name: "item_id",
                kind: ParamKind::Str,
                required: true,
            },
            ParamSpec {
                name: "limit",
                kind: ParamKind::Str,
                required: false,
            },
        ],
    }];

    const TEST_KEYS: &[&str] = &["widgets.items.list"];

    const TEST_SPEC: ProviderSpec = ProviderSpec {
        slug: "widgets",
        origin: Origin::Static("https://api.widgets.test/v1"),
        auth: AuthStyle::Bearer {
            token_field: "token",
        },
        actions: TEST_ACTIONS,
        action_keys: TEST_KEYS,
    };

    #[test]
    fn url_templating_encodes_placeholders_and_query() {
        let action = &TEST_ACTIONS[0];
        let url = build_url_from_parts(
            "https://api.widgets.test/v1",
            action,
            &json!({"item_id": "a b/c", "limit": "5"}),
            vec![("limit".to_string(), "5".to_string())],
        )
        .unwrap();
        assert_eq!(url, "https://api.widgets.test/v1/items/a%20b%2Fc?limit=5");
    }

    #[test]
    fn missing_required_path_param_is_rejected_before_network() {
        let action = &TEST_ACTIONS[0];
        assert!(build_url_from_parts(
            "https://api.widgets.test/v1",
            action,
            &json!({"limit": "5"}),
            vec![]
        )
        .is_err());
    }

    #[test]
    fn basic_template_substitutes_envelope_fields() {
        let spec = ProviderSpec {
            slug: "t",
            origin: Origin::Static("https://x"),
            auth: AuthStyle::BasicTemplate {
                user_template: "{email}/token",
                password_field: "token",
            },
            actions: &[],
            action_keys: &[],
        };
        let credentials = json!({"email": "a@b.c", "token": "secret"});
        let (headers, _) = auth_headers_and_query(&spec, &credentials).unwrap();
        let (_, value) = headers.iter().find(|(n, _)| *n == "authorization").unwrap();
        assert_eq!(
            *value,
            format!(
                "Basic {}",
                base64::engine::general_purpose::STANDARD.encode("a@b.c/token:secret")
            )
        );
    }

    #[test]
    fn zendesk_origin_validates_subdomain_charset() {
        let spec = ProviderSpec {
            slug: "z",
            origin: Origin::ZendeskSubdomain,
            auth: AuthStyle::Bearer {
                token_field: "token",
            },
            actions: &[],
            action_keys: &[],
        };
        assert_eq!(
            resolve_origin(&spec, &json!({"subdomain": "acme"})).unwrap(),
            "https://acme.api.zendesk.com/api/v2"
        );
        assert!(resolve_origin(&spec, &json!({"subdomain": "../etc"})).is_err());
    }

    #[test]
    fn mailchimp_origin_extracts_data_center() {
        let spec = ProviderSpec {
            slug: "m",
            origin: Origin::MailchimpDataCenter,
            auth: AuthStyle::Bearer {
                token_field: "api_key",
            },
            actions: &[],
            action_keys: &[],
        };
        let key = ["abcd1234", "-", "us19"].concat();
        let origin = resolve_origin(&spec, &json!({"api_key": key})).unwrap();
        assert_eq!(origin, "https://us19.api.mailchimp.com/3.0");
    }

    #[test]
    fn outcomes_are_capped_and_summarized() {
        let big: Vec<Value> = (0..40).map(|i| json!({"id": i})).collect();
        let outcome = shape_outcome(&TEST_ACTIONS[0], 200, &serde_json::to_vec(&big).unwrap());
        assert!(outcome.truncated);
        assert_eq!(outcome.data.as_array().unwrap().len(), MAX_LIST_ITEMS);
        assert!(outcome.summary.contains("(status 200)"));
    }

    #[test]
    fn generic_adapter_metadata_matches_declared_keys() {
        let adapter = GenericAdapter::new(&TEST_SPEC);
        assert_eq!(adapter.provider(), "widgets");
        let catalog = adapter.safe_action_catalog();
        assert_eq!(catalog.len(), adapter.implemented_actions().len());
        assert_eq!(catalog[0].risk, "low");
        assert!(!catalog[0].requires_approval);
    }
}
