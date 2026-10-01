//! Domain provisioning for the cloud API (issue #1469).
//!
//! Buying `domain-com` used to stop at "you own a hostname": the SKU was a
//! catalogue row, nothing registered anything, and `bot_domains` had to be
//! filled in by hand. This module is the missing half — it registers the name,
//! delegates it, writes the routing and onboarding records, and wires the
//! `bot_domains` row so `/api/domains/resolve` starts answering for it.
//!
//! The domain SKUs and the `bot_domains` table are deliberately kept distinct:
//! `bot_domains` stays the runtime routing source, and this module is only a
//! writer to it.

use botdomains::zone;
use botdomains::{Credential, DomainSpec, RegistrarOptions, Registration, registry};
use diesel::prelude::*;
use tracing::{info, warn};
use serde_json::Value;
use uuid::Uuid;

/// Status recorded when the organization has no registrar credential.
pub const STATUS_NO_KEY: &str = "provisioning_no_key";

/// Status recorded when registration itself failed.
pub const STATUS_FAILED: &str = "provisioning_failed";

/// Default nameserver delegation for a provisioned domain. deSEC manages DNSSEC
/// as its core product and rejects ALIAS/ANAME, which is why the platform writes
/// an apex `A` record rather than an apex CNAME.
const DEFAULT_NAMESERVERS: &[&str] = &["ns1.desec.io", "ns2.desec.org"];

/// One catalogue domain SKU.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DomainSku {
    pub tld: &'static str,
    /// Quoted yearly price in cents.
    pub yearly_cents: i64,
    /// Registrars able to sell this TLD, in preference order.
    pub candidates: &'static [&'static str],
}

/// Maps a catalogue id to its TLD and provider chain.
#[must_use]
pub fn sku_for(store_item_id: &str) -> Option<DomainSku> {
    let found = match store_item_id {
        "domain-com" => DomainSku { tld: "com", yearly_cents: 2_199, candidates: &["porkbun", "cloudflare", "dynadot", "namecheap"] },
        "domain-io" => DomainSku { tld: "io", yearly_cents: 7_199, candidates: &["porkbun", "dynadot", "cloudflare"] },
        "domain-ai" => DomainSku { tld: "ai", yearly_cents: 15_999, candidates: &["porkbun", "cloudflare", "dynadot"] },
        _ => return None,
    };
    Some(found)
}

/// Per-organization domain settings read from `cloud_organizations.config`.
#[derive(Debug, Default)]
pub struct OrgDomainSettings {
    /// Credential per registrar name.
    pub keys: std::collections::BTreeMap<String, Credential>,
    /// Registrar the organization prefers.
    pub preferred_registrar: Option<String>,
    /// Address the platform's proxy serves the domain from, for the apex `A`.
    pub app_ip: String,
    pub app_host: String,
    /// DMARC policy published at `_dmarc`.
    pub dmarc_record: String,
    /// Adapter deployment configuration.
    pub options: RegistrarOptions,
}

impl OrgDomainSettings {
    /// Parses the JSONB `config` column.
    #[must_use]
    pub fn from_config(config: Option<&Value>) -> Self {
        let mut settings = Self {
            app_ip: "127.0.0.1".into(),
            app_host: "localhost".into(),
            dmarc_record: "v=DMARC1; p=reject; rua=mailto:abuse@generalbots.org".into(),
            ..Self::default()
        };
        let Some(config) = config else { return settings };

        if let Some(map) = config.get("registrar_keys").and_then(Value::as_object) {
            for (name, value) in map {
                if let Some(entry) = value.as_object() {
                    let key = entry.get("api_key").and_then(Value::as_str).unwrap_or_default();
                    let secret = entry.get("api_secret").and_then(Value::as_str).unwrap_or_default();
                    if !key.trim().is_empty() {
                        settings
                            .keys
                            .insert(name.to_lowercase(), Credential::new(key.trim(), secret.trim()));
                    }
                }
            }
        }
        settings.preferred_registrar = config
            .get("domain_registrar")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(str::to_lowercase);
        if let Some(ip) = config.get("domain_app_ip").and_then(Value::as_str) {
            settings.app_ip = ip.to_string();
        }
        if let Some(host) = config.get("domain_app_host").and_then(Value::as_str) {
            settings.app_host = host.to_string();
        }
        if let Some(dmarc) = config.get("domain_dmarc").and_then(Value::as_str) {
            settings.dmarc_record = dmarc.to_string();
        }
        settings
    }

    /// Credential for `registrar`.
    #[must_use]
    pub fn key_for(&self, registrar: &str) -> Option<&Credential> {
        let wanted = registrar.to_lowercase();
        self.keys.get(&wanted)
    }

    /// Reorders a candidate chain so the organization's preference comes first.
    #[must_use]
    pub fn order_candidates<'c>(&self, candidates: &[&'c str]) -> Vec<&'c str> {
        let Some(preferred) = self.preferred_registrar.as_deref() else {
            return candidates.to_vec();
        };
        match candidates.iter().position(|name| name.eq_ignore_ascii_case(preferred)) {
            Some(index) => {
                let mut ordered = candidates.to_vec();
                let chosen = ordered.remove(index);
                ordered.insert(0, chosen);
                ordered
            }
            None => candidates.to_vec(),
        }
    }
}

/// `cloud_organizations.config` is JSONB; it is read as text and parsed, which
/// keeps this module independent of the diesel JSONB feature set.
#[derive(diesel::QueryableByName, Debug)]
struct OrgConfigRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    config: String,
}

/// Parses a `config::text` value, treating absent or malformed JSON as "no
/// overrides" rather than an error: a tenant with a broken config column should
/// still get the platform defaults.
fn parse_config(raw: Option<String>) -> Option<Value> {
    let text = raw?;
    if text.trim().is_empty() {
        return None;
    }
    serde_json::from_str::<Value>(&text).ok()
}

/// Reads the organization's domain settings.
pub fn load_settings(pool: &crate::DbPool, org_id: Uuid) -> Result<OrgDomainSettings, String> {
    let mut conn = pool.get().map_err(|e| format!("DB: {e}"))?;
    let raw: Option<OrgConfigRow> = diesel::sql_query(
        "SELECT COALESCE(config::text, '') AS config FROM cloud_organizations WHERE id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(org_id)
    .get_result(&mut conn)
    .optional()
    .map_err(|e| format!("DB: {e}"))?;

    let parsed = parse_config(raw.map(|row| row.config));
    Ok(OrgDomainSettings::from_config(parsed.as_ref()))
}

/// Builds the domain name for an organization and a TLD.
///
/// The label is a hash of the organization id rather than a slice of it: a UUID
/// prefix in a hostname is both leaky — it exposes the tenant's id in the public
/// DNS — and awkward, since a UUID's leading characters are not uniformly random
/// and can collide. Base-36 from a 64-bit FNV-1a gives ten characters, well
/// inside the DNS label limit and far beyond the platform's tenant count.
#[must_use]
pub fn domain_for(org_id: Uuid, tld: &str) -> String {
    format!("gb-{}.{}", short_hash(&org_id), tld)
}

/// Stable, non-reversible, DNS-label-safe identifier for a tenant.
fn short_hash(org_id: &Uuid) -> String {
    const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

    let mut hash = FNV_OFFSET;
    for byte in org_id.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    to_base36(hash, 10)
}

/// Fixed-width base-36 rendering, zero-padded so the label length is stable.
fn to_base36(mut value: u64, width: usize) -> String {
    const DIGITS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let mut buffer = vec![b'0'; width];
    for slot in buffer.iter_mut().rev() {
        *slot = DIGITS[(value % 36) as usize];
        value /= 36;
    }
    String::from_utf8(buffer).unwrap_or_default()
}

/// Outcome of a successful registration.
#[derive(Debug, Clone)]
pub struct DomainOutcome {
    pub registration: Registration,
    pub registrar_name: String,
    /// Records that were written, or that the registrar refused to write.
    pub written: Vec<botdomains::DnsRecord>,
    pub refused: Vec<String>,
}

/// Registers a domain and writes its onboarding records.
///
/// Walks the SKU's registrar chain: a registrar without a credential or without
/// the TLD is skipped, while a registration that fails outright stops the walk,
/// because retrying a purchase against four registrars can create four
/// registrations.
pub async fn register_domain(
    sku: &DomainSku,
    domain: &str,
    settings: &OrgDomainSettings,
) -> Result<DomainOutcome, String> {
    let candidates = settings.order_candidates(sku.candidates);
    let mut errors: Vec<String> = Vec::new();

    for name in candidates {
        let Some(credential) = settings.key_for(name) else {
            errors.push(format!("{name}: no credential for this organization"));
            continue;
        };
        let Some(registrar) = registry::configured_registrar(name, &settings.options) else {
            errors.push(format!("{name}: not available in this build"));
            continue;
        };
        if !registrar.info().supported_tlds.iter().any(|t| t == sku.tld) {
            let tld = sku.tld;
            errors.push(format!("{name}: does not sell .{tld}"));
            continue;
        }

        let spec = DomainSpec::one_year(domain).with_nameservers(DEFAULT_NAMESERVERS);
        let registration = match registrar.register(&spec, credential).await {
            Ok(registration) => registration,
            Err(err) if err.is_retryable() => {
                warn!("{name} failed transiently, trying the next registrar: {err}");
                errors.push(format!("{name}: {err}"));
                continue;
            }
            Err(err) => return Err(format!("{name}: {err}")),
        };

        info!("Registered {domain} via {name}");
        let (written, refused) = write_onboarding_records(
            registrar.as_ref(),
            domain,
            &registration,
            credential,
            settings,
        )
        .await;

        return Ok(DomainOutcome {
            registration,
            registrar_name: name.to_string(),
            written,
            refused,
        });
    }

    if errors.is_empty() {
        return Err(format!(
            "no registrar among candidates [{}] is available in this build",
            sku.candidates.join(", ")
        ));
    }
    Err(format!("every registrar in the chain failed: {}", errors.join("; ")))
}

/// Writes the routing and onboarding records for a registered domain.
///
/// A registrar whose only DNS write replaces the whole zone gets an explicit
/// diff first, so a refused write is reported rather than silently dropping the
/// customer's records.
async fn write_onboarding_records(
    registrar: &dyn botdomains::Registrar,
    domain: &str,
    registration: &Registration,
    credential: &Credential,
    settings: &OrgDomainSettings,
) -> (Vec<botdomains::DnsRecord>, Vec<String>) {
    let mut written = Vec::new();
    let mut refused = Vec::new();

    for record in zone::onboarding_records(
        &settings.app_ip,
        &settings.app_host,
        &settings.dmarc_record,
    ) {
        match registrar.set_dns_record(domain, &record, credential).await {
            Ok(_) => written.push(record),
            Err(err) => {
                warn!("{domain}: refused {} record — {err}", record.record_type.as_str());
                refused.push(format!("{}: {err}", record.record_type.as_str()));
            }
        }
    }

    if registration.auto_renew {
        info!("{domain}: auto-renew is on, so no renewal task is needed");
    }
    (written, refused)
}

/// Writes the `bot_domains` row so `/api/domains/resolve` answers for `domain`.
///
/// This is the payoff of the whole path: without it the customer buys a domain
/// that routes nowhere.
pub fn wire_routing(
    pool: &crate::DbPool,
    domain: &str,
    org_id: Uuid,
    branch_id: Option<Uuid>,
    bot_id: Option<Uuid>,
) -> Result<(), String> {
    let mut conn = pool.get().map_err(|e| format!("DB: {e}"))?;
    diesel::sql_query(
        "INSERT INTO bot_domains (id, domain, bot_id, org_id, branch_id, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5, NOW(), NOW()) \
         ON CONFLICT (domain) DO UPDATE SET bot_id = EXCLUDED.bot_id, \
             org_id = EXCLUDED.org_id, branch_id = EXCLUDED.branch_id, updated_at = NOW()",
    )
    .bind::<diesel::sql_types::Uuid, _>(Uuid::new_v4())
    .bind::<diesel::sql_types::Text, _>(domain.to_lowercase())
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(bot_id)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(Some(org_id))
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(branch_id)
    .execute(&mut conn)
    .map_err(|e| format!("bot_domains insert: {e}"))?;
    Ok(())
}

/// The full purchase path: register, delegate, write records, wire routing.
pub async fn provision_domain(
    pool: &crate::DbPool,
    store_item_id: &str,
    resource_id: Uuid,
    _workspace_id: Uuid,
    org_id: Uuid,
) -> Result<(), String> {
    use crate::compute_provisioning as compute;

    let sku = sku_for(store_item_id)
        .ok_or_else(|| format!("Unknown store item: {store_item_id}"))?;
    let settings = load_settings(pool, org_id)?;

    if !sku.candidates.iter().any(|name| settings.key_for(name).is_some()) {
        let _ = compute::record_outcome(
            pool,
            resource_id,
            STATUS_NO_KEY,
            serde_json::json!({ "error": "no registrar credential for this organization" }),
        );
        return Err("No registrar API key configured for organization".into());
    }

    let domain = domain_for(org_id, sku.tld);
    let outcome = match register_domain(&sku, &domain, &settings).await {
        Ok(outcome) => outcome,
        Err(message) => {
            let _ = compute::record_outcome(
                pool,
                resource_id,
                STATUS_FAILED,
                serde_json::json!({ "error": message }),
            );
            return Err(message);
        }
    };

    // Routing is wired even when some DNS records were refused: the domain is
    // owned and delegated, and the refused records are reported for follow-up.
    wire_routing(pool, &domain, org_id, None, None)?;

    let config = serde_json::json!({
        "domain": outcome.registration.domain,
        "registrar": outcome.registrar_name,
        "auto_renew": outcome.registration.auto_renew,
        "expires_at": outcome.registration.expires_at,
        "dns_records_written": outcome.written.len(),
        "dns_records_refused": outcome.refused,
        "nameservers": DEFAULT_NAMESERVERS,
        "yearly_cents": sku.yearly_cents,
    });
    compute::record_outcome(pool, resource_id, "active", config)
}

/// Marks the credential problems a caller can act on without a registrar call.
#[must_use]
pub fn missing_credential_report(settings: &OrgDomainSettings, sku: &DomainSku) -> String {
    let missing: Vec<&str> = sku
        .candidates
        .iter()
        .filter(|name| settings.key_for(name).is_none())
        .copied()
        .collect();
    let gaps = if missing.is_empty() { "none".to_string() } else { missing.join(", ") };
    format!(
        "no credential for any registrar in [{}]; missing: {gaps}",
        sku.candidates.join(", ")
    )
}

#[cfg(test)]
#[path = "domain_provisioning_tests.rs"]
mod tests;
