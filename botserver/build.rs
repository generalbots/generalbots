#[path = "crates/botcore/build_stamp.rs"]
mod build_stamp;

fn main() {
if std::path::Path::new("../botui/ui/suite/").exists() {
println!("cargo:rerun-if-changed=../botui/ui/suite/");
}
println!("cargo:rerun-if-changed=3rdparty.toml");
println!("cargo:rerun-if-changed=.env.embedded");

build_stamp::emit();
}
