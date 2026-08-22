use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Featured,
    Productivity,
    Developer,
    Startups,
    SmallBusiness,
    Finance,
    SocialMessaging,
    Lifestyle,
}

impl Category {
    pub const ALL: [Self; 8] = [
        Self::Featured,
        Self::Productivity,
        Self::Developer,
        Self::Startups,
        Self::SmallBusiness,
        Self::Finance,
        Self::SocialMessaging,
        Self::Lifestyle,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Featured => "featured",
            Self::Productivity => "productivity",
            Self::Developer => "developer",
            Self::Startups => "startups",
            Self::SmallBusiness => "small_business",
            Self::Finance => "finance",
            Self::SocialMessaging => "social_messaging",
            Self::Lifestyle => "lifestyle",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Featured => "Featured",
            Self::Productivity => "Productivity",
            Self::Developer => "Developer",
            Self::Startups => "Startups",
            Self::SmallBusiness => "Small Business",
            Self::Finance => "Finance",
            Self::SocialMessaging => "Social & Messaging",
            Self::Lifestyle => "Lifestyle",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Strategy {
    Integrate,
    Improve,
    NewApp,
    Verify,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Built,
    Partial,
    Planned,
    Unsupported,
}

impl Status {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Built => "built",
            Self::Partial => "partial",
            Self::Planned => "planned",
            Self::Unsupported => "unsupported",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Must,
    Nice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthMethod {
    #[serde(rename = "oauth2")]
    OAuth2,
    ApiKey,
    Basic,
    Token,
    Protocol,
    AccessKey,
    Unknown,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InputType {
    Text,
    Password,
    Number,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AuthField {
    pub key: &'static str,
    pub label: &'static str,
    pub input_type: InputType,
    pub secret: bool,
    pub required: bool,
    pub placeholder: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AuthProfile {
    pub method: AuthMethod,
    pub fields: &'static [AuthField],
    pub instructions: &'static str,
    pub least_privilege: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ParameterType {
    String,
    Integer,
    DateTime,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Parameter {
    pub name: &'static str,
    #[serde(rename = "type")]
    pub parameter_type: ParameterType,
    pub required: bool,
    pub description: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Risk {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Surface {
    Chat,
    Ui,
    Api,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ActionTemplate {
    pub key: &'static str,
    pub verb: &'static str,
    pub label: &'static str,
    pub summary: &'static str,
    pub params: &'static [Parameter],
    pub risk: Risk,
    pub requires_approval: bool,
    pub surfaces: &'static [Surface],
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderAction {
    pub name: String,
    pub verb: &'static str,
    pub label: &'static str,
    pub summary: &'static str,
    pub params: &'static [Parameter],
    pub risk: Risk,
    pub requires_approval: bool,
    pub surfaces: &'static [Surface],
    pub implemented: bool,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ProviderSeed {
    pub id: &'static str,
    pub name: &'static str,
    pub category: Category,
    pub strategy: Strategy,
    pub status: Status,
    pub priority: Priority,
    pub module: Option<&'static str>,
    pub official_docs: Option<&'static str>,
    pub auth: &'static AuthProfile,
    pub actions: &'static [ActionTemplate],
    pub llm_available: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderItem {
    pub id: &'static str,
    pub name: &'static str,
    pub category: Category,
    pub strategy: Strategy,
    pub status: Status,
    pub priority: Priority,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub official_docs: Option<&'static str>,
    pub auth: AuthProfile,
    pub actions: Vec<ProviderAction>,
    pub action_count: usize,
    pub llm_available: bool,
}

#[derive(Debug, Serialize)]
pub struct CategorySummary {
    pub id: &'static str,
    pub label: &'static str,
    pub count: usize,
}

#[derive(Debug, Serialize)]
pub struct CatalogTotals {
    pub providers: usize,
    pub actions: usize,
    pub implemented_actions: usize,
}

#[derive(Debug, Serialize)]
pub struct CatalogResponse {
    pub providers: Vec<ProviderItem>,
    pub categories: Vec<CategorySummary>,
    pub totals: CatalogTotals,
    pub provider_count: usize,
    pub action_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct LlmAction {
    pub name: String,
    pub summary: &'static str,
    pub params: &'static [Parameter],
    pub risk: Risk,
    pub requires_approval: bool,
    pub implemented: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct LlmProviderSummary {
    pub id: &'static str,
    pub name: &'static str,
    pub category: Category,
    pub status: Status,
    pub action_count: usize,
    pub llm_available: bool,
}
