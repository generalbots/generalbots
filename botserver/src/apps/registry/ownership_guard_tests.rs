//! #2120 — the ownership half of the supported-matrix guard.
//!
//! `botserver/.product.matrix` is the product decision; `OWNERSHIP` is its
//! Rust projection. These tests keep the two honest without needing a compile
//! of the whole server: an app without an owner, an owner without an app, a
//! deprecated app in `apps=`, or a "supported" app that the launcher hides all
//! fail here.

use super::{all_apps, AppStatus, OWNERSHIP, UNOWNED};
use std::collections::HashSet;

/// Apps the suite shell always provides without listing them in `.product`.
const CORE_APPS: &[&str] = &["about", "settings"];

/// `apps=` line of a product profile, as a set.
fn advertised(raw: &str) -> HashSet<&str> {
    raw.lines()
        .find(|line| line.starts_with("apps="))
        .map(|line| {
            line.trim_start_matches("apps=")
                .split(',')
                .map(str::trim)
                .filter(|id| !id.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

/// Both shipped profiles: the cloud edition and the desktop-local edition.
fn product_apps() -> Vec<(&'static str, HashSet<&'static str>)> {
    vec![
        (".product", advertised(include_str!("../../../.product"))),
        (
            ".product.desktop-local",
            advertised(include_str!("../../../.product.desktop-local")),
        ),
    ]
}

#[test]
fn every_app_declares_an_owner_and_a_status() {
    let apps = all_apps();
    assert!(!apps.is_empty(), "the app registry is empty");
    for app in &apps {
        assert!(
            !app.owner.trim().is_empty(),
            "`{}` has an empty owner (#2120)",
            app.id
        );
        assert_ne!(
            app.owner, UNOWNED,
            "`{}` has no row in OWNERSHIP (#2120)",
            app.id
        );
    }
}

#[test]
fn ownership_table_covers_the_registry_exactly_once() {
    let registry: HashSet<String> = all_apps().into_iter().map(|app| app.id).collect();
    let mut seen: HashSet<&str> = HashSet::new();
    for (id, owner, _) in OWNERSHIP {
        assert!(seen.insert(id), "OWNERSHIP lists `{id}` twice");
        assert!(!owner.trim().is_empty(), "OWNERSHIP leaves `{id}` unowned");
        assert!(
            registry.contains(*id),
            "OWNERSHIP names `{id}`, which is not a registry app"
        );
    }
    let missing: Vec<&str> = registry
        .iter()
        .map(String::as_str)
        .filter(|id| !seen.contains(id))
        .collect();
    assert!(
        missing.is_empty(),
        "registry apps missing from OWNERSHIP: {}",
        missing.join(", ")
    );
}

#[test]
fn deprecated_and_hidden_apps_are_not_advertised() {
    for (label, declared) in product_apps() {
        for app in all_apps() {
            if app.status != AppStatus::Deprecated && app.status != AppStatus::Hidden {
                continue;
            }
            assert!(
                !declared.contains(app.id.as_str()),
                "`{}` is {:?} but {label} lists it in apps= (#2120)",
                app.id,
                app.status
            );
        }
    }
}

#[test]
fn supported_apps_are_the_advertised_surface() {
    let profiles = product_apps();
    for (label, declared) in &profiles {
        for id in declared {
            let status = all_apps()
                .into_iter()
                .find(|app| app.id == *id)
                .map(|app| app.status);
            assert_eq!(
                status,
                Some(AppStatus::Supported),
                "`{id}` is in {label} apps= but its status is not supported (#2120)"
            );
        }
    }
    let first = &profiles[0].1;
    for app in all_apps() {
        if app.status != AppStatus::Supported || CORE_APPS.contains(&app.id.as_str()) {
            continue;
        }
        assert!(
            first.contains(app.id.as_str()),
            "`{}` is supported but the profiles do not advertise it (#2120)",
            app.id
        );
    }
}
