use crate::channels::{ChannelManager, ChannelType, PostContent};
use crate::core::shared::models::UserSession;
use crate::core::shared::state::AppState;
use rhai::{Dynamic, Engine, EvalAltResult, Map};
use std::sync::Arc;

pub fn post_to_keyword(state: &Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(state);
    let user_clone = user.clone();

    engine.register_fn(
        "POST_TO",
        move |channel: &str, message: &str| -> Result<Dynamic, Box<EvalAltResult>> {
            post_to_impl(&state_clone, &user_clone, channel, message, None, None)
        },
    );

    let state_clone = Arc::clone(state);
    let user_clone = user.clone();

    engine.register_fn(
        "POST_TO_WITH_IMAGE",
        move |channel: &str, message: &str, image: &str| -> Result<Dynamic, Box<EvalAltResult>> {
            post_to_impl(&state_clone, &user_clone, channel, message, Some(image), None)
        },
    );

    let state_clone = Arc::clone(state);
    let user_clone = user.clone();

    engine.register_fn(
        "POST_TO_WITH_VIDEO",
        move |channel: &str, message: &str, video: &str| -> Result<Dynamic, Box<EvalAltResult>> {
            post_to_impl(&state_clone, &user_clone, channel, message, None, Some(video))
        },
    );

    let state_clone = Arc::clone(state);
    let user_clone = user.clone();

    engine.register_fn(
        "POST_TO_MULTIPLE",
        move |channels: &str, message: &str| -> Result<Dynamic, Box<EvalAltResult>> {
            post_to_multiple_impl(&state_clone, &user_clone, channels, message, None, None)
        },
    );

    let state_clone = Arc::clone(state);
    let user_clone = user.clone();

    engine.register_fn(
        "POST_TO_ADVANCED",
        move |options: Map| -> Result<Dynamic, Box<EvalAltResult>> {
            post_to_advanced_impl(&state_clone, &user_clone, options)
        },
    );
}

fn post_to_impl(
    state: &Arc<AppState>,
    user: &UserSession,
    channel: &str,
    message: &str,
    image: Option<&str>,
    video: Option<&str>,
) -> Result<Dynamic, Box<EvalAltResult>> {
    let channel_manager = get_channel_manager(state)?;
    let account_name = channel.to_string();

    let mut content = PostContent::text(message);

    if let Some(img) = image {
        content = content.with_image(img);
    }

    if let Some(vid) = video {
        content = content.with_video(vid);
    }

    let rt = tokio::runtime::Handle::try_current().map_err(|e| {
        Box::new(EvalAltResult::ErrorRuntime(
            format!("No async runtime available: {}", e).into(),
            rhai::Position::NONE,
        ))
    })?;

    let result = rt.block_on(async { channel_manager.post_to(&account_name, &content).await });

    match result {
        Ok(post_result) => {
            let mut map = Map::new();
            map.insert("success".into(), Dynamic::from(post_result.success));
            map.insert(
                "channel".into(),
                Dynamic::from(post_result.channel_type.to_string()),
            );
            if let Some(post_id) = post_result.post_id {
                map.insert("post_id".into(), Dynamic::from(post_id));
            }
            if let Some(url) = post_result.url {
                map.insert("url".into(), Dynamic::from(url));
            }
            if let Some(error) = post_result.error {
                map.insert("error".into(), Dynamic::from(error));
            }
            Ok(Dynamic::from_map(map))
        }
        Err(e) => {
            let mut map = Map::new();
            map.insert("success".into(), Dynamic::from(false));
            map.insert("error".into(), Dynamic::from(e.to_string()));
            Ok(Dynamic::from_map(map))
        }
    }
}

fn post_to_multiple_impl(
    state: &Arc<AppState>,
    user: &UserSession,
    channels: &str,
    message: &str,
    image: Option<&str>,
    video: Option<&str>,
) -> Result<Dynamic, Box<EvalAltResult>> {
    let channel_manager = get_channel_manager(state)?;

    let account_names: Vec<String> = channels
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let mut content = PostContent::text(message);

    if let Some(img) = image {
        content = content.with_image(img);
    }

    if let Some(vid) = video {
        content = content.with_video(vid);
    }

    let rt = tokio::runtime::Handle::try_current().map_err(|e| {
        Box::new(EvalAltResult::ErrorRuntime(
            format!("No async runtime available: {}", e).into(),
            rhai::Position::NONE,
        ))
    })?;

    let results =
        rt.block_on(async { channel_manager.post_to_multiple(&account_names, &content).await });

    let mut total = 0;
    let mut successful = 0;
    let mut failed = 0;
    let mut result_list = Vec::new();

    for result in results {
        total += 1;
        match result {
            Ok(post_result) => {
                if post_result.success {
                    successful += 1;
                } else {
                    failed += 1;
                }
                let mut item = Map::new();
                item.insert("success".into(), Dynamic::from(post_result.success));
                item.insert(
                    "channel".into(),
                    Dynamic::from(post_result.channel_type.to_string()),
                );
                if let Some(post_id) = post_result.post_id {
                    item.insert("post_id".into(), Dynamic::from(post_id));
                }
                if let Some(url) = post_result.url {
                    item.insert("url".into(), Dynamic::from(url));
                }
                if let Some(error) = post_result.error {
                    item.insert("error".into(), Dynamic::from(error));
                }
                result_list.push(Dynamic::from_map(item));
            }
            Err(e) => {
                failed += 1;
                let mut item = Map::new();
                item.insert("success".into(), Dynamic::from(false));
                item.insert("error".into(), Dynamic::from(e.to_string()));
                result_list.push(Dynamic::from_map(item));
            }
        }
    }

    let mut map = Map::new();
    map.insert("total".into(), Dynamic::from(total as i64));
    map.insert("successful".into(), Dynamic::from(successful as i64));
    map.insert("failed".into(), Dynamic::from(failed as i64));
    map.insert("results".into(), Dynamic::from(result_list));

    Ok(Dynamic::from_map(map))
}

fn post_to_advanced_impl(
    state: &Arc<AppState>,
    user: &UserSession,
    options: Map,
) -> Result<Dynamic, Box<EvalAltResult>> {
    let channel_manager = get_channel_manager(state)?;

    let channel = options
        .get("channel")
        .and_then(|v| v.clone().into_string().ok())
        .ok_or_else(|| {
            Box::new(EvalAltResult::ErrorRuntime(
                "Missing 'channel' in options".into(),
                rhai::Position::NONE,
            ))
        })?;

    let message = options
        .get("message")
        .and_then(|v| v.clone().into_string().ok())
        .unwrap_or_default();

    let mut content = PostContent::text(message);

    if let Some(image) = options.get("image").and_then(|v| v.clone().into_string().ok()) {
        content = content.with_image(image);
    }

    if let Some(images) = options.get("images") {
        if let Some(arr) = images.clone().try_cast::<rhai::Array>() {
            for img in arr {
                if let Ok(url) = img.into_string() {
                    content = content.with_image(url);
                }
            }
        }
    }

    if let Some(video) = options.get("video").and_then(|v| v.clone().into_string().ok()) {
        content = content.with_video(video);
    }

    if let Some(link) = options.get("link").and_then(|v| v.clone().into_string().ok()) {
        content = content.with_link(link);
    }

    if let Some(hashtags) = options.get("hashtags") {
        if let Some(arr) = hashtags.clone().try_cast::<rhai::Array>() {
            let tags: Vec<String> = arr
                .into_iter()
                .filter_map(|v| v.into_string().ok())
                .collect();
            content = content.with_hashtags(tags);
        }
    }

    if let Some(mentions) = options.get("mentions") {
        if let Some(arr) = mentions.clone().try_cast::<rhai::Array>() {
            let mention_list: Vec<String> = arr
                .into_iter()
                .filter_map(|v| v.into_string().ok())
                .collect();
            content = content.with_mentions(mention_list);
        }
    }

    let rt = tokio::runtime::Handle::try_current().map_err(|e| {
        Box::new(EvalAltResult::ErrorRuntime(
            format!("No async runtime available: {}", e).into(),
            rhai::Position::NONE,
        ))
    })?;

    let result = rt.block_on(async { channel_manager.post_to(&channel, &content).await });

    match result {
        Ok(post_result) => {
            let mut map = Map::new();
            map.insert("success".into(), Dynamic::from(post_result.success));
            map.insert(
                "channel".into(),
                Dynamic::from(post_result.channel_type.to_string()),
            );
            if let Some(post_id) = post_result.post_id {
                map.insert("post_id".into(), Dynamic::from(post_id));
            }
            if let Some(url) = post_result.url {
                map.insert("url".into(), Dynamic::from(url));
            }
            if let Some(error) = post_result.error {
                map.insert("error".into(), Dynamic::from(error));
            }
            Ok(Dynamic::from_map(map))
        }
        Err(e) => {
            let mut map = Map::new();
            map.insert("success".into(), Dynamic::from(false));
            map.insert("error".into(), Dynamic::from(e.to_string()));
            Ok(Dynamic::from_map(map))
        }
    }
}

fn get_channel_manager(state: &Arc<AppState>) -> Result<Arc<ChannelManager>, Box<EvalAltResult>> {
    state.channel_manager.clone().ok_or_else(|| {
        Box::new(EvalAltResult::ErrorRuntime(
            "Channel manager not configured".into(),
            rhai::Position::NONE,
        ))
    })
}

pub fn get_channel_limits(channel_name: &str) -> Result<Dynamic, Box<EvalAltResult>> {
    let channel_type: ChannelType = channel_name.parse().map_err(|e: crate::channels::ChannelError| {
        Box::new(EvalAltResult::ErrorRuntime(
            e.to_string().into(),
            rhai::Position::NONE,
        ))
    })?;

    let mut map = Map::new();

    let (max_text, supports_images, supports_video, supports_links) = match channel_type {
        ChannelType::Twitter => (280, true, true, true),
        ChannelType::Bluesky => (300, true, false, true),
        ChannelType::Threads => (500, true, true, true),
        ChannelType::Instagram => (2200, true, true, true),
        ChannelType::Facebook => (63206, true, true, true),
        ChannelType::LinkedIn => (3000, true, true, true),
        ChannelType::Discord => (2000, true, true, true),
        ChannelType::Telegram => (4096, true, true, true),
        ChannelType::WhatsApp => (4096, true, true, true),
        ChannelType::Reddit => (40000, true, true, true),
        ChannelType::TikTok => (2200, false, true, true),
        ChannelType::YouTube => (5000, false, true, true),
        ChannelType::Pinterest => (500, true, false, true),
        ChannelType::Snapchat => (250, true, true, false),
        ChannelType::WeChat => (2000, true, true, true),
        ChannelType::TwilioSms => (1600, false, false, true),
    };

    map.insert("max_text_length".into(), Dynamic::from(max_text as i64));
    map.insert("supports_images".into(), Dynamic::from(supports_images));
    map.insert("supports_video".into(), Dynamic::from(supports_video));
    map.insert("supports_links".into(), Dynamic::from(supports_links));

    Ok(Dynamic::from_map(map))
}
