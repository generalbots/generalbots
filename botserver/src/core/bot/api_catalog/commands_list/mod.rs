use super::*;

/// The stable command list rendered into the prompt and used by discovery.
/// This now comes from the single source of truth in `crate::apps::commands`.
pub fn commands_list() -> Vec<&'static AppCommand> {
    crate::apps::commands::all_commands().iter().collect()
}
/// Full REST surface (method, path, one-line summary). Not injected — only
/// searched on demand through `api.find`, so the LLM knows the entire scale
/// without a giant system prompt. Keep entries grouped by domain.
mod endpoints;

/// #2372 — how many usage-ranked commands may join the named shortlist.
///
/// The baseline list stays in place so a fresh install still has a working
/// surface; this only lets real usage add the busiest extra commands on top.
const USAGE_RANKED_COMMANDS: usize = 4;

/// Full REST surface (method, path, one-line summary). Not injected — only
/// searched on demand through `api.find`, so the LLM knows the entire scale
/// without a giant system prompt. Keep entries grouped by domain.
pub fn all_endpoints() -> &'static [ApiEndpoint] {
    endpoints::all()
}
/// Compact system-prompt fragment teaching the `__api_call__` contract and
/// the routing rule, shortened to the current user's role so admin-only
/// commands are never proposed to regular users.
/// Compact system-prompt fragment teaching the `__api_call__` contract and the
/// discovery round-trip. The injected surface is intentionally tiny — instead
/// of dumping every command (there are now 1000+ harvested endpoints), the LLM
/// is taught to DISCOVER the exact command/endpoint with `api.find`, then EXECUTE
/// it with `api.exec` (or a named curated command). This keeps the prompt small
/// while unlocking the full action surface on every channel.
pub fn api_command_instructions(role: &str) -> String {
    let mut lines = Vec::with_capacity(24);
    lines.push("---".to_string());
    lines.push("## API Commands".to_string());
    // #2562 — suppress "or search the web" when the feature is not compiled.
    let intro = if cfg!(feature = "research") || cfg!(feature = "web-search") {
        "You can call backend commands to read or change business data (fiscal, banking, drive files) \
         or search the web. Answer general-knowledge questions (trivia, definitions, opinions) DIRECTLY \
         with no command."
    } else {
        "You can call backend commands to read or change business data (fiscal, banking, drive files). \
         Answer general-knowledge questions (trivia, definitions, opinions) DIRECTLY \
         with no command."
    };
    lines.push(intro.to_string());
    lines.push(format!(
        "Your role: {role}. Only issue commands you are allowed to use."
    ));
    lines.push("To call one, make the FIRST line of your reply exactly a JSON object:".to_string());
    lines.push("{\"__api_call__\": {\"name\": \"<command>\", \"params\": {<named params>}, \"compose\": true}}".to_string());
    lines.push("then add a short user-facing note after it.".to_string());

    // Discovery contract — never list the full surface in the prompt.
    lines.push("Discovery (2-step, prefer this over guessing a command):".to_string());
    lines.push("  1. `api.find` with params {query}: returns the matching curated command OR a harvested endpoint (kind, name/method/path, summary).".to_string());
    lines.push("  2. Run it: `api.exec` with params {method, path, params} executes any registered endpoint; or use the curated command name when returned.".to_string());
    // Models guessed plausible-looking routes (/api/products) that are not in
    // the router and the turn died with "not a registered route" (#2204).
    lines.push("  NEVER invent a REST path: only paths returned by `api.find` are registered. When a curated command below covers the request, call it directly.".to_string());

    // Only the most common commands are named explicitly to avoid prompt bloat.
    // #2561 — the gblite edition has no billing, banking, tax, vibe, tasks or
    // integrations; listing those commands made the model call them and trigger
    // consent prompts / errors for tools that do not exist.
    let prompt_commands: &[&str] = if cfg!(feature = "gblite") {
        &[
            "apps.find",
            "api.find",
            "api.exec",
            "drive.list",
            "drive.search",
            "settings.read",
        ]
    } else {
        &[
            "apps.find",
            "api.find",
            "api.exec",
            "tasks.autotask.create",
            "tasks.autotask.list",
            "vibe.project.change",
            "integrations.actions.list",
            "service.tax",
            "banking.diagnosis",
            "drive.list",
            "products.items.list",
            "billing.invoice.list",
            "billing.payment.list",
            "billing.quote.list",
        ]
    };
    // #2372 — the busiest curated commands join the fixed shortlist, so the
    // named surface follows what people actually run. Capped, deduplicated and
    // role-filtered: an admin-only command recorded by an admin never reaches a
    // regular user's prompt.
    let mut listed: Vec<&'static str> = prompt_commands.to_vec();
    let mut ranked_added = 0usize;
    // Filter before capping: the stats map also holds raw `METHOD /path` keys
    // from api.exec, and on a busy deployment they outnumber curated commands.
    // A fixed pre-filter window would let those raw keys fill every slot and
    // hide the curated commands entirely, so inspect the full ranking and cap
    // only the entries that survive the filter.
    for (name, _count) in crate::apps::top_commands(usize::MAX) {
        if ranked_added >= USAGE_RANKED_COMMANDS {
            break;
        }
        let Some(command) = command_by_name(&name) else {
            continue;
        };
        if listed.contains(&command.name) || (command.admin_only && role != "admin") {
            continue;
        }
        listed.push(command.name);
        ranked_added += 1;
    }
    lines.push("Common commands you may call directly:".to_string());
    for name in listed {
        let Some(cmd) = command_by_name(name) else {
            continue;
        };
        let params = cmd
            .params
            .iter()
            .map(|(n, d)| format!("{n}: {d}"))
            .collect::<Vec<_>>()
            .join("; ");
        lines.push(format!(
            "- {name}: {summary} (params: {params})",
            summary = cmd.summary
        ));
    }
    let rules = if cfg!(feature = "gblite") {
        "Rules: use compose=true when the answer needs prose from the data. Never mention JSON or \
         commands to the user. Discovery: api.find matches command NAMES and SUMMARIES by keyword, not \
         free-form intent, so search with domain words such as 'drive', 'files', 'settings' \
         rather than the user's own phrasing. When the command is listed above, call it directly by name \
         instead of searching."
    } else {
        "Rules: use compose=true when the answer needs prose from the data. Never mention JSON or \
         commands to the user. Discovery: api.find matches command NAMES and SUMMARIES by keyword, not \
         free-form intent, so search with domain words such as 'task', 'automation', 'drive', 'invoice' \
         rather than the user's own phrasing. When the command is listed above, call it directly by name \
         instead of searching. To build a new automation from a plain-language request, call \
         tasks.autotask.create with the request as the intent parameter."
    };
    lines.push(rules.to_string());
    lines.join("\n")
}
/// Shared keyword scorer: how many query tokens appear in the haystack.
pub(super) fn score_tokens(hay: &str, tokens: &[String]) -> i32 {
    tokens.iter().filter(|t| hay.contains(t.as_str())).count() as i32
}
pub(super) fn tokens_of(query: &str) -> Vec<String> {
    query
        .to_lowercase()
        .split_whitespace()
        .map(|t| t.to_string())
        .collect()
}
/// App registry search used by the `apps.find` command (covers all apps).
pub(super) fn find_apps(query: &str) -> Vec<Value> {
    let q = tokens_of(query);
    if q.is_empty() {
        return Vec::new();
    }
    let apps = registry::all_apps();
    let mut scored: Vec<(i32, &registry::AppDefinition)> = apps
        .iter()
        .filter_map(|app| {
            let hay = format!(
                "{} {} {} {}",
                app.title.to_lowercase(),
                app.id.to_lowercase(),
                app.description.to_lowercase(),
                app.keywords.to_lowercase()
            );
            let score = score_tokens(&hay, &q);
            if score > 0 {
                Some((score, app))
            } else {
                None
            }
        })
        .collect();
    scored.sort_by_key(|a| std::cmp::Reverse(a.0));
    scored
        .into_iter()
        .take(3)
        .map(|(_, app)| {
            json!({
                "id": app.id,
                "title": app.title,
                "description": app.description,
                "url": app.url,
            })
        })
        .collect()
}

#[cfg(test)]
mod prompt_surface_tests {
    use super::api_command_instructions;

    /// #2035 — the injected surface is the discovery contract plus ten named
    /// commands. Adding verbs to the catalogue must not grow the prompt.
    #[test]
    fn prompt_stays_small_regardless_of_command_count() {
        let prompt = api_command_instructions("user");
        assert!(prompt.len() < 6000, "prompt grew to {} bytes", prompt.len());
        assert!(prompt.contains("api.find"));
        assert!(prompt.contains("api.exec"));
        // Verbs that exist only in the catalogue must not be listed by name.
        assert!(!prompt.contains("notes.create"));
        assert!(!prompt.contains("database.rows.insert"));
        assert!(!prompt.contains("sheet.cells.set"));
    }

    /// #2372 — a curated command the deployment actually runs joins the
    /// shortlist, so "common commands" follows usage instead of a fixed list.
    #[test]
    fn usage_ranked_commands_reach_the_prompt() {
        crate::apps::record_command_usage("settings.read", Some("settings"), Some("Read settings"));
        let prompt = api_command_instructions("user");
        assert!(
            prompt.contains("settings.read"),
            "usage-ranked command missing from the prompt"
        );
    }

    /// #2372 — the ranking never proposes an admin-only command to a user.
    #[cfg(not(feature = "gblite"))]
    #[test]
    fn admin_only_usage_ranked_commands_are_role_filtered() {
        for _ in 0..10 {
            crate::apps::record_command_usage("database.rows.delete", Some("database"), None);
        }
        assert!(!api_command_instructions("user").contains("database.rows.delete"));
        assert!(api_command_instructions("admin").contains("database.rows.delete"));
    }

    /// #2372 — raw `METHOD /path` keys must not crowd curated commands out of
    /// a fixed pre-filter window. They are filtered by `command_by_name`
    /// before the shortlist cap is applied.
    #[test]
    fn raw_endpoint_usage_does_not_hide_curated_commands() {
        for index in 0..80 {
            let key = format!("GET /api/zz.rank.raw.{index}");
            for _ in 0..100 {
                crate::apps::record_command_usage(&key, None, None);
            }
        }
        crate::apps::record_command_usage("chat.resume", Some("chat"), None);
        assert!(
            api_command_instructions("user").contains("chat.resume"),
            "curated command hidden behind raw endpoint keys"
        );
    }
}
