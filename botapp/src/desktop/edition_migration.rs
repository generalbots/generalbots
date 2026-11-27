//! Carries an install of the Lite edition over to its own identity.
//!
//! The edition used to share the platform desktop's bundle identifier and data
//! folder. It now has its own identifier (Tauri derives the WebView profile —
//! localStorage, sessions, wizard flags — from it) and its own stack folder
//! (`data_root`). On the first start of the renamed build both are moved,
//! only when the old stack's product files show it is this edition's:
//!
//! - a plain rename, never a copy, for the stack (it holds the database, the
//!   vault and the drive), with the files that hold its absolute path
//!   repointed (`stack_relocate`): a rename on one volume is atomic, and when
//!   it cannot happen (files held open, another volume) the stack is used
//!   where it is and the move is tried again on the next start;
//! - a rename for the WebView profile, with the non-destructive copy of
//!   `legacy_data` as the fallback, and a copy when both editions used it;
//! - nothing at all when the target exists, or when an override
//!   (`BOTSERVER_STACK_PATH`, `WEBVIEW2_USER_DATA_FOLDER`, the test rig) shows
//!   this run is not the person's default install.

use super::data_root::{edition_stack, legacy_stack, owner_of, StackOwner};
use std::path::Path;

/// The identifier the edition shared with the platform desktop.
const OLD_IDENTIFIER: &str = "org.generalbots.botapp";

/// The edition's identifier, as in `tauri.gblite.conf.json`.
pub const IDENTIFIER: &str = "org.generalbots.lite";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StackMove {
    TargetExists,
    NothingToMove,
    /// The old stack is another build's, or unmarked: never taken.
    NotThisEdition,
    /// Both editions used it: it stays where it is, in use by both.
    KeepShared,
    Move,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProfileMove {
    Skip,
    Move,
    Copy,
}

/// Whether this run may touch the person's default install at all.
fn migration_allowed(stack_override: bool, webview_override: bool, test_rig: bool) -> bool {
    !(stack_override || webview_override || test_rig)
}

/// Only a binary that carries the edition's identifier may move the profile:
/// one that still runs as the old identifier would recreate an empty profile
/// under it right after the move.
fn identity_matches(compiled_identifier: &str) -> bool {
    compiled_identifier == IDENTIFIER
}

fn plan_stack(edition_exists: bool, legacy_owner: StackOwner) -> StackMove {
    if edition_exists {
        return StackMove::TargetExists;
    }
    match legacy_owner {
        StackOwner::Missing => StackMove::NothingToMove,
        StackOwner::Lite => StackMove::Move,
        StackOwner::Shared => StackMove::KeepShared,
        StackOwner::Other | StackOwner::Unmarked => StackMove::NotThisEdition,
    }
}

/// Whose the old WebView profile is, from the stack it was used with: the old
/// stack while it is there, else the moved one.
fn profile_owner(legacy_owner: StackOwner, edition_owner: StackOwner) -> StackOwner {
    match legacy_owner {
        StackOwner::Missing if edition_owner == StackOwner::Lite => StackOwner::Lite,
        owner => owner,
    }
}

fn plan_profile(new_exists: bool, old_is_dir: bool, owner: StackOwner) -> ProfileMove {
    if new_exists || !old_is_dir {
        return ProfileMove::Skip;
    }
    match owner {
        StackOwner::Lite => ProfileMove::Move,
        StackOwner::Shared => ProfileMove::Copy,
        StackOwner::Other | StackOwner::Unmarked | StackOwner::Missing => ProfileMove::Skip,
    }
}

fn env_set(name: &str) -> bool {
    std::env::var(name).is_ok_and(|value| !value.trim().is_empty())
}

/// Runs before any webview exists and before the stack is resolved, with the
/// identifier compiled into this binary.
pub fn migrate(compiled_identifier: &str) {
    if !identity_matches(compiled_identifier) {
        log::warn!(
            "this build runs as {compiled_identifier}, not {IDENTIFIER}; the edition migration is skipped"
        );
        return;
    }
    let allowed = migration_allowed(
        env_set("BOTSERVER_STACK_PATH"),
        env_set("WEBVIEW2_USER_DATA_FOLDER"),
        super::splash::is_test_rig(),
    );
    if !allowed {
        return;
    }
    let Some(data_local) = dirs::data_local_dir() else {
        log::error!("could not resolve the local data dir for the edition migration");
        return;
    };
    let legacy = legacy_stack(&data_local);
    let legacy_owner = owner_of(&legacy);
    let edition = edition_stack(&data_local);
    let owner = profile_owner(legacy_owner, owner_of(&edition));

    let mut roots = Vec::new();
    roots.extend(dirs::data_dir());
    if !roots.contains(&data_local) {
        roots.push(data_local.clone());
    }
    for root in roots {
        migrate_profile(&root, owner);
    }
    migrate_stack(&legacy, &edition, legacy_owner);
}

fn migrate_profile(root: &Path, owner: StackOwner) {
    let old = root.join(OLD_IDENTIFIER);
    let new = root.join(IDENTIFIER);
    match plan_profile(new.exists(), old.is_dir(), owner) {
        ProfileMove::Skip => {}
        ProfileMove::Move => match std::fs::rename(&old, &new) {
            Ok(()) => log::info!("app data moved from {} to {}", old.display(), new.display()),
            Err(e) => {
                log::warn!("could not move {} ({e}); copying it instead", old.display());
                copy_profile(&old, &new);
            }
        },
        ProfileMove::Copy => copy_profile(&old, &new),
    }
}

fn copy_profile(old: &Path, new: &Path) {
    match super::legacy_data::copy_dir(old, new) {
        Ok(()) => log::info!("app data copied from {} to {} (kept)", old.display(), new.display()),
        Err(e) => log::error!("could not copy app data from {}: {e}", old.display()),
    }
}

fn migrate_stack(legacy: &Path, edition: &Path, legacy_owner: StackOwner) {
    match plan_stack(edition.is_dir(), legacy_owner) {
        StackMove::TargetExists | StackMove::NothingToMove => {}
        StackMove::NotThisEdition => log::info!(
            "{} belongs to another build ({legacy_owner:?}); the edition starts in its own folder",
            legacy.display()
        ),
        StackMove::KeepShared => log::warn!(
            "{} was used by this edition and another build; it stays in place and in use",
            legacy.display()
        ),
        StackMove::Move => {
            if let Some(parent) = edition.parent() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    log::error!("could not create {}: {e}", parent.display());
                    return;
                }
            }
            match super::stack_relocate::relocate(legacy, edition) {
                Ok(repointed) => log::info!(
                    "stack moved from {} to {} ({repointed} path files repointed)",
                    legacy.display(),
                    edition.display()
                ),
                Err(e) => log::warn!(
                    "could not move the stack from {} to {} ({e}); using it in place, the move is retried on the next start",
                    legacy.display(),
                    edition.display()
                ),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::desktop::data_root::{
        stack_under, DATA_FOLDER, LEGACY_FOLDER, LITE_MARKER, OTHER_MARKER,
    };
    use std::path::PathBuf;

    #[test]
    fn the_identifier_matches_the_bundle_config() {
        let conf = include_str!("../../tauri.gblite.conf.json");
        assert!(conf.contains(&format!("\"identifier\": \"{IDENTIFIER}\"")));
        assert_ne!(DATA_FOLDER, LEGACY_FOLDER);
    }

    #[test]
    fn this_build_carries_the_edition_identity() {
        let context: tauri::Context<tauri::Wry> = tauri::generate_context!();
        let identifier = &context.config().identifier;
        let name = &context.package_info().name;
        println!("compiled identity: {identifier} / {name}");
        assert_eq!(identifier, IDENTIFIER);
        assert_eq!(name, "General Bots Lite");
        assert!(identity_matches(identifier));
    }

    #[test]
    fn a_build_with_another_identifier_never_migrates() {
        assert!(!identity_matches(OLD_IDENTIFIER));
        assert!(!identity_matches(""));
        assert!(!identity_matches("org.generalbots.lite.dev"));
        assert!(identity_matches(IDENTIFIER));
    }

    #[test]
    fn overrides_and_the_rig_never_touch_the_default_install() {
        assert!(migration_allowed(false, false, false));
        assert!(!migration_allowed(true, false, false));
        assert!(!migration_allowed(false, true, false));
        assert!(!migration_allowed(false, false, true));
    }

    #[test]
    fn only_this_editions_stack_is_moved() {
        assert_eq!(plan_stack(false, StackOwner::Lite), StackMove::Move);
        assert_eq!(plan_stack(true, StackOwner::Lite), StackMove::TargetExists);
        assert_eq!(plan_stack(false, StackOwner::Missing), StackMove::NothingToMove);
        assert_eq!(plan_stack(false, StackOwner::Other), StackMove::NotThisEdition);
        assert_eq!(plan_stack(false, StackOwner::Unmarked), StackMove::NotThisEdition);
        assert_eq!(plan_stack(false, StackOwner::Shared), StackMove::KeepShared);
    }

    #[test]
    fn the_profile_follows_its_stack() {
        assert_eq!(plan_profile(false, true, StackOwner::Lite), ProfileMove::Move);
        assert_eq!(plan_profile(false, true, StackOwner::Shared), ProfileMove::Copy);
        assert_eq!(plan_profile(false, true, StackOwner::Other), ProfileMove::Skip);
        assert_eq!(plan_profile(false, true, StackOwner::Unmarked), ProfileMove::Skip);
        assert_eq!(plan_profile(true, true, StackOwner::Lite), ProfileMove::Skip);
        assert_eq!(plan_profile(false, false, StackOwner::Lite), ProfileMove::Skip);
        // The stack already moved on an earlier start: the profile still goes.
        assert_eq!(profile_owner(StackOwner::Missing, StackOwner::Lite), StackOwner::Lite);
        assert_eq!(profile_owner(StackOwner::Missing, StackOwner::Missing), StackOwner::Missing);
        assert_eq!(profile_owner(StackOwner::Other, StackOwner::Lite), StackOwner::Other);
    }

    fn temp_root(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("gb-edition-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        root
    }

    fn make_stack(stack: &Path, markers: &[&str]) {
        std::fs::create_dir_all(stack.join("data")).expect("stack");
        std::fs::write(stack.join("data").join("PG_VERSION"), "17").expect("data file");
        for marker in markers {
            std::fs::write(stack.join(marker), "x").expect("marker");
        }
    }

    #[test]
    fn a_lite_stack_is_renamed_with_its_content() {
        let root = temp_root("move");
        let (legacy, edition) = (legacy_stack(&root), edition_stack(&root));
        make_stack(&legacy, &[LITE_MARKER]);
        migrate_stack(&legacy, &edition, owner_of(&legacy));
        assert!(!legacy.exists());
        assert!(edition.join("data").join("PG_VERSION").is_file());
        assert_eq!(stack_under(&root), edition);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn an_existing_target_or_a_foreign_stack_is_left_alone() {
        let root = temp_root("keep");
        let (legacy, edition) = (legacy_stack(&root), edition_stack(&root));
        make_stack(&legacy, &[LITE_MARKER]);
        make_stack(&edition, &[LITE_MARKER]);
        migrate_stack(&legacy, &edition, owner_of(&legacy));
        assert!(legacy.join("data").join("PG_VERSION").is_file());
        let _ = std::fs::remove_dir_all(&root);

        for markers in [&[OTHER_MARKER][..], &[][..], &[LITE_MARKER, OTHER_MARKER][..]] {
            let root = temp_root("foreign");
            let (legacy, edition) = (legacy_stack(&root), edition_stack(&root));
            make_stack(&legacy, markers);
            migrate_stack(&legacy, &edition, owner_of(&legacy));
            assert!(legacy.join("data").join("PG_VERSION").is_file());
            assert!(!edition.exists());
            let _ = std::fs::remove_dir_all(&root);
        }
    }

    #[test]
    fn the_profile_is_moved_once_and_never_overwritten() {
        let root = temp_root("profile");
        let old = root.join(OLD_IDENTIFIER).join("EBWebView");
        std::fs::create_dir_all(&old).expect("profile");
        std::fs::write(old.join("Local State"), "old").expect("state");
        migrate_profile(&root, StackOwner::Lite);
        assert!(!root.join(OLD_IDENTIFIER).exists());
        let moved = root.join(IDENTIFIER).join("EBWebView").join("Local State");
        assert_eq!(std::fs::read_to_string(&moved).expect("moved"), "old");

        std::fs::create_dir_all(&old).expect("profile again");
        std::fs::write(old.join("Local State"), "newer").expect("state");
        migrate_profile(&root, StackOwner::Lite);
        assert_eq!(std::fs::read_to_string(&moved).expect("kept"), "old");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_shared_profile_is_copied_and_kept() {
        let root = temp_root("shared");
        let old = root.join(OLD_IDENTIFIER);
        std::fs::create_dir_all(&old).expect("profile");
        std::fs::write(old.join("flag"), "1").expect("flag");
        migrate_profile(&root, StackOwner::Shared);
        assert!(old.join("flag").is_file());
        assert!(root.join(IDENTIFIER).join("flag").is_file());
        let _ = std::fs::remove_dir_all(&root);
    }
}
