pub(crate) use super::helpers::{json, resource_id, s, s_req};
pub(crate) use super::{ActionSpec, AuthStyle, Origin, ProviderSpec, Risk};

mod cal_com_spec;
mod todoist_spec;
mod outlook_calendar_spec;

pub use cal_com_spec::*;
pub use todoist_spec::*;
pub use outlook_calendar_spec::*;
