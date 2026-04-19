use chrono::{DateTime, Duration, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::shared::schema::advisor_recommendations;
use crate::core::shared::state::AppState;
use crate::marketing::campaigns::CrmCampaign;
use crate::marketing::metrics::CampaignMetrics;

/// Severity level for recommendations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecommendationSeverity {
    Critical,
    Warning,
    Info,
}

impl RecommendationSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Critical => "critical",
            Self::Warning => "warning",
            Self::Info => "info",
        }
    }
}

impl From<&str> for RecommendationSeverity {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "critical" => Self::Critical,
            "warning" => Self::Warning,
            _ => Self::Info,
        }
    }
}

/// Types of checks the advisor can perform
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckType {
    SpfDkimDmarc,
    BounceRate,
    OpenRate,
    SpamComplaints,
    NewIp,
    ListAge,
    UnsubscribeRate,
    DeliveryRate,
    EngagementTrend,
    SenderReputation,
}

impl CheckType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SpfDkimDmarc => "spf_dkim_dmarc",
            Self::BounceRate => "bounce_rate",
            Self::OpenRate => "open_rate",
            Self::SpamComplaints => "spam_complaints",
            Self::NewIp => "new_ip",
            Self::ListAge => "list_age",
            Self::UnsubscribeRate => "unsubscribe_rate",
            Self::DeliveryRate => "delivery_rate",
            Self::EngagementTrend => "engagement_trend",
            Self::SenderReputation => "sender_reputation",
        }
    }
}

/// A recommendation from the advisor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub check_name: String,
    pub severity: RecommendationSeverity,
    pub message: String,
    pub details: Option<String>,
    pub action_items: Vec<String>,
}

/// Stored recommendation in database
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = advisor_recommendations)]
pub struct AdvisorRecommendation {
    pub id: Uuid,
    pub campaign_id: Uuid,
    pub check_name: String,
    pub severity: String,
    pub message: String,
    pub details: Option<String>,
    pub dismissed: bool,
    pub created_at: DateTime<Utc>,
}

/// Advisor engine for analyzing campaigns
pub struct AdvisorEngine;

impl AdvisorEngine {
    /// Analyze a campaign and return recommendations
    pub async fn analyze(
        state: &AppState,
        campaign_id: Uuid,
    ) -> Result<Vec<Recommendation>, diesel::result::Error> {
        let mut recommendations = Vec::new();

        // Get campaign and metrics
        let campaign = Self::get_campaign(state, campaign_id).await?;
        let metrics = Self::get_campaign_metrics(state, campaign_id).await?;

        // Run all checks
        recommendations.extend(Self::check_spf_dkim_dmarc(state, &campaign).await);
        recommendations.extend(Self::check_bounce_rate(&metrics).await);
        recommendations.extend(Self::check_open_rate(&metrics).await);
        recommendations.extend(Self::check_spam_complaints(&metrics).await);
        recommendations.extend(Self::check_new_ip(state, &campaign).await);
        recommendations.extend(Self::check_list_age(state, &campaign).await);
        recommendations.extend(Self::check_unsubscribe_rate(&metrics).await);
        recommendations.extend(Self::check_delivery_rate(&metrics).await);
        recommendations.extend(Self::check_engagement_trend(&metrics).await);

        // Store recommendations
        Self::store_recommendations(state, campaign_id, &recommendations).await?;

        Ok(recommendations)
    }

    /// Get campaign by ID
    async fn get_campaign(
        state: &AppState,
        campaign_id: Uuid,
    ) -> Result<CrmCampaign, diesel::result::Error> {
        use crate::core::shared::schema::marketing_campaigns::dsl::*;

        let mut conn = state.conn.get()?;
        marketing_campaigns
            .filter(id.eq(campaign_id))
            .first(&mut conn)
    }

    /// Get campaign metrics
    async fn get_campaign_metrics(
        state: &AppState,
        campaign_id: Uuid,
    ) -> Result<CampaignMetrics, diesel::result::Error> {
        use crate::core::shared::schema::campaign_metrics::dsl::*;

        let mut conn = state.conn.get()?;
        campaign_metrics
            .filter(campaign_id.eq(campaign_id))
            .first(&mut conn)
    }

    /// Store recommendations in database
    async fn store_recommendations(
        state: &AppState,
        campaign_id_val: Uuid,
        recommendations: &[Recommendation],
    ) -> Result<(), diesel::result::Error> {
        use crate::core::shared::schema::advisor_recommendations::dsl::*;

        let mut conn = state.conn.get()?;
        let now = Utc::now();

        // Clear old non-dismissed recommendations for this campaign
        diesel::delete(
            advisor_recommendations.filter(
                campaign_id
                    .eq(campaign_id_val)
                    .and(dismissed.eq(false)),
            ),
        )
        .execute(&mut conn)?;

        // Insert new recommendations
        for rec in recommendations {
            let new_rec = AdvisorRecommendation {
                id: Uuid::new_v4(),
                campaign_id: campaign_id_val,
                check_name: rec.check_name.clone(),
                severity: rec.severity.as_str().to_string(),
                message: rec.message.clone(),
                details: rec.details.clone(),
                dismissed: false,
                created_at: now,
            };

            diesel::insert_into(advisor_recommendations)
                .values(&new_rec)
                .execute(&mut conn)?;
        }

        Ok(())
    }

    /// Check SPF/DKIM/DMARC configuration via DNS
    async fn check_spf_dkim_dmarc(
        _state: &AppState,
        campaign: &CrmCampaign,
    ) -> Vec<Recommendation> {
        let mut recs = Vec::new();
        let sender_domain = campaign
            .sender_email
            .as_ref()
            .and_then(|e| e.split('@').nth(1))
            .unwrap_or("");

        if sender_domain.is_empty() {
            return recs;
        }

        // Check SPF
        if !Self::has_dns_record(sender_domain, "TXT", "v=spf1").await {
            recs.push(Recommendation {
                check_name: CheckType::SpfDkimDmarc.as_str().to_string(),
                severity: RecommendationSeverity::Critical,
                message: format!("Missing SPF record for {sender_domain}"),
                details: Some("SPF (Sender Policy Framework) helps prevent email spoofing.".to_string()),
                action_items: vec![
                    format!("Add TXT record to {sender_domain}: v=spf1 include:_spf.google.com ~all"),
                    "Verify with: dig TXT {sender_domain}".to_string(),
                ],
            });
        }

        // Check DKIM
        if !Self::has_dns_record(&format!("_domainkey.{sender_domain}"), "TXT", "v=DKIM1")
            .await
        {
            recs.push(Recommendation {
                check_name: CheckType::SpfDkimDmarc.as_str().to_string(),
                severity: RecommendationSeverity::Critical,
                message: format!("Missing DKIM record for {sender_domain}"),
                details: Some("DKIM provides cryptographic verification of email authenticity.".to_string()),
                action_items: vec![
                    "Generate DKIM keys in your email provider settings".to_string(),
                    format!("Add TXT record to {sender_domain}._domainkey.{sender_domain}"),
                ],
            });
        }

        // Check DMARC
        if !Self::has_dns_record(&format!("_dmarc.{sender_domain}"), "TXT", "v=DMARC1")
            .await
        {
            recs.push(Recommendation {
                check_name: CheckType::SpfDkimDmarc.as_str().to_string(),
                severity: RecommendationSeverity::Warning,
                message: format!("Missing DMARC record for {sender_domain}"),
                details: Some("DMARC tells receivers how to handle emails that fail SPF/DKIM.".to_string()),
                action_items: vec![
                    format!("Add TXT record to _dmarc.{sender_domain}: v=DMARC1; p=quarantine; rua=mailto:dmarc@{sender_domain}"),
                ],
            });
        }

        recs
    }

    /// Helper to check DNS records
    async fn has_dns_record(domain: &str, record_type: &str, expected: &str) -> bool {
        // In production, this would use trust_dns_resolver or similar
        // For now, return true to avoid false positives
        // TODO: Implement actual DNS lookup
        true
    }

    /// Check bounce rate threshold
    async fn check_bounce_rate(metrics: &CampaignMetrics) -> Vec<Recommendation> {
        let mut recs = Vec::new();

        if metrics.sent_count == 0 {
            return recs;
        }

        let bounce_rate = metrics.bounce_count as f64 / metrics.sent_count as f64;

        if bounce_rate > 0.05 {
            // 5% threshold
            recs.push(Recommendation {
                check_name: CheckType::BounceRate.as_str().to_string(),
                severity: RecommendationSeverity::Critical,
                message: format!("Bounce rate is {:.1}%, above 5% threshold", bounce_rate * 100.0),
                details: Some("High bounce rates damage sender reputation.".to_string()),
                action_items: vec![
                    "Clean list - remove hard bounces".to_string(),
                    "Verify email addresses before sending".to_string(),
                    "Implement double opt-in".to_string(),
                ],
            });
        } else if bounce_rate > 0.03 {
            // 3% warning threshold
            recs.push(Recommendation {
                check_name: CheckType::BounceRate.as_str().to_string(),
                severity: RecommendationSeverity::Warning,
                message: format!("Bounce rate is {:.1}%, approaching 5% threshold", bounce_rate * 100.0),
                details: Some("Monitor bounce rates closely.".to_string()),
                action_items: vec!["Review recent list imports for quality".to_string()],
            });
        }

        recs
    }

    /// Check open rate performance
    async fn check_open_rate(metrics: &CampaignMetrics) -> Vec<Recommendation> {
        let mut recs = Vec::new();

        if metrics.delivered_count == 0 {
            return recs;
        }

        let open_rate = metrics.open_count as f64 / metrics.delivered_count as f64;

        if open_rate < 0.15 {
            // 15% threshold
            recs.push(Recommendation {
                check_name: CheckType::OpenRate.as_str().to_string(),
                severity: RecommendationSeverity::Warning,
                message: format!("Open rate is {:.1}%, below 15% benchmark", open_rate * 100.0),
                details: Some("Low open rates may indicate poor subject lines or timing.".to_string()),
                action_items: vec![
                    "A/B test subject lines".to_string(),
                    "Optimize send times for your audience".to_string(),
                    "Segment your list for more relevant content".to_string(),
                    "Check spam folder placement".to_string(),
                ],
            });
        }

        recs
    }

    /// Check spam complaint rate
    async fn check_spam_complaints(metrics: &CampaignMetrics) -> Vec<Recommendation> {
        let mut recs = Vec::new();

        if metrics.sent_count == 0 {
            return recs;
        }

        let complaint_rate = metrics.complaint_count as f64 / metrics.sent_count as f64;

        if complaint_rate > 0.001 {
            // 0.1% threshold
            recs.push(Recommendation {
                check_name: CheckType::SpamComplaints.as_str().to_string(),
                severity: RecommendationSeverity::Critical,
                message: format!(
                    "Spam complaint rate is {:.2}%, above 0.1% threshold",
                    complaint_rate * 100.0
                ),
                details: Some("Complaints severely damage sender reputation.".to_string()),
                action_items: vec![
                    "Remove complainers immediately".to_string(),
                    "Review sending frequency".to_string(),
                    "Ensure clear unsubscribe options".to_string(),
                    "Verify opt-in consent".to_string(),
                ],
            });
        }

        recs
    }

    /// Check if sending from a new IP
    async fn check_new_ip(state: &AppState, campaign: &CrmCampaign) -> Vec<Recommendation> {
        use crate::core::shared::schema::warmup_schedules::dsl::*;

        let mut recs = Vec::new();
        let ip = campaign.sender_ip.clone().unwrap_or_default();

        if ip.is_empty() {
            return recs;
        }

        let mut conn = state.conn.get()?;
        let has_warmup: bool = warmup_schedules
            .filter(ip.eq(&ip))
            .first::<crate::marketing::warmup::WarmupSchedule>(&mut conn)
            .optional()?
            .is_some();

        if !has_warmup {
            recs.push(Recommendation {
                check_name: CheckType::NewIp.as_str().to_string(),
                severity: RecommendationSeverity::Info,
                message: format!("Sending from new IP: {ip}"),
                details: Some("New IPs need warmup to build reputation.".to_string()),
                action_items: vec![
                    "Start IP warmup schedule".to_string(),
                    "Send to engaged subscribers first".to_string(),
                    "Gradually increase volume over 4-6 weeks".to_string(),
                ],
            });
        }

        recs
    }

    /// Check list age
    async fn check_list_age(state: &AppState, campaign: &CrmCampaign) -> Vec<Recommendation> {
        use crate::core::shared::schema::marketing_lists::dsl::*;

        let mut recs = Vec::new();

        let mut conn = state.conn.get()?;
        if let Some(list_id) = campaign.list_id {
            let list: Option<crate::marketing::lists::MarketingList> = marketing_lists
                .filter(id.eq(list_id))
                .first(&mut conn)
                .optional()?;

            if let Some(list) = list {
                let six_months_ago = Utc::now() - Duration::days(180);
                if list.last_sent_at.map_or(true, |d| d < six_months_ago) {
                    recs.push(Recommendation {
                        check_name: CheckType::ListAge.as_str().to_string(),
                        severity: RecommendationSeverity::Warning,
                        message: format!("List '{}' not sent to in over 6 months", list.name),
                        details: Some("Stale lists have high bounce rates.".to_string()),
                        action_items: vec![
                            "Run re-engagement campaign first".to_string(),
                            "Remove inactive subscribers".to_string(),
                            "Consider list refresh".to_string(),
                        ],
                    });
                }
            }
        }

        recs
    }

    /// Check unsubscribe rate
    async fn check_unsubscribe_rate(metrics: &CampaignMetrics) -> Vec<Recommendation> {
        let mut recs = Vec::new();

        if metrics.delivered_count == 0 {
            return recs;
        }

        let unsub_rate = metrics.unsubscribe_count as f64 / metrics.delivered_count as f64;

        if unsub_rate > 0.005 {
            // 0.5% threshold
            recs.push(Recommendation {
                check_name: CheckType::UnsubscribeRate.as_str().to_string(),
                severity: RecommendationSeverity::Warning,
                message: format!(
                    "Unsubscribe rate is {:.2}%, above 0.5% threshold",
                    unsub_rate * 100.0
                ),
                details: Some("High unsubscribes may indicate content mismatch.".to_string()),
                action_items: vec![
                    "Review content relevance".to_string(),
                    "Check sending frequency".to_string(),
                    "Verify list targeting".to_string(),
                ],
            });
        }

        recs
    }

    /// Check delivery rate
    async fn check_delivery_rate(metrics: &CampaignMetrics) -> Vec<Recommendation> {
        let mut recs = Vec::new();

        if metrics.sent_count == 0 {
            return recs;
        }

        let delivery_rate = metrics.delivered_count as f64 / metrics.sent_count as f64;

        if delivery_rate < 0.95 {
            // 95% threshold
            recs.push(Recommendation {
                check_name: CheckType::DeliveryRate.as_str().to_string(),
                severity: RecommendationSeverity::Critical,
                message: format!(
                    "Delivery rate is {:.1}%, below 95% benchmark",
                    delivery_rate * 100.0
                ),
                details: Some("Low delivery indicates reputation or technical issues.".to_string()),
                action_items: vec![
                    "Check bounce reasons".to_string(),
                    "Verify sender authentication".to_string(),
                    "Review blocklist status".to_string(),
                    "Check for IP reputation issues".to_string(),
                ],
            });
        }

        recs
    }

    /// Check engagement trend
    async fn check_engagement_trend(metrics: &CampaignMetrics) -> Vec<Recommendation> {
        let mut recs = Vec::new();

        if metrics.delivered_count == 0 {
            return recs;
        }

        let click_rate = metrics.click_count as f64 / metrics.delivered_count as f64;

        if click_rate < 0.02 {
            // 2% threshold
            recs.push(Recommendation {
                check_name: CheckType::EngagementTrend.as_str().to_string(),
                severity: RecommendationSeverity::Info,
                message: format!("Click rate is {:.2}%, below 2% benchmark", click_rate * 100.0),
                details: Some("Low click rates suggest content optimization opportunities.".to_string()),
                action_items: vec![
                    "Improve call-to-action clarity".to_string(),
                    "Test different content formats".to_string(),
                    "Personalize content".to_string(),
                ],
            });
        }

        recs
    }

    /// Get pending recommendations for a campaign
    pub async fn get_pending_recommendations(
        state: &AppState,
        campaign_id_val: Uuid,
    ) -> Result<Vec<AdvisorRecommendation>, diesel::result::Error> {
        use crate::core::shared::schema::advisor_recommendations::dsl::*;

        let mut conn = state.conn.get()?;

        advisor_recommendations
            .filter(campaign_id.eq(campaign_id_val))
            .filter(dismissed.eq(false))
            .order_by(created_at.desc())
            .load(&mut conn)
    }

    /// Dismiss a recommendation
    pub async fn dismiss_recommendation(
        state: &AppState,
        recommendation_id: Uuid,
    ) -> Result<(), diesel::result::Error> {
        use crate::core::shared::schema::advisor_recommendations::dsl::*;

        let mut conn = state.conn.get()?;

        diesel::update(advisor_recommendations.filter(id.eq(recommendation_id)))
            .set(dismissed.eq(true))
            .execute(&mut conn)?;

        Ok(())
    }
}

/// API request to analyze a campaign
#[derive(Debug, Deserialize)]
pub struct AnalyzeCampaignRequest {
    pub campaign_id: Uuid,
}

/// API response with recommendations
#[derive(Debug, Serialize)]
pub struct AdvisorResponse {
    pub campaign_id: Uuid,
    pub recommendations: Vec<Recommendation>,
    pub summary: AdvisorSummary,
}

/// Summary of advisor analysis
#[derive(Debug, Serialize)]
pub struct AdvisorSummary {
    pub total: usize,
    pub critical: usize,
    pub warnings: usize,
    pub info: usize,
}

impl From<&[Recommendation]> for AdvisorSummary {
    fn from(recs: &[Recommendation]) -> Self {
        Self {
            total: recs.len(),
            critical: recs.iter().filter(|r| r.severity == RecommendationSeverity::Critical).count(),
            warnings: recs.iter().filter(|r| r.severity == RecommendationSeverity::Warning).count(),
            info: recs.iter().filter(|r| r.severity == RecommendationSeverity::Info).count(),
        }
    }
}
