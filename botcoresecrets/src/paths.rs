#[derive(Debug)]
pub struct SecretPaths;

impl SecretPaths {
    pub const DIRECTORY: &'static str = "gbo/directory";
    pub const TABLES: &'static str = "gbo/tables";
    pub const DRIVE: &'static str = "gbo/drive";
    pub const CACHE: &'static str = "gbo/cache";
    pub const EMAIL: &'static str = "gbo/email";
    pub const LLM: &'static str = "gbo/llm";
    pub const ENCRYPTION: &'static str = "gbo/encryption";
    pub const JWT: &'static str = "gbo/jwt";
    pub const MEET: &'static str = "gbo/meet";
    pub const ALM: &'static str = "gbo/alm";
    pub const VECTORDB: &'static str = "gbo/vectordb";
    pub const OBSERVABILITY: &'static str = "gbo/system/observability";
    pub const SECURITY: &'static str = "gbo/system/security";
    pub const CLOUD: &'static str = "gbo/system/cloud";
    pub const APP: &'static str = "gbo/system/app";
    pub const MODELS: &'static str = "gbo/system/models";

    pub fn tenant_infrastructure(tenant: &str) -> String {
        format!("gbo/tenants/{}/infrastructure", tenant)
    }

    pub fn tenant_config(tenant: &str) -> String {
        format!("gbo/tenants/{}/config", tenant)
    }

    pub fn org_bot(org_id: &str, bot_id: &str) -> String {
        format!("gbo/orgs/{}/bots/{}", org_id, bot_id)
    }

    pub fn org_user(org_id: &str, user_id: &str) -> String {
        format!("gbo/orgs/{}/users/{}", org_id, user_id)
    }

    pub fn org_config(org_id: &str) -> String {
        format!("gbo/orgs/{}/config", org_id)
    }
}
