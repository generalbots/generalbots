use super::AppCommand;
use std::sync::OnceLock;

mod part1;
mod part2;
mod part3;
mod part4;
mod part5;

static ALL: OnceLock<Vec<AppCommand>> = OnceLock::new();

pub fn all() -> &'static [AppCommand] {
    ALL.get_or_init(|| {
        let mut v = Vec::with_capacity(
            part1::PART_1.len()
                + part2::PART_2.len()
                + part3::PART_3.len()
                + part4::PART_4.len()
                + part5::PART_5.len(),
        );
        v.extend_from_slice(part1::PART_1);
        v.extend_from_slice(part2::PART_2);
        v.extend_from_slice(part3::PART_3);
        v.extend_from_slice(part4::PART_4);
        v.extend_from_slice(part5::PART_5);
        v
    })
    .as_slice()
}
