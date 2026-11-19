use super::{bind, VerbEndpoint};
use std::sync::OnceLock;

mod table_part1;
mod table_part2;

static DATA: OnceLock<Vec<VerbEndpoint>> = OnceLock::new();

pub fn all() -> &'static [VerbEndpoint] {
    DATA.get_or_init(|| {
        let mut v = Vec::with_capacity(table_part1::PART1.len() + table_part2::PART2.len());
        v.extend_from_slice(table_part1::PART1);
        v.extend_from_slice(table_part2::PART2);
        v
    })
    .as_slice()
}
