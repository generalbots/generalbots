use botsecurity::command_guard::SafeCommand;
use std::path::PathBuf;
use uuid::Uuid;

pub const MARKETPLACE_BUCKET: &str = "marketplace.gbai";

pub fn valid_slug(slug: &str) -> bool {
    !slug.is_empty()
        && slug.len() <= 80
        && slug
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

pub fn valid_version(version: &str) -> bool {
    !version.is_empty()
        && version.len() <= 32
        && version
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_')
}

pub fn package_object_key(slug: &str, version: &str) -> String {
    format!("{MARKETPLACE_BUCKET}/packages/{slug}/{version}.gbskill")
}

pub fn bot_object_key(bot_id: &Uuid, relative_path: &str) -> String {
    format!("{bot_id}.gbai/{bot_id}.gbdialog/{relative_path}")
}

fn temp_file(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("gbmkt_{label}_{}", Uuid::new_v4()))
}

fn run_mc(mc_bin: &str, args: &[String]) -> Result<std::process::Output, String> {
    let mut cmd = SafeCommand::new(mc_bin).map_err(|e| format!("mc guard: {e}"))?;
    for a in args {
        cmd = cmd.arg(a.as_str()).map_err(|e| format!("mc arg guard: {e}"))?;
    }
    cmd.execute().map_err(|e| format!("mc exec: {e}"))
}

fn mc_stderr(output: &std::process::Output) -> String {
    let text = String::from_utf8_lossy(&output.stderr);
    text.chars().take(200).collect()
}

fn upload_bytes(mc_bin: &str, mc_alias: &str, object_key: &str, content: &[u8]) -> Result<(), String> {
    let tmp = temp_file("put");
    std::fs::write(&tmp, content).map_err(|e| format!("temp write: {e}"))?;
    let tmp_str = tmp.to_string_lossy().to_string();
    let remote = format!("{mc_alias}/{object_key}");
    let args = vec![
        "cp".to_string(),
        tmp_str.clone(),
        remote,
    ];
    let result = run_mc(mc_bin, &args);
    let _ = std::fs::remove_file(&tmp);
    let output = result?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!("mc cp upload failed: {}", mc_stderr(&output)))
    }
}

fn download_bytes(mc_bin: &str, mc_alias: &str, object_key: &str) -> Result<Vec<u8>, String> {
    let tmp = temp_file("get");
    let tmp_str = tmp.to_string_lossy().to_string();
    let remote = format!("{mc_alias}/{object_key}");
    let args = vec![
        "cp".to_string(),
        remote,
        tmp_str.clone(),
    ];
    let result = run_mc(mc_bin, &args);
    if let Err(e) = result {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }
    match std::fs::read(&tmp) {
        Ok(bytes) => {
            let _ = std::fs::remove_file(&tmp);
            Ok(bytes)
        }
        Err(e) => {
            let _ = std::fs::remove_file(&tmp);
            Err(format!("temp read: {e}"))
        }
    }
}

pub fn put_package(
    mc_bin: &str,
    mc_alias: &str,
    slug: &str,
    version: &str,
    content_bytes: &[u8],
) -> Result<String, String> {
    if !valid_slug(slug) {
        return Err(format!("Invalid package slug '{slug}'"));
    }
    if !valid_version(version) {
        return Err(format!("Invalid package version '{version}'"));
    }
    if content_bytes.is_empty() {
        return Err("Package content is empty".to_string());
    }
    let object_key = package_object_key(slug, version);
    upload_bytes(mc_bin, mc_alias, &object_key, content_bytes)?;
    Ok(object_key)
}

pub fn get_package(mc_bin: &str, mc_alias: &str, object_key: &str) -> Result<Vec<u8>, String> {
    download_bytes(mc_bin, mc_alias, object_key)
}

pub fn upload_to_bot_bucket(
    mc_bin: &str,
    mc_alias: &str,
    bot_id: &Uuid,
    relative_path: &str,
    content: &[u8],
) -> Result<(), String> {
    upload_bytes(mc_bin, mc_alias, &bot_object_key(bot_id, relative_path), content)
}

pub fn remove_bot_bucket_prefix(mc_bin: &str, mc_alias: &str, bot_id: &Uuid, relative_prefix: &str) -> Result<(), String> {
    let remote = format!("{mc_alias}/{}", bot_object_key(bot_id, relative_prefix));
    let args = vec![
        "rm".to_string(),
        "--recursive".to_string(),
        "--force".to_string(),
        remote,
    ];
    let output = run_mc(mc_bin, &args)?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!("mc rm failed: {}", mc_stderr(&output)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_well_formed_slugs() {
        for s in ["expense-parser", "kb_quizmaster", "Webhook2", "a"] {
            assert!(valid_slug(s), "{s} should be valid");
        }
    }

    #[test]
    fn rejects_bad_slugs_and_versions() {
        for s in ["", "../etc", "has space", "sl/ash", "dot.dot"] {
            assert!(!valid_slug(s), "{s} should be invalid");
        }
        let too_long = "x".repeat(81);
        assert!(!valid_slug(&too_long));
        assert!(valid_version("1.2.0"));
        assert!(valid_version("0.1.0-beta_1"));
        assert!(!valid_version(""));
        assert!(!valid_version("1 2"));
        let long_version = "9".repeat(33);
        assert!(!valid_version(&long_version));
    }

    #[test]
    fn object_keys_follow_layout() {
        let bot = Uuid::nil();
        assert_eq!(
            package_object_key("expense-parser", "1.0.0"),
            "marketplace.gbai/packages/expense-parser/1.0.0.gbskill"
        );
        assert_eq!(
            bot_object_key(&bot, "skills/csv-cleaner/manifest.json"),
            format!("{bot}.gbai/{bot}.gbdialog/skills/csv-cleaner/manifest.json")
        );
    }
}
