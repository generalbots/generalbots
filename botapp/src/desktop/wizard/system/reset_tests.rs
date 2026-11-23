//! #2689 — offline regression tests for the destructive stack reset.
//!
//! The deletion helper is exercised only against a private temp stack: no test
//! touches the configured install directory or any real user data.

use super::{empty_stack_dir, entry_is_inside_stack, reset_target_is_exact};
use std::path::{Path, PathBuf};

struct TempStack(PathBuf);

impl TempStack {
    fn new(tag: &str) -> Self {
        let path = std::env::temp_dir().join(format!("gb-reset-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        assert!(std::fs::create_dir_all(&path).is_ok(), "create temp stack");
        Self(path)
    }
}

impl Drop for TempStack {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn only_the_exact_canonical_stack_authorizes_a_reset() {
    let stack = TempStack::new("target");
    assert!(reset_target_is_exact(&stack.0, &stack.0));
    let child = stack.0.join("workspace");
    assert!(std::fs::create_dir(&child).is_ok());
    assert!(!reset_target_is_exact(&stack.0, &child));
    assert!(!reset_target_is_exact(
        &stack.0,
        stack.0.parent().unwrap_or(Path::new("/"))
    ));
    assert!(!reset_target_is_exact(
        Path::new("/no/such/stack"),
        Path::new("/no/such/stack")
    ));
}

#[test]
fn entries_outside_the_canonical_stack_are_refused() {
    let stack = TempStack::new("entry");
    let file = stack.0.join("data.env");
    assert!(std::fs::write(&file, b"ok").is_ok());
    assert!(entry_is_inside_stack(&stack.0, &file));
    assert!(!entry_is_inside_stack(&stack.0, &stack.0.join("missing")));
    let outside = std::env::temp_dir().join(format!("gb-reset-outside-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&outside);
    assert!(std::fs::create_dir(&outside).is_ok());
    assert!(!entry_is_inside_stack(&stack.0, &outside));
    #[cfg(unix)]
    {
        let link = stack.0.join("link");
        assert!(std::os::unix::fs::symlink(&outside, &link).is_ok());
        assert!(!entry_is_inside_stack(&stack.0, &link));
    }
    let _ = std::fs::remove_dir_all(&outside);
}

#[test]
fn a_reset_empties_only_the_temporary_stack() {
    let stack = TempStack::new("wipe");
    let outside = std::env::temp_dir().join(format!("gb-reset-keep-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&outside);
    assert!(std::fs::create_dir(&outside).is_ok());
    let victim = outside.join("victim.txt");
    assert!(std::fs::write(&victim, b"keep").is_ok());
    assert!(std::fs::write(stack.0.join("state.env"), b"gone").is_ok());
    let sub = stack.0.join("data");
    assert!(std::fs::create_dir(&sub).is_ok());
    assert!(std::fs::write(sub.join("db"), b"gone").is_ok());

    assert!(empty_stack_dir(&stack.0).is_ok());

    assert!(
        std::fs::read_dir(&stack.0)
            .map(|mut entries| entries.next().is_none())
            .unwrap_or(false),
        "temporary stack was not emptied"
    );
    assert!(victim.is_file(), "reset deleted a file outside the stack");
    let _ = std::fs::remove_dir_all(&outside);
}
