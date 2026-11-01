fn main() {
    // The suite and the first-run wizard are embedded from botui/ui at build
    // time, so a change there must relink the application.
    println!("cargo:rerun-if-changed=../botui/ui");
    tauri_build::build();
}
