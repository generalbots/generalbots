//! Shared, exact admin-group-name resolver (#2633).
//!
//! Role derivation must never use a substring test such as
//! `name.to_lowercase().contains("admin")`: that grants full admin to groups
//! like `notadmin`, `admintrainee` or `admins-in-training`. All call sites now
//! use [`is_admin_group_name`], which matches only an explicit, case-insensitive
//! set of names that the project actually uses for administrator groups.

/// Canonical administrator group/role names (lowercase).
///
/// Compared with [`str::eq_ignore_ascii_case`], so the case variants used by
/// the directory (`Admin`, `SuperAdmin`) and JWT role maps (`ADMIN`,
/// `super_admin`) match without widening the set to substrings.
pub const ADMIN_GROUP_NAMES: &[&str] = &[
    "admin",
    "administrator",
    "superadmin",
    "super_admin",
    "super-admin",
    "super",
];

/// Whether `name` is exactly one of the recognised administrator group names.
///
/// Comparison is case-insensitive and ignores surrounding whitespace, but it
/// is a full-string match: near misses cannot acquire admin privileges.
pub fn is_admin_group_name(name: &str) -> bool {
    let candidate = name.trim();
    ADMIN_GROUP_NAMES
        .iter()
        .any(|admin| candidate.eq_ignore_ascii_case(admin))
}

#[cfg(test)]
mod tests {
    use super::{is_admin_group_name, ADMIN_GROUP_NAMES};

    #[test]
    fn exact_admin_names_match() {
        for name in ADMIN_GROUP_NAMES {
            assert!(
                is_admin_group_name(name),
                "canonical admin name '{name}' must match"
            );
        }
    }

    #[test]
    fn case_variants_match() {
        for name in [
            "Admin",
            "ADMIN",
            "Administrator",
            "SuperAdmin",
            "SUPERADMIN",
            "Super_Admin",
            "super-ADMIN",
        ] {
            assert!(
                is_admin_group_name(name),
                "case variant '{name}' must match"
            );
        }
    }

    #[test]
    fn whitespace_around_admin_name_matches() {
        assert!(is_admin_group_name("  admin  "));
        assert!(is_admin_group_name("\tSuperAdmin\n"));
    }

    #[test]
    fn near_miss_names_do_not_match() {
        for name in [
            "notadmin",
            "not-admin",
            "not_admin",
            "admintrainee",
            "admins-in-training",
            "admin-readonly",
            "administrators",
            "superadmins",
            "subadmin",
            "admin2",
            "admin.view",
            "superuser",
            "user",
            "viewer",
            "moderator",
            "",
        ] {
            assert!(
                !is_admin_group_name(name),
                "near-miss name '{name}' must NOT grant admin"
            );
        }
    }
}
