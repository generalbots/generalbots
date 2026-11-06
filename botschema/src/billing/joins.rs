use super::*;
use crate::core::branches;

diesel::joinable!(products -> branches (branch_id));
diesel::joinable!(services -> branches (branch_id));
diesel::joinable!(product_categories -> branches (branch_id));
diesel::joinable!(price_lists -> branches (branch_id));
diesel::joinable!(price_list_items -> price_lists (price_list_id));
diesel::joinable!(price_list_items -> products (product_id));
diesel::joinable!(price_list_items -> services (service_id));
diesel::joinable!(inventory_movements -> branches (branch_id));
diesel::joinable!(inventory_movements -> products (product_id));
diesel::joinable!(product_variants -> products (product_id));

diesel::joinable!(billing_invoices -> branches (branch_id));
diesel::joinable!(billing_invoice_items -> billing_invoices (invoice_id));
diesel::joinable!(billing_payments -> branches (branch_id));
diesel::joinable!(billing_payments -> billing_invoices (invoice_id));
diesel::joinable!(billing_quotes -> branches (branch_id));
diesel::joinable!(billing_quote_items -> billing_quotes (quote_id));
diesel::joinable!(billing_recurring -> branches (branch_id));
diesel::joinable!(billing_tax_rates -> branches (branch_id));

diesel::allow_tables_to_appear_in_same_query!(
    billing_invoices,
    billing_payments,
    billing_quotes,
    billing_recurring,
    billing_tax_rates,
    products,
    price_lists,
    billing_alert_history,
    billing_notification_preferences,
);

diesel::joinable!(billing_usage_alerts -> branches (branch_id));
diesel::joinable!(billing_alert_history -> branches (branch_id));
diesel::joinable!(billing_notification_preferences -> branches (branch_id));
diesel::joinable!(billing_grace_periods -> branches (branch_id));
