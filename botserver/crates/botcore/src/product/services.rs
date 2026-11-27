//! Which bundled local services an edition installs and starts.
//!
//! A product file lists the services it does without in `services_off=`
//! (names as in `3rdparty.toml`). A service that is off is never installed,
//! started, waited for or health-checked; its binaries and data already on
//! disk are left untouched, so removing the name from the list (or naming it
//! in `GB_SERVICES_ON`) starts it again exactly as before. `cache` in the list
//! means the server's built-in cache replaces the bundled cache service.

use super::*;

/// Bundled services a product file may switch off.
pub const SWITCHABLE_SERVICES: &[&str] = &["cache", "vector_db", "directory"];

/// Environment variable naming services to keep on even when the product
/// file switches them off (comma separated), e.g. `GB_SERVICES_ON=directory`.
pub const SERVICES_ON_ENV: &str = "GB_SERVICES_ON";

/// Parses the `services_off=` value. Unknown names are ignored with a warning
/// so a typo never switches off something the edition needs.
pub(super) fn parse_services_off(value: &str) -> HashSet<String> {
    let mut off = HashSet::new();
    for name in value.split(',') {
        let name = name.trim().to_lowercase();
        if name.is_empty() {
            continue;
        }
        if SWITCHABLE_SERVICES.contains(&name.as_str()) {
            off.insert(name);
        } else {
            warn!("Unknown service in services_off: {name}");
        }
    }
    off
}

/// Services switched off by `product_off` once `services_on` (the value of
/// [`SERVICES_ON_ENV`]) has turned some back on.
fn effective_off(product_off: &HashSet<String>, services_on: Option<&str>) -> HashSet<String> {
    let on = services_on.map(parse_services_off).unwrap_or_else(|| {
        log::debug!("{} is not set; no service is turned back on", SERVICES_ON_ENV);
        HashSet::new()
    });
    product_off.difference(&on).cloned().collect()
}

impl ProductConfig {
    /// Whether the product file switches the bundled service `name` off.
    pub fn is_service_off(&self, name: &str) -> bool {
        self.services_off.contains(&name.to_lowercase())
    }

    /// Services the product file switches off, sorted.
    pub fn get_services_off(&self) -> Vec<String> {
        let mut off: Vec<String> = self.services_off.iter().cloned().collect();
        off.sort();
        off
    }
}

/// Whether the bundled local service `name` (`cache`, `vector_db`,
/// `directory`) is switched off for this run: listed in the product file's
/// `services_off=` and not named in `GB_SERVICES_ON`. An unreadable
/// configuration answers `false`, so every service runs as before.
pub fn is_service_off(name: &str) -> bool {
    let product_off = PRODUCT_CONFIG
        .read()
        .map(|c| c.services_off.clone())
        .unwrap_or_else(|e| {
            log::debug!("product config lock is unavailable ({e}); treating every service as on");
            HashSet::new()
        });
    let services_on = std::env::var(SERVICES_ON_ENV).ok();
    effective_off(&product_off, services_on.as_deref()).contains(&name.to_lowercase())
}

/// Whether the server's built-in cache replaces the bundled cache service
/// (`cache` switched off): the installer and boot then never install or start
/// Redis/Valkey, and the server serves the cache itself.
pub fn uses_builtin_cache() -> bool {
    is_service_off("cache")
}

/// Keeps the entries of `items` whose service (`name_of`) is not switched off,
/// in order, and logs one line per service left out.
pub fn services_on<T>(
    items: Vec<T>,
    name_of: impl Fn(&T) -> &str,
    is_off: impl Fn(&str) -> bool,
) -> Vec<T> {
    items
        .into_iter()
        .filter(|item| {
            let name = name_of(item);
            let off = is_off(name);
            if off {
                info!("{name} is switched off for this edition; not installing or starting it");
            }
            !off
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(names: &[&str]) -> HashSet<String> {
        names.iter().map(|n| n.to_string()).collect()
    }

    #[test]
    fn the_list_is_parsed_case_insensitively_and_unknown_names_are_dropped() {
        assert_eq!(
            parse_services_off(" Cache , vector_db,DIRECTORY,tables,,drive "),
            set(&["cache", "vector_db", "directory"])
        );
        assert!(parse_services_off("").is_empty());
    }

    #[test]
    fn the_product_file_key_reaches_the_config() {
        let config = ProductConfig::parse("apps=chat\nservices_off=cache,vector_db,directory").unwrap();
        assert!(config.is_service_off("cache"));
        assert!(config.is_service_off("Vector_DB"));
        assert!(config.is_service_off("directory"));
        assert!(!config.is_service_off("tables"));
        assert_eq!(config.get_services_off(), vec!["cache", "directory", "vector_db"]);
    }

    #[test]
    fn without_the_key_every_service_is_on() {
        let config = ProductConfig::parse("apps=chat").unwrap();
        assert!(config.get_services_off().is_empty());
        assert!(!ProductConfig::default().is_service_off("directory"));
    }

    #[test]
    fn the_environment_turns_a_service_back_on() {
        let off = set(&["cache", "vector_db", "directory"]);
        assert_eq!(effective_off(&off, None), off);
        assert_eq!(effective_off(&off, Some("directory, vector_db")), set(&["cache"]));
        assert_eq!(effective_off(&off, Some("")), off);
    }

    #[test]
    fn the_start_and_install_plans_leave_switched_off_services_out() {
        let off = set(&["cache", "vector_db", "directory"]);
        let is_off = |name: &str| off.contains(name);
        let start = services_on(
            vec![("vector_db", 45), ("tables", 0), ("cache", 30), ("drive", 0), ("directory", 60), ("llm", 60)],
            |service| service.0,
            is_off,
        );
        assert_eq!(start, vec![("tables", 0), ("drive", 0), ("llm", 60)]);
        let install = services_on(
            vec!["tables", "cache", "drive", "directory", "llm", "vector_db"],
            |name| *name,
            is_off,
        );
        assert_eq!(install, vec!["tables", "drive", "llm"]);
        let all_on = services_on(vec!["tables", "cache"], |name| *name, |_| false);
        assert_eq!(all_on, vec!["tables", "cache"]);
    }
}
