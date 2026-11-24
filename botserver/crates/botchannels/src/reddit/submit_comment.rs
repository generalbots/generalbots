use super::*;

impl RedditChannel {
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
            let error_text = crate::response_error_text(response).await;
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
            id: crate::required_string(comment_data.id, "comment id"),
            body: crate::required_string(comment_data.body, "comment body"),
            author: crate::required_string(comment_data.author, "comment author"),
            score: comment_data.score.unwrap_or(0),
            created_utc: comment_data.created_utc.unwrap_or(0.0),
            parent_id: crate::required_string(comment_data.parent_id, "comment parent_id"),
            link_id: crate::required_string(comment_data.link_id, "comment link_id"),
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
            let error_text = crate::response_error_text(response).await;
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
            id: crate::required_string(post_data.id, "post id"),
            subreddit: crate::required_string(post_data.subreddit, "post subreddit"),
            title: crate::required_string(post_data.title, "post title"),
            selftext: post_data.selftext,
            url: post_data.url,
            author: crate::required_string(post_data.author, "post author"),
            score: post_data.score.unwrap_or(0),
            num_comments: post_data.num_comments.unwrap_or(0),
            created_utc: post_data.created_utc.unwrap_or(0.0),
            permalink: crate::required_string(post_data.permalink, "post permalink"),
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
            let error_text = crate::response_error_text(response).await;
            return Err(RedditError::ApiError(error_text));
        }

        let about: AboutResponse = response
            .json()
            .await
            .map_err(|e| RedditError::ParseError(e.to_string()))?;

        let data = about.data;

        Ok(Subreddit {
            name: crate::required_string(data.name, "subreddit name"),
            display_name: crate::required_string(data.display_name, "subreddit display name"),
            title: crate::required_string(data.title, "subreddit title"),
            description: data.description,
            subscribers: data.subscribers.unwrap_or(0),
            public_description: data.public_description,
            subreddit_type: crate::string_or_empty(data.subreddit_type),
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
            let error_text = crate::response_error_text(response).await;
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
                id: crate::required_string(d.id, "post id"),
                subreddit: crate::required_string(d.subreddit, "post subreddit"),
                title: crate::required_string(d.title, "post title"),
                selftext: d.selftext,
                url: d.url,
                author: crate::required_string(d.author, "post author"),
                score: d.score.unwrap_or(0),
                num_comments: d.num_comments.unwrap_or(0),
                created_utc: d.created_utc.unwrap_or(0.0),
                permalink: crate::required_string(d.permalink, "post permalink"),
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
            let error_text = crate::response_error_text(response).await;
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
            let error_text = crate::response_error_text(response).await;
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
            let error_text = crate::response_error_text(response).await;
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
            let error_text = crate::response_error_text(response).await;
            return Err(RedditError::ApiError(error_text));
        }

        Ok(())
    }

    pub async fn set_tokens(&self, tokens: RedditTokens) {
        let mut token_guard = self.tokens.write().await;
        *token_guard = Some(tokens);
    }
}
