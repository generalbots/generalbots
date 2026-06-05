//! Delivery platform (iFood, Rappi, Uber Eats, etc.) connector types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Delivery platform identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeliveryPlatform {
    /// iFood (Brazil).
    IFood,
    /// Rappi.
    Rappi,
    /// Uber Eats.
    UberEats,
    /// Mercado Livre / Mercado Pago Entregas.
    MercadoEntregas,
    /// Lalamove (logistics).
    Lalamove,
    /// Loggi.
    Loggi,
    /// Other / custom.
    Other,
}

impl DeliveryPlatform {
    /// Returns the typical settlement window in days.
    pub fn settlement_window_days(self) -> u32 {
        match self {
            Self::IFood | Self::Rappi | Self::UberEats => 14,
            Self::MercadoEntregas => 21,
            Self::Lalamove | Self::Loggi => 7,
            Self::Other => 14,
        }
    }
}

/// Kind of integration for the connector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectorKind {
    /// Pull-based, polled at `sync_interval_min`.
    Pull,
    /// Webhook-based (push).
    Webhook,
    /// Manual CSV upload.
    Manual,
}

/// Per-platform connector configuration stored in the Sources module.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformConnector {
    /// Server-assigned connector ID.
    pub id: Uuid,
    /// Tenant.
    pub tenant_id: String,
    /// Platform.
    pub platform: DeliveryPlatform,
    /// Display name.
    pub name: String,
    /// Kind.
    pub kind: ConnectorKind,
    /// Vault path holding the connector credentials.
    pub credentials_ref: String,
    /// When the connector was created.
    pub created_at: DateTime<Utc>,
    /// Last successful pull.
    pub last_pulled_at: Option<DateTime<Utc>>,
}

/// Errors raised by connectors.
#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum ConnectorError {
    /// HTTP failure.
    #[error("transport: {0}")]
    Transport(String),
    /// Authentication refused.
    #[error("auth: {0}")]
    Auth(String),
    /// Platform returned a 4xx/5xx.
    #[error("http {status}: {body}")]
    Http {
        /// Status.
        status: u16,
        /// Body.
        body: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settlement_window_defaults() {
        assert_eq!(DeliveryPlatform::IFood.settlement_window_days(), 14);
        assert_eq!(DeliveryPlatform::Loggi.settlement_window_days(), 7);
    }
}
