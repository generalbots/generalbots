//! Unit tests for `consent_action_class`.

#[cfg(feature = "consent")]
use super::handle::consent_action_class;

#[cfg(feature = "consent")]
#[test]
fn read_only_commands_get_read_class() {
    for name in ["api.find", "drive.list", "drive.search", "settings.read", "products.items.list"] {
        assert_eq!(consent_action_class(name), "read", "{name} should be read");
    }
}

#[cfg(feature = "consent")]
#[test]
fn write_and_delete_classes() {
    assert_eq!(consent_action_class("drive.write"), "update");
    assert_eq!(consent_action_class("api.exec"), "update");
    assert_eq!(consent_action_class("database.rows.delete"), "delete");
    assert_eq!(consent_action_class("integrations.api-keys.revoke"), "delete");
}
