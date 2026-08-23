use super::types::{AuthField, AuthMethod, AuthProfile, InputType};

const fn field(
    key: &'static str,
    label: &'static str,
    input_type: InputType,
    secret: bool,
    required: bool,
    placeholder: &'static str,
) -> AuthField {
    AuthField {
        key,
        label,
        input_type,
        secret,
        required,
        placeholder,
    }
}

const OAUTH_FIELDS: &[AuthField] = &[
    field(
        "client_id",
        "Client ID",
        InputType::Text,
        false,
        false,
        "Managed OAuth app or client ID",
    ),
    field(
        "client_secret",
        "Client Secret",
        InputType::Password,
        true,
        false,
        "Stored in the secrets vault",
    ),
];

const API_KEY_FIELDS: &[AuthField] = &[field(
    "api_key",
    "API Key",
    InputType::Password,
    true,
    true,
    "Stored in the secrets vault",
)];

const TOKEN_FIELDS: &[AuthField] = &[field(
    "token",
    "Access Token",
    InputType::Password,
    true,
    true,
    "Stored in the secrets vault",
)];

const ZENDESK_TOKEN_FIELDS: &[AuthField] = &[
    field(
        "subdomain",
        "Zendesk Subdomain",
        InputType::Text,
        false,
        true,
        "acme (from acme.zendesk.com)",
    ),
    field(
        "email",
        "Account Email",
        InputType::Text,
        false,
        true,
        "agent@company.com",
    ),
    field(
        "token",
        "API Token",
        InputType::Password,
        true,
        true,
        "Stored in the secrets vault",
    ),
];

const TRELLO_KEY_TOKEN_FIELDS: &[AuthField] = &[
    field(
        "key",
        "API Key",
        InputType::Password,
        true,
        true,
        "From trello.com/power-ups/admin",
    ),
    field(
        "token",
        "Member Token",
        InputType::Password,
        true,
        true,
        "Generated next to the API key",
    ),
];

const BASIC_FIELDS: &[AuthField] = &[
    field(
        "username",
        "Username",
        InputType::Text,
        false,
        true,
        "Account or API username",
    ),
    field(
        "password",
        "Password or API Secret",
        InputType::Password,
        true,
        true,
        "Stored in the secrets vault",
    ),
];

const IMAP_FIELDS: &[AuthField] = &[
    field(
        "host",
        "IMAP Host",
        InputType::Text,
        false,
        true,
        "imap.example.com",
    ),
    field("port", "IMAP Port", InputType::Number, false, true, "993"),
    field(
        "username",
        "Mailbox Username",
        InputType::Text,
        false,
        true,
        "user@example.com",
    ),
    field(
        "password",
        "Password, App Password, or Token",
        InputType::Password,
        true,
        true,
        "Stored in the secrets vault",
    ),
];

const AWS_FIELDS: &[AuthField] = &[
    field(
        "access_key_id",
        "Access Key ID",
        InputType::Password,
        true,
        true,
        "Stored in the secrets vault",
    ),
    field(
        "secret_access_key",
        "Secret Access Key",
        InputType::Password,
        true,
        true,
        "Stored in the secrets vault",
    ),
    field(
        "session_token",
        "Session Token",
        InputType::Password,
        true,
        false,
        "Optional temporary STS token",
    ),
    field(
        "region",
        "Region",
        InputType::Text,
        false,
        false,
        "us-east-1",
    ),
];

pub(crate) static OAUTH2: AuthProfile = AuthProfile {
    method: AuthMethod::OAuth2,
    fields: OAUTH_FIELDS,
    instructions: "Authorize through the provider OAuth 2.0 consent flow. OAuth credentials are stored only in the secrets vault.",
    least_privilege: "Request only scopes required by enabled actions and prefer short-lived, refreshable grants.",
};

pub(crate) static GOOGLE_OAUTH2: AuthProfile = AuthProfile {
    method: AuthMethod::OAuth2,
    fields: OAUTH_FIELDS,
    instructions: "Use Google OAuth 2.0 and grant access through the Google consent screen. Tokens remain outside LLM context.",
    least_privilege: "Select the narrowest Google API scopes needed by enabled actions; avoid broad Workspace scopes.",
};

pub(crate) static MICROSOFT_OAUTH2: AuthProfile = AuthProfile {
    method: AuthMethod::OAuth2,
    fields: OAUTH_FIELDS,
    instructions: "Use Microsoft identity platform OAuth 2.0 for Microsoft Graph. Tokens remain in the secrets vault.",
    least_privilege: "Prefer delegated permissions and the narrowest Graph scopes required by enabled actions.",
};

pub(crate) static API_KEY: AuthProfile = AuthProfile {
    method: AuthMethod::ApiKey,
    fields: API_KEY_FIELDS,
    instructions: "Create a provider API key and store it in the secrets vault. The key is never included in LLM context.",
    least_privilege: "Use a dedicated restricted key and disable unused products, resources, and write permissions.",
};

pub(crate) static TOKEN: AuthProfile = AuthProfile {
    method: AuthMethod::Token,
    fields: TOKEN_FIELDS,
    instructions: "Create a dedicated access token and store it in the secrets vault. Tokens are never exposed to the LLM.",
    least_privilege: "Grant only scopes needed by enabled actions and use an expiry when the provider supports one.",
};

pub(crate) static ZENDESK_TOKEN: AuthProfile = AuthProfile {
    method: AuthMethod::Token,
    fields: ZENDESK_TOKEN_FIELDS,
    instructions: "In Zendesk Admin Center create an API token, then provide your tenant subdomain, account email and the token. All three are required for Basic {email}/token authentication over TLS.",
    least_privilege: "Use a dedicated agent account whose role covers only ticket operations enabled here.",
};

pub(crate) static TRELLO_KEY_TOKEN: AuthProfile = AuthProfile {
    method: AuthMethod::ApiKey,
    fields: TRELLO_KEY_TOKEN_FIELDS,
    instructions: "Create a Power-Up (or use your developer API key) on trello.com and generate a member token for it. Both the key and the token are stored in the secrets vault.",
    least_privilege: "Scope the token to read or write only the boards used by the enabled actions.",
};

pub(crate) static BASIC: AuthProfile = AuthProfile {
    method: AuthMethod::Basic,
    fields: BASIC_FIELDS,
    instructions: "Use dedicated API credentials over TLS. Secrets are stored in the vault and excluded from LLM context.",
    least_privilege: "Use a non-human integration account with only the permissions required by enabled actions.",
};

pub(crate) static IMAP: AuthProfile = AuthProfile {
    method: AuthMethod::Protocol,
    fields: IMAP_FIELDS,
    instructions: "Connect with IMAP over TLS. Prefer OAuth or an app password when the mail provider supports it.",
    least_privilege: "Use a dedicated mailbox or read-only mailbox grant unless message mutation is explicitly enabled.",
};

pub(crate) static AWS: AuthProfile = AuthProfile {
    method: AuthMethod::AccessKey,
    fields: AWS_FIELDS,
    instructions: "Prefer workload identity or an assumed STS role. Use an IAM user only when workload identity or an STS role is unavailable. Attach a minimum custom IAM policy; never use root or AdministratorAccess. Credentials stay in the secrets vault and never enter LLM context.",
    least_privilege: "Scope the custom IAM policy to the exact actions and resource ARNs required. Use temporary STS credentials whenever possible and rotate long-lived keys.",
};

pub(crate) static UNKNOWN: AuthProfile = AuthProfile {
    method: AuthMethod::Unknown,
    fields: &[],
    instructions: "Authentication must be verified against current official provider documentation before implementation.",
    least_privilege: "Do not collect credentials until the supported authentication method and minimum scopes are confirmed.",
};

pub(crate) static UNSUPPORTED: AuthProfile = AuthProfile {
    method: AuthMethod::Unsupported,
    fields: &[],
    instructions:
        "No supported official public API authentication flow is available for this catalog entry.",
    least_privilege: "Do not request or store credentials for unsupported providers.",
};
