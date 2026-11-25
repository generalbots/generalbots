// Only the mail helpers resolve `super::notification_email` through the parent
// glob; gate it so no-default-features builds stay warning-free.
#[cfg(feature = "mail")]
use super::*;

// Split by responsibility to stay under the 450-line file limit: request
// handlers live in `handlers`, @mention email helpers in `mentions`.
// All items stay reachable through `collab_routes::comments::*`.
mod handlers;
#[cfg(feature = "mail")]
mod mentions;

pub use handlers::*;
#[cfg(feature = "mail")]
pub(crate) use mentions::*;
