//! Pure branch-scoping rules for Stripe customer/card resolution (issue #2869).
//!
//! An email address is not an identity boundary: the same address can belong
//! to admins in several branches. A Stripe customer discovered by email may
//! only be adopted by a branch when no *other* branch already owns that
//! customer id. These helpers contain no I/O so the rule is unit-testable in
//! isolation.

use uuid::Uuid;

/// Decides whether `caller_branch` may adopt a Stripe customer id that is
/// already mapped (in `billing_customers`) to `existing_owner`.
///
/// - `None` — the id has no local branch mapping: safe to adopt.
/// - `Some(owner)` where `owner == caller_branch` — the caller already owns
///   it, so re-adoption is idempotent and safe.
/// - `Some(owner)` where `owner != caller_branch` — reject. A different branch
///   owns the customer, and adopting it would bind the caller's cards to the
///   victim branch's Stripe customer (and re-attribute them on webhook sync).
pub(crate) fn customer_adoptable_by_branch(
    existing_owner: Option<Uuid>,
    caller_branch: Uuid,
) -> bool {
    match existing_owner {
        None => true,
        Some(owner) => owner == caller_branch,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn branch(n: u128) -> Uuid {
        Uuid::from_u128(n)
    }

    #[test]
    fn unowned_customer_can_be_adopted() {
        assert!(customer_adoptable_by_branch(None, branch(1)));
    }

    #[test]
    fn customer_owned_by_caller_can_be_adopted_again() {
        let caller = branch(1);
        assert!(customer_adoptable_by_branch(Some(caller), caller));
    }

    #[test]
    fn customer_owned_by_another_branch_is_rejected() {
        assert!(!customer_adoptable_by_branch(Some(branch(1)), branch(2)));
    }

    #[test]
    fn same_email_two_branches_never_share_a_customer() {
        // Two callers carrying the same email must not converge on one id.
        let branch_a = branch(1);
        let branch_b = branch(2);
        // Branch A already owns the customer found by the shared email.
        assert!(customer_adoptable_by_branch(Some(branch_a), branch_a));
        // Branch B must not adopt it; it creates a fresh, unowned customer.
        assert!(!customer_adoptable_by_branch(Some(branch_a), branch_b));
        assert!(customer_adoptable_by_branch(None, branch_b));
    }
}
