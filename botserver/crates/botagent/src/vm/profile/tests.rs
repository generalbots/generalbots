//! `profile::tests` — split from `profile.rs` per the AGENTS.md file-size rule.

use super::*;

fn executable_lines(block: &str) -> String {
    block
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n")
        .to_ascii_lowercase()
}

#[test]
fn office_is_the_default_profile_for_bots() {
    assert_eq!(profile_for_bot(KIND_BOT, None), VmProfile::Office);
    assert_eq!(profile_for_bot("BOT", Some("")), VmProfile::Office);
    // An unknown request must not leave the workspace without documents.
    assert_eq!(profile_for_bot(KIND_BOT, Some("gpu")), VmProfile::Office);
    // app/website VMs serve HTTP and keep today's base image.
    assert_eq!(profile_for_bot("apps", None), VmProfile::Base);
    assert_eq!(profile_for_bot("website", None), VmProfile::Base);
}

#[test]
fn media_and_data_require_an_explicit_request() {
    for kind in [KIND_BOT, "apps", "website", "unknown"] {
        assert_ne!(profile_for_bot(kind, None), VmProfile::Media, "{kind}");
        assert_ne!(profile_for_bot(kind, None), VmProfile::Data, "{kind}");
    }
    assert_eq!(profile_for_bot(KIND_BOT, Some("media")), VmProfile::Media);
    assert_eq!(profile_for_bot(KIND_BOT, Some("data")), VmProfile::Data);
    assert_eq!(profile_for_bot(KIND_BOT, Some("base")), VmProfile::Base);
}

#[test]
fn the_budget_constants_match_the_issue() {
    assert_eq!(OFFICE_MAX_DELTA_BYTES, 200_000_000);
    assert_eq!(MEDIA_EXTRA_BYTES, 250_000_000);
    assert_eq!(DATA_EXTRA_BYTES, 180_000_000);

    let office = VmProfile::Office.extra_bytes();
    assert!(
        office <= OFFICE_MAX_DELTA_BYTES,
        "office profile adds {office} bytes, budget {OFFICE_MAX_DELTA_BYTES}"
    );
    // The budget must still buy a browser: a profile that dropped Chromium
    // would pass the ceiling while failing the acceptance criterion.
    assert!(office >= 150_000_000, "office profile is suspiciously small");
    assert_eq!(
        VmProfile::Media.extra_bytes() - office,
        MEDIA_EXTRA_BYTES
    );
    assert_eq!(VmProfile::Data.extra_bytes() - office, DATA_EXTRA_BYTES);
    assert_eq!(VmProfile::Base.extra_bytes(), 0);
}

#[test]
fn the_default_profile_has_no_large_stack() {
    let commands = executable_lines(&install_block(VmProfile::Office));
    for forbidden in ["libreoffice", "torch", "tensorflow", "transformers", "cuda"] {
        assert!(!commands.contains(forbidden), "office installs {forbidden}");
    }
    // ffmpeg is opt-in (`media`) only: media generation runs in BotModels.
    assert!(!commands.contains("ffmpeg"));
    // The Debian chromium package is only the online fallback; the static
    // build from the cache is what the budget counts.
    assert!(commands.contains("chromium-headless.tar.zst"));
    assert!(commands.contains("fonts-noto-core"));
    assert!(commands.contains("python-docx"));
}

#[test]
fn office_prints_html_to_pdf_with_chromium_instead_of_weasyprint() {
    let block = install_block(VmProfile::Office);
    assert!(block.contains("--headless=new"));
    assert!(block.contains("--print-to-pdf="));
    assert!(block.contains("/usr/local/bin/html2pdf"));
    assert!(!block.to_ascii_lowercase().contains("weasyprint"));
}

#[test]
fn opt_in_profiles_add_only_their_own_stack() {
    let media = executable_lines(&install_block(VmProfile::Media));
    assert!(media.contains("ffmpeg"));
    assert!(media.contains("chromium-headless.tar.zst"));
    let data = executable_lines(&install_block(VmProfile::Data));
    assert!(data.contains("scipy"));
    assert!(data.contains("python-docx"));
    assert!(!data.contains("ffmpeg"));
    assert!(install_block(VmProfile::Base).is_empty());
}

#[test]
fn a_failed_artifact_is_fatal_and_recorded() {
    let block = install_block(VmProfile::Office);
    assert!(block.contains("gb_profile_fail"));
    assert!(block.contains(PROFILE_ERROR_FILE));
    assert!(block.contains("exit 78"));
    // No `|| true` on the profile steps: a silent partial install is what
    // the issue forbids.
    assert!(!executable_lines(&block).contains("|| true"));
}

#[test]
fn an_offline_host_without_the_image_fails_loudly() {
    let availability = Availability {
        prefab_image: false,
        offline: true,
    };
    let error = ensure_available(VmProfile::Office, &availability)
        .expect_err("offline office must not degrade silently");
    assert_eq!(error.profile, VmProfile::Office);
    assert!(error.artifacts.contains(&"chromium-headless"));
    let message = error.to_string();
    assert!(message.contains("scripts/build_vm_image.sh"));
    assert!(message.contains("BotModels"));
    assert!(message.contains("Refusing to fall back"));
}

#[test]
fn a_prefabricated_image_or_an_online_host_is_enough() {
    let prefab = Availability {
        prefab_image: true,
        offline: true,
    };
    assert!(ensure_available(VmProfile::Office, &prefab).is_ok());
    let online = Availability {
        prefab_image: false,
        offline: false,
    };
    assert!(ensure_available(VmProfile::Media, &online).is_ok());
    let offline = Availability {
        prefab_image: false,
        offline: true,
    };
    // The base profile needs nothing beyond today's image.
    assert!(ensure_available(VmProfile::Base, &offline).is_ok());
    assert!(ensure_available(VmProfile::Data, &offline).is_err());
}

#[test]
fn the_image_resolver_prefers_the_prefabricated_image() {
    let available = vec!["images:debian/13".to_string(), DEFAULT_PREFAB_IMAGE.to_string()];
    let (image, prefab) = resolve_image(None, DEFAULT_PREFAB_IMAGE, &available);
    assert_eq!(image, DEFAULT_PREFAB_IMAGE);
    assert!(prefab);
    let (image, prefab) = resolve_image(None, DEFAULT_PREFAB_IMAGE, &["images:debian/13".to_string()]);
    assert_eq!(image, BASE_IMAGE);
    assert!(!prefab);
    // An explicit image always wins, even a pinned one.
    let (image, prefab) = resolve_image(Some("my-image"), DEFAULT_PREFAB_IMAGE, &available);
    assert_eq!(image, "my-image");
    assert!(!prefab);
}

#[test]
fn a_profile_change_reprovisions_once() {
    for profile in [VmProfile::Base, VmProfile::Office, VmProfile::Media, VmProfile::Data] {
        let marker = profile.prepare_version();
        assert!(marker.contains(profile.as_str()));
    }
    assert_ne!(
        VmProfile::Office.prepare_version(),
        VmProfile::Base.prepare_version()
    );
}
