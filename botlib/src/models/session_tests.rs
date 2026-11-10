//! Contract tests for `Session` / `UserSession` (#2064).
//!
//! Expiry is the only branching logic here, and it is what hides an expired
//! conversation from history — so both sides of the comparison are pinned.

use super::{Session, UserSession};
use chrono::{Duration, Utc};
use uuid::Uuid;

#[test]
fn a_new_session_is_active_and_unexpired() {
    let user_id = Uuid::new_v4();
    let bot_id = Uuid::new_v4();
    let session = Session::new(user_id, bot_id, "Chat");

    assert_eq!(session.user_id, user_id);
    assert_eq!(session.bot_id, bot_id);
    assert_eq!(session.title, "Chat");
    assert_eq!(session.created_at, session.updated_at);
    assert!(session.expires_at.is_none());
    assert!(session.is_active());
    assert!(!session.is_expired());
    assert!(session.remaining_time().is_none());
}

#[test]
fn an_expiry_in_the_past_is_expired() {
    let session = Session::new(Uuid::new_v4(), Uuid::new_v4(), "Chat")
        .with_expiry(Utc::now() - Duration::seconds(1));

    assert!(session.is_expired());
    assert!(!session.is_active());
    assert!(session.remaining_time().is_some_and(|d| d.num_seconds() < 0));
}

#[test]
fn an_expiry_in_the_future_leaves_time_remaining() {
    let session = Session::new(Uuid::new_v4(), Uuid::new_v4(), "Chat")
        .with_expiry(Utc::now() + Duration::hours(1));

    assert!(!session.is_expired());
    assert!(session.is_active());
    let remaining = session.remaining_time().expect("expiry set");
    assert!(remaining.num_minutes() >= 59 && remaining.num_minutes() <= 60);
}

#[test]
fn every_session_gets_its_own_id() {
    let a = Session::new(Uuid::nil(), Uuid::nil(), "a");
    let b = Session::new(Uuid::nil(), Uuid::nil(), "b");
    assert_ne!(a.id, b.id);
}

#[test]
fn a_new_user_session_starts_unlocked_and_branchless() {
    let session = UserSession::new(Uuid::new_v4(), Uuid::new_v4(), "Chat");

    assert_eq!(session.branch_id, Uuid::nil());
    assert!(!session.locked);
    assert!(session.lock_pin_hash.is_none());
    assert_eq!(session.context_data, serde_json::json!({}));
    assert!(session.current_tool.is_none());
    assert_eq!(session.created_at, session.updated_at);
}

#[test]
fn a_user_session_round_trips_through_json() {
    let mut session = UserSession::new(Uuid::new_v4(), Uuid::new_v4(), "Chat");
    session.locked = true;
    session.context_data = serde_json::json!({"roles": ["admin"]});

    let encoded = serde_json::to_string(&session).expect("serializable");
    let decoded: UserSession = serde_json::from_str(&encoded).expect("deserializable");

    assert!(decoded.locked, "#1682: the locked flag must survive a round-trip");
    assert_eq!(decoded.context_data, session.context_data);
    assert_eq!(decoded.id, session.id);
}
