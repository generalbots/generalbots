use super::*;

impl PostContent {
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            text: Some(text.into()),
            image_urls: vec![],
            video_url: None,
            link: None,
            hashtags: vec![],
            mentions: vec![],
            scheduled_at: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_image(mut self, url: impl Into<String>) -> Self {
        self.image_urls.push(url.into());
        self
    }

    pub fn with_images(mut self, urls: Vec<String>) -> Self {
        self.image_urls.extend(urls);
        self
    }

    pub fn with_video(mut self, url: impl Into<String>) -> Self {
        self.video_url = Some(url.into());
        self
    }

    pub fn with_link(mut self, url: impl Into<String>) -> Self {
        self.link = Some(url.into());
        self
    }

    pub fn with_hashtags(mut self, tags: Vec<String>) -> Self {
        self.hashtags.extend(tags);
        self
    }

    pub fn with_mentions(mut self, mentions: Vec<String>) -> Self {
        self.mentions.extend(mentions);
        self
    }

    pub fn scheduled(mut self, at: chrono::DateTime<chrono::Utc>) -> Self {
        self.scheduled_at = Some(at);
        self
    }
}

#[cfg(test)]
mod post_content_tests {
    use super::*;

    #[test]
    fn a_text_post_carries_only_its_text() {
        let content = PostContent::text("hello");
        assert_eq!(content.text.as_deref(), Some("hello"));
        assert!(content.image_urls.is_empty());
        assert!(content.video_url.is_none());
        assert!(content.link.is_none());
        assert!(content.hashtags.is_empty());
        assert!(content.mentions.is_empty());
        assert!(content.scheduled_at.is_none());
        assert!(content.metadata.is_empty());
    }

    #[test]
    fn builders_accumulate_media_and_tags() {
        let content = PostContent::text("launch")
            .with_image("https://cdn/a.png")
            .with_images(vec![
                "https://cdn/b.png".to_string(),
                "https://cdn/c.png".to_string(),
            ])
            .with_video("https://cdn/clip.mp4")
            .with_link("https://example.com")
            .with_hashtags(vec!["#launch".to_string()])
            .with_hashtags(vec!["#gb".to_string()])
            .with_mentions(vec!["@acme".to_string()]);

        assert_eq!(content.image_urls.len(), 3);
        assert_eq!(content.image_urls[0], "https://cdn/a.png");
        assert_eq!(content.video_url.as_deref(), Some("https://cdn/clip.mp4"));
        assert_eq!(content.link.as_deref(), Some("https://example.com"));
        assert_eq!(content.hashtags, vec!["#launch", "#gb"]);
        assert_eq!(content.mentions, vec!["@acme"]);
    }

    #[test]
    fn scheduling_sets_the_timestamp() {
        let when = chrono::DateTime::from_timestamp(1_800_000_000, 0).expect("ts");
        let content = PostContent::text("later").scheduled(when);
        assert_eq!(content.scheduled_at, Some(when));
    }
}
