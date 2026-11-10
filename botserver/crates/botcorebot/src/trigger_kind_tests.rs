//! Offline unit tests for the automation trigger kind (#2064).
//!
//! `system_automations.kind` is an integer column; `TriggerKind::from_i32` is
//! the only decoder. An unknown value must return `None` (the caller logs and
//! skips the row) instead of defaulting to `Scheduled`, which would fire a cron
//! job for a row that is not a cron job.

use super::TriggerKind;

const ALL: [(i32, TriggerKind); 12] = [
    (0, TriggerKind::Scheduled),
    (1, TriggerKind::TableUpdate),
    (2, TriggerKind::TableInsert),
    (3, TriggerKind::TableDelete),
    (4, TriggerKind::Webhook),
    (5, TriggerKind::EmailReceived),
    (6, TriggerKind::FolderChange),
    (7, TriggerKind::DealStageChange),
    (8, TriggerKind::ContactChange),
    (9, TriggerKind::EmailOpened),
    (10, TriggerKind::MediaUploaded),
    (11, TriggerKind::MessageReceived),
];

#[test]
fn every_persisted_kind_round_trips() {
    for (value, expected) in ALL {
        assert_eq!(TriggerKind::from_i32(value), Some(expected), "kind {value}");
    }
}

#[test]
fn the_discriminant_matches_the_column_value() {
    for (value, kind) in ALL {
        assert_eq!(kind as i32, value, "{kind:?} must encode as {value}");
    }
}

#[test]
fn unknown_and_negative_values_are_rejected() {
    for value in [-1, 12, 13, 99, i32::MIN, i32::MAX] {
        assert_eq!(TriggerKind::from_i32(value), None, "kind {value}");
    }
}

#[test]
fn kinds_are_distinct() {
    let mut seen: Vec<TriggerKind> = Vec::new();
    for (_, kind) in ALL {
        assert!(!seen.contains(&kind), "{kind:?} is duplicated");
        seen.push(kind);
    }
    assert_eq!(seen.len(), 12);
}
