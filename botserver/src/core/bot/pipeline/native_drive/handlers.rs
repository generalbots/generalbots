use super::*;

pub(super) async fn run_drive_fetch(identity: &SessionIdentity<'_>, args: &str) {
    let parsed = parse(args);
    let reference = parsed
        .get("path")
        .or_else(|| parsed.get("name"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    let SessionIdentity {
        state,
        session_id,
        user_id,
        bot_uuid,
        ..
    } = *identity;
    if reference.is_empty() {
        send(identity, "Tell me which Drive file to fetch (`path`).").await;
        return;
    }
    if !identity.agent_vm_enabled {
        send(identity, "Agent/VM is disabled for this conversation").await;
        return;
    }
    agent_vm_hook::ensure_agent_vm(session_id, user_id, bot_uuid).await;

    let scope = match drive_bridge::scope(state, &bot_uuid, user_id) {
        Some(scope) => scope,
        None => {
            send(identity, "Drive is not available for this workspace.").await;
            return;
        }
    };
    let found = drive_bridge::find_object(state, &scope, &reference).await;
    let Some((key, relative)) = found else {
        send(
            identity,
            &format!("I could not find `{reference}` in your Drive."),
        )
        .await;
        return;
    };
    let Some(bytes) = drive_bridge::read_object(state, &scope, &key).await else {
        send(
            identity,
            &format!("Could not download `{relative}` from Drive."),
        )
        .await;
        return;
    };
    if bytes.len() as u64 > drive_bridge::MAX_FETCH_BYTES {
        send(
            identity,
            &format!(
                "`{relative}` is {} and exceeds the 100 MB transfer limit.",
                human_size(bytes.len() as u64)
            ),
        )
        .await;
        return;
    }
    let name = parsed
        .get("as")
        .and_then(|v| v.as_str())
        .map(sanitize_name)
        .unwrap_or_else(|| sanitize_name(&relative));
    let target = format!("data/{name}");
    let written = agent_vm_hook::write_session_file(user_id, session_id, &target, &bytes).await;
    let message = match written {
        Ok(size) => {
            let listing = agent_vm_hook::list_session_folder(user_id, session_id, "data")
                .await
                .unwrap_or_else(Vec::new);
            let names: Vec<String> = listing
                .iter()
                .map(|(name, size)| format!("{name} ({})", human_size(*size)))
                .collect();
            format!(
                "Fetched `{relative}` from Drive into `{target}` ({}).\n\n**data/** {}",
                human_size(size as u64),
                names.join(", ")
            )
        }
        Err(e) => format!("Could not copy `{relative}` into the workspace: {e}"),
    };
    send(identity, &message).await;
    persist(state, session_id, user_id, &message).await;
}

const DRIVE_LIST_DEFAULT_LIMIT: usize = 20;
const DRIVE_LIST_MAX_LIMIT: usize = 100;
const DRIVE_LIST_SEARCH_SCAN_LIMIT: usize = 200;

/// #2164 — lists files in the bot's `.gbdrive` so the model can answer
/// "open my last document about X" without the user naming a file.
pub(super) async fn run_drive_list(identity: &SessionIdentity<'_>, args: &str) {
    let parsed = parse(args);
    let limit = parsed
        .get("limit")
        .and_then(|v| v.as_u64())
        .map(|n| n as usize)
        .unwrap_or(DRIVE_LIST_DEFAULT_LIMIT)
        .clamp(1, DRIVE_LIST_MAX_LIMIT);
    let query = parsed
        .get("query")
        .or_else(|| parsed.get("q"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    let SessionIdentity {
        state,
        session_id,
        user_id,
        bot_uuid,
        ..
    } = *identity;
    let Some(scope) = drive_bridge::scope(state, &bot_uuid, user_id) else {
        send(identity, "Drive is not available for this workspace.").await;
        return;
    };
    let scan_limit = if query.is_some() {
        DRIVE_LIST_SEARCH_SCAN_LIMIT
    } else {
        limit
    };
    let Some(items) = drive_bridge::list_bot_drive_objects(state, &scope, scan_limit).await else {
        send(identity, "Could not list files in your bot Drive.").await;
        return;
    };
    let filtered: Vec<_> = if let Some(ref q) = query {
        items
            .into_iter()
            .filter(|item| matches_drive_query(item, q))
            .take(limit)
            .collect()
    } else {
        items.into_iter().take(limit).collect()
    };
    if filtered.is_empty() {
        let message = if let Some(ref q) = query {
            format!("No files matching \"{q}\" were found in the bot Drive (`.gbdrive`).")
        } else {
            "No files found in the bot Drive (`.gbdrive`).".to_string()
        };
        send(identity, &message).await;
        persist(state, session_id, user_id, &message).await;
        return;
    }
    let heading = if let Some(ref q) = query {
        format!("Bot Drive files matching \"{q}\" (newest first, up to {limit}):")
    } else {
        format!("Bot Drive files (newest first, up to {limit}):")
    };
    let mut lines = vec![heading];
    for item in &filtered {
        let name = sanitize_name(&item.relative);
        let entry = if is_document_file(&item.relative) {
            format!(
                "- {} · [Open in Drive]({}) ({})",
                docs_link(&name, &scope.bucket, &item.key),
                drive_href(&scope.bucket, &item.key),
                human_size(item.size)
            )
        } else {
            format!(
                "- {} ({})",
                drive_link(&name, &scope.bucket, &item.key),
                human_size(item.size)
            )
        };
        lines.push(entry);
    }
    if query.is_some() {
        lines.push(String::new());
        lines.push(
            "To open a document in the editor, use its Docs link (`app://docs?file=<key>&bucket=<bucket>`)."
                .to_string(),
        );
    }
    let message = lines.join("\n");
    send(identity, &message).await;
    persist(state, session_id, user_id, &message).await;
}

pub(super) async fn run_drive_save(identity: &SessionIdentity<'_>, args: &str) {
    let parsed = parse(args);
    let SessionIdentity {
        state,
        session_id,
        user_id,
        bot_uuid,
        ..
    } = *identity;
    // #2239 — workspace output access must be refused when Agent/VM is off.
    if !identity.agent_vm_enabled {
        send(identity, "Agent/VM is disabled for this conversation").await;
        return;
    }
    let requested: Vec<String> = parsed
        .get("files")
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|v| v.as_str().map(sanitize_name))
                .collect()
        })
        .unwrap_or(Default::default());
    let folder = parsed
        .get("folder")
        .and_then(|v| v.as_str())
        .map(drive_bridge::slug)
        .filter(|f| !f.is_empty())
        .unwrap_or_else(|| String::new());

    let files = agent_vm_hook::list_session_folder(user_id, session_id, "output")
        .await
        .unwrap_or_else(Vec::new);
    if files.is_empty() {
        send(
            identity,
            "There are no generated files in `output/` to save.",
        )
        .await;
        return;
    }
    let selected: Vec<(String, u64)> = files
        .into_iter()
        .filter(|(name, _)| requested.is_empty() || requested.contains(name))
        .collect();
    if selected.is_empty() {
        send(identity, "None of the named files exist in `output/`.").await;
        return;
    }
    let folder = if folder.is_empty() {
        drive_bridge::conversation_title(state, session_id).await
    } else {
        folder
    };

    let mut saved = Vec::new();
    let mut failed = Vec::new();
    let mut total: u64 = 0;
    for (name, _) in &selected {
        match agent_vm_hook::read_session_file(user_id, session_id, &format!("output/{name}")).await
        {
            Ok(bytes) => {
                total += bytes.len() as u64;
                if total > botagent::transfer::MAX_TURN_BYTES as u64 {
                    failed.push(format!("{name} (turn limit)"));
                    continue;
                }
                match drive_bridge::save_object(state, &bot_uuid, &folder, name, bytes).await {
                    Some(path) => {
                        let (bucket, key) = split_bucket_path(&path);
                        saved.push(drive_link(name, bucket, key));
                    }
                    None => failed.push(name.to_string()),
                }
            }
            Err(e) => failed.push(format!("{name} ({e})")),
        }
    }
    let mut message = String::new();
    if !saved.is_empty() {
        message.push_str(&format!(
            "Saved to Drive under `conversations/{folder}/` ({}):\n",
            human_size(total)
        ));
        for item in &saved {
            message.push_str(&format!("- {item}\n"));
        }
    }
    if !failed.is_empty() {
        message.push_str(&format!("\nNot saved: {}", failed.join(", ")));
    }
    if message.is_empty() {
        message.push_str("Nothing was saved to Drive.");
    }
    send(identity, &message).await;
    persist(state, session_id, user_id, &message).await;
}
