//! Task ownership model for the browser-task API (#2877).
//!
//! The API used to authenticate the caller and then act on any task id from
//! the URL. Ownership is decided here by comparing a verified caller
//! principal with the scope recorded on the task at creation time.

use crate::models::BrowserTask;
use uuid::Uuid;

/// Verified identity used for task ownership decisions (#2877). A caller's
/// principal is built only from a signature-verified token; a task's principal
/// is the scope recorded when it was created. Both share this shape so the
/// pure [`ownership_matches`] predicate can compare them directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaskPrincipal {
    pub user_id: Uuid,
    pub org_id: Option<Uuid>,
    pub bot_id: Option<Uuid>,
}

impl TaskPrincipal {
    /// Ownership recorded on a persisted task.
    pub fn of_task(task: &BrowserTask) -> Self {
        Self {
            user_id: task.user_id,
            org_id: task.org_id,
            bot_id: task.bot_id,
        }
    }
}

/// Pure ownership predicate (#2877).
///
/// A caller may act on a task only when the verified `user_id` matches and
/// every scope component present on *either* side agrees. A component present
/// on only one side cannot be confirmed, so it fails closed; when both sides
/// omit it, the user id is the sole ownership boundary (legacy, unattributed
/// tokens).
pub fn ownership_matches(task: &TaskPrincipal, caller: &TaskPrincipal) -> bool {
    if task.user_id != caller.user_id {
        return false;
    }
    scope_component_matches(task.org_id, caller.org_id)
        && scope_component_matches(task.bot_id, caller.bot_id)
}

fn scope_component_matches(task: Option<Uuid>, caller: Option<Uuid>) -> bool {
    match (task, caller) {
        (Some(task), Some(caller)) => task == caller,
        (None, None) => true,
        // Exactly one side carries a scope the other cannot confirm -> deny.
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn principal(user: u128, org: Option<u128>, bot: Option<u128>) -> TaskPrincipal {
        TaskPrincipal {
            user_id: Uuid::from_u128(user),
            org_id: org.map(Uuid::from_u128),
            bot_id: bot.map(Uuid::from_u128),
        }
    }

    #[test]
    fn same_user_without_any_scope_is_owned() {
        assert!(ownership_matches(&principal(1, None, None), &principal(1, None, None)));
    }

    #[test]
    fn a_different_user_never_owns_the_task_even_with_matching_scope() {
        assert!(!ownership_matches(
            &principal(1, Some(10), Some(20)),
            &principal(2, Some(10), Some(20)),
        ));
    }

    #[test]
    fn matching_user_and_scope_is_owned() {
        assert!(ownership_matches(
            &principal(1, Some(10), Some(20)),
            &principal(1, Some(10), Some(20)),
        ));
    }

    #[test]
    fn a_mismatched_org_or_bot_denies_ownership() {
        assert!(!ownership_matches(
            &principal(1, Some(10), Some(20)),
            &principal(1, Some(11), Some(20)),
        ));
        assert!(!ownership_matches(
            &principal(1, Some(10), Some(20)),
            &principal(1, Some(10), Some(21)),
        ));
    }

    #[test]
    fn a_scope_present_on_only_one_side_denies_ownership() {
        let task = principal(1, Some(10), Some(20));
        // Caller has no scope claims -> cannot confirm the task's workspace.
        assert!(!ownership_matches(&task, &principal(1, None, None)));
        // Caller has extra scope claims -> cannot confirm they are the task's.
        assert!(!ownership_matches(&principal(1, None, None), &task));
        // Mixed: only org present on one side.
        assert!(!ownership_matches(
            &principal(1, Some(10), None),
            &principal(1, None, None),
        ));
    }
}
