//! Retail management (Bling-style): product variations, stock by branch,
//! price lists, promotions, NFCe integration (via `botbrazil`), simplified
//! POS interface.

#![deny(missing_docs)]
#![deny(unsafe_code)]

pub mod product;
pub mod stock;
pub mod pricing;
pub mod promotion;
pub mod pos;
pub mod db;
pub mod mutations;

pub use product::{Product, Variation, VariationKind, ProductStatus};
pub use stock::{StockLevel, StockMovement, MovementKind, Branch};
pub use pricing::{PriceList, PriceListEntry, PriceTier};
pub use promotion::{Promotion, DiscountKind, PromotionWindow};
pub use pos::{PosSession, PosLineItem, PosSale, PosPayment, PosError, PaymentMethod};

/// Builds the Axum router exposing the retail management HTTP handlers.
///
/// Handlers resolve the caller's branch/tenant from request headers and operate
/// against the shared database pool, so no application state is threaded through
/// the router.
pub fn configure() -> Router {
    use axum::routing::{get, post, put};

    Router::new()
        .route(
            "/api/retail/branches",
            get(mutations::list_branches).post(mutations::create_branch),
        )
        .route("/api/retail/branches/{id}", put(mutations::update_branch))
        .route(
            "/api/retail/promotions",
            get(mutations::list_promotions).post(mutations::create_promotion),
        )
        .route(
            "/api/retail/suppliers",
            get(mutations::list_suppliers).post(mutations::create_supplier),
        )
}
