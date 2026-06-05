use crate::campaign::models::{
    AiContentRequest, AiContentResponse, ContentPiece, ContentStatus, InstagramConfig,
    InstagramInsights,
};
use anyhow::{Context, Result};
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct InstagramService {
    client: Client,
    config: Arc<RwLock<InstagramConfig>>,
    llm_endpoint: String,
    llm_api_key: String,
    llm_model: String,
}

impl InstagramService {
    pub fn new(
        config: InstagramConfig,
        llm_endpoint: String,
        llm_api_key: String,
        llm_model: String,
    ) -> Self {
        Self {
            client: Client::new(),
            config: Arc::new(RwLock::new(config)),
            llm_endpoint,
            llm_api_key,
            llm_model,
        }
    }

    pub async fn update_config(&self, config: InstagramConfig) {
        let mut current = self.config.write().await;
        *current = config;
    }

    async fn call_llm(&self, payload: serde_json::Value) -> Result<serde_json::Value> {
        let response = self
            .client
            .post(&self.llm_endpoint)
            .header("Authorization", format!("Bearer {}", self.llm_api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .context("LLM request failed")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "LLM request failed with status {}",
                response.status()
            ));
        }

        let body: serde_json::Value = response.json().await.context("Failed to parse LLM response")?;

        body["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("LLM response missing content"))
            .and_then(|text| {
                serde_json::from_str(&text)
                    .map_err(|e| anyhow::anyhow!("Failed to parse LLM content JSON: {}", e))
            })
    }

    pub async fn generate_content(&self, request: AiContentRequest) -> Result<AiContentResponse> {
        let system = "Você é especialista em marketing digital para Instagram. \
                      Gere conteúdo persuasivo e criativo alinhado à marca. \
                      Responda JSON com caption, hashtags, image_prompt, alt_text, call_to_action.";

        let prompt = format!(
            "MARCA: {} | PRODUTO: {} | DESCRICAO: {} | PUBLICO: {} | \
             OBJETIVO: {} | TOM: {} | MAX: {} chars | HASHTAGS: {} | \
             ESTILO VISUAL: {}",
            request.brand_voice,
            request.product_name,
            request.product_description,
            request.target_audience,
            request.campaign_goal,
            request.tone,
            request.max_length,
            if request.include_hashtags {
                format!("{} tags", request.hashtag_count)
            } else {
                "nenhuma".to_string()
            },
            request.media_style,
        );

        let payload = serde_json::json!({
            "model": self.llm_model,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": prompt}
            ],
            "temperature": 0.8,
            "max_tokens": 1024,
            "response_format": {"type": "json_object"}
        });

        self.call_llm(payload).await
    }

    pub async fn generate_hashtags(&self, topic: &str, count: usize) -> Result<Vec<String>> {
        let payload = serde_json::json!({
            "model": self.llm_model,
            "messages": [
                {"role": "system", "content": "Gere hashtags para Instagram em JSON: {\"hashtags\": [string]}"},
                {"role": "user", "content": format!("Gera {} hashtags sobre '{}'", count, topic)}
            ],
            "temperature": 0.7,
            "max_tokens": 256,
            "response_format": {"type": "json_object"}
        });

        let result: serde_json::Value = self.call_llm(payload).await?;
        let hashtags: Vec<String> = result["hashtags"]
            .as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();

        Ok(hashtags)
    }

    pub async fn create_media_container(&self, image_url: &str, caption: &str) -> Result<String> {
        let config = self.config.read().await;
        let url = format!("{}/{}/media", config.api_base_url, config.business_account_id);

        let response = self
            .client
            .post(&url)
            .form(&[
                ("image_url", image_url.to_string()),
                ("caption", caption.to_string()),
                ("access_token", config.access_token.clone()),
            ])
            .send()
            .await
            .context("Failed to create media container")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Media container creation failed: {}", response.status()));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse media container response")?;

        body["id"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("Media container response missing id"))
    }

    pub async fn publish_container(&self, container_id: &str) -> Result<String> {
        let config = self.config.read().await;
        let url = format!("{}/{}/media_publish", config.api_base_url, config.business_account_id);

        let response = self
            .client
            .post(&url)
            .form(&[
                ("creation_id", container_id.to_string()),
                ("access_token", config.access_token.clone()),
            ])
            .send()
            .await
            .context("Failed to publish container")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Media publish failed: {}", response.status()));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse publish response")?;

        body["id"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("Publish response missing id"))
    }

    pub async fn publish_post(
        &self,
        content: &ContentPiece,
        ai_request: AiContentRequest,
    ) -> Result<ContentPiece> {
        let ai_content = self.generate_content(ai_request).await?;

        let caption = if ai_content.hashtags.is_empty() {
            format!("{}\n\n{}", ai_content.caption, ai_content.call_to_action)
        } else {
            format!(
                "{}\n\n{}\n\n{}",
                ai_content.caption,
                ai_content.call_to_action,
                ai_content.hashtags.join(" ")
            )
        };

        let mut published = content.clone();
        if let Some(media_url) = content.media_urls.first() {
            let container_id = self.create_media_container(media_url, &caption).await?;
            let post_id = self.publish_container(&container_id).await?;
            published.id = post_id;
        }

        published.body = caption;
        published.set_status(ContentStatus::Published);
        Ok(published)
    }

    pub async fn get_post_insights(&self, post_id: &str) -> Result<InstagramInsights> {
        let config = self.config.read().await;
        let url = format!("{}/{}", config.api_base_url, post_id);

        let fields = [
            "impressions", "reach", "engagement", "likes", "comments",
            "shares", "saves", "profile_visits", "follower_count",
        ];

        let response = self
            .client
            .get(&url)
            .query(&[
                ("fields", fields.join(",")),
                ("access_token", config.access_token.clone()),
            ])
            .send()
            .await
            .context("Failed to fetch insights")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Insights fetch failed: {}", response.status()));
        }

        let data: serde_json::Value = response.json().await.context("Failed to parse insights")?;
        let ext = |f: &str| -> u64 { data[f].as_u64().unwrap_or(0) };

        Ok(InstagramInsights {
            post_id: post_id.to_string(),
            impressions: ext("impressions"),
            reach: ext("reach"),
            engagement: ext("engagement"),
            likes: ext("likes"),
            comments: ext("comments"),
            shares: ext("shares"),
            saves: ext("saves"),
            profile_visits: ext("profile_visits"),
            follower_count: ext("follower_count"),
        })
    }

    pub async fn batch_generate_content(
        &self,
        requests: Vec<AiContentRequest>,
    ) -> Result<Vec<AiContentResponse>> {
        let mut results = Vec::with_capacity(requests.len());
        for request in requests {
            results.push(self.generate_content(request).await?);
        }
        Ok(results)
    }

    pub async fn generate_image_prompt(&self, prompt: &str, style: &str) -> Result<String> {
        let payload = serde_json::json!({
            "model": self.llm_model,
            "messages": [
                {"role": "system", "content": "Generate image prompts for Instagram marketing. Return JSON with improved_prompt and negative_prompt."},
                {"role": "user", "content": format!("Style: {}. Prompt: {}", style, prompt)}
            ],
            "temperature": 0.8,
            "max_tokens": 512,
            "response_format": {"type": "json_object"}
        });

        let result: serde_json::Value = self.call_llm(payload).await?;
        result["improved_prompt"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("Response missing improved_prompt"))
    }

    pub async fn schedule_post(
        &self,
        content: &ContentPiece,
        ai_request: AiContentRequest,
        _publish_at: &str,
    ) -> Result<ContentPiece> {
        let ai_content = self.generate_content(ai_request).await?;
        let caption = format!(
            "{}\n\n{}\n\n{}",
            ai_content.caption,
            ai_content.call_to_action,
            ai_content.hashtags.join(" ")
        );

        let mut scheduled = content.clone();
        scheduled.body = caption;
        scheduled.set_status(ContentStatus::Scheduled);
        Ok(scheduled)
    }

    pub fn hashtag_suggestions(&self, category: &str) -> Vec<String> {
        match category {
            "marketing" => vec![
                "#marketingdigital", "#estrategiadigital", "#marketingonline",
                "#crescimento", "#marketing360", "#digitalstrategy",
            ],
            "vendas" => vec![
                "#vendas", "#sales", "#negocios", "#empreendedorismo",
                "#sucesso", "#clientes",
            ],
            "produto" => vec![
                "#novoproduto", "#lancamento", "#produto", "#inovacao",
                "#qualidade", "#exclusivo",
            ],
            "engajamento" => vec![
                "#engajamento", "#interacao", "#comunidade", "#seguidores",
                "#conversa", "#feedback",
            ],
            _ => vec![
                "#instagram", "#instagood", "#photooftheday", "#love",
                "#beautiful", "#happy",
            ],
        }
    }
}
