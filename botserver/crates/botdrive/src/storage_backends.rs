//! Object-storage backend catalog (issue #1468).
//!
//! Drive is **private**: bytes reach a client by traversing botserver, gated by a
//! revocable share token (`drive_handlers::public_link_download`). Egress is
//! therefore bounded by how many share links tenants actually mint — not a
//! public CDN origin. That is what decides the default external backend.
//!
//! Measured list prices, USD per TB/month, mid-2026:
//!
//! | Backend  | Storage | Egress                    | Minimums                    |
//! |----------|---------|---------------------------|-----------------------------|
//! | MinIO    | n/a     | none (self-hosted)        | none                        |
//! | Backblaze B2 | $6.95 | free to 3x stored, then $0.01/GB | none           |
//! | Cloudflare R2 | $15.00 | **$0 unlimited**     | none                        |
//! | Wasabi   | $7.99   | $0, fair use 1:1          | **1 TB floor, 90-day term**  |
//!
//! **Wasabi is deliberately not the default.** Its 1 TB minimum means a platform
//! with many small organizations pays the floor per tenant, and its 90-day
//! retention bills deleted objects for the full window — a poor fit for `.ast`
//! recompiles and transient media, whose lifecycle is far shorter. It remains a
//! reasonable *archive* tier where the term is aligned with the data.

use std::sync::atomic::{AtomicU64, Ordering};

use log::warn;

/// The storage backends the platform can drive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageBackend {
    /// Self-hosted MinIO — the runtime default, unchanged.
    Minio,
    BackblazeB2,
    Wasabi,
    CloudflareR2,
}

impl StorageBackend {
    /// Identifier used in the Vault `secret/gbo/drive` `backend` key.
    #[must_use]
    pub const fn config_name(self) -> &'static str {
        match self {
            Self::Minio => "minio",
            Self::BackblazeB2 => "b2",
            Self::Wasabi => "wasabi",
            Self::CloudflareR2 => "r2",
        }
    }

    /// Resolves a configured name. Unknown names fall back to MinIO rather than
    /// failing the boot: a typo in a Vault value should not take Drive down.
    #[must_use]
    pub fn from_name(name: &str) -> Self {
        match name.trim().to_lowercase().as_str() {
            "b2" | "backblaze" | "backblaze-b2" | "backblazeb2" => Self::BackblazeB2,
            "wasabi" => Self::Wasabi,
            "r2" | "cloudflare" | "cloudflare-r2" => Self::CloudflareR2,
            _ => Self::Minio,
        }
    }

    /// Region this backend needs when the operator has not set one.
    ///
    /// B2 and R2 both require an explicit region; MinIO accepts the
    /// conventional `auto` sentinel and Wasabi ignores it in favour of the
    /// endpoint.
    #[must_use]
    pub const fn default_region(self) -> &'static str {
        match self {
            Self::Minio => "auto",
            Self::BackblazeB2 => "us-west-004",
            Self::Wasabi => "us-east-1",
            Self::CloudflareR2 => "auto",
        }
    }

    /// True when a misconfigured endpoint would break signing, because the
    /// backend derives its host from the region rather than the URL.
    #[must_use]
    pub const fn requires_explicit_region(self) -> bool {
        matches!(self, Self::BackblazeB2 | Self::CloudflareR2)
    }

    /// Smallest billable footprint, in GB. Wasabi's 1 TB floor is the reason it
    /// is not a Drive default.
    #[must_use]
    pub const fn minimum_billable_gb(self) -> u64 {
        match self {
            Self::Minio => 0,
            Self::BackblazeB2 => 0,
            Self::Wasabi => 1_024,
            Self::CloudflareR2 => 0,
        }
    }

    /// Minimum retention in days, charged in full for objects deleted sooner.
    #[must_use]
    pub const fn minimum_retention_days(self) -> u32 {
        match self {
            Self::Wasabi => 90,
            _ => 0,
        }
    }

    /// Bytes of egress included per byte stored before charges begin.
    ///
    /// B2 includes three times the stored volume; Wasabi allows fair use up to
    /// 1:1 and flags a breach; R2 egress is unmetered.
    #[must_use]
    pub const fn free_egress_ratio(self) -> f64 {
        match self {
            Self::Minio => f64::INFINITY,
            Self::BackblazeB2 => 3.0,
            Self::Wasabi => 1.0,
            Self::CloudflareR2 => f64::INFINITY,
        }
    }

    /// Whether the backend suits a per-tenant runtime bucket.
    ///
    /// Wasabi's floor and retention term are structurally wrong here, so the
    /// catalog says so and `crate::s3_repository` warns when one is selected.
    #[must_use]
    pub const fn suits_runtime_buckets(self) -> bool {
        !matches!(self, Self::Wasabi)
    }

    /// Reason this backend is unsuitable for a runtime bucket, if it is not.
    #[must_use]
    pub const fn runtime_warning(self) -> Option<&'static str> {
        match self {
            Self::Wasabi => Some(
                "Wasabi bills a 1 TB minimum and a 90-day retention term; \
                 per-tenant .gbdrive buckets are short-lived and small, so the \
                 floor is charged without being used. Use it as an archive tier.",
            ),
            _ => None,
        }
    }
}

/// A resolved backend selection, as read from configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageSelection {
    pub backend: StorageBackend,
    pub endpoint: String,
    pub region: String,
    pub bucket: String,
    /// Optional secondary endpoint used for reads when the primary is down.
    pub fallback_endpoint: Option<String>,
}

impl StorageSelection {
    /// Builds a selection from the raw configuration values.
    ///
    /// `region` falls back to the backend's default, which is what makes B2 and
    /// R2 work without an operator having to know their region name.
    #[must_use]
    pub fn new(
        backend_name: &str,
        endpoint: &str,
        region: &str,
        bucket: &str,
        fallback_endpoint: Option<String>,
    ) -> Self {
        let backend = StorageBackend::from_name(backend_name);
        let region = match region.trim() {
            "" | "auto" if !backend.requires_explicit_region() => backend.default_region().to_string(),
            "" => backend.default_region().to_string(),
            explicit => explicit.to_string(),
        };
        Self {
            backend,
            endpoint: endpoint.trim_end_matches('/').to_string(),
            region,
            bucket: bucket.to_string(),
            fallback_endpoint: fallback_endpoint
                .map(|value| value.trim().trim_end_matches('/').to_string())
                .filter(|value| !value.is_empty()),
        }
    }

    /// Warns once about a backend whose terms do not fit a runtime bucket.
    pub fn warn_if_unsuitable(&self) {
        if let Some(reason) = self.backend.runtime_warning() {
            warn!(
                "Drive backend {} selected for bucket {}: {reason}",
                self.backend.config_name(),
                self.bucket
            );
        }
    }

    /// True when the selection has a usable secondary endpoint.
    #[must_use]
    pub fn has_fallback(&self) -> bool {
        self.fallback_endpoint.is_some()
    }
}

/// Running egress counters, so a bill can be explained rather than surprising.
///
/// B2 charges egress past three times the stored volume and Wasabi flags a
/// fair-use breach; without a counter an operator learns about it from the
/// invoice.
#[derive(Debug, Default)]
pub struct EgressMeter {
    bytes_read: AtomicU64,
    bytes_written: AtomicU64,
}

impl EgressMeter {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_read(&self, bytes: u64) {
        self.bytes_read.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn record_write(&self, bytes: u64) {
        self.bytes_written.fetch_add(bytes, Ordering::Relaxed);
    }

    #[must_use]
    pub fn bytes_read(&self) -> u64 {
        self.bytes_read.load(Ordering::Relaxed)
    }

    #[must_use]
    pub fn bytes_written(&self) -> u64 {
        self.bytes_written.load(Ordering::Relaxed)
    }

    /// A snapshot suitable for logging or an API response.
    #[must_use]
    pub fn snapshot(&self) -> EgressSnapshot {
        EgressSnapshot {
            bytes_read: self.bytes_read(),
            bytes_written: self.bytes_written(),
        }
    }

    /// Bytes of egress that `stored_bytes` does not already cover for `backend`.
    ///
    /// The allowance is the free ratio times the stored volume, so the figure is
    /// what the provider would actually charge for.
    #[must_use]
    pub fn billable_egress(&self, backend: StorageBackend, stored_bytes: u64) -> u64 {
        let ratio = backend.free_egress_ratio();
        if !ratio.is_finite() {
            return 0;
        }
        let allowance = (stored_bytes as f64 * ratio) as u64;
        self.bytes_read().saturating_sub(allowance)
    }
}

/// Point-in-time egress counters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EgressSnapshot {
    pub bytes_read: u64,
    pub bytes_written: u64,
}

#[cfg(test)]
#[path = "storage_backends_tests.rs"]
mod tests;
