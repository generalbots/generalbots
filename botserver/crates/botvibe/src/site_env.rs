//! #1290 — environment-aware site targets for proxy-published projects.
//!
//! A website/python project keeps TWO websites, so an in-flight change never
//! overwrites the public page:
//!
//! | env        | host                          | payload dir            |
//! |------------|-------------------------------|------------------------|
//! | test       | `{slug}-test.{domain}`        | `websites/{slug}-test` |
//! | production | `{slug}.{domain}`             | `websites/{slug}`      |
//!
//! The production layout is the legacy one (existing sites keep their dir,
//! host and release ring). Each environment owns its payload directory,
//! release ring (`{dir}.prev-N`), python service port and Caddyfile block;
//! promote copies the test release ring head into the production target.
//! Rollback and unpublish operate per environment.
//!
//! The test target is the DEFAULT for site projects: only an explicit
//! production publish sanctioned by the deploy pipeline writes the public
//! slug (see `publish::do_publish`).

/// Which environment a site operation targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SiteEnv {
    Production,
    Test,
}

impl SiteEnv {
    /// Parse a publish `env` string. `test` is canonical; the legacy
    /// `development`/`dev` spellings stay accepted so existing callers and
    /// deployment records keep working.
    pub fn parse(env: &str) -> Option<Self> {
        match env.to_ascii_lowercase().as_str() {
            "production" | "prod" => Some(Self::Production),
            "test" | "testing" | "development" | "dev" => Some(Self::Test),
            _ => None,
        }
    }

    /// Canonical name used in deployment records and API payloads.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Production => "production",
            Self::Test => "test",
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
            SiteEnv::Test => Self {
                dir: format!("/opt/gbo/data/websites/{slug}-test"),
                host: format!("{slug}-test.{domain}"),
            },
        }
    }
}

/// Resolve the targets for a slug in BOTH environments (production, test).
pub fn both_targets(slug: &str, domain: &str) -> (SiteTarget, SiteTarget) {
    (
        SiteTarget::new(slug, SiteEnv::Production, domain),
        SiteTarget::new(slug, SiteEnv::Test, domain),
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
    fn test_target_gets_test_suffix() {
        let t = SiteTarget::new("site1276", SiteEnv::Test, "generalbots.org");
        assert_eq!(t.dir, "/opt/gbo/data/websites/site1276-test");
        assert_eq!(t.host, "site1276-test.generalbots.org");
    }

    #[test]
    fn parse_maps_publish_env_names() {
        assert_eq!(SiteEnv::parse("production"), Some(SiteEnv::Production));
        assert_eq!(SiteEnv::parse("prod"), Some(SiteEnv::Production));
        assert_eq!(SiteEnv::parse("test"), Some(SiteEnv::Test));
        assert_eq!(SiteEnv::parse("testing"), Some(SiteEnv::Test));
        // Legacy spellings keep resolving to the test twin.
        assert_eq!(SiteEnv::parse("development"), Some(SiteEnv::Test));
        assert_eq!(SiteEnv::parse("dev"), Some(SiteEnv::Test));
        assert_eq!(SiteEnv::parse("staging"), None);
    }

    #[test]
    fn both_targets_are_distinct() {
        let (prod, test) = both_targets("mysite", "generalbots.org");
        assert_ne!(prod.dir, test.dir);
        assert_ne!(prod.host, test.host);
    }
}
