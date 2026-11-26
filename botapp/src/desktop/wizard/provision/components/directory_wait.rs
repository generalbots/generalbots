//! Whether the first boot has to wait for the identity provider's config.
//!
//! The product file in the stack can switch the directory service off
//! (`services_off=` lists `directory`); the assistant then never writes
//! `directory_config.json` and the wizard must not wait for it. The
//! `GB_SERVICES_ON` environment variable turns a listed service back on, the
//! same rule the server applies.

use std::path::Path;

/// Whether the product file in `stack` switches the directory service off.
pub(super) fn directory_switched_off(stack: &Path) -> bool {
    let product = std::fs::read_to_string(stack.join(crate::desktop::sidecar::PRODUCT_FILE))
        .unwrap_or_default();
    let services_on = std::env::var("GB_SERVICES_ON").unwrap_or_default();
    is_off(&product, &services_on, "directory")
}

/// Whether `service` is listed in `services_off=` of `product` and not named
/// in `services_on`. When multiple `services_off=` lines exist, only the last
/// one counts (same semantics as the server's key=value parser, which assigns
/// rather than merges).
fn is_off(product: &str, services_on: &str, service: &str) -> bool {
    let listed = |list: &str| {
        list.split(',')
            .any(|name| name.trim().eq_ignore_ascii_case(service))
    };
    let last_value = product
        .lines()
        .map(str::trim)
        .filter(|line| !line.starts_with('#'))
        .filter_map(|line| line.split_once('='))
        .filter(|(key, _)| key.trim().eq_ignore_ascii_case("services_off"))
        .map(|(_, value)| value)
        .last();
    let off = last_value.is_some_and(|value| listed(value));
    off && !listed(services_on)
}

#[cfg(test)]
mod tests {
    use super::is_off;

    #[test]
    fn the_directory_is_off_only_when_listed_and_not_turned_back_on() {
        let product = "name=X\n# services_off=tables\nservices_off=cache, vector_db ,Directory\n";
        assert!(is_off(product, "", "directory"));
        assert!(!is_off(product, "vector_db,directory", "directory"));
        assert!(!is_off(product, "", "tables"));
        assert!(!is_off("name=X\n", "", "directory"));
        assert!(!is_off("", "", "directory"));
    }

    #[test]
    fn the_last_services_off_line_wins() {
        let two_lines = "services_off=cache,directory\nservices_off=cache\n";
        assert!(!is_off(two_lines, "", "directory"), "second line replaced the first");
        assert!(is_off(two_lines, "", "cache"));
        let reversed = "services_off=cache\nservices_off=cache,directory\n";
        assert!(is_off(reversed, "", "directory"), "last line lists directory");
    }
}
