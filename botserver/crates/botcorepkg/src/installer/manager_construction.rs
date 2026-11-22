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
        // #2582 — table_editor is out of scope for botserver (pending deletion):
        // it must not be installed or managed here. The registrar in
        // `installer_regs::register_workspace` is intentionally left uncalled.
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

    /// #2582 — `table_editor` is explicitly out of scope for botserver
    /// (pending deletion). The default registry assembled here is the single
    /// source of truth consumed by `register_components`, so the absence of
    /// the key proves no manager instance can expose it.
    #[test]
    fn all_components_exclude_table_editor() {
        let components = PackageManager::all_components();
        assert!(
            !components.contains_key("table_editor"),
            "table_editor must not be in the default manager component set (#2582)"
        );
    }

    /// A concrete `PackageManager` built through the normal constructor path
    /// must likewise have no `table_editor` in its `components` map.
    #[test]
    fn manager_components_exclude_table_editor() {
        let excluded = PackageManager::with_base_path(
            InstallMode::Local,
            None,
            PathBuf::from("/tmp/gb-manager-test"),
        )
        .map(|pm| !pm.components.contains_key("table_editor"))
        .unwrap_or(false);
        assert!(
            excluded,
            "constructed manager must not manage table_editor (#2582)"
        );
    }
}
