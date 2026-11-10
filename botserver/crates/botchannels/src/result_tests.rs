//! Aggregation contract for post attempts (CHAN-3, CHAN-4 in #2070).

use super::*;

fn ok(channel: ChannelType, id: &str) -> Result<PostResult, ChannelError> {
    Ok(PostResult::success(channel, id.to_string(), None))
}

fn failed(channel: ChannelType) -> Result<PostResult, ChannelError> {
    Ok(PostResult::error(channel, "provider refused"))
}

#[test]
fn a_successful_post_sets_its_id_and_clears_the_error() {
    let result = PostResult::success(
        ChannelType::Bluesky,
        "at://did:plc:1".to_string(),
        Some("https://bsky.app/p/1".to_string()),
    );

    assert!(result.success);
    assert_eq!(result.channel_type, ChannelType::Bluesky);
    assert_eq!(result.post_id.as_deref(), Some("at://did:plc:1"));
    assert_eq!(result.url.as_deref(), Some("https://bsky.app/p/1"));
    assert!(result.error.is_none());
    // Metadata is an empty map, never None: callers index into it directly.
    assert!(result.metadata.is_empty());
}

#[test]
fn a_failed_post_drops_the_id_and_url() {
    let result = PostResult::error(ChannelType::Discord, "missing permissions");

    assert!(!result.success);
    assert!(result.post_id.is_none());
    assert!(result.url.is_none());
    assert_eq!(result.error.as_deref(), Some("missing permissions"));
    assert!(result.metadata.is_empty());
}

#[test]
fn two_successes_and_two_failures_aggregate_to_four() {
    let aggregate = MultiPostResult::from_results(
        ChannelType::Instagram,
        vec![
            ok(ChannelType::Instagram, "ig-1"),
            ok(ChannelType::Instagram, "ig-2"),
            failed(ChannelType::Instagram),
            Err(ChannelError::RateLimited {
                retry_after: Some(30),
            }),
        ],
    );

    assert_eq!(aggregate.total, 4);
    assert_eq!(aggregate.successful, 2);
    assert_eq!(aggregate.failed, 2);
    assert_eq!(aggregate.results.len(), 4);
}

#[test]
fn an_error_arm_is_labelled_with_the_requested_channel_not_twitter() {
    // Regression: the error arm hardcoded `ChannelType::Twitter`, so an
    // Instagram failure surfaced as a Twitter post in the UI.
    let aggregate = MultiPostResult::from_results(
        ChannelType::Instagram,
        vec![Err(ChannelError::NotConfigured)],
    );

    assert_eq!(aggregate.results[0].channel_type, ChannelType::Instagram);
    assert_eq!(
        aggregate.results[0].error.as_deref(),
        Some("Channel not configured")
    );
    assert!(aggregate.results[0].post_id.is_none());
    assert!(!aggregate.results[0].success);
}

#[test]
fn a_provider_side_failure_keeps_its_own_channel() {
    let aggregate = MultiPostResult::from_results(
        ChannelType::Telegram,
        vec![failed(ChannelType::WhatsApp)],
    );

    // An `Ok` result already knows its channel: the default must not override.
    assert_eq!(aggregate.results[0].channel_type, ChannelType::WhatsApp);
    assert_eq!(aggregate.failed, 1);
    assert_eq!(aggregate.successful, 0);
}

#[test]
fn an_empty_run_is_zeroed() {
    let aggregate = MultiPostResult::from_results(ChannelType::Reddit, Vec::new());

    assert_eq!(aggregate.total, 0);
    assert_eq!(aggregate.successful, 0);
    assert_eq!(aggregate.failed, 0);
    assert!(aggregate.results.is_empty());
}
