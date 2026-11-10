//! The account-deletion state machine: grace period, single active request,
//! token-gated confirmation, cancellation by the owner only, and execution
//! only after confirmation. The service keeps its requests in memory, so these
//! run without a database.
use super::*;

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Runtime::new().expect("tokio runtime")
}

fn request(immediate: bool) -> CreateDeletionRequest {
    CreateDeletionRequest {
        reason: Some("no longer needed".to_string()),
        feedback: None,
        immediate,
    }
}

#[test]
fn the_grace_period_is_configurable() {
    assert_eq!(AccountDeletionService::new(7).grace_period_days(), 7);
    assert_eq!(AccountDeletionService::default().grace_period_days(), 30);
}

#[test]
fn a_new_request_awaits_confirmation_and_carries_a_token() {
    runtime().block_on(async {
        let service = AccountDeletionService::new(7);
        let user = Uuid::new_v4();

        let created = service
            .create_deletion_request(user, "u@example.com".into(), None, request(false))
            .await
            .expect("request created");

        assert_eq!(created.status, DeletionStatus::AwaitingConfirmation);
        assert_eq!(created.confirmation_token.len(), 16);
        assert!(created.confirmed_at.is_none());
        assert!(created.completed_at.is_none());
        assert!(created.deletion_report.is_none());
        assert_eq!(
            (created.scheduled_for - created.requested_at).num_days(),
            7,
            "the grace period must be applied"
        );
    });
}

#[test]
fn an_immediate_request_is_scheduled_for_now() {
    runtime().block_on(async {
        let service = AccountDeletionService::new(30);

        let created = service
            .create_deletion_request(Uuid::new_v4(), "u@example.com".into(), None, request(true))
            .await
            .expect("request created");

        assert_eq!(created.scheduled_for, created.requested_at);
    });
}

#[test]
fn a_user_cannot_stack_a_second_active_request() {
    runtime().block_on(async {
        let service = AccountDeletionService::new(30);
        let user = Uuid::new_v4();
        service
            .create_deletion_request(user, "u@example.com".into(), None, request(false))
            .await
            .expect("first request");

        let error = service
            .create_deletion_request(user, "u@example.com".into(), None, request(false))
            .await
            .expect_err("second request must be rejected");

        assert!(matches!(error, AccountDeletionError::RequestAlreadyExists));
    });
}

#[test]
fn a_wrong_token_cannot_confirm() {
    runtime().block_on(async {
        let service = AccountDeletionService::new(30);
        let created = service
            .create_deletion_request(Uuid::new_v4(), "u@example.com".into(), None, request(false))
            .await
            .expect("request created");

        let error = service
            .confirm_deletion(created.id, "0000000000000000")
            .await
            .expect_err("wrong token must be rejected");

        assert!(matches!(
            error,
            AccountDeletionError::InvalidConfirmationToken
        ));
    });
}

#[test]
fn confirming_with_the_issued_token_moves_to_confirmed_once() {
    runtime().block_on(async {
        let service = AccountDeletionService::new(30);
        let created = service
            .create_deletion_request(Uuid::new_v4(), "u@example.com".into(), None, request(false))
            .await
            .expect("request created");

        let confirmed = service
            .confirm_deletion(created.id, &created.confirmation_token)
            .await
            .expect("confirmation accepted");
        assert_eq!(confirmed.status, DeletionStatus::Confirmed);
        assert!(confirmed.confirmed_at.is_some());

        let error = service
            .confirm_deletion(created.id, &created.confirmation_token)
            .await
            .expect_err("second confirmation must be rejected");
        assert!(matches!(error, AccountDeletionError::InvalidStatus));
    });
}

#[test]
fn an_unknown_request_cannot_be_confirmed() {
    runtime().block_on(async {
        let service = AccountDeletionService::new(30);

        let error = service
            .confirm_deletion(Uuid::new_v4(), "token")
            .await
            .expect_err("unknown request must be rejected");

        assert!(matches!(error, AccountDeletionError::RequestNotFound));
    });
}

#[test]
fn only_the_owner_can_cancel() {
    runtime().block_on(async {
        let service = AccountDeletionService::new(30);
        let user = Uuid::new_v4();
        let created = service
            .create_deletion_request(user, "u@example.com".into(), None, request(false))
            .await
            .expect("request created");

        let error = service
            .cancel_deletion(created.id, Uuid::new_v4())
            .await
            .expect_err("a stranger must not cancel");
        assert!(matches!(error, AccountDeletionError::Unauthorized));

        let cancelled = service
            .cancel_deletion(created.id, user)
            .await
            .expect("owner may cancel");
        assert_eq!(cancelled.status, DeletionStatus::Cancelled);
    });
}

#[test]
fn cancelling_frees_the_user_to_open_a_new_request() {
    runtime().block_on(async {
        let service = AccountDeletionService::new(30);
        let user = Uuid::new_v4();
        let created = service
            .create_deletion_request(user, "u@example.com".into(), None, request(false))
            .await
            .expect("request created");
        service
            .cancel_deletion(created.id, user)
            .await
            .expect("cancelled");

        let again = service
            .create_deletion_request(user, "u@example.com".into(), None, request(false))
            .await;
        assert!(again.is_ok());
    });
}

#[test]
fn deletion_cannot_run_before_confirmation() {
    runtime().block_on(async {
        let service = AccountDeletionService::new(30);
        let created = service
            .create_deletion_request(Uuid::new_v4(), "u@example.com".into(), None, request(false))
            .await
            .expect("request created");

        let error = service
            .execute_deletion(created.id)
            .await
            .expect_err("unconfirmed request must not execute");

        assert!(matches!(error, AccountDeletionError::NotConfirmed));
    });
}

#[test]
fn a_confirmed_immediate_request_is_pending_then_completes() {
    runtime().block_on(async {
        let service = AccountDeletionService::new(30);
        let user = Uuid::new_v4();
        let created = service
            .create_deletion_request(user, "u@example.com".into(), None, request(true))
            .await
            .expect("request created");
        service
            .confirm_deletion(created.id, &created.confirmation_token)
            .await
            .expect("confirmed");

        let pending = service.get_pending_deletions().await;
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].id, created.id);

        let report = service
            .execute_deletion(created.id)
            .await
            .expect("execution succeeds");
        assert_eq!(report.user_id, user);
        assert!(report.confirmation_number.starts_with("DEL-"));
        assert!(report.total_records_deleted > 0);
        assert!(!report.data_deleted.is_empty());

        let stored = service
            .get_deletion_request(created.id)
            .await
            .expect("request still stored");
        assert_eq!(stored.status, DeletionStatus::Completed);
        assert!(stored.completed_at.is_some());
        assert!(stored.deletion_report.is_some());

        let error = service
            .execute_deletion(created.id)
            .await
            .expect_err("a completed request must not run again");
        assert!(matches!(error, AccountDeletionError::NotConfirmed));
    });
}

#[test]
fn a_confirmed_request_is_not_pending_before_its_grace_period_ends() {
    runtime().block_on(async {
        let service = AccountDeletionService::new(30);
        let created = service
            .create_deletion_request(Uuid::new_v4(), "u@example.com".into(), None, request(false))
            .await
            .expect("request created");
        service
            .confirm_deletion(created.id, &created.confirmation_token)
            .await
            .expect("confirmed");

        assert!(service.get_pending_deletions().await.is_empty());
    });
}

#[test]
fn a_completed_request_cannot_be_cancelled() {
    runtime().block_on(async {
        let service = AccountDeletionService::new(30);
        let user = Uuid::new_v4();
        let created = service
            .create_deletion_request(user, "u@example.com".into(), None, request(true))
            .await
            .expect("request created");
        service
            .confirm_deletion(created.id, &created.confirmation_token)
            .await
            .expect("confirmed");
        service
            .execute_deletion(created.id)
            .await
            .expect("executed");

        let error = service
            .cancel_deletion(created.id, user)
            .await
            .expect_err("completed deletion cannot be cancelled");

        assert!(matches!(error, AccountDeletionError::CannotCancel));
    });
}

#[test]
fn user_requests_are_filtered_by_user() {
    runtime().block_on(async {
        let service = AccountDeletionService::new(30);
        let first = Uuid::new_v4();
        let second = Uuid::new_v4();
        service
            .create_deletion_request(first, "a@example.com".into(), None, request(false))
            .await
            .expect("first request");
        service
            .create_deletion_request(second, "b@example.com".into(), None, request(false))
            .await
            .expect("second request");

        let mine = service.get_user_deletion_requests(first).await;

        assert_eq!(mine.len(), 1);
        assert_eq!(mine[0].user_id, first);
    });
}

#[test]
fn confirmation_tokens_are_hex_and_unique() {
    let first = generate_confirmation_token();
    let second = generate_confirmation_token();

    assert_eq!(first.len(), 16);
    assert!(first.chars().all(|c| c.is_ascii_hexdigit()));
    assert_ne!(first, second);
}
