use super::ApiEndpoint;
use std::sync::OnceLock;

mod endpoints_a;
mod endpoints_b;
mod endpoints_c;

static ALL: OnceLock<Vec<ApiEndpoint>> = OnceLock::new();

pub fn all() -> &'static [ApiEndpoint] {
    ALL.get_or_init(|| {
        let mut v = Vec::with_capacity(
            endpoints_a::PART_A.len() + endpoints_b::PART_B.len() + endpoints_c::PART_C.len(),
        );
        v.extend_from_slice(endpoints_a::PART_A);
        v.extend_from_slice(endpoints_b::PART_B);
        v.extend_from_slice(endpoints_c::PART_C);
        v
    })
    .as_slice()
}
