//! Where the per-user data stack lives, per edition.
//!
//! Every build used `<local data>/generalbots/stack`, so the Lite edition and
//! the platform's desktop would have shared one database, vault and drive.
//! The Lite edition has its own folder. A stack the Lite edition left in the
//! shared folder before the rename stays usable in place until
//! `edition_migration` has moved it, so a move that cannot happen yet (files
//! held open, another volume) never turns into an empty first run.

use std::path::{Path, PathBuf};

/// The folder every build used before the editions were separated.
pub const LEGACY_FOLDER: &str = "generalbots";

/// This edition's folder under the per-user local data directory.
pub const DATA_FOLDER: &str = if cfg!(feature = "gblite") {
    "GeneralBotsLite"
} else {
    LEGACY_FOLDER
};

pub const STACK_FOLDER: &str = "stack";

/// Product file the Lite edition keeps in its stack (the shell writes it at
/// every sidecar start), and the one every other desktop build keeps.
pub const LITE_MARKER: &str = ".product.gblite";
pub const OTHER_MARKER: &str = ".product.desktop-local";

/// Which edition a stack folder belongs to, read from its product files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackOwner {
    /// Only the Lite edition's product file is there.
    Lite,
    /// Only another build's product file is there.
    Other,
    /// Both editions have run on this stack.
    Shared,
    /// No product file: never started, or not a stack at all.
    Unmarked,
    /// The folder does not exist.
    Missing,
}

pub fn owner_from_markers(exists: bool, lite: bool, other: bool) -> StackOwner {
    match (exists, lite, other) {
        (false, _, _) => StackOwner::Missing,
        (true, true, false) => StackOwner::Lite,
        (true, false, true) => StackOwner::Other,
        (true, true, true) => StackOwner::Shared,
        (true, false, false) => StackOwner::Unmarked,
    }
}

pub fn owner_of(stack: &Path) -> StackOwner {
    owner_from_markers(
        stack.is_dir(),
        stack.join(LITE_MARKER).is_file(),
        stack.join(OTHER_MARKER).is_file(),
    )
}

pub fn legacy_stack(data_local: &Path) -> PathBuf {
    data_local.join(LEGACY_FOLDER).join(STACK_FOLDER)
}

pub fn edition_stack(data_local: &Path) -> PathBuf {
    data_local.join(DATA_FOLDER).join(STACK_FOLDER)
}

/// Picks the stack folder. The edition's own folder wins once it exists; until
/// then a legacy stack the Lite edition has used (alone or with another build)
/// is used where it is.
pub fn choose_stack(
    edition: PathBuf,
    edition_exists: bool,
    legacy: PathBuf,
    legacy_owner: StackOwner,
) -> PathBuf {
    if edition_exists || edition == legacy {
        return edition;
    }
    match legacy_owner {
        StackOwner::Lite | StackOwner::Shared => legacy,
        StackOwner::Other | StackOwner::Unmarked | StackOwner::Missing => edition,
    }
}

/// The stack folder under a per-user local data directory.
pub fn stack_under(data_local: &Path) -> PathBuf {
    let edition = edition_stack(data_local);
    let legacy = legacy_stack(data_local);
    if edition == legacy {
        return edition;
    }
    let edition_exists = edition.is_dir();
    let owner = if edition_exists {
        StackOwner::Missing
    } else {
        owner_of(&legacy)
    };
    choose_stack(edition, edition_exists, legacy, owner)
}

/// The stack folder for this user, without the `BOTSERVER_STACK_PATH`
/// override (callers apply it first).
pub fn default_stack(data_local: Option<PathBuf>) -> PathBuf {
    stack_under(&data_local.unwrap_or_else(|| PathBuf::from(".")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_markers_name_the_owner() {
        assert_eq!(owner_from_markers(false, true, true), StackOwner::Missing);
        assert_eq!(owner_from_markers(true, true, false), StackOwner::Lite);
        assert_eq!(owner_from_markers(true, false, true), StackOwner::Other);
        assert_eq!(owner_from_markers(true, true, true), StackOwner::Shared);
        assert_eq!(owner_from_markers(true, false, false), StackOwner::Unmarked);
    }

    #[test]
    fn the_folder_matches_the_edition() {
        if cfg!(feature = "gblite") {
            assert_eq!(DATA_FOLDER, "GeneralBotsLite");
        } else {
            assert_eq!(DATA_FOLDER, "generalbots");
        }
    }

    #[test]
    fn the_editions_own_stack_wins_once_it_exists() {
        let new = PathBuf::from("/d/GeneralBotsLite/stack");
        let old = PathBuf::from("/d/generalbots/stack");
        assert_eq!(choose_stack(new.clone(), true, old.clone(), StackOwner::Lite), new);
        assert_eq!(choose_stack(new.clone(), true, old.clone(), StackOwner::Shared), new);
    }

    #[test]
    fn a_lite_stack_not_moved_yet_is_used_in_place() {
        let new = PathBuf::from("/d/GeneralBotsLite/stack");
        let old = PathBuf::from("/d/generalbots/stack");
        assert_eq!(choose_stack(new.clone(), false, old.clone(), StackOwner::Lite), old);
        assert_eq!(choose_stack(new.clone(), false, old.clone(), StackOwner::Shared), old);
    }

    #[test]
    fn another_builds_stack_is_never_taken() {
        let new = PathBuf::from("/d/GeneralBotsLite/stack");
        let old = PathBuf::from("/d/generalbots/stack");
        assert_eq!(choose_stack(new.clone(), false, old.clone(), StackOwner::Other), new);
        assert_eq!(choose_stack(new.clone(), false, old.clone(), StackOwner::Unmarked), new);
        assert_eq!(choose_stack(new.clone(), false, old, StackOwner::Missing), new);
    }

    #[test]
    fn the_folder_on_disk_decides() {
        let root = std::env::temp_dir().join(format!("gb-data-root-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let legacy = legacy_stack(&root);
        if cfg!(feature = "gblite") {
            assert_eq!(stack_under(&root), edition_stack(&root));
            std::fs::create_dir_all(&legacy).expect("legacy stack");
            std::fs::write(legacy.join(LITE_MARKER), "x").expect("marker");
            assert_eq!(stack_under(&root), legacy);
            std::fs::create_dir_all(edition_stack(&root)).expect("edition stack");
            assert_eq!(stack_under(&root), edition_stack(&root));
        } else {
            assert_eq!(stack_under(&root), legacy);
        }
        let _ = std::fs::remove_dir_all(&root);
    }
}
