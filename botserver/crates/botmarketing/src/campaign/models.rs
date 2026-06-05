use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type CampaignId = String;
pub type ChannelTypeStr = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Campaign {
    pub id: CampaignId,
    pub name: String,
    pub description: Option<String>,
    pub channels: Vec<ChannelConfig>,
    pub target_audience: Option<TargetAudience>,
    pub schedule: Schedule,
    pub content: Vec<ContentPiece>,
    pub budget: Option<Budget>,
    pub status: CampaignStatus,
    pub metrics: CampaignMetrics,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    pub channel: ChannelTypeStr,
    pub enabled: bool,
    pub config: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetAudience {
    pub age_range: Option<(u32, u32)>,
    pub locations: Vec<String>,
    pub interests: Vec<String>,
    pub languages: Vec<String>,
    pub custom_attrs: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub frequency: String,
    pub time_slots: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentPiece {
    pub id: String,
    pub title: String,
    pub body: String,
    pub media_urls: Vec<String>,
    pub channel: ChannelTypeStr,
    pub status: ContentStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentStatus {
    Draft,
    Generated,
    Approved,
    Scheduled,
    Published,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Budget {
    pub total: f64,
    pub spent: f64,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CampaignStatus {
    Draft,
    Active,
    Paused,
    Completed,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CampaignMetrics {
    pub impressions: u64,
    pub clicks: u64,
    pub conversions: u64,
    pub engagement_rate: f64,
    pub roi: Option<f64>,
    pub channel_metrics: HashMap<String, ChannelMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChannelMetrics {
    pub impressions: u64,
    pub clicks: u64,
    pub conversions: u64,
    pub spend: f64,
    pub revenue: Option<f64>,
}

impl Campaign {
    pub fn new(id: CampaignId, name: String, schedule: Schedule) -> Self {
        Self {
            id,
            name,
            description: None,
            channels: Vec::new(),
            target_audience: None,
            schedule,
            content: Vec::new(),
            budget: None,
            status: CampaignStatus::Draft,
            metrics: CampaignMetrics::default(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn add_channel(&mut self, config: ChannelConfig) {
        self.channels.push(config);
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    pub fn add_content(&mut self, piece: ContentPiece) {
        self.content.push(piece);
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    pub fn set_status(&mut self, status: CampaignStatus) {
        self.status = status;
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, CampaignStatus::Active)
    }

    pub fn total_budget(&self) -> f64 {
        self.budget.as_ref().map_or(0.0, |b| b.total)
    }

    pub fn total_spent(&self) -> f64 {
        self.budget.as_ref().map_or(0.0, |b| b.spent)
    }
}

impl ContentPiece {
    pub fn new(id: String, title: String, body: String, channel: ChannelTypeStr) -> Self {
        Self {
            id,
            title,
            body,
            media_urls: Vec::new(),
            channel,
            status: ContentStatus::Draft,
        }
    }

    pub fn add_media(&mut self, url: String) {
        self.media_urls.push(url);
    }

    pub fn set_status(&mut self, status: ContentStatus) {
        self.status = status;
    }
}

impl Budget {
    pub fn new(total: f64, currency: String) -> Self {
        Self {
            total,
            spent: 0.0,
            currency,
        }
    }

    pub fn add_spend(&mut self, amount: f64) {
        self.spent += amount;
    }

    pub fn remaining(&self) -> f64 {
        (self.total - self.spent).max(0.0)
    }

    pub fn utilization_pct(&self) -> f64 {
        if self.total <= 0.0 {
            return 0.0;
        }
        (self.spent / self.total) * 100.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstagramConfig {
    pub api_base_url: String,
    pub access_token: String,
    pub business_account_id: String,
    pub media_endpoint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstagramPost {
    pub id: String,
    pub caption: String,
    pub media_url: Option<String>,
    pub media_type: InstagramMediaType,
    pub permalink: Option<String>,
    pub timestamp: String,
    pub like_count: u64,
    pub comment_count: u64,
    pub share_count: u64,
    pub save_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstagramMediaType {
    Image,
    Carousel,
    Reel,
    Story,
}

impl std::fmt::Display for InstagramMediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Image => write!(f, "IMAGE"),
            Self::Carousel => write!(f, "CAROUSEL"),
            Self::Reel => write!(f, "REELS"),
            Self::Story => write!(f, "STORIES"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiContentRequest {
    pub brand_voice: String,
    pub product_name: String,
    pub product_description: String,
    pub target_audience: String,
    pub campaign_goal: String,
    pub tone: String,
    pub max_length: usize,
    pub include_hashtags: bool,
    pub hashtag_count: usize,
    pub media_style: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiContentResponse {
    pub caption: String,
    pub hashtags: Vec<String>,
    pub image_prompt: String,
    pub alt_text: String,
    pub call_to_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstagramInsights {
    pub post_id: String,
    pub impressions: u64,
    pub reach: u64,
    pub engagement: u64,
    pub likes: u64,
    pub comments: u64,
    pub shares: u64,
    pub saves: u64,
    pub profile_visits: u64,
    pub follower_count: u64,
}

impl CampaignMetrics {
    pub fn update_channel_metrics(&mut self, channel: String, metrics: ChannelMetrics) {
        self.impressions = self.impressions.saturating_add(metrics.impressions);
        self.clicks = self.clicks.saturating_add(metrics.clicks);
        self.conversions = self.conversions.saturating_add(metrics.conversions);
        self.channel_metrics.insert(channel, metrics);
        self.recalculate_engagement_rate();
    }

    pub fn recalculate_engagement_rate(&mut self) {
        if self.impressions > 0 {
            self.engagement_rate = (self.clicks as f64 / self.impressions as f64) * 100.0;
        }
    }

    pub fn calculate_roi(&mut self) {
        let total_spend: f64 = self
            .channel_metrics
            .values()
            .map(|m| m.spend)
            .sum();
        let total_revenue: f64 = self
            .channel_metrics
            .values()
            .filter_map(|m| m.revenue)
            .sum();
        if total_spend > 0.0 {
            self.roi = Some(((total_revenue - total_spend) / total_spend) * 100.0);
        }
    }

    pub fn total_spend(&self) -> f64 {
        self.channel_metrics.values().map(|m| m.spend).sum()
    }

    pub fn total_revenue(&self) -> f64 {
        self.channel_metrics.values().filter_map(|m| m.revenue).sum()
    }
}
