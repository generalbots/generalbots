use super::*;

pub(super) fn installs() -> Arc<AppInstallStats> {
    INSTALLS
        .get_or_init(|| Arc::new(AppInstallStats::default()))
        .clone()
}

pub(crate) fn command_stats() -> Arc<CommandUsageStats> {
    COMMAND_STATS
        .get_or_init(|| Arc::new(CommandUsageStats::default()))
        .clone()
}

/// Records one command/API execution for usage stats (#2372).
pub fn record_command_usage(key: &str, app: Option<&str>, label: Option<&str>) {
    let stats = command_stats();
    let mut entries = match stats.entries.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let entry = entries
        .entry(key.to_string())
        .or_insert_with(|| CommandUsageEntry {
            count: 0,
            app: app.map(str::to_string),
            label: label.map(str::to_string),
        });
    entry.count = entry.count.saturating_add(1);
    if entry.app.is_none() {
        entry.app = app.map(str::to_string);
    }
    if entry.label.is_none() {
        entry.label = label.map(str::to_string);
    }
}

pub fn register<S: Clone + Send + Sync + 'static>(r: Router<S>) -> Router<S> {
    let router = r;
    #[cfg(feature = "uifragments")]
    let router = router.merge(botuifragments::register(Router::new()));
    let router = router
        .route("/api/apps/catalog", get(catalog_handler))
        .route("/api/apps/install", post(install_handler))
        .route("/api/apps/install/stats", get(install_stats_handler))
        .route("/api/commands/stats", get(command_stats_handler))
        .route("/api/agent/plan", post(agent_plan_handler));
    integration_catalog::register(router)
}
