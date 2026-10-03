//! Compute provisioning for the cloud API (issue #1470).
//!
//! Extracted from `api.rs`, which was already over the file-size limit, so this
//! module owns the parts of a provisioning run that have a policy: candidate
//! selection, per-organization credentials, region choice, the retry chain and
//! cost reconciliation. `api.rs` keeps only the HTTP handler.

use botproviders::{MachineSpec, ProviderError, ProvisionResult, registry};
use diesel::prelude::*;
use tracing::{info, warn};
use serde_json::Value;
use uuid::Uuid;

/// Status recorded on the resource when the organization has no credential for
/// any candidate provider.
pub const STATUS_NO_KEY: &str = "provisioning_no_key";

/// Status recorded when every candidate provider was tried and failed.
pub const STATUS_FAILED: &str = "provisioning_failed";

/// Key holding the JSON map of per-provider credentials in
/// `cloud_organizations.config`.
const PROVIDER_KEYS_FIELD: &str = "provider_keys";

/// Key holding the organization's preferred provider name.
const PROVIDER_PREFERENCE_FIELD: &str = "compute_provider";

/// One catalogue SKU: the machine it sells and the providers able to deliver it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkuSpec {
    pub machine: MachineSpec,
    /// Ordered cheapest-fit first; a candidate that fails is skipped for the next.
    pub candidates: &'static [&'static str],
}

fn spec(
    cpu_cores: u32,
    ram_gb: u32,
    disk_gb: u32,
    gpu_type: Option<&str>,
    bandwidth_tb: u32,
    candidates: &'static [&'static str],
) -> SkuSpec {
    SkuSpec {
        machine: MachineSpec {
            cpu_cores,
            ram_gb,
            disk_gb,
            gpu_type: gpu_type.map(str::to_string),
            gpu_count: u32::from(gpu_type.is_some()),
            bandwidth_tb,
            use_spot: false,
        },
        candidates,
    }
}

/// Maps a catalogue id to its machine spec and provider candidate chain.
///
/// `vps-large` and `vps-xl` previously listed only `contabo`, which made them
/// unprovisionable the moment Contabo was out; both now carry a chain.
#[must_use]
pub fn sku_for(store_item_id: &str) -> Option<SkuSpec> {
    let found = match store_item_id {
        "vps-small" => spec(4, 8, 100, None, 2, &["hetzner", "digitalocean", "vultr", "contabo", "vast"]),
        "vps-medium" => spec(6, 16, 200, None, 4, &["hetzner", "digitalocean", "oracle", "vultr", "contabo", "vast"]),
        "vps-large" => spec(8, 32, 400, None, 8, &["digitalocean", "oracle", "vultr", "contabo", "hetzner", "vast"]),
        "vps-xl" => spec(16, 64, 800, None, 16, &["oracle", "vultr", "contabo", "hetzner", "vast"]),
        "gpu-basic" => spec(4, 8, 50, Some("RTX 3060 12 GB"), 2, &["vast", "ovh", "runpod"]),
        "gpu-pro" => spec(8, 32, 200, Some("RTX 4090 24 GB"), 4, &["vast", "ovh", "runpod", "contabo"]),
        "gpu-enterprise" => spec(16, 64, 500, Some("A100"), 8, &["ovh", "vast", "runpod", "contabo"]),
        _ => return None,
    };
    Some(found)
}

/// Per-organization compute settings read from `cloud_organizations.config`.
///
/// `provider_api_key` predates multi-provider support and holds one
/// unnamespaced secret. It is still honoured, applied to whichever provider the
/// chain reaches first, so an existing organization keeps working after the
/// upgrade.
#[derive(Debug, Default)]
pub struct OrgComputeSettings {
    /// Credential per provider name.
    pub keys: std::collections::BTreeMap<String, String>,
    /// Provider the organization prefers, when it has one.
    pub preferred_provider: Option<String>,
    /// The legacy single key, applied as a fallback for any provider without an
    /// entry in `keys`.
    pub legacy_key: Option<String>,
}

impl OrgComputeSettings {
    /// Parses the JSONB `config` column.
    #[must_use]
    pub fn from_config(config: Option<&Value>) -> Self {
        let mut settings = Self::default();
        let Some(config) = config else {
            return settings;
        };

        if let Some(map) = config.get(PROVIDER_KEYS_FIELD).and_then(Value::as_object) {
            for (provider, value) in map {
                if let Some(key) = value.as_str().map(str::trim).filter(|k| !k.is_empty()) {
                    settings.keys.insert(provider.to_lowercase(), key.to_string());
                }
            }
        }
        settings.preferred_provider = config
            .get(PROVIDER_PREFERENCE_FIELD)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(str::to_lowercase);
        settings.legacy_key = config
            .get("provider_api_key")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|key| !key.is_empty())
            .map(str::to_string);
        settings
    }

    /// Credential for `provider`, falling back to the legacy single key.
    #[must_use]
    pub fn key_for(&self, provider: &str) -> Option<&str> {
        let wanted = provider.to_lowercase();
        self.keys.get(&wanted).map(String::as_str).or(self.legacy_key.as_deref())
    }

    /// Reorders a candidate chain so the organization's preference comes first.
    #[must_use]
    pub fn order_candidates<'c>(&self, candidates: &[&'c str]) -> Vec<&'c str> {
        let Some(preferred) = self.preferred_provider.as_deref() else {
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

/// Reads the organization's compute settings.
pub fn load_settings(pool: &crate::DbPool, org_id: Uuid) -> Result<OrgComputeSettings, String> {
    let mut conn = pool.get().map_err(|e| format!("DB: {e}"))?;
    let raw: Option<OrgConfigRow> = diesel::sql_query(
        "SELECT COALESCE(config::text, '') AS config FROM cloud_organizations WHERE id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(org_id)
    .get_result(&mut conn)
    .optional()
    .map_err(|e| format!("DB: {e}"))?;

    let parsed = parse_config(raw.map(|row| row.config));
    Ok(OrgComputeSettings::from_config(parsed.as_ref()))
}

/// Compares what the provider actually charged against the catalogue price.
///
/// `ProvisionResult::hourly_cost` was recorded but never read back, so a
/// provisioning path could silently bill above the advertised price. A variance
/// beyond [`COST_TOLERANCE_PCT`] is logged and stored on the resource so support
/// can see it; a provider that reports no price at all is not treated as free.
pub fn reconcile_cost(result_hourly: f64, catalogue_monthly: f64) -> Option<f64> {
    if result_hourly <= 0.0 || catalogue_monthly <= 0.0 {
        return None;
    }
    // Catalogue prices are per month; the provider reports per hour.
    let catalogue_hourly = catalogue_monthly / 730.0;
    let variance = (result_hourly - catalogue_hourly) / catalogue_hourly * 100.0;
    (variance.abs() > COST_TOLERANCE_PCT).then_some(variance)
}

/// Percentage a provider may exceed the catalogue price before it is reported.
pub const COST_TOLERANCE_PCT: f64 = 5.0;

/// One attempt's outcome, kept so the caller can record why provisioning failed.
#[derive(Debug)]
pub struct ProvisionOutcome {
    pub result: ProvisionResult,
    pub provider_name: String,
    /// Percentage over the catalogue price, when the provider reported one and
    /// it exceeded the tolerance.
    pub cost_variance_pct: Option<f64>,
}

/// Walks a SKU's candidate chain until one provider provisions the machine.
///
/// Capacity and authentication failures move on to the next candidate — a busy
/// host or a key scoped to a different vendor is not a reason to fail the
/// customer. Anything else stops the walk, because retrying the same bad request
/// against five vendors helps nobody.
pub async fn provision_with_fallback(
    sku: &SkuSpec,
    region: &str,
    settings: &OrgComputeSettings,
) -> Result<ProvisionOutcome, String> {
    let candidates = settings.order_candidates(sku.candidates);
    let mut errors: Vec<String> = Vec::new();

    for name in candidates {
        let Some(provider) = registry::provider_for_name(name) else {
            errors.push(format!("{name}: not compiled into this build"));
            continue;
        };
        let Some(key) = settings.key_for(name) else {
            errors.push(format!("{name}: no credential for this organization"));
            continue;
        };

        match provider.provision(&sku.machine, region, key).await {
            Ok(result) => {
                let provider_name = provider.name().to_string();
                let region = result.region.clone();
                let instance_id = result.instance_id.clone();
                info!("Provisioned {provider_name}: instance={instance_id} region={region}");
                return Ok(ProvisionOutcome {
                    result,
                    provider_name,
                    cost_variance_pct: None,
                });
            }
            Err(ProviderError::Capacity(message)) => {
                warn!("{name} has no capacity, trying the next candidate: {message}");
                errors.push(format!("{name}: {message}"));
            }
            Err(ProviderError::Auth(message)) => {
                warn!("{name} rejected the credential, trying the next candidate: {message}");
                errors.push(format!("{name}: {message}"));
            }
            Err(err) => {
                return Err(format!("{name}: {err}"));
            }
        }
    }

    if errors.is_empty() {
        return Err(format!(
            "no provider in the chain [{}] is compiled into this build",
            sku.candidates.join(", ")
        ));
    }
    Err(format!(
        "every provider in the chain failed: {}",
        errors.join("; ")
    ))
}

/// Records the outcome on the resource row.
pub fn record_outcome(
    pool: &crate::DbPool,
    resource_id: Uuid,
    status: &str,
    config: Value,
) -> Result<(), String> {
    use crate::schema_ext::workspace_resources::dsl as wr;

    let mut conn = pool.get().map_err(|e| format!("DB: {e}"))?;
    diesel::update(wr::workspace_resources.filter(wr::id.eq(resource_id)))
        .set((wr::status.eq(status), wr::config.eq(Some(config))))
        .execute(&mut conn)
        .map_err(|e| format!("DB: {e}"))?;
    Ok(())
}

/// Builds the `config` JSON stored on the resource for a successful launch.
#[must_use]
pub fn success_config(outcome: &ProvisionOutcome) -> Value {
    let mut config = serde_json::json!({
        "provider": outcome.provider_name,
        "instance_id": outcome.result.instance_id,
        "ip": outcome.result.ip_address,
        "region": outcome.result.region,
        "hourly_cost": outcome.result.hourly_cost,
        "status": outcome.result.status,
    });
    if let Some(variance) = outcome.cost_variance_pct {
        config["cost_variance_pct"] = serde_json::json!(variance);
    }
    config
}

#[cfg(test)]
#[path = "compute_provisioning_tests.rs"]
mod tests;
