//! Registrar adapters for domain provisioning (issue #1469).
//!
//! The store sells `domain-com`/`domain-io`/`domain-ai` as priced SKUs with no
//! registrar behind them: buying one stopped at "you own a hostname". This crate
//! adds the missing half — a [`Registrar`] trait plus adapters for the vendors
//! whose APIs actually support per-record DNS writes, which is the criterion the
//! platform needs because it writes A/AAAA/CAA/TXT on every domain it provisions.
//!
//! Adapter choice is deliberate:
//!
//! * **Porkbun** — the default. Open API, flat pricing, no renewal cliff.
//! * **Cloudflare Registrar** — at-cost, and one dashboard when Cloudflare DNS
//!   is already authoritative.
//! * **Dynadot** — breadth (805 TLDs) at flat renewal pricing.
//! * **deSEC** — DNSSEC-first, but DNS-only: no registration.
//! * **Namecheap** — selectable, but its only DNS write replaces the entire
//!   zone, which is why [`namecheap`] refuses to write when it cannot first read
//!   the current record set.

pub mod cloudflare;
pub mod desec;
pub mod dynadot;
pub mod http;
pub mod namecheap;
pub mod porkbun;
pub mod registry;
pub mod types;
pub mod xml;
pub mod zone;

pub use registry::{
    RegistrarOptions, available_registrars, configured_registrar, default_registrar,
    registrar_for_name,
};
pub use types::{
    Availability, Credential, DEFAULT_TTL, DnsRecord, DomainSpec, RecordType, RegistrarError,
    RegistrarInfo, Registration, TldPrice,
};

use async_trait::async_trait;

/// Lifecycle and DNS operations the platform needs from a registrar.
///
/// Every method takes a [`Credential`] explicitly rather than holding one, so an
/// adapter is stateless and a per-organization credential never leaks into a
/// shared instance.
#[async_trait]
pub trait Registrar: Send + Sync {
    /// Stable identifier used in configuration.
    fn name(&self) -> &str;

    /// What this adapter can do, for the UI and for pre-flight checks.
    fn info(&self) -> RegistrarInfo;

    /// Probes whether a name can be registered.
    async fn check(&self, domain: &str, credential: &Credential)
        -> Result<Availability, RegistrarError>;

    /// Registers a name for the requested term.
    async fn register(
        &self,
        spec: &DomainSpec,
        credential: &Credential,
    ) -> Result<Registration, RegistrarError>;

    /// Extends an existing registration.
    async fn renew(
        &self,
        domain: &str,
        years: u32,
        credential: &Credential,
    ) -> Result<Registration, RegistrarError>;

    /// Starts an inbound transfer to this account.
    async fn transfer(
        &self,
        domain: &str,
        credential: &Credential,
    ) -> Result<Registration, RegistrarError>;

    /// Delegates the zone to a nameserver set.
    async fn set_nameservers(
        &self,
        domain: &str,
        nameservers: &[String],
        credential: &Credential,
    ) -> Result<(), RegistrarError>;

    /// Reads one record. `Ok(None)` means "no such record", which is different
    /// from the zone being unreadable.
    async fn get_dns_record(
        &self,
        domain: &str,
        host: &str,
        credential: &Credential,
    ) -> Result<Option<DnsRecord>, RegistrarError>;

    /// Creates or replaces one record.
    async fn set_dns_record(
        &self,
        domain: &str,
        record: &DnsRecord,
        credential: &Credential,
    ) -> Result<DnsRecord, RegistrarError>;

    /// Removes one record by its registrar-assigned id.
    async fn delete_dns_record(
        &self,
        domain: &str,
        record_id: &str,
        credential: &Credential,
    ) -> Result<(), RegistrarError>;

    /// Published prices, so the store can show what a TLD actually costs.
    async fn tld_list(&self, credential: &Credential) -> Result<Vec<TldPrice>, RegistrarError>;
}
