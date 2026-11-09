use super::*;

impl ChannelManager {
    pub fn new() -> Self {
        Self {
            accounts: Arc::new(RwLock::new(HashMap::new())),
            providers: HashMap::new(),
        }
    }

    /// #2055 — the manager a real caller gets: every provider implemented in
    /// this crate is registered, so `post_to` can reach one. The returned list
    /// is also the startup log, which makes a missing registration visible
    /// instead of surfacing as a silent `NotConfigured`.
    pub fn with_defaults() -> Self {
        let mut manager = Self::new();
        let registered = manager.register_default_providers();
        log::info!("channel providers registered: {registered:?}");
        manager
    }

    /// Registers the implemented providers and returns their channel types.
    pub fn register_default_providers(&mut self) -> Vec<ChannelType> {
        let providers: Vec<Arc<dyn ChannelProvider>> = vec![
            Arc::new(bluesky::BlueskyProvider::new()),
            Arc::new(threads::ThreadsProvider::new()),
            Arc::new(discord::DiscordProvider::default()),
            Arc::new(pinterest::PinterestProvider::default()),
            Arc::new(snapchat::SnapchatProvider::default()),
            Arc::new(tiktok::TikTokProvider::default()),
            Arc::new(wechat::WeChatProvider::default()),
        ];
        let types: Vec<ChannelType> =
            providers.iter().map(|provider| provider.channel_type()).collect();
        for provider in providers {
            self.register_provider(provider);
        }
        types
    }

    /// Channel types with a registered provider (the catalog's truth).
    pub fn registered_channels(&self) -> Vec<ChannelType> {
        let mut types: Vec<ChannelType> = self.providers.keys().cloned().collect();
        types.sort_by_key(|channel| format!("{channel:?}"));
        types
    }

    pub fn register_provider(&mut self, provider: Arc<dyn ChannelProvider>) {
        self.providers.insert(provider.channel_type(), provider);
    }

    pub async fn add_account(&self, account: ChannelAccount) {
        let mut accounts = self.accounts.write().await;
        accounts.insert(account.name.clone(), account);
    }

    pub async fn get_account(&self, name: &str) -> Option<ChannelAccount> {
        let accounts = self.accounts.read().await;
        accounts.get(name).cloned()
    }

    pub async fn remove_account(&self, name: &str) -> Option<ChannelAccount> {
        let mut accounts = self.accounts.write().await;
        accounts.remove(name)
    }

    pub async fn list_accounts(&self) -> Vec<ChannelAccount> {
        let accounts = self.accounts.read().await;
        accounts.values().cloned().collect()
    }

    pub async fn post_to(
        &self,
        account_name: &str,
        content: &PostContent,
    ) -> Result<PostResult, ChannelError> {
        let account = self
            .get_account(account_name)
            .await
            .ok_or_else(|| ChannelError::AccountNotFound(account_name.to_string()))?;

        let provider = self
            .providers
            .get(&account.channel_type)
            .ok_or(ChannelError::NotConfigured)?;

        provider.post(&account, content).await
    }

    pub async fn post_to_multiple(
        &self,
        account_names: &[String],
        content: &PostContent,
    ) -> Vec<Result<PostResult, ChannelError>> {
        let mut results = Vec::with_capacity(account_names.len());

        for name in account_names {
            let result = self.post_to(name, content).await;
            results.push(result);
        }

        results
    }

    pub async fn post_to_channels(
        &self,
        channels: &[ChannelType],
        content: &PostContent,
    ) -> Vec<Result<PostResult, ChannelError>> {
        let accounts = self.accounts.read().await;
        let mut results = Vec::new();

        for channel_type in channels {
            let matching_accounts: Vec<_> = accounts
                .values()
                .filter(|a| &a.channel_type == channel_type && a.is_active)
                .collect();

            for account in matching_accounts {
                if let Some(provider) = self.providers.get(channel_type) {
                    let result = provider.post(account, content).await;
                    results.push(result);
                } else {
                    results.push(Err(ChannelError::NotConfigured));
                }
            }
        }

        results
    }

    pub fn get_channel_limits(&self, channel_type: &ChannelType) -> Option<ChannelLimits> {
        self.providers.get(channel_type).map(|p| ChannelLimits {
            max_text_length: p.max_text_length(),
            supports_images: p.supports_images(),
            supports_video: p.supports_video(),
            supports_links: p.supports_links(),
        })
    }
}
impl Default for ChannelManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// #2055 — the five placeholder providers are gone; the manager must be
    /// populated with the real implementations so post_to can reach them.
    #[test]
    fn the_default_manager_registers_every_implemented_provider() {
        let manager = ChannelManager::with_defaults();
        let channels = manager.registered_channels();
        for channel in [
            ChannelType::Bluesky,
            ChannelType::Threads,
            ChannelType::Discord,
            ChannelType::Pinterest,
            ChannelType::Snapchat,
            ChannelType::TikTok,
            ChannelType::WeChat,
        ] {
            assert!(
                channels.contains(&channel),
                "{channel:?} must be registered"
            );
            assert!(
                manager.get_channel_limits(&channel).is_some(),
                "{channel:?} limits must be readable"
            );
        }
        assert_eq!(channels.len(), 7);
    }

    /// A registered channel reaches the provider; an account that does not
    /// exist is reported as such (never as a silent success).
    #[tokio::test]
    async fn post_to_reports_a_missing_account_instead_of_pretending() {
        let manager = ChannelManager::with_defaults();
        let content = PostContent {
            text: Some("hello".to_string()),
            image_urls: Vec::new(),
            video_url: None,
            link: None,
            hashtags: Vec::new(),
            mentions: Vec::new(),
            scheduled_at: None,
            metadata: HashMap::new(),
        };
        match manager.post_to("missing", &content).await {
            Err(ChannelError::AccountNotFound(name)) => assert_eq!(name, "missing"),
            other => panic!("expected AccountNotFound, got {other:?}"),
        }
    }
}
