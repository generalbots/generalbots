//! #1290 — environment-aware site targets for proxy-published projects.
//!
//! A website/python project publishes to TWO independent environments, mirroring
//! the VM projects' dev→prod model:
//!
//! | env        | host                          | payload dir            |
//! |------------|-------------------------------|------------------------|
//! | production | `{slug}.{domain}`             | `websites/{slug}`      |
//! | dev        | `{slug}-dev.{domain}`         | `websites/{slug}-dev`  |
//!
//! The production layout is unchanged (existing sites keep their dir, host and
//! release ring). Each environment owns its own payload directory, release
//! ring (`{dir}.prev-N`), python service port and Caddyfile block; promote
//! dev→prod copies the dev release ring head into the prod target. Rollback
//! and unpublish operate per environment.

/// Which environment a site operation targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SiteEnv {
    Production,
    Dev,
}

impl SiteEnv {
    /// Parse a publish `env` string ("production" / "development").
    pub fn parse(env: &str) -> Option<Self> {
        match env {
            "production" => Some(Self::Production),
            "development" | "dev" => Some(Self::Dev),
            _ => None,
        }
    }

    /// Canonical name used in deployment records and API payloads.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Production => "production",
            Self::Dev => "development",
        }
    }
}

/// Resolved filesystem/host targets for one (slug, env) pair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiteTarget {
    /// Payload directory inside the proxy (`websites/{dir_name}`).
    pub dir: String,
    /// Public hostname (Caddyfile block key).
    pub host: String,
}

impl SiteTarget {
    pub fn new(slug: &str, env: SiteEnv, domain: &str) -> Self {
        match env {
            SiteEnv::Production => Self {
                dir: format!("/opt/gbo/data/websites/{slug}"),
                host: format!("{slug}.{domain}"),
            },
            SiteEnv::Dev => Self {
                dir: format!("/opt/gbo/data/websites/{slug}-dev"),
                host: format!("{slug}-dev.{domain}"),
            },
        }
    }
}

/// Resolve the targets for a slug in BOTH environments (prod, dev).
pub fn both_targets(slug: &str, domain: &str) -> (SiteTarget, SiteTarget) {
    (
        SiteTarget::new(slug, SiteEnv::Production, domain),
        SiteTarget::new(slug, SiteEnv::Dev, domain),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prod_target_keeps_legacy_layout() {
        let t = SiteTarget::new("site1276", SiteEnv::Production, "generalbots.org");
        assert_eq!(t.dir, "/opt/gbo/data/websites/site1276");
        assert_eq!(t.host, "site1276.generalbots.org");
    }

    #[test]
    fn dev_target_gets_dev_suffix() {
        let t = SiteTarget::new("site1276", SiteEnv::Dev, "generalbots.org");
        assert_eq!(t.dir, "/opt/gbo/data/websites/site1276-dev");
        assert_eq!(t.host, "site1276-dev.generalbots.org");
    }

    #[test]
    fn parse_maps_publish_env_names() {
        assert_eq!(SiteEnv::parse("production"), Some(SiteEnv::Production));
        assert_eq!(SiteEnv::parse("development"), Some(SiteEnv::Dev));
        assert_eq!(SiteEnv::parse("dev"), Some(SiteEnv::Dev));
        assert_eq!(SiteEnv::parse("staging"), None);
    }
}
