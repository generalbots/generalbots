//! WeChat ChannelProvider trait implementation

use super::client::WeChatProvider;
use crate::channels::{
    ChannelAccount, ChannelCredentials, ChannelError, ChannelProvider, ChannelType, PostContent,
    PostResult,
};

#[async_trait::async_trait]
impl ChannelProvider for WeChatProvider {
    fn channel_type(&self) -> ChannelType {
        ChannelType::WeChat
    }

    fn max_text_length(&self) -> usize {
        600 // WeChat article summary limit
    }

    fn supports_images(&self) -> bool {
        true
    }

    fn supports_video(&self) -> bool {
        true
    }

    fn supports_links(&self) -> bool {
        true
    }

    async fn post(
        &self,
        account: &ChannelAccount,
        content: &PostContent,
    ) -> Result<PostResult, ChannelError> {
        let (app_id, app_secret) = match &account.credentials {
            ChannelCredentials::ApiKey { api_key, api_secret } => {
                let secret = api_secret.as_ref().ok_or_else(|| {
                    ChannelError::AuthenticationFailed("Missing app_secret".to_string())
                })?;
                (api_key.clone(), secret.clone())
            }
            _ => {
                return Err(ChannelError::AuthenticationFailed(
                    "API key credentials required for WeChat".to_string(),
                ))
            }
        };

        let access_token = self.get_access_token(&app_id, &app_secret).await?;
        let text = content.text.as_deref().unwrap_or("");

        // Create a news article draft and publish it
        let article = super::types::NewsArticle {
            title: content
                .metadata
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("Post")
                .to_string(),
            author: content
                .metadata
                .get("author")
                .and_then(|v| v.as_str())
                .map(String::from),
            digest: Some(text.chars().take(120).collect()),
            content: text.to_string(),
            content_source_url: content.link.clone(),
            thumb_media_id: content
                .metadata
                .get("thumb_media_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            need_open_comment: Some(1),
            only_fans_can_comment: Some(0),
        };

        let media_id = self.create_draft(&access_token, &[article]).await?;
        let publish_result = self.publish_draft(&access_token, &media_id).await?;

        Ok(PostResult::success(
            ChannelType::WeChat,
            publish_result.publish_id,
            None,
        ))
    }

    async fn validate_credentials(
        &self,
        credentials: &ChannelCredentials,
    ) -> Result<bool, ChannelError> {
        match credentials {
            ChannelCredentials::ApiKey { api_key, api_secret } => {
                if let Some(secret) = api_secret {
                    match self.get_access_token(api_key, secret).await {
                        Ok(_) => Ok(true),
                        Err(ChannelError::AuthenticationFailed(_)) => Ok(false),
                        Err(e) => Err(e),
                    }
                } else {
                    Ok(false)
                }
            }
            _ => Ok(false),
        }
    }

    async fn refresh_token(&self, _account: &mut ChannelAccount) -> Result<(), ChannelError> {
        // WeChat uses app_id/app_secret, tokens are auto-refreshed via get_access_token
        Ok(())
    }
}
