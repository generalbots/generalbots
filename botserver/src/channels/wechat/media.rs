//! WeChat media upload functionality

use super::client::WeChatProvider;
use super::types::{
    MediaUploadResult, MediaType, PermanentMediaResult, VideoDescription,
};
use crate::channels::ChannelError;

impl WeChatProvider {
    /// Upload temporary media (image, voice, video, thumb)
    pub async fn upload_temp_media(
        &self,
        access_token: &str,
        media_type: MediaType,
        file_name: &str,
        file_data: &[u8],
    ) -> Result<MediaUploadResult, ChannelError> {
        let url = format!(
            "{}/cgi-bin/media/upload?access_token={}&type={}",
            self.api_base_url,
            access_token,
            media_type.as_str()
        );

        let part = reqwest::multipart::Part::bytes(file_data.to_vec())
            .file_name(file_name.to_string())
            .mime_str(media_type.mime_type())
            .map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })?;

        let form = reqwest::multipart::Form::new().part("media", part);

        let response = self
            .client
            .post(&url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        let result: super::types::MediaUploadResponse =
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

        Ok(MediaUploadResult {
            media_type: result.media_type.unwrap_or_default(),
            media_id: result.media_id.ok_or_else(|| ChannelError::ApiError {
                code: None,
                message: "No media_id in response".to_string(),
            })?,
            created_at: result.created_at,
        })
    }

    /// Upload permanent media
    pub async fn upload_permanent_media(
        &self,
        access_token: &str,
        media_type: MediaType,
        file_name: &str,
        file_data: &[u8],
        description: Option<&VideoDescription>,
    ) -> Result<PermanentMediaResult, ChannelError> {
        let url = format!(
            "{}/cgi-bin/material/add_material?access_token={}&type={}",
            self.api_base_url,
            access_token,
            media_type.as_str()
        );

        let part = reqwest::multipart::Part::bytes(file_data.to_vec())
            .file_name(file_name.to_string())
            .mime_str(media_type.mime_type())
            .map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })?;

        let mut form = reqwest::multipart::Form::new().part("media", part);

        if let Some(desc) = description {
            let desc_json = serde_json::to_string(desc).map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })?;
            form = form.text("description", desc_json);
        }

        let response = self
            .client
            .post(&url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        let result: super::types::PermanentMediaResponse =
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

        Ok(PermanentMediaResult {
            media_id: result.media_id.ok_or_else(|| ChannelError::ApiError {
                code: None,
                message: "No media_id in response".to_string(),
            })?,
            url: result.url,
        })
    }
}
