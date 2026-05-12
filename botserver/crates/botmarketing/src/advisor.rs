use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::campaigns::CrmCampaign;
use crate::schema::advisor_recommendations;
use crate::state::AppState;

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
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub check_name: String,
    pub severity: RecommendationSeverity,
    pub message: String,
    pub details: Option<String>,
    pub action_items: Vec<String>,
}

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

#[derive(Debug, Deserialize)]
pub struct AnalyzeCampaignRequest {
    pub campaign_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct AdvisorResponse {
    pub campaign_id: Uuid,
    pub recommendations: Vec<Recommendation>,
    pub summary: AdvisorSummary,
}

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
            critical: recs
                .iter()
                .filter(|r| r.severity == RecommendationSeverity::Critical)
                .count(),
            warnings: recs
                .iter()
                .filter(|r| r.severity == RecommendationSeverity::Warning)
                .count(),
            info: recs
                .iter()
                .filter(|r| r.severity == RecommendationSeverity::Info)
                .count(),
        }
    }
}

#[derive(Debug, Clone, Queryable)]
#[diesel(table_name = crate::schema::campaign_metrics)]
#[allow(dead_code)]
struct DbMetricsRow {
    campaign_id: Uuid,
    sent_count: i64,
    delivered_count: i64,
    bounce_count: i64,
    open_count: i64,
    click_count: i64,
    complaint_count: i64,
    unsubscribe_count: i64,
    reply_count: i64,
}

pub struct AdvisorEngine;

impl AdvisorEngine {
    pub async fn analyze(
        state: &AppState,
        campaign_id: Uuid,
    ) -> Result<Vec<Recommendation>, String> {
        let mut recommendations = Vec::new();

        let _campaign = Self::get_campaign(state, campaign_id).await?;
        let metrics = Self::get_campaign_metrics(state, campaign_id).await?;

        recommendations.extend(Self::check_bounce_rate(&metrics));
        recommendations.extend(Self::check_open_rate(&metrics));
        recommendations.extend(Self::check_spam_complaints(&metrics));
        recommendations.extend(Self::check_unsubscribe_rate(&metrics));
        recommendations.extend(Self::check_delivery_rate(&metrics));
        recommendations.extend(Self::check_engagement_trend(&metrics));

        Self::store_recommendations(state, campaign_id, &recommendations).await?;

        Ok(recommendations)
    }

    async fn get_campaign(
        state: &AppState,
        campaign_id: Uuid,
    ) -> Result<CrmCampaign, String> {
        use crate::schema::marketing_campaigns::dsl::*;
        let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
        marketing_campaigns
            .filter(id.eq(campaign_id))
            .first(&mut conn)
            .map_err(|e| format!("Campaign not found: {e}"))
    }

    async fn get_campaign_metrics(
        state: &AppState,
        campaign_id_val: Uuid,
    ) -> Result<DbMetricsRow, String> {
        use crate::schema::campaign_metrics::dsl::*;
        let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
        campaign_metrics
            .filter(campaign_id.eq(campaign_id_val))
            .first(&mut conn)
            .map_err(|e| format!("Metrics not found: {e}"))
    }

    async fn store_recommendations(
        state: &AppState,
        campaign_id_val: Uuid,
        recommendations: &[Recommendation],
    ) -> Result<(), String> {
        use crate::schema::advisor_recommendations::dsl::*;
        let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
        let now = Utc::now();

        diesel::delete(
            advisor_recommendations
                .filter(campaign_id.eq(campaign_id_val).and(dismissed.eq(false))),
        )
        .execute(&mut conn)
        .map_err(|e| format!("Delete error: {e}"))?;

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
                .execute(&mut conn)
                .map_err(|e| format!("Insert error: {e}"))?;
        }

        Ok(())
    }

    fn check_bounce_rate(metrics: &DbMetricsRow) -> Vec<Recommendation> {
        let mut recs = Vec::new();
        if metrics.sent_count == 0 {
            return recs;
        }

        let bounce_rate = metrics.bounce_count as f64 / metrics.sent_count as f64;

        if bounce_rate > 0.05 {
            recs.push(Recommendation {
                check_name: CheckType::BounceRate.as_str().to_string(),
                severity: RecommendationSeverity::Critical,
                message: format!(
                    "Bounce rate is {:.1}%, above 5% threshold",
                    bounce_rate * 100.0
                ),
                details: Some("High bounce rates damage sender reputation.".to_string()),
                action_items: vec![
                    "Clean list - remove hard bounces".to_string(),
                    "Verify email addresses before sending".to_string(),
                    "Implement double opt-in".to_string(),
                ],
            });
        } else if bounce_rate > 0.03 {
            recs.push(Recommendation {
                check_name: CheckType::BounceRate.as_str().to_string(),
                severity: RecommendationSeverity::Warning,
                message: format!(
                    "Bounce rate is {:.1}%, approaching 5% threshold",
                    bounce_rate * 100.0
                ),
                details: Some("Monitor bounce rates closely.".to_string()),
                action_items: vec!["Review recent list imports for quality".to_string()],
            });
        }

        recs
    }

    fn check_open_rate(metrics: &DbMetricsRow) -> Vec<Recommendation> {
        let mut recs = Vec::new();
        if metrics.delivered_count == 0 {
            return recs;
        }

        let open_rate = metrics.open_count as f64 / metrics.delivered_count as f64;

        if open_rate < 0.15 {
            recs.push(Recommendation {
                check_name: CheckType::OpenRate.as_str().to_string(),
                severity: RecommendationSeverity::Warning,
                message: format!(
                    "Open rate is {:.1}%, below 15% benchmark",
                    open_rate * 100.0
                ),
                details: Some(
                    "Low open rates may indicate poor subject lines or timing.".to_string(),
                ),
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

    fn check_spam_complaints(metrics: &DbMetricsRow) -> Vec<Recommendation> {
        let mut recs = Vec::new();
        if metrics.sent_count == 0 {
            return recs;
        }

        let complaint_rate = metrics.complaint_count as f64 / metrics.sent_count as f64;

        if complaint_rate > 0.001 {
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

    fn check_unsubscribe_rate(metrics: &DbMetricsRow) -> Vec<Recommendation> {
        let mut recs = Vec::new();
        if metrics.delivered_count == 0 {
            return recs;
        }

        let unsub_rate = metrics.unsubscribe_count as f64 / metrics.delivered_count as f64;

        if unsub_rate > 0.005 {
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

    fn check_delivery_rate(metrics: &DbMetricsRow) -> Vec<Recommendation> {
        let mut recs = Vec::new();
        if metrics.sent_count == 0 {
            return recs;
        }

        let delivery_rate = metrics.delivered_count as f64 / metrics.sent_count as f64;

        if delivery_rate < 0.95 {
            recs.push(Recommendation {
                check_name: CheckType::DeliveryRate.as_str().to_string(),
                severity: RecommendationSeverity::Critical,
                message: format!(
                    "Delivery rate is {:.1}%, below 95% benchmark",
                    delivery_rate * 100.0
                ),
                details: Some(
                    "Low delivery indicates reputation or technical issues.".to_string(),
                ),
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

    fn check_engagement_trend(metrics: &DbMetricsRow) -> Vec<Recommendation> {
        let mut recs = Vec::new();
        if metrics.delivered_count == 0 {
            return recs;
        }

        let click_rate = metrics.click_count as f64 / metrics.delivered_count as f64;

        if click_rate < 0.02 {
            recs.push(Recommendation {
                check_name: CheckType::EngagementTrend.as_str().to_string(),
                severity: RecommendationSeverity::Info,
                message: format!(
                    "Click rate is {:.2}%, below 2% benchmark",
                    click_rate * 100.0
                ),
                details: Some(
                    "Low click rates suggest content optimization opportunities.".to_string(),
                ),
                action_items: vec![
                    "Improve call-to-action clarity".to_string(),
                    "Test different content formats".to_string(),
                    "Personalize content".to_string(),
                ],
            });
        }

        recs
    }

    pub async fn get_pending_recommendations(
        state: &AppState,
        campaign_id_val: Uuid,
    ) -> Result<Vec<AdvisorRecommendation>, String> {
        use crate::schema::advisor_recommendations::dsl::*;
        let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
        advisor_recommendations
            .filter(campaign_id.eq(campaign_id_val))
            .filter(dismissed.eq(false))
            .order_by(created_at.desc())
            .load(&mut conn)
            .map_err(|e| format!("Query error: {e}"))
    }

    pub async fn dismiss_recommendation(
        state: &AppState,
        recommendation_id: Uuid,
    ) -> Result<(), String> {
        use crate::schema::advisor_recommendations::dsl::*;
        let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
        diesel::update(advisor_recommendations.filter(id.eq(recommendation_id)))
            .set(dismissed.eq(true))
            .execute(&mut conn)
            .map_err(|e| format!("Update error: {e}"))?;
        Ok(())
    }
}
