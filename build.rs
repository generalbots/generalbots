fn main() {
    println!("cargo:rerun-if-changed=../botui/ui/suite/");
    println!("cargo:rerun-if-changed=3rdparty.toml");
    println!("cargo:rerun-if-changed=.env.embedded");
}
