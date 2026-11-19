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

/// Most-executed commands, busiest first (#2372).
///
/// The ranking is what the prompt uses to surface "common commands" from real
/// usage instead of a fixed list; ties break on the command name so two calls
/// with equal counts always return the same order.
pub fn top_commands(limit: usize) -> Vec<(String, u64)> {
    if limit == 0 {
        return Vec::new();
    }
    let stats = command_stats();
    let entries = match stats.entries.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let mut ranked: Vec<(String, u64)> = entries
        .iter()
        .filter(|(_, entry)| entry.count > 0)
        .map(|(name, entry)| (name.clone(), entry.count))
        .collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    ranked.truncate(limit);
    ranked
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

#[cfg(test)]
mod top_commands_tests {
    use super::{record_command_usage, top_commands};

    /// #2372 — the ranking is count first, then name, so the prompt order is
    /// stable for equal counts. Only this test's own keys are asserted: other
    /// tests in the binary share the process-wide counter.
    #[test]
    fn ranking_is_count_first_then_name() {
        record_command_usage("zz.rank.second", None, None);
        record_command_usage("zz.rank.first", None, None);
        record_command_usage("zz.rank.first", None, None);
        record_command_usage("zz.rank.third", None, None);

        let ranked = top_commands(usize::MAX);
        let names: Vec<&str> = ranked
            .iter()
            .map(|(name, _)| name.as_str())
            .filter(|name| name.starts_with("zz.rank."))
            .collect();
        assert_eq!(
            names,
            vec!["zz.rank.first", "zz.rank.second", "zz.rank.third"]
        );
    }

    /// #2372 — a zero limit must stay cheap; callers use it to disable the
    /// usage-driven tail entirely.
    #[test]
    fn zero_limit_returns_nothing() {
        record_command_usage("zz.zero.limit", None, None);
        assert!(top_commands(0).is_empty());
    }
}
