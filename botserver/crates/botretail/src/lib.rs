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

pub use product::{Product, Variation, VariationKind, ProductStatus};
pub use stock::{StockLevel, StockMovement, MovementKind, Branch};
pub use pricing::{PriceList, PriceListEntry, PriceTier};
pub use promotion::{Promotion, DiscountKind, PromotionWindow};
pub use pos::{PosSession, PosLineItem, PosSale, PosPayment, PosError, PaymentMethod};
