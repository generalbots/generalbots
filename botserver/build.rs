fn main() {
    if std::path::Path::new("../botui/ui/suite/").exists() {
        println!("cargo:rerun-if-changed=../botui/ui/suite/");
    }
    println!("cargo:rerun-if-changed=3rdparty.toml");
    println!("cargo:rerun-if-changed=.env.embedded");

    // Pass build metadata to the binary via option_env!
    if let Ok(date) = std::env::var("BOTSERVER_BUILD_DATE") {
        println!("cargo:rustc-env=BOTSERVER_BUILD_DATE={}", date);
    }
    if let Ok(commit) = std::env::var("BOTSERVER_COMMIT") {
        println!("cargo:rustc-env=BOTSERVER_COMMIT={}", commit);
    }
}
