use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use uuid::Uuid;

use crate::schema::{ip_reputation, ip_rotations, org_ips};
use crate::state::AppState;

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

#[derive(Debug, Clone, Serialize)]
pub struct ScoredIp {
    pub ip: IpAddr,
    pub score: f64,
    pub delivery_rate: f64,
    pub bounce_rate: f64,
    pub complaint_rate: f64,
    pub provider: String,
}

#[derive(Debug)]
pub enum IpRouterError {
    Database(diesel::result::Error),
    Pool(diesel::r2d2::PoolError),
    NoAvailableIps,
    InvalidIp(String),
}

impl std::fmt::Display for IpRouterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Database(e) => write!(f, "Database error: {e}"),
            Self::Pool(e) => write!(f, "Pool error: {e}"),
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

impl From<diesel::r2d2::PoolError> for IpRouterError {
    fn from(e: diesel::r2d2::PoolError) -> Self {
        Self::Pool(e)
    }
}

pub struct IpRouter {
    state: Arc<AppState>,
    org_id: Uuid,
}

impl IpRouter {
    pub fn new(state: Arc<AppState>, org_id: Uuid) -> Self {
        Self { state, org_id }
    }

    pub async fn select(&self, destination_domain: &str) -> Result<IpAddr, IpRouterError> {
        let provider = Self::classify_provider(destination_domain);
        let available_ips = self.get_available_ips()?;

        if available_ips.is_empty() {
            return Err(IpRouterError::NoAvailableIps);
        }

        let mut scored_ips = Vec::new();
        for ip in available_ips {
            let reputation = self.get_ip_reputation(&ip, &provider)?;
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

        scored_ips.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        scored_ips
            .first()
            .map(|s| s.ip)
            .ok_or(IpRouterError::NoAvailableIps)
    }

    pub async fn select_with_load_balancing(
        &self,
        destination_domain: &str,
    ) -> Result<IpAddr, IpRouterError> {
        let provider = Self::classify_provider(destination_domain);
        let available_ips = self.get_available_ips()?;

        if available_ips.is_empty() {
            return Err(IpRouterError::NoAvailableIps);
        }

        let mut scored_ips = Vec::new();
        for ip in &available_ips {
            let reputation = self.get_ip_reputation(ip, &provider)?;
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

        let mut score_groups: HashMap<u64, Vec<ScoredIp>> = HashMap::new();
        for ip in scored_ips {
            let score_key = (ip.score * 100.0) as u64;
            score_groups.entry(score_key).or_default().push(ip);
        }

        let max_score = score_groups.keys().copied().max().unwrap_or(0);
        let top_group = score_groups.get(&max_score).unwrap();

        self.round_robin_select(top_group)
    }

    fn get_available_ips(&self) -> Result<Vec<IpAddr>, IpRouterError> {
        let mut conn = self.state.conn.get()?;

        let ip_strings: Vec<String> = org_ips::table
            .filter(org_ips::org_id.eq(self.org_id))
            .filter(org_ips::is_active.eq(true))
            .select(org_ips::ip_address)
            .load(&mut conn)?;

        ip_strings
            .into_iter()
            .map(|s| s.parse().map_err(|_| IpRouterError::InvalidIp(s)))
            .collect::<Result<Vec<_>, _>>()
    }

    fn get_ip_reputation(
        &self,
        ip_addr: &IpAddr,
        provider_name: &str,
    ) -> Result<IpReputation, IpRouterError> {
        let mut conn = self.state.conn.get()?;
        let ip_str = ip_addr.to_string();

        let reputation: Option<IpReputation> = ip_reputation::table
            .filter(ip_reputation::ip.eq(&ip_str))
            .filter(ip_reputation::provider.eq(provider_name))
            .first(&mut conn)
            .optional()?;

        match reputation {
            Some(r) => Ok(r),
            None => {
                let new_rep = IpReputation {
                    id: Uuid::new_v4(),
                    org_id: self.org_id,
                    ip: ip_str,
                    provider: provider_name.to_string(),
                    delivered: 0,
                    bounced: 0,
                    complained: 0,
                    window_start: Utc::now(),
                    updated_at: Some(Utc::now()),
                };

                diesel::insert_into(ip_reputation::table)
                    .values(&new_rep)
                    .execute(&mut conn)?;

                Ok(new_rep)
            }
        }
    }

    pub fn calculate_score(reputation: &IpReputation) -> f64 {
        let delivery_rate = Self::calculate_delivery_rate(reputation);
        let bounce_rate = Self::calculate_bounce_rate(reputation);
        let complaint_rate = Self::calculate_complaint_rate(reputation);

        let score = delivery_rate - (bounce_rate * 10.0) - (complaint_rate * 100.0);
        score.max(0.0)
    }

    pub fn calculate_delivery_rate(reputation: &IpReputation) -> f64 {
        let total = reputation.delivered + reputation.bounced;
        if total == 0 {
            return 1.0;
        }
        reputation.delivered as f64 / total as f64
    }

    pub fn calculate_bounce_rate(reputation: &IpReputation) -> f64 {
        let total = reputation.delivered + reputation.bounced;
        if total == 0 {
            return 0.0;
        }
        reputation.bounced as f64 / total as f64
    }

    pub fn calculate_complaint_rate(reputation: &IpReputation) -> f64 {
        if reputation.delivered == 0 {
            return 0.0;
        }
        reputation.complained as f64 / reputation.delivered as f64
    }

    fn round_robin_select(
        &self,
        group: &[ScoredIp],
    ) -> Result<IpAddr, IpRouterError> {
        let mut conn = self.state.conn.get()?;
        let now = Utc::now();

        let ip_strings: Vec<String> = group.iter().map(|s| s.ip.to_string()).collect();

        let next_ip: Option<String> = ip_rotations::table
            .filter(ip_rotations::ip_address.eq_any(&ip_strings))
            .filter(ip_rotations::org_id.eq(self.org_id))
            .order(ip_rotations::last_used.asc())
            .select(ip_rotations::ip_address)
            .first(&mut conn)
            .optional()?;

        match next_ip {
            Some(ip_str) => {
                diesel::update(ip_rotations::table.filter(ip_rotations::ip_address.eq(&ip_str)))
                    .set(ip_rotations::last_used.eq(now))
                    .execute(&mut conn)?;

                ip_str
                    .parse()
                    .map_err(|_| IpRouterError::InvalidIp(ip_str))
            }
            None => group
                .first()
                .map(|s| s.ip)
                .ok_or(IpRouterError::NoAvailableIps),
        }
    }

    pub fn classify_provider(domain: &str) -> String {
        let domain_lower = domain.to_lowercase();

        if domain_lower.contains("gmail") || domain_lower.contains("google") {
            "gmail".to_string()
        } else if domain_lower.contains("outlook")
            || domain_lower.contains("hotmail")
            || domain_lower.contains("live")
            || domain_lower.contains("msn")
        {
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

    pub async fn update_reputation(
        &self,
        ip_addr: &IpAddr,
        provider_name: &str,
        delivered: i64,
        bounced: i64,
        complained: i64,
    ) -> Result<(), IpRouterError> {
        let mut conn = self.state.conn.get()?;
        let ip_str = ip_addr.to_string();
        let now = Utc::now();

        let existing: Option<IpReputation> = ip_reputation::table
            .filter(ip_reputation::ip.eq(&ip_str))
            .filter(ip_reputation::provider.eq(provider_name))
            .first(&mut conn)
            .optional()?;

        if let Some(mut rep) = existing {
            let window_duration = chrono::Duration::hours(24);
            if now - rep.window_start > window_duration {
                rep.delivered = delivered;
                rep.bounced = bounced;
                rep.complained = complained;
                rep.window_start = now;
            } else {
                rep.delivered += delivered;
                rep.bounced += bounced;
                rep.complained += complained;
            }
            rep.updated_at = Some(now);

            diesel::update(ip_reputation::table.filter(ip_reputation::id.eq(rep.id)))
                .set(&rep)
                .execute(&mut conn)?;
        } else {
            let new_rep = IpReputation {
                id: Uuid::new_v4(),
                org_id: self.org_id,
                ip: ip_str,
                provider: provider_name.to_string(),
                delivered,
                bounced,
                complained,
                window_start: now,
                updated_at: Some(now),
            };

            diesel::insert_into(ip_reputation::table)
                .values(&new_rep)
                .execute(&mut conn)?;
        }

        Ok(())
    }

    pub async fn get_reputation_report(
        &self,
    ) -> Result<Vec<IpReputationReport>, IpRouterError> {
        let mut conn = self.state.conn.get()?;

        let reputations: Vec<IpReputation> = ip_reputation::table
            .filter(ip_reputation::org_id.eq(self.org_id))
            .load(&mut conn)?;

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

#[derive(Debug, Deserialize)]
pub struct SelectIpRequest {
    pub destination_domain: String,
}

#[derive(Debug, Serialize)]
pub struct SelectIpResponse {
    pub selected_ip: String,
    pub provider: String,
    pub score: f64,
    pub delivery_rate: f64,
    pub bounce_rate: f64,
    pub complaint_rate: f64,
}

#[derive(Debug, Deserialize)]
pub struct UpdateReputationRequest {
    pub ip: String,
    pub provider: String,
    pub delivered: i64,
    pub bounced: i64,
    pub complained: i64,
}

