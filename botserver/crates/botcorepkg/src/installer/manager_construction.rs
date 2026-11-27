use super::*;

impl PackageManager {
    pub fn new(mode: InstallMode, tenant: Option<String>) -> Result<Self> {
        let os_type = detect_os();
        let base_path = if mode == InstallMode::Container {
            PathBuf::from("/opt/gbo")
        } else if let Ok(custom_path) = std::env::var("BOTSERVER_STACK_PATH") {
            PathBuf::from(custom_path)
        } else {
            std::env::current_dir()?.join("botserver-stack")
        };
        let tenant = tenant.unwrap_or_else(|| "default".to_string());

        let mut pm = Self {
            mode,
            os_type,
            base_path,
            tenant,
            components: HashMap::new(),
        };
        pm.register_components();
        Ok(pm)
    }

    pub fn with_base_path(
        mode: InstallMode,
        tenant: Option<String>,
        base_path: PathBuf,
    ) -> Result<Self> {
        let os_type = detect_os();
        let tenant = tenant.unwrap_or_else(|| "default".to_string());

        let mut pm = Self {
            mode,
            os_type,
            base_path,
            tenant,
            components: HashMap::new(),
        };
        pm.register_components();
        Ok(pm)
    }

    /// Build the full component registry (single source of truth, shared
    /// with `botcore`'s runtime PackageManager).
    pub fn all_components() -> HashMap<String, ComponentConfig> {
        let mut components = HashMap::new();
        installer_regs::register_drive(&mut components);
        installer_regs::register_tables(&mut components);
        installer_regs::register_cache(&mut components);
        installer_regs::register_llm(&mut components);
        installer_regs::register_email(&mut components);
        installer_regs::register_proxy(&mut components);
        installer_regs::register_directory(&mut components);
        installer_regs::register_alm(&mut components);
        installer_regs::register_alm_ci(&mut components);
        installer_regs::register_dns(&mut components);
        installer_regs::register_meeting(&mut components);
        installer_regs::register_remote_desktop(&mut components);
        installer_regs::register_vector_db(&mut components);
        installer_regs::register_timeseries_db(&mut components);
        installer_regs::register_vault(&mut components);
        installer_regs::register_observability(&mut components);
        installer_regs::register_host(&mut components);
        installer_regs::register_botmodels(&mut components);
        components
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// #2582 — keys botserver must never register or manage.
    /// `devtools` was a no-op placeholder and `remote_terminal` was renamed to
    /// `remote_desktop`. Their absence from the registry is what keeps them off
    /// the CLI/install surface.
    const EXCLUDED_KEYS: &[&str] = &["devtools", "remote_terminal"];

    /// #2582 — keys that must be present in the botserver registry.
    const REQUIRED_KEYS: &[&str] = &["remote_desktop", "timeseries_db", "observability"];

    /// Assert the full membership contract (both exclusions and inclusions)
    /// for one view of the registry. Shared by the static registry and the
    /// constructed manager so both surfaces are held to the same bar.
    fn assert_expected_membership(components: &HashMap<String, ComponentConfig>, label: &str) {
        for key in REQUIRED_KEYS {
            assert!(
                components.contains_key(*key),
                "{label} must include '{key}' (#2582)"
            );
        }
        for key in EXCLUDED_KEYS {
            assert!(
                !components.contains_key(*key),
                "{label} must exclude '{key}' (#2582)"
            );
        }
    }

    fn test_manager() -> PackageManager {
        PackageManager::with_base_path(
            InstallMode::Local,
            None,
            PathBuf::from("/tmp/gb-manager-test"),
        )
        .expect("with_base_path must succeed with an explicit base path")
    }

    /// The static registry assembled by `all_components()` is the single
    /// source of truth consumed by `register_components`, so its membership
    /// proves no manager instance can expose an excluded key.
    #[test]
    fn all_components_have_expected_membership() {
        let components = PackageManager::all_components();
        assert_expected_membership(&components, "PackageManager::all_components()");
    }

    /// A concrete `PackageManager` built through the normal constructor path
    /// must satisfy the same membership contract as the static registry.
    #[test]
    fn manager_components_have_expected_membership() {
        let manager = test_manager();
        assert_expected_membership(&manager.components, "constructed PackageManager.components");
    }

    /// Guard against future divergence: the constructed manager's map must
    /// expose exactly the keys produced by `all_components()`.
    #[test]
    fn manager_components_keys_match_all_components() {
        let all = PackageManager::all_components();
        let manager = test_manager();
        let all_keys: HashSet<&String> = all.keys().collect();
        let manager_keys: HashSet<&String> = manager.components.keys().collect();
        assert_eq!(
            all_keys, manager_keys,
            "constructed manager must expose exactly all_components() keys (#2582)"
        );
    }
}
