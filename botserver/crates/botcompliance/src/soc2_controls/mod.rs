//! Default SOC 2 control catalog (Trust Service Criteria).
//! Data-only module split from soc2.rs for file-size limits.

use crate::soc2::Soc2Control;

mod controls;
mod trust_service_controls;

/// The full default control catalog, both catalogue halves in order.
pub(crate) fn get_default_controls() -> Vec<Soc2Control> {
    let mut controls = controls::controls();
    controls.extend(trust_service_controls::controls());
    controls
}
