// Billing, catalog and POS tables (#1497).
//
// Split by responsibility: invoice/quote documents, the product catalog,
// inventory movements, usage alerts/notifications, point-of-sale and the
// join/allow relations. Every `table!` stays inside `botschema`; the root
// re-exports each table module so `botschema::billing::<table>` and (through
// `pub use self::billing::*;` in lib.rs) `botschema::<table>` keep their
// historical paths.

mod alerts;
mod catalog;
mod inventory;
mod invoices;
mod joins;
mod payments;
mod pos;

pub use alerts::{
    billing_alert_history, billing_grace_periods, billing_notification_preferences,
    billing_usage_alerts,
};
pub use catalog::{
    price_list_items, price_lists, product_categories, products, services,
};
pub use inventory::{inventory_movements, product_variants};
pub use invoices::{billing_invoice_items, billing_invoices};
pub use payments::{
    billing_payments, billing_quote_items, billing_quotes, billing_recurring, billing_tax_rates,
};
pub use pos::{
    pos_sales, pos_sessions, product_price_lists, product_prices, product_promotions,
    product_stock, product_variations,
};
