use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use uuid::Uuid;

use crate::core::shared::schema::ip_reputation;
use crate::core::shared::state::AppState;

/// IP reputation tracking for optimized delivery
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = ip_reputation)]
pub struct IpReputation {
    pub id: Uuid,
    pub org_id: Uuid,
    pub ip: String,
    pub provider: String,
    pub delivered: i64,
    pub bounced: i64,
    pub complained: i64,
    pub window_start: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Scored IP for routing decision
#[derive(Debug, Clone, Serialize)]
pub struct ScoredIp {
    pub ip: IpAddr,
    pub score: f64,
    pub delivery_rate: f64,
    pub bounce_rate: f64,
    pub complaint_rate: f64,
    pub provider: String,
}

/// IP Router for optimized email delivery
pub struct IpRouter {
    state: std::sync::Arc<AppState>,
    org_id: Uuid,
}

impl IpRouter {
    /// Create new IP router
    pub fn new(state: std::sync::Arc<AppState>, org_id: Uuid) -> Self {
        Self { state, org_id }
    }

    /// Select best IP for sending to a destination domain
    pub async fn select(&self, destination_domain: &str) -> Result<IpAddr, IpRouterError> {
        let provider = Self::classify_provider(destination_domain);
        let available_ips = self.get_available_ips().await?;

        if available_ips.is_empty() {
            return Err(IpRouterError::NoAvailableIps);
        }

        // Get reputation data for each IP
        let mut scored_ips = Vec::new();
        for ip in available_ips {
            let reputation = self.get_ip_reputation(&ip, &provider).await?;
            let score = Self::calculate_score(&reputation);

            scored_ips.push(ScoredIp {
                ip,
                score,
                delivery_rate: Self::calculate_delivery_rate(&reputation),
                bounce_rate: Self::calculate_bounce_rate(&reputation),
                complaint_rate: Self::calculate_complaint_rate(&reputation),
                provider: provider.clone(),
            });
        }

        // Sort by score descending
        scored_ips.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // Return highest scored IP
        scored_ips
            .first()
            .map(|s| s.ip)
            .ok_or(IpRouterError::NoAvailableIps)
    }

    /// Select IP with load balancing (round-robin for same score)
    pub async fn select_with_load_balancing(
        &self,
        destination_domain: &str,
    ) -> Result<IpAddr, IpRouterError> {
        let provider = Self::classify_provider(destination_domain);
        let available_ips = self.get_available_ips().await?;

        if available_ips.is_empty() {
            return Err(IpRouterError::NoAvailableIps);
        }

        // Get reputation data and scores
        let mut scored_ips = Vec::new();
        for ip in &available_ips {
            let reputation = self.get_ip_reputation(ip, &provider).await?;
            let score = Self::calculate_score(&reputation);

            scored_ips.push(ScoredIp {
                ip: *ip,
                score,
                delivery_rate: Self::calculate_delivery_rate(&reputation),
                bounce_rate: Self::calculate_bounce_rate(&reputation),
                complaint_rate: Self::calculate_complaint_rate(&reputation),
                provider: provider.clone(),
            });
        }

        // Group by score (rounded to 2 decimal places)
        let mut score_groups: HashMap<u64, Vec<ScoredIp>> = HashMap::new();
        for ip in scored_ips {
            let score_key = (ip.score * 100.0) as u64;
            score_groups.entry(score_key).or_default().push(ip);
        }

        // Get highest score group
        let max_score = score_groups.keys().copied().max().unwrap_or(0);
        let top_group = score_groups.get(&max_score).unwrap();

        // Round-robin within top group
        self.round_robin_select(top_group).await
    }

    /// Get available sending IPs for the organization
    async fn get_available_ips(&self) -> Result<Vec<IpAddr>, IpRouterError> {
        use crate::core::shared::schema::org_ips::dsl::*;

        let mut conn = self.state.conn.get().map_err(IpRouterError::Database)?;

        let ip_strings: Vec<String> = org_ips
            .filter(org_id.eq(self.org_id))
            .filter(is_active.eq(true))
            .select(ip_address)
            .load(&mut conn)
            .map_err(IpRouterError::Database)?;

        ip_strings
            .into_iter()
            .map(|s| s.parse().map_err(|_| IpRouterError::InvalidIp(s)))
            .collect::<Result<Vec<_>, _>>()
    }

    /// Get or create reputation record for an IP
    async fn get_ip_reputation(
        &self,
        ip: &IpAddr,
        provider: &str,
    ) -> Result<IpReputation, IpRouterError> {
        use crate::core::shared::schema::ip_reputation::dsl::*;

        let mut conn = self.state.conn.get().map_err(IpRouterError::Database)?;
        let ip_str = ip.to_string();

        let reputation: Option<IpReputation> = ip_reputation
            .filter(ip.eq(&ip_str))
            .filter(provider.eq(provider))
            .first(&mut conn)
            .optional()
            .map_err(IpRouterError::Database)?;

        match reputation {
            Some(r) => Ok(r),
            None => {
                // Create new reputation record
                let new_rep = IpReputation {
                    id: Uuid::new_v4(),
                    org_id: self.org_id,
                    ip: ip_str,
                    provider: provider.to_string(),
                    delivered: 0,
                    bounced: 0,
                    complained: 0,
                    window_start: Utc::now(),
                    updated_at: Some(Utc::now()),
                };

                diesel::insert_into(ip_reputation)
                    .values(&new_rep)
                    .execute(&mut conn)
                    .map_err(IpRouterError::Database)?;

                Ok(new_rep)
            }
        }
    }

    /// Calculate IP score: delivery_rate - (bounce_rate * 10) - (complaint_rate * 100)
    fn calculate_score(reputation: &IpReputation) -> f64 {
        let delivery_rate = Self::calculate_delivery_rate(reputation);
        let bounce_rate = Self::calculate_bounce_rate(reputation);
        let complaint_rate = Self::calculate_complaint_rate(reputation);

        let score = delivery_rate - (bounce_rate * 10.0) - (complaint_rate * 100.0);
        score.max(0.0)
    }

    /// Calculate delivery rate
    fn calculate_delivery_rate(reputation: &IpReputation) -> f64 {
        let total = reputation.delivered + reputation.bounced;
        if total == 0 {
            return 1.0; // Default to 100% for new IPs
        }
        reputation.delivered as f64 / total as f64
    }

    /// Calculate bounce rate
    fn calculate_bounce_rate(reputation: &IpReputation) -> f64 {
        let total = reputation.delivered + reputation.bounced;
        if total == 0 {
            return 0.0;
        }
        reputation.bounced as f64 / total as f64
    }

    /// Calculate complaint rate
    fn calculate_complaint_rate(reputation: &IpReputation) -> f64 {
        if reputation.delivered == 0 {
            return 0.0;
        }
        reputation.complained as f64 / reputation.delivered as f64
    }

    /// Round-robin selection from a group of IPs
    async fn round_robin_select(&self, group: &[ScoredIp]) -> Result<IpAddr, IpRouterError> {
        use crate::core::shared::schema::ip_rotations::dsl::*;

        let mut conn = self.state.conn.get().map_err(IpRouterError::Database)?;
        let now = Utc::now();

        // Find the IP with oldest last_used timestamp
        let ip_strings: Vec<String> = group.iter().map(|s| s.ip.to_string()).collect();

        let next_ip: Option<String> = ip_rotations
            .filter(ip_address.eq_any(&ip_strings))
            .filter(org_id.eq(self.org_id))
            .order_by(last_used.asc())
            .select(ip_address)
            .first(&mut conn)
            .optional()
            .map_err(IpRouterError::Database)?;

        match next_ip {
            Some(ip_str) => {
                // Update last_used
                diesel::update(ip_rotations.filter(ip_address.eq(&ip_str)))
                    .set(last_used.eq(now))
                    .execute(&mut conn)
                    .map_err(IpRouterError::Database)?;

                ip_str.parse().map_err(|_| IpRouterError::InvalidIp(ip_str))
            }
            None => {
                // No rotation record, use first IP
                group
                    .first()
                    .map(|s| s.ip)
                    .ok_or(IpRouterError::NoAvailableIps)
            }
        }
    }

    /// Classify email provider from domain
    fn classify_provider(domain: &str) -> String {
        let domain_lower = domain.to_lowercase();

        if domain_lower.contains("gmail") || domain_lower.contains("google") {
            "gmail".to_string()
        } else if domain_lower.contains("outlook") || domain_lower.contains("hotmail") || domain_lower.contains("live") || domain_lower.contains("msn") {
            "outlook".to_string()
        } else if domain_lower.contains("yahoo") {
            "yahoo".to_string()
        } else if domain_lower.contains("icloud") || domain_lower.contains("me.") {
            "icloud".to_string()
        } else if domain_lower.contains("proton") {
            "proton".to_string()
        } else {
            "other".to_string()
        }
    }

    /// Update reputation metrics after sending
    pub async fn update_reputation(
        &self,
        ip: &IpAddr,
        provider: &str,
        delivered: i64,
        bounced: i64,
        complained: i64,
    ) -> Result<(), IpRouterError> {
        use crate::core::shared::schema::ip_reputation::dsl::*;

        let mut conn = self.state.conn.get().map_err(IpRouterError::Database)?;
        let ip_str = ip.to_string();
        let now = Utc::now();

        // Check if record exists
        let existing: Option<IpReputation> = ip_reputation
            .filter(ip.eq(&ip_str))
            .filter(provider.eq(provider))
            .first(&mut conn)
            .optional()
            .map_err(IpRouterError::Database)?;

        if let Some(mut rep) = existing {
            // Update rolling window - keep last 24h
            let window_duration = chrono::Duration::hours(24);
            if now - rep.window_start > window_duration {
                // Reset window
                rep.delivered = delivered;
                rep.bounced = bounced;
                rep.complained = complained;
                rep.window_start = now;
            } else {
                // Add to existing
                rep.delivered += delivered;
                rep.bounced += bounced;
                rep.complained += complained;
            }
            rep.updated_at = Some(now);

            diesel::update(ip_reputation.filter(id.eq(rep.id)))
                .set(&rep)
                .execute(&mut conn)
                .map_err(IpRouterError::Database)?;
        } else {
            // Create new record
            let new_rep = IpReputation {
                id: Uuid::new_v4(),
                org_id: self.org_id,
                ip: ip_str,
                provider: provider.to_string(),
                delivered,
                bounced,
                complained,
                window_start: now,
                updated_at: Some(now),
            };

            diesel::insert_into(ip_reputation)
                .values(&new_rep)
                .execute(&mut conn)
                .map_err(IpRouterError::Database)?;
        }

        Ok(())
    }

    /// Get reputation report for all IPs
    pub async fn get_reputation_report(
        &self,
    ) -> Result<Vec<IpReputationReport>, IpRouterError> {
        use crate::core::shared::schema::ip_reputation::dsl::*;

        let mut conn = self.state.conn.get().map_err(IpRouterError::Database)?;

        let reputations: Vec<IpReputation> = ip_reputation
            .filter(org_id.eq(self.org_id))
            .load(&mut conn)
            .map_err(IpRouterError::Database)?;

        let reports = reputations
            .into_iter()
            .map(|r| {
                let total = r.delivered + r.bounced;
                IpReputationReport {
                    ip: r.ip.clone(),
                    provider: r.provider.clone(),
                    delivered: r.delivered,
                    bounced: r.bounced,
                    complained: r.complained,
                    delivery_rate: if total > 0 {
                        r.delivered as f64 / total as f64
                    } else {
                        1.0
                    },
                    bounce_rate: if total > 0 {
                        r.bounced as f64 / total as f64
                    } else {
                        0.0
                    },
                    complaint_rate: if r.delivered > 0 {
                        r.complained as f64 / r.delivered as f64
                    } else {
                        0.0
                    },
                    score: Self::calculate_score(&r),
                    window_start: r.window_start,
                }
            })
            .collect();

        Ok(reports)
    }
}

/// IP Router errors
#[derive(Debug)]
pub enum IpRouterError {
    Database(diesel::result::Error),
    NoAvailableIps,
    InvalidIp(String),
}

impl std::fmt::Display for IpRouterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Database(e) => write!(f, "Database error: {e}"),
            Self::NoAvailableIps => write!(f, "No available IPs for routing"),
            Self::InvalidIp(s) => write!(f, "Invalid IP address: {s}"),
        }
    }
}

impl std::error::Error for IpRouterError {}

impl From<diesel::result::Error> for IpRouterError {
    fn from(e: diesel::result::Error) -> Self {
        Self::Database(e)
    }
}

/// Reputation report for display
#[derive(Debug, Serialize)]
pub struct IpReputationReport {
    pub ip: String,
    pub provider: String,
    pub delivered: i64,
    pub bounced: i64,
    pub complained: i64,
    pub delivery_rate: f64,
    pub bounce_rate: f64,
    pub complaint_rate: f64,
    pub score: f64,
    pub window_start: DateTime<Utc>,
}

/// API request for IP selection
#[derive(Debug, Deserialize)]
pub struct SelectIpRequest {
    pub destination_domain: String,
}

/// API response for IP selection
#[derive(Debug, Serialize)]
pub struct SelectIpResponse {
    pub selected_ip: String,
    pub provider: String,
    pub score: f64,
    pub delivery_rate: f64,
    pub bounce_rate: f64,
    pub complaint_rate: f64,
}

/// API request for reputation update
#[derive(Debug, Deserialize)]
pub struct UpdateReputationRequest {
    pub ip: String,
    pub provider: String,
    pub delivered: i64,
    pub bounced: i64,
    pub complained: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_provider() {
        assert_eq!(IpRouter::classify_provider("gmail.com"), "gmail");
        assert_eq!(IpRouter::classify_provider("outlook.com"), "outlook");
        assert_eq!(IpRouter::classify_provider("yahoo.com"), "yahoo");
        assert_eq!(IpRouter::classify_provider("icloud.com"), "icloud");
        assert_eq!(IpRouter::classify_provider("protonmail.com"), "proton");
        assert_eq!(IpRouter::classify_provider("example.com"), "other");
    }

    #[test]
    fn test_calculate_score() {
        let rep = IpReputation {
            id: Uuid::new_v4(),
            org_id: Uuid::new_v4(),
            ip: "10.0.0.1".to_string(),
            provider: "gmail".to_string(),
            delivered: 100,
            bounced: 5,
            complained: 0,
            window_start: Utc::now(),
            updated_at: Some(Utc::now()),
        };

        // delivery_rate = 100/105 = 0.952
        // bounce_rate = 5/105 = 0.0476
        // complaint_rate = 0
        // score = 0.952 - (0.0476 * 10) - 0 = 0.952 - 0.476 = 0.476
        let score = IpRouter::calculate_score(&rep);
        assert!(score > 0.0);

        // Test with complaints
        let rep2 = IpReputation {
            id: Uuid::new_v4(),
            org_id: Uuid::new_v4(),
            ip: "10.0.0.2".to_string(),
            provider: "gmail".to_string(),
            delivered: 1000,
            bounced: 50,
            complained: 5,
            window_start: Utc::now(),
            updated_at: Some(Utc::now()),
        };

        // complaint_rate = 5/1000 = 0.005
        // score penalty from complaints = 0.005 * 100 = 0.5
        let score2 = IpRouter::calculate_score(&rep2);
        assert!(score2 < score); // Complaints should lower score
    }
}
