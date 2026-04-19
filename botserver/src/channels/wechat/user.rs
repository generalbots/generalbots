//! WeChat user management functionality

use super::client::WeChatProvider;
use super::types::{FollowerList, WeChatUser, WeChatUserResponse};
use crate::channels::ChannelError;

impl WeChatProvider {
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

        let result: super::types::FollowerListResponse =
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
}
