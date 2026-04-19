//! WeChat QR code and URL utilities

use super::client::WeChatProvider;
use super::types::{QRCodeRequest, QRCodeResult};
use crate::channels::ChannelError;

impl WeChatProvider {
    /// Create QR code (temporary or permanent)
    pub async fn create_qrcode(
        &self,
        access_token: &str,
        request: &QRCodeRequest,
    ) -> Result<QRCodeResult, ChannelError> {
        let url = format!(
            "{}/cgi-bin/qrcode/create?access_token={}",
            self.api_base_url, access_token
        );

        let response = self
            .client
            .post(&url)
            .json(request)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        let result: super::types::QRCodeResponse =
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

        let ticket = result.ticket.ok_or_else(|| ChannelError::ApiError {
            code: None,
            message: "No ticket in response".to_string(),
        })?;

        Ok(QRCodeResult {
            ticket: ticket.clone(),
            expire_seconds: result.expire_seconds,
            url: result.url.unwrap_or_default(),
            qrcode_url: format!(
                "https://mp.weixin.qq.com/cgi-bin/showqrcode?ticket={}",
                urlencoding::encode(&ticket)
            ),
        })
    }

    /// Shorten URL
    pub async fn shorten_url(
        &self,
        access_token: &str,
        long_url: &str,
    ) -> Result<String, ChannelError> {
        let url = format!(
            "{}/cgi-bin/shorturl?access_token={}",
            self.api_base_url, access_token
        );

        let request_body = serde_json::json!({
            "action": "long2short",
            "long_url": long_url
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

        let result: super::types::ShortUrlResponse =
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

        result.short_url.ok_or_else(|| ChannelError::ApiError {
            code: None,
            message: "No short_url in response".to_string(),
        })
    }
}
