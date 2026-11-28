//! Rooted and prefixed path forms. On Windows `\x`, `C:x`, `\\server\share`
//! and `\\?\C:\x` are not `is_absolute()`, or not only, yet `base.join(p)`
//! replaces the base with them. None of them may land outside the base.

use super::*;

const HOSTILE_FORMS: [&str; 8] = [
    r"\etc\passwd",
    "/etc/passwd",
    "C:foo",
    r"C:\Windows",
    r"\\server\share\x",
    r"\\?\C:\x",
    r"..\x",
    r"a\..\..\x",
];

fn base() -> PathBuf {
    std::env::temp_dir().join("pg-windows-forms-base")
}

fn stays_under(landed: &Path, base: &Path) -> bool {
    landed.starts_with(base) && !landed.components().any(|c| c == Component::ParentDir)
}

#[test]
fn no_hostile_form_lands_outside_the_base() {
    let base = base();
    let open = PathGuard::new(PathGuardConfig::new().allow_hidden(true));
    let rooted = PathGuard::new(PathGuardConfig::new().with_root(base.clone()));
    for form in HOSTILE_FORMS {
        let input = Path::new(form);
        let cases = [
            ("validate", open.validate(input).map(|p| base.join(p))),
            (
                "validate_relative_under",
                open.validate_relative_under(&base, input),
            ),
            (
                "validate_and_resolve without roots",
                open.validate_and_resolve(&base, input),
            ),
            (
                "validate_and_resolve with root",
                rooted.validate_and_resolve(&base, input),
            ),
            ("join_safe", join_safe(&base, input)),
        ];
        for (function, result) in cases {
            if let Ok(landed) = result {
                assert!(
                    stays_under(&landed, &base),
                    "{function}({form:?}) lands at {} outside {}",
                    landed.display(),
                    base.display()
                );
            }
        }
        if let Ok(landed) = canonicalize_safe(input, &base) {
            assert!(
                !is_rooted(&landed) || landed.starts_with(&base),
                "canonicalize_safe({form:?}) lands at {}",
                landed.display()
            );
        }
    }
}

#[test]
fn hostile_forms_are_refused_by_validate_relative_under() {
    let guard = PathGuard::new(PathGuardConfig::new().allow_hidden(true));
    let base = base();
    for form in HOSTILE_FORMS {
        // On a POSIX host `C:foo` is an ordinary file name inside the base.
        if form == "C:foo" && cfg!(not(windows)) {
            continue;
        }
        assert!(
            guard.validate_relative_under(&base, Path::new(form)).is_err(),
            "{form:?} must be refused"
        );
    }
}

#[test]
fn rooted_input_is_refused_when_no_root_is_configured() {
    let guard = PathGuard::new(PathGuardConfig::new().allow_hidden(true));
    for form in ["/etc/passwd", "/data/uploads/a.txt"] {
        assert_eq!(
            guard.validate(Path::new(form)).unwrap_err(),
            PathGuardError::AbsolutePath,
            "{form:?}"
        );
    }
    assert!(!is_safe_path(Path::new("/etc/passwd")));
}

#[test]
fn rooted_input_cannot_climb_out_of_its_allowed_root() {
    let guard = PathGuard::new(PathGuardConfig::new().with_root("/data/uploads"));
    assert!(guard
        .validate(Path::new("/data/uploads/../../etc/passwd"))
        .is_err());
    assert!(guard
        .validate(Path::new("/data/uploads/../uploads2/a.txt"))
        .is_err());
    assert_eq!(
        guard.validate(Path::new("/data/uploads/a/../b.txt")).ok(),
        Some(PathBuf::from("/data/uploads/b.txt"))
    );
}

#[test]
fn is_rooted_matches_every_form_that_replaces_a_base_on_join() {
    assert!(is_rooted(Path::new("/etc/passwd")));
    assert!(!is_rooted(Path::new("a/b.txt")));
    assert!(!is_rooted(Path::new("./a")));
    #[cfg(windows)]
    for form in [r"\etc\passwd", "C:foo", r"C:\Windows", r"\\server\share\x", r"\\?\C:\x"] {
        assert!(is_rooted(Path::new(form)), "{form:?}");
        let joined = Path::new(r"C:\stack").join(form);
        assert!(!joined.starts_with(r"C:\stack"), "{form:?} replaces the base");
    }
}
