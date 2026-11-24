//! Pure binding checks for `checkout_success` (#2665).
//!
//! Every value below comes either from the Stripe session fetched
//! server-side with our API key, from the verified JWT tenant claim, or from
//! our own billing tables. None of them is a client-declared owner: the
//! `invoice` query parameter is only a hint that must agree with the
//! server-set session metadata before any invoice row may be touched.

use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CheckoutBinding<'a> {
    /// `session_id` as supplied to the lookup (owned by the caller's URL).
    pub(crate) requested_session_id: &'a str,
    /// `id` returned by Stripe for the looked-up session.
    pub(crate) verified_session_id: &'a str,
    /// Checkout mode on the verified session (only `subscription` pays an invoice).
    pub(crate) session_mode: &'a str,
    /// `metadata["invoice_id"]` written by us when the session was created.
    pub(crate) metadata_invoice_id: Option<&'a str>,
    /// UUID parsed from the `invoice` query parameter.
    pub(crate) requested_invoice_id: Uuid,
    /// `customer` on the verified Stripe session.
    pub(crate) session_customer: Option<&'a str>,
    /// Stripe customer id mapped locally to the invoice's branch.
    pub(crate) mapped_customer: Option<&'a str>,
    /// Branch resolved from the authenticated (signature-verified) JWT.
    pub(crate) authenticated_branch_id: Option<Uuid>,
    /// Branch that actually owns the invoice row.
    pub(crate) invoice_branch_id: Uuid,
    /// `org_id` stored on the invoice row.
    pub(crate) invoice_org_id: Uuid,
    /// `org_id` of the branch resolved from the JWT.
    pub(crate) authenticated_org_id: Uuid,
}

/// Returns the first reason the checkout must not be allowed to transition
/// the invoice, or `None` when the verified session, the caller's tenant and
/// the invoice all describe the same purchase. Pure: no I/O, no side effects.
pub(crate) fn binding_failure(binding: &CheckoutBinding<'_>) -> Option<&'static str> {
    if binding.requested_session_id != binding.verified_session_id {
        return Some("session id returned by Stripe differs from the requested id");
    }
    if binding.session_mode != "subscription" {
        return Some("only a subscription checkout can settle an invoice");
    }

    let Some(metadata_invoice) = binding.metadata_invoice_id else {
        return Some("session has no server-set invoice_id metadata");
    };
    let Ok(metadata_uuid) = Uuid::parse_str(metadata_invoice) else {
        return Some("session invoice_id metadata is not a UUID");
    };
    if metadata_uuid != binding.requested_invoice_id {
        return Some("session invoice_id does not match the requested invoice");
    }

    let (session_customer, mapped_customer) = match (
        binding.session_customer,
        binding.mapped_customer,
    ) {
        (Some(session_customer), Some(mapped_customer)) => (session_customer, mapped_customer),
        _ => return Some("session customer is not mapped to the invoice's tenant"),
    };
    if session_customer != mapped_customer {
        return Some("session customer belongs to another tenant");
    }

    let Some(authenticated_branch) = binding.authenticated_branch_id else {
        return Some("request has no authenticated tenant");
    };
    if authenticated_branch != binding.invoice_branch_id {
        return Some("invoice belongs to another tenant");
    }
    if binding.invoice_org_id != binding.authenticated_org_id {
        return Some("invoice org does not match the authenticated tenant's org");
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn branch(n: u8) -> Uuid {
        Uuid::from_bytes([n; 16])
    }

    fn same_tenant_binding<'a>(
        invoice_id: Uuid,
        invoice_metadata: &'a str,
        customer: &'a str,
        session_id: &'a str,
    ) -> CheckoutBinding<'a> {
        CheckoutBinding {
            requested_session_id: session_id,
            verified_session_id: session_id,
            session_mode: "subscription",
            metadata_invoice_id: Some(invoice_metadata),
            requested_invoice_id: invoice_id,
            session_customer: Some(customer),
            mapped_customer: Some(customer),
            authenticated_branch_id: Some(branch(1)),
            invoice_branch_id: branch(1),
            invoice_org_id: branch(10),
            authenticated_org_id: branch(10),
        }
    }

    #[test]
    fn matching_session_tenant_and_invoice_are_allowed() {
        let invoice = Uuid::new_v4();
        let metadata = invoice.to_string();
        let binding = same_tenant_binding(invoice, &metadata, "cus_owner", "cs_ok");
        assert_eq!(binding_failure(&binding), None);
    }

    #[test]
    fn unrelated_invoice_id_is_rejected() {
        let invoice = Uuid::new_v4();
        let metadata = invoice.to_string();
        let mut binding = same_tenant_binding(invoice, &metadata, "cus_owner", "cs_ok");
        binding.requested_invoice_id = Uuid::new_v4();
        assert_eq!(
            binding_failure(&binding),
            Some("session invoice_id does not match the requested invoice")
        );
    }

    #[test]
    fn missing_or_malformed_metadata_is_rejected() {
        let invoice = Uuid::new_v4();
        let metadata = invoice.to_string();
        let mut binding = same_tenant_binding(invoice, &metadata, "cus_owner", "cs_ok");
        binding.metadata_invoice_id = None;
        assert_eq!(
            binding_failure(&binding),
            Some("session has no server-set invoice_id metadata")
        );

        binding.metadata_invoice_id = Some("not-a-uuid");
        assert_eq!(
            binding_failure(&binding),
            Some("session invoice_id metadata is not a UUID")
        );
    }

    #[test]
    fn unrelated_customer_is_rejected() {
        let invoice = Uuid::new_v4();
        let metadata = invoice.to_string();
        let mut binding = same_tenant_binding(invoice, &metadata, "cus_owner", "cs_ok");
        binding.mapped_customer = Some("cus_victim");
        assert_eq!(
            binding_failure(&binding),
            Some("session customer belongs to another tenant")
        );

        binding.session_customer = None;
        assert_eq!(
            binding_failure(&binding),
            Some("session customer is not mapped to the invoice's tenant")
        );
    }

    #[test]
    fn cross_tenant_invoice_is_rejected_even_with_matching_metadata() {
        // Two tenants: the attacker's session is unchanged; only the target
        // invoice's branch/org/customer differ. This is the #2665 attack.
        let attacker_invoice = Uuid::new_v4();
        let metadata = attacker_invoice.to_string();
        let attacker = same_tenant_binding(attacker_invoice, &metadata, "cus_attacker", "cs_attacker");
        assert_eq!(binding_failure(&attacker), None);

        let victim = CheckoutBinding {
            mapped_customer: Some("cus_victim"),
            authenticated_branch_id: Some(branch(2)),
            invoice_branch_id: branch(3),
            invoice_org_id: branch(30),
            authenticated_org_id: branch(20),
            ..attacker.clone()
        };
        assert_eq!(
            binding_failure(&victim),
            Some("session customer belongs to another tenant")
        );

        let victim_same_customer = CheckoutBinding {
            mapped_customer: Some("cus_attacker"),
            authenticated_branch_id: Some(branch(2)),
            invoice_branch_id: branch(3),
            invoice_org_id: branch(30),
            authenticated_org_id: branch(20),
            ..attacker.clone()
        };
        assert_eq!(
            binding_failure(&victim_same_customer),
            Some("invoice belongs to another tenant")
        );

        let victim_org_mismatch = CheckoutBinding {
            authenticated_branch_id: Some(branch(3)),
            invoice_branch_id: branch(3),
            authenticated_org_id: branch(99),
            ..victim_same_customer
        };
        assert_eq!(
            binding_failure(&victim_org_mismatch),
            Some("invoice org does not match the authenticated tenant's org")
        );
    }

    #[test]
    fn missing_tenant_wrong_mode_and_wrong_session_are_rejected() {
        let invoice = Uuid::new_v4();
        let metadata = invoice.to_string();
        let mut binding = same_tenant_binding(invoice, &metadata, "cus_owner", "cs_ok");

        binding.authenticated_branch_id = None;
        assert_eq!(
            binding_failure(&binding),
            Some("request has no authenticated tenant")
        );

        binding = same_tenant_binding(invoice, &metadata, "cus_owner", "cs_ok");
        binding.session_mode = "setup";
        assert_eq!(
            binding_failure(&binding),
            Some("only a subscription checkout can settle an invoice")
        );

        binding = same_tenant_binding(invoice, &metadata, "cus_owner", "cs_ok");
        binding.verified_session_id = "cs_other";
        assert_eq!(
            binding_failure(&binding),
            Some("session id returned by Stripe differs from the requested id")
        );
    }
}
