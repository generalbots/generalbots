use crate::channels::{
    ChannelAccount, ChannelCredentials, ChannelError, ChannelProvider, ChannelType,
    PostContent, PostResult,
};
use serde::{Deserialize, Serialize};

pub struct BlueskyProvider {
    client: reqwest::Client,
}

impl BlueskyProvider {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    async fn create_session(
        &self,
        identifier: &str,
        password: &str,
    ) -> Result<BlueskySession, ChannelError> {
        let request = CreateSessionRequest {
            identifier: identifier.to_string(),
            password: password.to_string(),
        };

        let response = self
            .client
            .post("https://bsky.social/xrpc/com.atproto.server.createSession")
            .json(&request)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ChannelError::AuthenticationFailed(error_text));
        }

        response
            .json::<BlueskySession>()
            .await
            .map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })
    }

    async fn create_post(
        &self,
        session: &BlueskySession,
        text: &str,
        images: &[UploadedBlob],
        link: Option<&str>,
    ) -> Result<CreateRecordResponse, ChannelError> {
        let mut record = PostRecord {
            text: text.to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            embed: None,
            facets: vec![],
        };

        if !images.is_empty() {
            let image_embeds: Vec<ImageEmbed> = images
                .iter()
                .map(|blob| ImageEmbed {
                    alt: String::new(),
                    image: blob.blob.clone(),
                })
                .collect();

            record.embed = Some(PostEmbed::Images {
                embed_type: "app.bsky.embed.images".to_string(),
                images: image_embeds,
            });
        } else if let Some(url) = link {
            record.embed = Some(PostEmbed::External {
                embed_type: "app.bsky.embed.external".to_string(),
                external: ExternalEmbed {
                    uri: url.to_string(),
                    title: String::new(),
                    description: String::new(),
                },
            });
        }

        record.facets = self.extract_facets(text);

        let request = CreateRecordRequest {
            repo: session.did.clone(),
            collection: "app.bsky.feed.post".to_string(),
            record,
        };

        let response = self
            .client
            .post("https://bsky.social/xrpc/com.atproto.repo.createRecord")
            .header("Authorization", format!("Bearer {}", session.access_jwt))
            .json(&request)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            if status.as_u16() == 429 {
                return Err(ChannelError::RateLimited { retry_after: None });
            }

            return Err(ChannelError::ApiError {
                code: Some(status.to_string()),
                message: error_text,
            });
        }

        response
            .json::<CreateRecordResponse>()
            .await
            .map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })
    }

    fn extract_facets(&self, text: &str) -> Vec<Facet> {
        let mut facets = Vec::new();

        for (idx, word) in text.split_whitespace().enumerate() {
            let byte_start = text
                .split_whitespace()
                .take(idx)
                .map(|w| w.len() + 1)
                .sum::<usize>();
            let byte_end = byte_start + word.len();

            if word.starts_with('@') && word.len() > 1 {
                facets.push(Facet {
                    index: FacetIndex {
                        byte_start,
                        byte_end,
                    },
                    features: vec![FacetFeature::Mention {
                        feature_type: "app.bsky.richtext.facet#mention".to_string(),
                        did: word[1..].to_string(),
                    }],
                });
            } else if word.starts_with("https://") || word.starts_with("http://") {
                facets.push(Facet {
                    index: FacetIndex {
                        byte_start,
                        byte_end,
                    },
                    features: vec![FacetFeature::Link {
                        feature_type: "app.bsky.richtext.facet#link".to_string(),
                        uri: word.to_string(),
                    }],
                });
            } else if word.starts_with('#') && word.len() > 1 {
                facets.push(Facet {
                    index: FacetIndex {
                        byte_start,
                        byte_end,
                    },
                    features: vec![FacetFeature::Tag {
                        feature_type: "app.bsky.richtext.facet#tag".to_string(),
                        tag: word[1..].to_string(),
                    }],
                });
            }
        }

        facets
    }

    fn build_post_url(handle: &str, rkey: &str) -> String {
        format!("https://bsky.app/profile/{}/post/{}", handle, rkey)
    }
}

impl Default for BlueskyProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl ChannelProvider for BlueskyProvider {
    fn channel_type(&self) -> ChannelType {
        ChannelType::Bluesky
    }

    fn max_text_length(&self) -> usize {
        300
    }

    fn supports_images(&self) -> bool {
        true
    }

    fn supports_video(&self) -> bool {
        false
    }

    fn supports_links(&self) -> bool {
        true
    }

    async fn post(
        &self,
        account: &ChannelAccount,
        content: &PostContent,
    ) -> Result<PostResult, ChannelError> {
        let (identifier, password) = match &account.credentials {
            ChannelCredentials::UsernamePassword {
                username,
                password,
                app_password,
            } => {
                let pwd = app_password.as_ref().unwrap_or(password);
                (username.clone(), pwd.clone())
            }
            _ => {
                return Err(ChannelError::AuthenticationFailed(
                    "Invalid credentials type for Bluesky".to_string(),
                ))
            }
        };

        let session = self.create_session(&identifier, &password).await?;

        let text = content.text.as_deref().unwrap_or("");

        if text.len() > self.max_text_length() {
            return Err(ChannelError::ContentTooLong {
                max_length: self.max_text_length(),
                actual_length: text.len(),
            });
        }

        let images: Vec<UploadedBlob> = Vec::new();

        let link = content.link.as_deref();

        let response = self.create_post(&session, text, &images, link).await?;

        let rkey = response
            .uri
            .split('/')
            .last()
            .unwrap_or("")
            .to_string();

        let url = Self::build_post_url(&session.handle, &rkey);

        Ok(PostResult::success(
            ChannelType::Bluesky,
            response.uri,
            Some(url),
        ))
    }

    async fn validate_credentials(
        &self,
        credentials: &ChannelCredentials,
    ) -> Result<bool, ChannelError> {
        let (identifier, password) = match credentials {
            ChannelCredentials::UsernamePassword {
                username,
                password,
                app_password,
            } => {
                let pwd = app_password.as_ref().unwrap_or(password);
                (username.clone(), pwd.clone())
            }
            _ => return Ok(false),
        };

        match self.create_session(&identifier, &password).await {
            Ok(_) => Ok(true),
            Err(ChannelError::AuthenticationFailed(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }

    async fn refresh_token(&self, _account: &mut ChannelAccount) -> Result<(), ChannelError> {
        Ok(())
    }
}

#[derive(Debug, Serialize)]
struct CreateSessionRequest {
    identifier: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct BlueskySession {
    did: String,
    handle: String,
    #[serde(rename = "accessJwt")]
    access_jwt: String,
}

#[derive(Debug, Serialize)]
struct CreateRecordRequest {
    repo: String,
    collection: String,
    record: PostRecord,
}

#[derive(Debug, Serialize)]
struct PostRecord {
    text: String,
    #[serde(rename = "createdAt")]
    created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    embed: Option<PostEmbed>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    facets: Vec<Facet>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum PostEmbed {
    Images {
        #[serde(rename = "$type")]
        embed_type: String,
        images: Vec<ImageEmbed>,
    },
    External {
        #[serde(rename = "$type")]
        embed_type: String,
        external: ExternalEmbed,
    },
}

#[derive(Debug, Serialize)]
struct ImageEmbed {
    alt: String,
    image: BlobRef,
}

#[derive(Debug, Serialize)]
struct ExternalEmbed {
    uri: String,
    title: String,
    description: String,
}

#[derive(Debug, Serialize)]
struct Facet {
    index: FacetIndex,
    features: Vec<FacetFeature>,
}

#[derive(Debug, Serialize)]
struct FacetIndex {
    #[serde(rename = "byteStart")]
    byte_start: usize,
    #[serde(rename = "byteEnd")]
    byte_end: usize,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum FacetFeature {
    Mention {
        #[serde(rename = "$type")]
        feature_type: String,
        did: String,
    },
    Link {
        #[serde(rename = "$type")]
        feature_type: String,
        uri: String,
    },
    Tag {
        #[serde(rename = "$type")]
        feature_type: String,
        tag: String,
    },
}

#[derive(Debug, Deserialize)]
struct CreateRecordResponse {
    uri: String,
}

#[derive(Debug, Deserialize)]
struct UploadedBlob {
    blob: BlobRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BlobRef {
    #[serde(rename = "$type")]
    blob_type: String,
    #[serde(rename = "ref")]
    reference: BlobLink,
    #[serde(rename = "mimeType")]
    mime_type: String,
    size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BlobLink {
    #[serde(rename = "$link")]
    link: String,
}
