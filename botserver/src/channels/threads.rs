use crate::channels::{
    ChannelAccount, ChannelCredentials, ChannelError, ChannelProvider, ChannelType,
    PostContent, PostResult,
};

pub struct ThreadsProvider {
    client: reqwest::Client,
    base_url: String,
}

impl ThreadsProvider {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: "https://graph.threads.net/v1.0".to_string(),
        }
    }

    async fn create_media_container(
        &self,
        access_token: &str,
        user_id: &str,
        text: &str,
        media_type: &str,
        image_url: Option<&str>,
        video_url: Option<&str>,
    ) -> Result<String, ChannelError> {
        let mut params = vec![
            ("media_type", media_type.to_string()),
            ("text", text.to_string()),
            ("access_token", access_token.to_string()),
        ];

        if let Some(url) = image_url {
            params.push(("image_url", url.to_string()));
        }

        if let Some(url) = video_url {
            params.push(("video_url", url.to_string()));
        }

        let response = self
            .client
            .post(format!("{}/{}/threads", self.base_url, user_id))
            .form(&params)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ChannelError::ApiError {
                code: None,
                message: error_text,
            });
        }

        #[derive(serde::Deserialize)]
        struct ContainerResponse {
            id: String,
        }

        let container: ContainerResponse = response
            .json()
            .await
            .map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })?;

        Ok(container.id)
    }

    async fn publish_container(
        &self,
        access_token: &str,
        user_id: &str,
        container_id: &str,
    ) -> Result<String, ChannelError> {
        let params = vec![
            ("creation_id", container_id.to_string()),
            ("access_token", access_token.to_string()),
        ];

        let response = self
            .client
            .post(format!("{}/{}/threads_publish", self.base_url, user_id))
            .form(&params)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ChannelError::ApiError {
                code: None,
                message: error_text,
            });
        }

        #[derive(serde::Deserialize)]
        struct PublishResponse {
            id: String,
        }

        let published: PublishResponse = response
            .json()
            .await
            .map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })?;

        Ok(published.id)
    }

    async fn get_user_profile(&self, access_token: &str) -> Result<ThreadsUser, ChannelError> {
        let response = self
            .client
            .get(format!("{}/me", self.base_url))
            .query(&[
                ("fields", "id,username,threads_profile_picture_url"),
                ("access_token", access_token),
            ])
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ChannelError::AuthenticationFailed(error_text));
        }

        response
            .json::<ThreadsUser>()
            .await
            .map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })
    }
}

impl Default for ThreadsProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl ChannelProvider for ThreadsProvider {
    fn channel_type(&self) -> ChannelType {
        ChannelType::Threads
    }

    fn max_text_length(&self) -> usize {
        500
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
        let access_token = match &account.credentials {
            ChannelCredentials::OAuth { access_token, .. } => access_token.clone(),
            _ => {
                return Err(ChannelError::AuthenticationFailed(
                    "Invalid credentials type for Threads".to_string(),
                ))
            }
        };

        let user = self.get_user_profile(&access_token).await?;
        let text = content.text.as_deref().unwrap_or("");

        if text.len() > self.max_text_length() {
            return Err(ChannelError::ContentTooLong {
                max_length: self.max_text_length(),
                actual_length: text.len(),
            });
        }

        let (media_type, image_url, video_url) = if content.video_url.is_some() {
            ("VIDEO", None, content.video_url.as_deref())
        } else if !content.image_urls.is_empty() {
            ("IMAGE", content.image_urls.first().map(|s| s.as_str()), None)
        } else {
            ("TEXT", None, None)
        };

        let container_id = self
            .create_media_container(&access_token, &user.id, text, media_type, image_url, video_url)
            .await?;

        if media_type == "VIDEO" {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }

        let post_id = self
            .publish_container(&access_token, &user.id, &container_id)
            .await?;

        let url = format!("https://www.threads.net/@{}/post/{}", user.username, post_id);

        Ok(PostResult::success(ChannelType::Threads, post_id, Some(url)))
    }

    async fn validate_credentials(
        &self,
        credentials: &ChannelCredentials,
    ) -> Result<bool, ChannelError> {
        let access_token = match credentials {
            ChannelCredentials::OAuth { access_token, .. } => access_token,
            _ => return Ok(false),
        };

        match self.get_user_profile(access_token).await {
            Ok(_) => Ok(true),
            Err(ChannelError::AuthenticationFailed(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }

    async fn refresh_token(&self, account: &mut ChannelAccount) -> Result<(), ChannelError> {
        let (access_token, _refresh_token) = match &account.credentials {
            ChannelCredentials::OAuth {
                access_token,
                refresh_token,
                ..
            } => (access_token.clone(), refresh_token.clone()),
            _ => return Err(ChannelError::AuthenticationFailed("Invalid credentials".to_string())),
        };

        let response = self
            .client
            .get(format!("{}/refresh_access_token", self.base_url))
            .query(&[
                ("grant_type", "th_refresh_token"),
                ("access_token", &access_token),
            ])
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ChannelError::ApiError {
                code: None,
                message: error_text,
            });
        }

        #[derive(serde::Deserialize)]
        struct RefreshResponse {
            access_token: String,
            expires_in: i64,
        }

        let refreshed: RefreshResponse = response
            .json()
            .await
            .map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })?;

        let expires_at = chrono::Utc::now() + chrono::Duration::seconds(refreshed.expires_in);

        account.credentials = ChannelCredentials::OAuth {
            access_token: refreshed.access_token,
            refresh_token: None,
            expires_at: Some(expires_at),
            scope: None,
        };

        Ok(())
    }
}

#[derive(Debug, serde::Deserialize)]
struct ThreadsUser {
    id: String,
    username: String,
}
