#[path = "build_stamp.rs"]
mod build_stamp;

fn main() {
    build_stamp::emit();
}
