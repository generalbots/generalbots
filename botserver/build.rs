fn main() {
if std::path::Path::new("../botui/ui/suite/").exists() {
println!("cargo:rerun-if-changed=../botui/ui/suite/");
}
println!("cargo:rerun-if-changed=3rdparty.toml");
println!("cargo:rerun-if-changed=.env.embedded");

if let Ok(date) = std::env::var("BOTSERVER_BUILD_DATE") {
println!("cargo:rustc-env=BOTSERVER_BUILD_DATE={}", date);
} else {
println!("cargo:rustc-env=BOTSERVER_BUILD_DATE={}", chrono_now());
}

let commit = std::env::var("BOTSERVER_COMMIT")
.ok()
.or_else(|| git_commit_hash());
if let Some(hash) = commit {
println!("cargo:rustc-env=BOTSERVER_COMMIT={}", hash);
}
}

fn git_commit_hash() -> Option<String> {
let output = std::process::Command::new("git")
.args(["rev-parse", "--short", "HEAD"])
.output()
.ok()?;
if !output.status.success() {
return None;
}
String::from_utf8(output.stdout).ok().map(|s| s.trim().to_string())
}

fn chrono_now() -> String {
    let output = match std::process::Command::new("date")
        .args(["+%Y-%m-%dT%H:%M:%S"])
        .output()
    {
        Ok(o) => o,
        Err(_) => return "unknown".to_string(),
    };
    if !output.status.success() {
        return "unknown".to_string();
    }
    String::from_utf8(output.stdout)
        .ok()
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}
