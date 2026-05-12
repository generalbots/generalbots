use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedditConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub user_agent: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedditTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedditPost {
    pub id: String,
    pub subreddit: String,
    pub title: String,
    pub selftext: Option<String>,
    pub url: Option<String>,
    pub author: String,
    pub score: i64,
    pub num_comments: u64,
    pub created_utc: f64,
    pub permalink: String,
    pub is_self: bool,
    pub link_flair_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedditComment {
    pub id: String,
    pub body: String,
    pub author: String,
    pub score: i64,
    pub created_utc: f64,
    pub parent_id: String,
    pub link_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subreddit {
    pub name: String,
    pub display_name: String,
    pub title: String,
    pub description: Option<String>,
    pub subscribers: u64,
    pub public_description: Option<String>,
    pub subreddit_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitPostRequest {
    pub subreddit: String,
    pub title: String,
    pub kind: PostKind,
    pub content: String,
    pub flair_id: Option<String>,
    pub flair_text: Option<String>,
    pub nsfw: bool,
    pub spoiler: bool,
    pub send_replies: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PostKind {
    #[serde(rename = "self")]
    Text,
    Link,
    Image,
    Video,
    Poll,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitCommentRequest {
    pub parent_id: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedditUser {
    pub id: String,
    pub name: String,
    pub created_utc: f64,
    pub link_karma: i64,
    pub comment_karma: i64,
    pub is_gold: bool,
    pub is_mod: bool,
    pub has_verified_email: bool,
    pub icon_img: Option<String>,
}

pub struct RedditChannel {
    config: RedditConfig,
    http_client: Client,
    tokens: Arc<RwLock<Option<RedditTokens>>>,
    base_url: String,
    oauth_url: String,
}

impl RedditChannel {
    pub fn new(config: RedditConfig) -> Self {
        let http_client = Client::builder()
            .user_agent(&config.user_agent)
            .build()
            .unwrap_or_default();

        Self {
            config,
            http_client,
            tokens: Arc::new(RwLock::new(None)),
            base_url: "https://oauth.reddit.com".to_string(),
            oauth_url: "https://www.reddit.com/api/v1".to_string(),
        }
    }

    pub fn get_authorization_url(&self, state: &str, scopes: &[&str]) -> String {
        let scope = scopes.join(" ");
        format!(
            "{}/authorize?client_id={}&response_type=code&state={}&redirect_uri={}&duration=permanent&scope={}",
            self.oauth_url,
            self.config.client_id,
            state,
            urlencoding::encode(&self.config.redirect_uri),
            urlencoding::encode(&scope)
        )
    }

    pub async fn exchange_code(&self, code: &str) -> Result<RedditTokens, RedditError> {
        let response = self
            .http_client
            .post(format!("{}/access_token", self.oauth_url))
            .basic_auth(&self.config.client_id, Some(&self.config.client_secret))
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", code),
                ("redirect_uri", &self.config.redirect_uri),
            ])
            .send()
            .await
            .map_err(|e| RedditError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(RedditError::ApiError(error_text));
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| RedditError::ParseError(e.to_string()))?;

        let tokens = RedditTokens {
            access_token: token_response.access_token,
            refresh_token: token_response.refresh_token,
            expires_at: Utc::now() + chrono::Duration::seconds(token_response.expires_in as i64),
            scope: token_response.scope,
        };

        let mut token_guard = self.tokens.write().await;
        *token_guard = Some(tokens.clone());

        Ok(tokens)
    }

    pub async fn refresh_token(&self) -> Result<RedditTokens, RedditError> {
        let tokens = self.tokens.read().await;
        let refresh_token = tokens
            .as_ref()
            .and_then(|t| t.refresh_token.clone())
            .ok_or(RedditError::NotAuthenticated)?;
        drop(tokens);

        let response = self
            .http_client
            .post(format!("{}/access_token", self.oauth_url))
            .basic_auth(&self.config.client_id, Some(&self.config.client_secret))
            .form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", &refresh_token),
            ])
            .send()
            .await
            .map_err(|e| RedditError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(RedditError::ApiError(error_text));
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| RedditError::ParseError(e.to_string()))?;

        let new_tokens = RedditTokens {
            access_token: token_response.access_token,
            refresh_token: token_response.refresh_token.or(Some(refresh_token)),
            expires_at: Utc::now() + chrono::Duration::seconds(token_response.expires_in as i64),
            scope: token_response.scope,
        };

        let mut token_guard = self.tokens.write().await;
        *token_guard = Some(new_tokens.clone());

        Ok(new_tokens)
    }

    pub async fn authenticate_script(&self) -> Result<RedditTokens, RedditError> {
        let username = self
            .config
            .username
            .as_ref()
            .ok_or(RedditError::ConfigError("Username required for script auth".to_string()))?;
        let password = self
            .config
            .password
            .as_ref()
            .ok_or(RedditError::ConfigError("Password required for script auth".to_string()))?;

        let response = self
            .http_client
            .post(format!("{}/access_token", self.oauth_url))
            .basic_auth(&self.config.client_id, Some(&self.config.client_secret))
            .form(&[
                ("grant_type", "password"),
                ("username", username.as_str()),
                ("password", password.as_str()),
            ])
            .send()
            .await
            .map_err(|e| RedditError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(RedditError::ApiError(error_text));
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| RedditError::ParseError(e.to_string()))?;

        let tokens = RedditTokens {
            access_token: token_response.access_token,
            refresh_token: token_response.refresh_token,
            expires_at: Utc::now() + chrono::Duration::seconds(token_response.expires_in as i64),
            scope: token_response.scope,
        };

        let mut token_guard = self.tokens.write().await;
        *token_guard = Some(tokens.clone());

        Ok(tokens)
    }

    async fn get_access_token(&self) -> Result<String, RedditError> {
        let tokens = self.tokens.read().await;
        match tokens.as_ref() {
            Some(t) if t.expires_at > Utc::now() => Ok(t.access_token.clone()),
            Some(_) => {
                drop(tokens);
                let new_tokens = self.refresh_token().await?;
                Ok(new_tokens.access_token)
            }
            None => Err(RedditError::NotAuthenticated),
        }
    }

    pub async fn get_me(&self) -> Result<RedditUser, RedditError> {
        let token = self.get_access_token().await?;

        let response = self
            .http_client
            .get(format!("{}/api/v1/me", self.base_url))
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| RedditError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(RedditError::ApiError(error_text));
        }

        let user: RedditUser = response
            .json()
            .await
            .map_err(|e| RedditError::ParseError(e.to_string()))?;

        Ok(user)
    }

    pub async fn submit_post(&self, request: SubmitPostRequest) -> Result<RedditPost, RedditError> {
        let token = self.get_access_token().await?;

        let kind = match request.kind {
            PostKind::Text => "self",
            PostKind::Link => "link",
            PostKind::Image => "image",
            PostKind::Video => "video",
            PostKind::Poll => "poll",
        };

        let mut params: HashMap<&str, String> = HashMap::new();
        params.insert("sr", request.subreddit.clone());
        params.insert("title", request.title.clone());
        params.insert("kind", kind.to_string());
        params.insert("api_type", "json".to_string());
        params.insert("send_replies", request.send_replies.to_string());
        params.insert("nsfw", request.nsfw.to_string());
        params.insert("spoiler", request.spoiler.to_string());

        match request.kind {
            PostKind::Text => {
                params.insert("text", request.content.clone());
            }
            PostKind::Link | PostKind::Image | PostKind::Video => {
                params.insert("url", request.content.clone());
            }
            PostKind::Poll => {
                params.insert("text", request.content.clone());
            }
        }

        if let Some(flair_id) = &request.flair_id {
            params.insert("flair_id", flair_id.clone());
        }
        if let Some(flair_text) = &request.flair_text {
            params.insert("flair_text", flair_text.clone());
        }

        let response = self
            .http_client
            .post(format!("{}/api/submit", self.base_url))
            .bearer_auth(&token)
            .form(&params)
            .send()
            .await
            .map_err(|e| RedditError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(RedditError::ApiError(error_text));
        }

        let submit_response: SubmitResponse = response
            .json()
            .await
            .map_err(|e| RedditError::ParseError(e.to_string()))?;

        if let Some(errors) = submit_response.json.errors {
            if !errors.is_empty() {
                let error_msg = errors
                    .iter()
                    .map(|e| e.join(": "))
                    .collect::<Vec<_>>()
                    .join(", ");
                return Err(RedditError::ApiError(error_msg));
            }
        }

        let post_id = submit_response
            .json
            .data
            .ok_or_else(|| RedditError::ApiError("No post ID returned".to_string()))?
            .id;

        self.get_post(&post_id).await
    }

    pub async fn submit_comment(&self, request: SubmitCommentRequest) -> Result<RedditComment, RedditError> {
        let token = self.get_access_token().await?;

        let response = self
            .http_client
            .post(format!("{}/api/comment", self.base_url))
            .bearer_auth(&token)
            .form(&[
                ("api_type", "json"),
                ("thing_id", &request.parent_id),
                ("text", &request.text),
            ])
            .send()
            .await
            .map_err(|e| RedditError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(RedditError::ApiError(error_text));
        }

        let comment_response: CommentResponse = response
            .json()
            .await
            .map_err(|e| RedditError::ParseError(e.to_string()))?;

        if let Some(errors) = comment_response.json.errors {
            if !errors.is_empty() {
                let error_msg = errors
                    .iter()
                    .map(|e| e.join(": "))
                    .collect::<Vec<_>>()
                    .join(", ");
                return Err(RedditError::ApiError(error_msg));
            }
        }

        let comment_data = comment_response
            .json
            .data
            .and_then(|d| d.things.into_iter().next())
            .and_then(|t| t.data)
            .ok_or_else(|| RedditError::ApiError("No comment data returned".to_string()))?;

        Ok(RedditComment {
            id: comment_data.id.unwrap_or_default(),
            body: comment_data.body.unwrap_or_default(),
            author: comment_data.author.unwrap_or_default(),
            score: comment_data.score.unwrap_or(0),
            created_utc: comment_data.created_utc.unwrap_or(0.0),
            parent_id: comment_data.parent_id.unwrap_or_default(),
            link_id: comment_data.link_id.unwrap_or_default(),
        })
    }

    pub async fn get_post(&self, post_id: &str) -> Result<RedditPost, RedditError> {
        let token = self.get_access_token().await?;

        let id = if post_id.starts_with("t3_") {
            post_id.to_string()
        } else {
            format!("t3_{}", post_id)
        };

        let response = self
            .http_client
            .get(format!("{}/api/info?id={}", self.base_url, id))
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| RedditError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(RedditError::ApiError(error_text));
        }

        let listing: ListingResponse = response
            .json()
            .await
            .map_err(|e| RedditError::ParseError(e.to_string()))?;

        let post_data = listing
            .data
            .children
            .into_iter()
            .next()
            .and_then(|c| c.data)
            .ok_or(RedditError::PostNotFound)?;

        Ok(RedditPost {
            id: post_data.id.unwrap_or_default(),
            subreddit: post_data.subreddit.unwrap_or_default(),
            title: post_data.title.unwrap_or_default(),
            selftext: post_data.selftext,
            url: post_data.url,
            author: post_data.author.unwrap_or_default(),
            score: post_data.score.unwrap_or(0),
            num_comments: post_data.num_comments.unwrap_or(0),
            created_utc: post_data.created_utc.unwrap_or(0.0),
            permalink: post_data.permalink.unwrap_or_default(),
            is_self: post_data.is_self.unwrap_or(false),
            link_flair_text: post_data.link_flair_text,
        })
    }

    pub async fn get_subreddit(&self, name: &str) -> Result<Subreddit, RedditError> {
        let token = self.get_access_token().await?;

        let response = self
            .http_client
            .get(format!("{}/r/{}/about", self.base_url, name))
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| RedditError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(RedditError::ApiError(error_text));
        }

        let about: AboutResponse = response
            .json()
            .await
            .map_err(|e| RedditError::ParseError(e.to_string()))?;

        let data = about.data;

        Ok(Subreddit {
            name: data.name.unwrap_or_default(),
            display_name: data.display_name.unwrap_or_default(),
            title: data.title.unwrap_or_default(),
            description: data.description,
            subscribers: data.subscribers.unwrap_or(0),
            public_description: data.public_description,
            subreddit_type: data.subreddit_type.unwrap_or_default(),
        })
    }

    pub async fn get_subreddit_posts(
        &self,
        subreddit: &str,
        sort: PostSort,
        limit: u32,
    ) -> Result<Vec<RedditPost>, RedditError> {
        let token = self.get_access_token().await?;

        let sort_str = match sort {
            PostSort::Hot => "hot",
            PostSort::New => "new",
            PostSort::Top => "top",
            PostSort::Rising => "rising",
            PostSort::Controversial => "controversial",
        };

        let response = self
            .http_client
            .get(format!(
                "{}/r/{}/{}?limit={}",
                self.base_url, subreddit, sort_str, limit
            ))
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| RedditError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(RedditError::ApiError(error_text));
        }

        let listing: ListingResponse = response
            .json()
            .await
            .map_err(|e| RedditError::ParseError(e.to_string()))?;

        let posts = listing
            .data
            .children
            .into_iter()
            .filter_map(|c| c.data)
            .map(|d| RedditPost {
                id: d.id.unwrap_or_default(),
                subreddit: d.subreddit.unwrap_or_default(),
                title: d.title.unwrap_or_default(),
                selftext: d.selftext,
                url: d.url,
                author: d.author.unwrap_or_default(),
                score: d.score.unwrap_or(0),
                num_comments: d.num_comments.unwrap_or(0),
                created_utc: d.created_utc.unwrap_or(0.0),
                permalink: d.permalink.unwrap_or_default(),
                is_self: d.is_self.unwrap_or(false),
                link_flair_text: d.link_flair_text,
            })
            .collect();

        Ok(posts)
    }

    pub async fn vote(&self, thing_id: &str, direction: VoteDirection) -> Result<(), RedditError> {
        let token = self.get_access_token().await?;

        let dir = match direction {
            VoteDirection::Up => "1",
            VoteDirection::Down => "-1",
            VoteDirection::None => "0",
        };

        let response = self
            .http_client
            .post(format!("{}/api/vote", self.base_url))
            .bearer_auth(&token)
            .form(&[("id", thing_id), ("dir", dir)])
            .send()
            .await
            .map_err(|e| RedditError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(RedditError::ApiError(error_text));
        }

        Ok(())
    }

    pub async fn delete(&self, thing_id: &str) -> Result<(), RedditError> {
        let token = self.get_access_token().await?;

        let response = self
            .http_client
            .post(format!("{}/api/del", self.base_url))
            .bearer_auth(&token)
            .form(&[("id", thing_id)])
            .send()
            .await
            .map_err(|e| RedditError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(RedditError::ApiError(error_text));
        }

        Ok(())
    }

    pub async fn edit(&self, thing_id: &str, new_text: &str) -> Result<(), RedditError> {
        let token = self.get_access_token().await?;

        let response = self
            .http_client
            .post(format!("{}/api/editusertext", self.base_url))
            .bearer_auth(&token)
            .form(&[
                ("api_type", "json"),
                ("thing_id", thing_id),
                ("text", new_text),
            ])
            .send()
            .await
            .map_err(|e| RedditError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(RedditError::ApiError(error_text));
        }

        Ok(())
    }

    pub async fn subscribe(&self, subreddit: &str, subscribe: bool) -> Result<(), RedditError> {
        let token = self.get_access_token().await?;

        let action = if subscribe { "sub" } else { "unsub" };

        let response = self
            .http_client
            .post(format!("{}/api/subscribe", self.base_url))
            .bearer_auth(&token)
            .form(&[
                ("action", action),
                ("sr_name", subreddit),
            ])
            .send()
            .await
            .map_err(|e| RedditError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(RedditError::ApiError(error_text));
        }

        Ok(())
    }

    pub async fn set_tokens(&self, tokens: RedditTokens) {
        let mut token_guard = self.tokens.write().await;
        *token_guard = Some(tokens);
    }

    pub async fn is_authenticated(&self) -> bool {
        let tokens = self.tokens.read().await;
        tokens.is_some()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PostSort {
    Hot,
    New,
    Top,
    Rising,
    Controversial,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum VoteDirection {
    Up,
    Down,
    None,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: u64,
    scope: String,
}

#[derive(Debug, Deserialize)]
struct SubmitResponse {
    json: SubmitJsonResponse,
}

#[derive(Debug, Deserialize)]
struct SubmitJsonResponse {
    errors: Option<Vec<Vec<String>>>,
    data: Option<SubmitDataResponse>,
}

#[derive(Debug, Deserialize)]
struct SubmitDataResponse {
    id: String,
}

#[derive(Debug, Deserialize)]
struct CommentResponse {
    json: CommentJsonResponse,
}

#[derive(Debug, Deserialize)]
struct CommentJsonResponse {
    errors: Option<Vec<Vec<String>>>,
    data: Option<CommentDataResponse>,
}

#[derive(Debug, Deserialize)]
struct CommentDataResponse {
    things: Vec<ThingWrapper>,
}

#[derive(Debug, Deserialize)]
struct ThingWrapper {
    data: Option<ThingData>,
}

#[derive(Debug, Deserialize)]
struct ThingData {
    id: Option<String>,
    body: Option<String>,
    author: Option<String>,
    score: Option<i64>,
    created_utc: Option<f64>,
    parent_id: Option<String>,
    link_id: Option<String>,
    subreddit: Option<String>,
    title: Option<String>,
    selftext: Option<String>,
    url: Option<String>,
    num_comments: Option<u64>,
    permalink: Option<String>,
    is_self: Option<bool>,
    link_flair_text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ListingResponse {
    data: ListingData,
}

#[derive(Debug, Deserialize)]
struct ListingData {
    children: Vec<ThingWrapper>,
}

#[derive(Debug, Deserialize)]
struct AboutResponse {
    data: AboutData,
}

#[derive(Debug, Deserialize)]
struct AboutData {
    name: Option<String>,
    display_name: Option<String>,
    title: Option<String>,
    description: Option<String>,
    subscribers: Option<u64>,
    public_description: Option<String>,
    subreddit_type: Option<String>,
}

#[derive(Debug, Clone)]
pub enum RedditError {
    NetworkError(String),
    ApiError(String),
    ParseError(String),
    ConfigError(String),
    NotAuthenticated,
    PostNotFound,
    RateLimited,
}

impl std::fmt::Display for RedditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NetworkError(e) => write!(f, "Network error: {}", e),
            Self::ApiError(e) => write!(f, "Reddit API error: {}", e),
            Self::ParseError(e) => write!(f, "Parse error: {}", e),
            Self::ConfigError(e) => write!(f, "Configuration error: {}", e),
            Self::NotAuthenticated => write!(f, "Not authenticated with Reddit"),
            Self::PostNotFound => write!(f, "Post not found"),
            Self::RateLimited => write!(f, "Rate limited by Reddit API"),
        }
    }
}

impl std::error::Error for RedditError {}

pub fn create_reddit_config(
    client_id: &str,
    client_secret: &str,
    redirect_uri: &str,
    user_agent: &str,
) -> RedditConfig {
    RedditConfig {
        client_id: client_id.to_string(),
        client_secret: client_secret.to_string(),
        redirect_uri: redirect_uri.to_string(),
        user_agent: user_agent.to_string(),
        username: None,
        password: None,
    }
}

pub fn create_script_config(
    client_id: &str,
    client_secret: &str,
    username: &str,
    password: &str,
    user_agent: &str,
) -> RedditConfig {
    RedditConfig {
        client_id: client_id.to_string(),
        client_secret: client_secret.to_string(),
        redirect_uri: String::new(),
        user_agent: user_agent.to_string(),
        username: Some(username.to_string()),
        password: Some(password.to_string()),
    }
}
