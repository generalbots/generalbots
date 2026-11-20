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
    /// Signing key for OAuth `state` tokens (#2130): field `secret`. When
    /// absent, the OAuth flow falls back to `JWT` and then to an ephemeral
    /// per-process key.
    pub const OAUTH_STATE: &'static str = "gbo/oauth-state";
    pub const MEET: &'static str = "gbo/meet";
    pub const ALM: &'static str = "gbo/alm";
    pub const VECTORDB: &'static str = "gbo/vectordb";
    pub const OBSERVABILITY: &'static str = "gbo/system/observability";
    pub const SECURITY: &'static str = "gbo/system/security";
    pub const CLOUD: &'static str = "gbo/system/cloud";
    pub const APP: &'static str = "gbo/system/app";
    pub const MODELS: &'static str = "gbo/system/models";
    pub const VDI: &'static str = "gbo/vdi";
    /// Peer-network settings (#1968): `enabled`, `node-id`, `node-name`,
    /// `advertise-addr`, `data-dir`, `heartbeat-ttl-secs`, `token`,
    /// `signing-key`.
    pub const PEER: &'static str = "gbo/peer";

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_paths_are_stable() {
        assert_eq!(SecretPaths::DIRECTORY, "gbo/directory");
        assert_eq!(SecretPaths::TABLES, "gbo/tables");
        assert_eq!(SecretPaths::DRIVE, "gbo/drive");
        assert_eq!(SecretPaths::CACHE, "gbo/cache");
        assert_eq!(SecretPaths::LLM, "gbo/llm");
        assert_eq!(SecretPaths::JWT, "gbo/jwt");
        assert_eq!(SecretPaths::OAUTH_STATE, "gbo/oauth-state");
        assert_eq!(SecretPaths::SECURITY, "gbo/system/security");
        assert_eq!(SecretPaths::PEER, "gbo/peer");
    }

    #[test]
    fn test_tenant_paths_are_scoped_and_distinct() {
        let infrastructure = SecretPaths::tenant_infrastructure("acme");
        assert_eq!(infrastructure, "gbo/tenants/acme/infrastructure");
        assert_ne!(infrastructure, SecretPaths::tenant_infrastructure("other"));
        let config = SecretPaths::tenant_config("acme");
        assert_eq!(config, "gbo/tenants/acme/config");
        assert_ne!(config, infrastructure);
    }

    #[test]
    fn test_org_paths_separate_bots_users_and_config() {
        let bot = SecretPaths::org_bot("org-1", "same-id");
        let user = SecretPaths::org_user("org-1", "same-id");
        assert_eq!(bot, "gbo/orgs/org-1/bots/same-id");
        assert_eq!(user, "gbo/orgs/org-1/users/same-id");
        assert_ne!(bot, user);
        assert_eq!(SecretPaths::org_config("org-1"), "gbo/orgs/org-1/config");
    }
}
