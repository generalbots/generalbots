//! WeChat message sending functionality

use super::client::WeChatProvider;
use super::types::{
    CustomerMessage, ReplyArticle, ReplyContent, ReplyMessage, TemplateMessage,
    TemplateMessageResult, WeChatApiResponse,
};
use crate::channels::ChannelError;

impl WeChatProvider {
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

    /// Verify webhook signature
    pub fn verify_signature(
        &self,
        token: &str,
        timestamp: &str,
        nonce: &str,
        signature: &str,
    ) -> bool {
        use sha1::{Digest, Sha1};

        let mut params = [token, timestamp, nonce];
        params.sort();
        let joined = params.join("");

        let mut hasher = Sha1::new();
        hasher.update(joined.as_bytes());
        let result = hasher.finalize();
        let computed = hex::encode(result);

        computed == signature
    }

    /// Parse incoming message XML
    pub fn parse_message(&self, xml: &str) -> Result<super::types::IncomingMessage, ChannelError> {
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

        Ok(super::types::IncomingMessage {
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
                    .map(|a: &ReplyArticle| {
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
}
