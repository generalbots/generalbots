//! WeChat menu management functionality

use super::client::WeChatProvider;
use super::types::{Menu, WeChatApiResponse};
use crate::channels::ChannelError;

impl WeChatProvider {
    /// Create a menu
    pub async fn create_menu(
        &self,
        access_token: &str,
        menu: &Menu,
    ) -> Result<(), ChannelError> {
        let url = format!(
            "{}/cgi-bin/menu/create?access_token={}",
            self.api_base_url, access_token
        );

        let response = self
            .client
            .post(&url)
            .json(menu)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        let result: WeChatApiResponse<()> =
            response.json().await.map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })?;

        self.check_error(&result)?;

        Ok(())
    }

    /// Delete menu
    pub async fn delete_menu(&self, access_token: &str) -> Result<(), ChannelError> {
        let url = format!(
            "{}/cgi-bin/menu/delete?access_token={}",
            self.api_base_url, access_token
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        let result: WeChatApiResponse<()> =
            response.json().await.map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })?;

        self.check_error(&result)?;

        Ok(())
    }
}
