//! WeChat Official Account and Mini Program API Integration
//!
//! Provides messaging, media upload, and content publishing capabilities.
//! Supports both Official Account and Mini Program APIs.

use crate::channels::{
    ChannelAccount, ChannelCredentials, ChannelError, ChannelProvider, ChannelType, PostContent,
    PostResult,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// WeChat API provider for Official Accounts and Mini Programs
pub struct WeChatProvider {
    client: reqwest::Client,
    api_base_url: String,
    /// Cache for access tokens (app_id -> token info)
    token_cache: Arc<RwLock<HashMap<String, CachedToken>>>,
}

#[derive(Debug, Clone)]
struct CachedToken {
    access_token: String,
    expires_at: chrono::DateTime<chrono::Utc>,
}

impl WeChatProvider {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            api_base_url: "https://api.weixin.qq.com".to_string(),
            token_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get access token (with caching)
    pub async fn get_access_token(
        &self,
        app_id: &str,
        app_secret: &str,
    ) -> Result<String, ChannelError> {
        // Check cache first
        {
            let cache = self.token_cache.read().await;
            if let Some(cached) = cache.get(app_id) {
                if cached.expires_at > chrono::Utc::now() + chrono::Duration::minutes(5) {
                    return Ok(cached.access_token.clone());
                }
            }
        }

        // Fetch new token
        let url = format!(
            "{}/cgi-bin/token?grant_type=client_credential&appid={}&secret={}",
            self.api_base_url, app_id, app_secret
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

        let token_response: AccessTokenResponse =
            response.json().await.map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })?;

        if let Some(errcode) = token_response.errcode {
            if errcode != 0 {
                return Err(ChannelError::ApiError {
                    code: Some(errcode.to_string()),
                    message: token_response.errmsg.unwrap_or_default(),
                });
            }
        }

        let access_token = token_response.access_token.ok_or_else(|| {
            ChannelError::ApiError {
                code: None,
                message: "No access token in response".to_string(),
            }
        })?;

        let expires_in = token_response.expires_in.unwrap_or(7200);
        let expires_at = chrono::Utc::now() + chrono::Duration::seconds(expires_in as i64);

        // Cache the token
        {
            let mut cache = self.token_cache.write().await;
            cache.insert(
                app_id.to_string(),
                CachedToken {
                    access_token: access_token.clone(),
                    expires_at,
                },
            );
        }

        Ok(access_token)
    }

    /// Send template message to user
    pub async fn send_template_message(
        &self,
        access_token: &str,
        message: &TemplateMessage,
    ) -> Result<TemplateMessageResult, ChannelError> {
        let url = format!(
            "{}/cgi-bin/message/template/send?access_token={}",
            self.api_base_url, access_token
        );

        let response = self
            .client
            .post(&url)
            .json(message)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        let result: WeChatApiResponse<TemplateMessageResult> =
            response.json().await.map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })?;

        self.check_error(&result)?;

        Ok(TemplateMessageResult {
            msgid: result.msgid,
        })
    }

    /// Send customer service message
    pub async fn send_customer_message(
        &self,
        access_token: &str,
        message: &CustomerMessage,
    ) -> Result<(), ChannelError> {
        let url = format!(
            "{}/cgi-bin/message/custom/send?access_token={}",
            self.api_base_url, access_token
        );

        let response = self
            .client
            .post(&url)
            .json(message)
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

        let result: MediaUploadResponse =
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

        let result: PermanentMediaResponse =
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

        let result: DraftResponse =
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

        let result: PublishResponse =
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

        let result: PublishStatusResponse =
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

    /// Get user info
    pub async fn get_user_info(
        &self,
        access_token: &str,
        openid: &str,
    ) -> Result<WeChatUser, ChannelError> {
        let url = format!(
            "{}/cgi-bin/user/info?access_token={}&openid={}&lang=zh_CN",
            self.api_base_url, access_token, openid
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

        let result: WeChatUserResponse =
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

        Ok(WeChatUser {
            subscribe: result.subscribe.unwrap_or(0),
            openid: result.openid.unwrap_or_default(),
            nickname: result.nickname,
            sex: result.sex,
            language: result.language,
            city: result.city,
            province: result.province,
            country: result.country,
            headimgurl: result.headimgurl,
            subscribe_time: result.subscribe_time,
            unionid: result.unionid,
            remark: result.remark,
            groupid: result.groupid,
            tagid_list: result.tagid_list,
            subscribe_scene: result.subscribe_scene,
            qr_scene: result.qr_scene,
            qr_scene_str: result.qr_scene_str,
        })
    }

    /// Get follower list
    pub async fn get_followers(
        &self,
        access_token: &str,
        next_openid: Option<&str>,
    ) -> Result<FollowerList, ChannelError> {
        let mut url = format!(
            "{}/cgi-bin/user/get?access_token={}",
            self.api_base_url, access_token
        );

        if let Some(openid) = next_openid {
            url = format!("{}&next_openid={}", url, openid);
        }

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        let result: FollowerListResponse =
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

        Ok(FollowerList {
            total: result.total.unwrap_or(0),
            count: result.count.unwrap_or(0),
            openids: result
                .data
                .and_then(|d| d.openid)
                .unwrap_or_default(),
            next_openid: result.next_openid,
        })
    }

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

        let result: QRCodeResponse =
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

        let result: ShortUrlResponse =
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

    /// Verify webhook signature
    pub fn verify_signature(
        &self,
        token: &str,
        timestamp: &str,
        nonce: &str,
        signature: &str,
    ) -> bool {
        use sha1::{Digest, Sha1};

        let mut params = vec![token, timestamp, nonce];
        params.sort();
        let joined = params.join("");

        let mut hasher = Sha1::new();
        hasher.update(joined.as_bytes());
        let result = hasher.finalize();
        let computed = hex::encode(result);

        computed == signature
    }

    /// Parse incoming message XML
    pub fn parse_message(&self, xml: &str) -> Result<IncomingMessage, ChannelError> {
        // Simple XML parsing - in production, use a proper XML parser
        let get_value = |tag: &str| -> Option<String> {
            let start_tag = format!("<{}>", tag);
            let end_tag = format!("</{}>", tag);
            if let Some(start) = xml.find(&start_tag) {
                if let Some(end) = xml.find(&end_tag) {
                    let value_start = start + start_tag.len();
                    if value_start < end {
                        let value = &xml[value_start..end];
                        // Handle CDATA
                        if value.starts_with("<![CDATA[") && value.ends_with("]]>") {
                            return Some(value[9..value.len() - 3].to_string());
                        }
                        return Some(value.to_string());
                    }
                }
            }
            None
        };

        let msg_type = get_value("MsgType").ok_or_else(|| ChannelError::ApiError {
            code: None,
            message: "Missing MsgType in message".to_string(),
        })?;

        Ok(IncomingMessage {
            to_user_name: get_value("ToUserName").unwrap_or_default(),
            from_user_name: get_value("FromUserName").unwrap_or_default(),
            create_time: get_value("CreateTime")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            msg_type,
            msg_id: get_value("MsgId"),
            content: get_value("Content"),
            pic_url: get_value("PicUrl"),
            media_id: get_value("MediaId"),
            format: get_value("Format"),
            recognition: get_value("Recognition"),
            thumb_media_id: get_value("ThumbMediaId"),
            location_x: get_value("Location_X").and_then(|s| s.parse().ok()),
            location_y: get_value("Location_Y").and_then(|s| s.parse().ok()),
            scale: get_value("Scale").and_then(|s| s.parse().ok()),
            label: get_value("Label"),
            title: get_value("Title"),
            description: get_value("Description"),
            url: get_value("Url"),
            event: get_value("Event"),
            event_key: get_value("EventKey"),
            ticket: get_value("Ticket"),
            latitude: get_value("Latitude").and_then(|s| s.parse().ok()),
            longitude: get_value("Longitude").and_then(|s| s.parse().ok()),
            precision: get_value("Precision").and_then(|s| s.parse().ok()),
        })
    }

    /// Build reply message XML
    pub fn build_reply(&self, reply: &ReplyMessage) -> String {
        let timestamp = chrono::Utc::now().timestamp();

        match &reply.content {
            ReplyContent::Text { content } => {
                format!(
                    r#"<xml>
<ToUserName><![CDATA[{}]]></ToUserName>
<FromUserName><![CDATA[{}]]></FromUserName>
<CreateTime>{}</CreateTime>
<MsgType><![CDATA[text]]></MsgType>
<Content><![CDATA[{}]]></Content>
</xml>"#,
                    reply.to_user, reply.from_user, timestamp, content
                )
            }
            ReplyContent::Image { media_id } => {
                format!(
                    r#"<xml>
<ToUserName><![CDATA[{}]]></ToUserName>
<FromUserName><![CDATA[{}]]></FromUserName>
<CreateTime>{}</CreateTime>
<MsgType><![CDATA[image]]></MsgType>
<Image>
<MediaId><![CDATA[{}]]></MediaId>
</Image>
</xml>"#,
                    reply.to_user, reply.from_user, timestamp, media_id
                )
            }
            ReplyContent::Voice { media_id } => {
                format!(
                    r#"<xml>
<ToUserName><![CDATA[{}]]></ToUserName>
<FromUserName><![CDATA[{}]]></FromUserName>
<CreateTime>{}</CreateTime>
<MsgType><![CDATA[voice]]></MsgType>
<Voice>
<MediaId><![CDATA[{}]]></MediaId>
</Voice>
</xml>"#,
                    reply.to_user, reply.from_user, timestamp, media_id
                )
            }
            ReplyContent::Video {
                media_id,
                title,
                description,
            } => {
                format!(
                    r#"<xml>
<ToUserName><![CDATA[{}]]></ToUserName>
<FromUserName><![CDATA[{}]]></FromUserName>
<CreateTime>{}</CreateTime>
<MsgType><![CDATA[video]]></MsgType>
<Video>
<MediaId><![CDATA[{}]]></MediaId>
<Title><![CDATA[{}]]></Title>
<Description><![CDATA[{}]]></Description>
</Video>
</xml>"#,
                    reply.to_user,
                    reply.from_user,
                    timestamp,
                    media_id,
                    title.as_deref().unwrap_or(""),
                    description.as_deref().unwrap_or("")
                )
            }
            ReplyContent::News { articles } => {
                let article_xml: String = articles
                    .iter()
                    .map(|a| {
                        format!(
                            r#"<item>
<Title><![CDATA[{}]]></Title>
<Description><![CDATA[{}]]></Description>
<PicUrl><![CDATA[{}]]></PicUrl>
<Url><![CDATA[{}]]></Url>
</item>"#,
                            a.title,
                            a.description.as_deref().unwrap_or(""),
                            a.pic_url.as_deref().unwrap_or(""),
                            a.url.as_deref().unwrap_or("")
                        )
                    })
                    .collect();

                format!(
                    r#"<xml>
<ToUserName><![CDATA[{}]]></ToUserName>
<FromUserName><![CDATA[{}]]></FromUserName>
<CreateTime>{}</CreateTime>
<MsgType><![CDATA[news]]></MsgType>
<ArticleCount>{}</ArticleCount>
<Articles>{}</Articles>
</xml>"#,
                    reply.to_user,
                    reply.from_user,
                    timestamp,
                    articles.len(),
                    article_xml
                )
            }
        }
    }

    fn check_error<T>(&self, response: &WeChatApiResponse<T>) -> Result<(), ChannelError> {
        if let Some(errcode) = response.errcode {
            if errcode != 0 {
                return Err(ChannelError::ApiError {
                    code: Some(errcode.to_string()),
                    message: response.errmsg.clone().unwrap_or_default(),
                });
            }
        }
        Ok(())
    }

    async fn parse_error_response(&self, response: reqwest::Response) -> ChannelError {
        let status = response.status();

        if status.as_u16() == 401 {
            return ChannelError::AuthenticationFailed("Invalid credentials".to_string());
        }

        let error_text = response.text().await.unwrap_or_default();

        if let Ok(api_response) = serde_json::from_str::<WeChatApiResponse<()>>(&error_text) {
            if let Some(errcode) = api_response.errcode {
                return ChannelError::ApiError {
                    code: Some(errcode.to_string()),
                    message: api_response.errmsg.unwrap_or_default(),
                };
            }
        }

        ChannelError::ApiError {
            code: Some(status.to_string()),
            message: error_text,
        }
    }
}

impl Default for WeChatProvider {
    fn default() -> Self {
        Self::new()
    }
}

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
        let article = NewsArticle {
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

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessTokenResponse {
    pub access_token: Option<String>,
    pub expires_in: Option<u64>,
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeChatApiResponse<T> {
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
    #[serde(flatten)]
    pub data: Option<T>,
    pub msgid: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMessage {
    pub touser: String,
    pub template_id: String,
    pub url: Option<String>,
    pub miniprogram: Option<MiniProgram>,
    pub data: HashMap<String, TemplateDataItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiniProgram {
    pub appid: String,
    pub pagepath: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateDataItem {
    pub value: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMessageResult {
    pub msgid: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "msgtype", rename_all = "lowercase")]
pub enum CustomerMessage {
    Text {
        touser: String,
        text: TextContent,
    },
    Image {
        touser: String,
        image: MediaContent,
    },
    Voice {
        touser: String,
        voice: MediaContent,
    },
    Video {
        touser: String,
        video: VideoContent,
    },
    Music {
        touser: String,
        music: MusicContent,
    },
    News {
        touser: String,
        news: NewsContent,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextContent {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaContent {
    pub media_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoContent {
    pub media_id: String,
    pub thumb_media_id: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicContent {
    pub title: Option<String>,
    pub description: Option<String>,
    pub musicurl: String,
    pub hqmusicurl: String,
    pub thumb_media_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsContent {
    pub articles: Vec<NewsItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsItem {
    pub title: String,
    pub description: Option<String>,
    pub url: Option<String>,
    pub picurl: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaType {
    Image,
    Voice,
    Video,
    Thumb,
}

impl MediaType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Image => "image",
            Self::Voice => "voice",
            Self::Video => "video",
            Self::Thumb => "thumb",
        }
    }

    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Image => "image/jpeg",
            Self::Voice => "audio/amr",
            Self::Video => "video/mp4",
            Self::Thumb => "image/jpeg",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaUploadResponse {
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
    #[serde(rename = "type")]
    pub media_type: Option<String>,
    pub media_id: Option<String>,
    pub created_at: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct MediaUploadResult {
    pub media_type: String,
    pub media_id: String,
    pub created_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoDescription {
    pub title: String,
    pub introduction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermanentMediaResponse {
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
    pub media_id: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PermanentMediaResult {
    pub media_id: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsArticle {
    pub title: String,
    pub author: Option<String>,
    pub digest: Option<String>,
    pub content: String,
    pub content_source_url: Option<String>,
    pub thumb_media_id: String,
    pub need_open_comment: Option<i32>,
    pub only_fans_can_comment: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftResponse {
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
    pub media_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishResponse {
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
    pub publish_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PublishResult {
    pub publish_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishStatusResponse {
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
    pub publish_status: Option<i32>,
    pub article_id: Option<String>,
    pub article_detail: Option<ArticleDetail>,
    pub fail_idx: Option<Vec<i32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleDetail {
    pub count: Option<i32>,
    pub item: Option<Vec<ArticleItem>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleItem {
    pub idx: Option<i32>,
    pub article_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PublishStatus {
    pub publish_id: String,
    pub publish_status: i32, // 0=success, 1=publishing, 2=failed
    pub article_id: Option<String>,
    pub article_detail: Option<ArticleDetail>,
    pub fail_idx: Option<Vec<i32>>,
}

impl PublishStatus {
    pub fn is_success(&self) -> bool {
        self.publish_status == 0
    }

    pub fn is_publishing(&self) -> bool {
        self.publish_status == 1
    }

    pub fn is_failed(&self) -> bool {
        self.publish_status == 2
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeChatUserResponse {
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
    pub subscribe: Option<i32>,
    pub openid: Option<String>,
    pub nickname: Option<String>,
    pub sex: Option<i32>,
    pub language: Option<String>,
    pub city: Option<String>,
    pub province: Option<String>,
    pub country: Option<String>,
    pub headimgurl: Option<String>,
    pub subscribe_time: Option<i64>,
    pub unionid: Option<String>,
    pub remark: Option<String>,
    pub groupid: Option<i32>,
    pub tagid_list: Option<Vec<i32>>,
    pub subscribe_scene: Option<String>,
    pub qr_scene: Option<i32>,
    pub qr_scene_str: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WeChatUser {
    pub subscribe: i32,
    pub openid: String,
    pub nickname: Option<String>,
    pub sex: Option<i32>,
    pub language: Option<String>,
    pub city: Option<String>,
    pub province: Option<String>,
    pub country: Option<String>,
    pub headimgurl: Option<String>,
    pub subscribe_time: Option<i64>,
    pub unionid: Option<String>,
    pub remark: Option<String>,
    pub groupid: Option<i32>,
    pub tagid_list: Option<Vec<i32>>,
    pub subscribe_scene: Option<String>,
    pub qr_scene: Option<i32>,
    pub qr_scene_str: Option<String>,
}

impl WeChatUser {
    pub fn is_subscribed(&self) -> bool {
        self.subscribe == 1
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowerListResponse {
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
    pub total: Option<i32>,
    pub count: Option<i32>,
    pub data: Option<FollowerData>,
    pub next_openid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowerData {
    pub openid: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct FollowerList {
    pub total: i32,
    pub count: i32,
    pub openids: Vec<String>,
    pub next_openid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Menu {
    pub button: Vec<MenuButton>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuButton {
    #[serde(rename = "type")]
    pub button_type: Option<String>,
    pub name: String,
    pub key: Option<String>,
    pub url: Option<String>,
    pub media_id: Option<String>,
    pub appid: Option<String>,
    pub pagepath: Option<String>,
    pub article_id: Option<String>,
    pub sub_button: Option<Vec<MenuButton>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QRCodeRequest {
    pub expire_seconds: Option<i32>,
    pub action_name: String, // "QR_SCENE", "QR_STR_SCENE", "QR_LIMIT_SCENE", "QR_LIMIT_STR_SCENE"
    pub action_info: ActionInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionInfo {
    pub scene: Scene,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    pub scene_id: Option<i32>,
    pub scene_str: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QRCodeResponse {
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
    pub ticket: Option<String>,
    pub expire_seconds: Option<i32>,
    pub url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct QRCodeResult {
    pub ticket: String,
    pub expire_seconds: Option<i32>,
    pub url: String,
    pub qrcode_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortUrlResponse {
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
    pub short_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct IncomingMessage {
    pub to_user_name: String,
    pub from_user_name: String,
    pub create_time: i64,
    pub msg_type: String,
    pub msg_id: Option<String>,
    pub content: Option<String>,
    pub pic_url: Option<String>,
    pub media_id: Option<String>,
    pub format: Option<String>,
    pub recognition: Option<String>,
    pub thumb_media_id: Option<String>,
    pub location_x: Option<f64>,
    pub location_y: Option<f64>,
    pub scale: Option<i32>,
    pub label: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub event: Option<String>,
    pub event_key: Option<String>,
    pub ticket: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub precision: Option<f64>,
}

impl IncomingMessage {
    pub fn is_text(&self) -> bool {
        self.msg_type == "text"
    }

    pub fn is_image(&self) -> bool {
        self.msg_type == "image"
    }

    pub fn is_voice(&self) -> bool {
        self.msg_type == "voice"
    }

    pub fn is_video(&self) -> bool {
        self.msg_type == "video"
    }

    pub fn is_location(&self) -> bool {
        self.msg_type == "location"
    }

    pub fn is_link(&self) -> bool {
        self.msg_type == "link"
    }

    pub fn is_event(&self) -> bool {
        self.msg_type == "event"
    }

    pub fn is_subscribe_event(&self) -> bool {
        self.is_event() && self.event.as_deref() == Some("subscribe")
    }

    pub fn is_unsubscribe_event(&self) -> bool {
        self.is_event() && self.event.as_deref() == Some("unsubscribe")
    }

    pub fn is_scan_event(&self) -> bool {
        self.is_event() && self.event.as_deref() == Some("SCAN")
    }

    pub fn is_click_event(&self) -> bool {
        self.is_event() && self.event.as_deref() == Some("CLICK")
    }
}

#[derive(Debug, Clone)]
pub struct ReplyMessage {
    pub to_user: String,
    pub from_user: String,
    pub content: ReplyContent,
}

#[derive(Debug, Clone)]
pub enum ReplyContent {
    Text { content: String },
    Image { media_id: String },
    Voice { media_id: String },
    Video {
        media_id: String,
        title: Option<String>,
        description: Option<String>,
    },
    News { articles: Vec<ReplyArticle> },
}

#[derive(Debug, Clone)]
pub struct ReplyArticle {
    pub title: String,
    pub description: Option<String>,
    pub pic_url: Option<String>,
    pub url: Option<String>,
}

// ============================================================================
// Error Codes
// ============================================================================

pub struct WeChatErrorCodes;

impl WeChatErrorCodes {
    pub const SUCCESS: i32 = 0;
    pub const INVALID_CREDENTIAL: i32 = 40001;
    pub const INVALID_GRANT_TYPE: i32 = 40002;
    pub const INVALID_OPENID: i32 = 40003;
    pub const INVALID_MEDIA_TYPE: i32 = 40004;
    pub const INVALID_MEDIA_ID: i32 = 40007;
    pub const INVALID_MESSAGE_TYPE: i32 = 40008;
    pub const INVALID_IMAGE_SIZE: i32 = 40009;
    pub const INVALID_VOICE_SIZE: i32 = 40010;
    pub const INVALID_VIDEO_SIZE: i32 = 40011;
    pub const INVALID_THUMB_SIZE: i32 = 40012;
    pub const INVALID_APPID: i32 = 40013;
    pub const INVALID_ACCESS_TOKEN: i32 = 40014;
    pub const INVALID_MENU_TYPE: i32 = 40015;
    pub const INVALID_BUTTON_COUNT: i32 = 40016;
    pub const ACCESS_TOKEN_EXPIRED: i32 = 42001;
    pub const REQUIRE_SUBSCRIBE: i32 = 43004;
    pub const API_LIMIT_REACHED: i32 = 45009;
    pub const API_BLOCKED: i32 = 48001;
}
