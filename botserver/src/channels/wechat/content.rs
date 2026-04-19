//! WeChat content publishing functionality

use super::client::WeChatProvider;
use super::types::{NewsArticle, PublishResult, PublishStatus};
use crate::channels::ChannelError;

impl WeChatProvider {
    /// Create a news article (draft)
    pub async fn create_draft(
        &self,
        access_token: &str,
        articles: &[NewsArticle],
    ) -> Result<String, ChannelError> {
        let url = format!(
            "{}/cgi-bin/draft/add?access_token={}",
            self.api_base_url, access_token
        );

        let request_body = serde_json::json!({
            "articles": articles
        });

        let response = self
            .client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        let result: super::types::DraftResponse =
            response.json().await.map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })?;

        if let Some(errcode) = result.errcode {
            if errcode != 0 {
                return Err(ChannelError::ApiError {
                    code: Some(errcode.to_string()),
                    message: result.errmsg.unwrap_or_default(),
                });
            }
        }

        result.media_id.ok_or_else(|| ChannelError::ApiError {
            code: None,
            message: "No media_id in response".to_string(),
        })
    }

    /// Publish a draft
    pub async fn publish_draft(
        &self,
        access_token: &str,
        media_id: &str,
    ) -> Result<PublishResult, ChannelError> {
        let url = format!(
            "{}/cgi-bin/freepublish/submit?access_token={}",
            self.api_base_url, access_token
        );

        let request_body = serde_json::json!({
            "media_id": media_id
        });

        let response = self
            .client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        let result: super::types::PublishResponse =
            response.json().await.map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })?;

        if let Some(errcode) = result.errcode {
            if errcode != 0 {
                return Err(ChannelError::ApiError {
                    code: Some(errcode.to_string()),
                    message: result.errmsg.unwrap_or_default(),
                });
            }
        }

        Ok(PublishResult {
            publish_id: result.publish_id.ok_or_else(|| ChannelError::ApiError {
                code: None,
                message: "No publish_id in response".to_string(),
            })?,
        })
    }

    /// Get publish status
    pub async fn get_publish_status(
        &self,
        access_token: &str,
        publish_id: &str,
    ) -> Result<PublishStatus, ChannelError> {
        let url = format!(
            "{}/cgi-bin/freepublish/get?access_token={}",
            self.api_base_url, access_token
        );

        let request_body = serde_json::json!({
            "publish_id": publish_id
        });

        let response = self
            .client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        let result: super::types::PublishStatusResponse =
            response.json().await.map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })?;

        if let Some(errcode) = result.errcode {
            if errcode != 0 {
                return Err(ChannelError::ApiError {
                    code: Some(errcode.to_string()),
                    message: result.errmsg.unwrap_or_default(),
                });
            }
        }

        Ok(PublishStatus {
            publish_id: publish_id.to_string(),
            publish_status: result.publish_status.unwrap_or(0),
            article_id: result.article_id,
            article_detail: result.article_detail,
            fail_idx: result.fail_idx,
        })
    }
}
