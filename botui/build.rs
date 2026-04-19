fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let ui_path = std::path::Path::new(&manifest_dir).join("ui");
    println!("cargo:rustc-env=BOTUI_UI_PATH={}", ui_path.display());
}