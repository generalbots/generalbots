pub(crate) use super::helpers::{json, resource_id, s, s_req};
pub(crate) use super::{ActionSpec, AuthStyle, Origin, ProviderSpec, Risk};

mod slack_spec;
mod spotify_spec;
mod basecamp_spec;

pub use slack_spec::*;
pub use spotify_spec::*;
pub use basecamp_spec::*;
