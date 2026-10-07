//! Turning a `.bas` source into a `.ast` artifact, and resolving the work-dir
//! paths and bot/branch ids that compilation needs.
//!
//! Split out of `drive_compiler.rs` (450-line rule). The *decision* to compile
//! lives in `queue.rs`; this module is the *mechanism*.

use super::*;

impl DriveCompiler {
    /// Compilar arquivo .bas → .ast DIRETAMENTE em work/{bot}.gbai/{bot}.gbdialog/
    async fn compile_file(&self, _bot_id: Uuid, fp: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        // fp formats:
        // - {bot}.gbai/{bot}.gbdialog/{tool}.bas (full path with bucket prefix)
        // - {bot}.gbdialog/{tool}.bas (without bucket prefix)
        // - {bot}.gbkb/{doc}.txt (KB files - skip compilation)
        let parts: Vec<&str> = fp.split('/').collect();
        if parts.len() < 2 {
            return Err("Invalid file path format".into());
        }

    // Determine branch name, bot name, and work directory structure.
    // Structure: {org}.gborg/{branch}.gbai/{bot}.gbdialog/{tool}.ast
    // .gborg = organization/tenant, .gbai = branch, .gbdialog = bot
    // Branch and bot are separate: multiple bots can exist under one branch.
    let (branch_name, bot_name, work_dir) = if parts[0].ends_with(".gbai") {
        // Full path: {branch}.gbai/{bot}.gbdialog/{tool}.bas
        let branch_name = parts[0].strip_suffix(".gbai").unwrap_or(parts[0]);
        let bot_name = if parts.len() >= 2 {
            parts[1].strip_suffix(".gbdialog").unwrap_or(parts[1])
        } else {
            branch_name
        };
        let work_dir = self.work_root.join(format!("{branch_name}.gborg/{branch_name}.gbai/{bot_name}.gbdialog"));
        (branch_name.to_string(), bot_name.to_string(), work_dir)
    } else if parts.len() >= 2 && parts[0].ends_with(".gbdialog") {
        // Short path (legacy): {bot}.gbdialog/{tool}.bas
        let bot_name = parts[0].strip_suffix(".gbdialog").unwrap_or(parts[0]);
        let work_dir = self.work_root.join(format!("{bot_name}.gborg/{bot_name}.gbai/{bot_name}.gbdialog"));
        (bot_name.to_string(), bot_name.to_string(), work_dir)
    } else if parts.len() >= 2 && parts[0].ends_with(".gbkb") {
        // KB file: {bot}.gbkb/{doc}.txt - skip compilation
        debug!("Skipping KB file: {}", fp);
        return Ok(());
    } else {
        warn!("Unknown file path format: {}", fp);
        return Err("Invalid file path format".into());
    };

    // Look up the real bot_id from the database using the bot name
    let real_bot_id = Self::resolve_bot_id(&self.state, &bot_name);

    // Create work directory
    std::fs::create_dir_all(&work_dir)?;

    // Determine tool name from last part of path
    let tool_name = parts.last().unwrap_or(&"unknown").strip_suffix(".bas").unwrap_or(parts.last().unwrap_or(&"unknown"));

        // Caminho do .bas no work
        let work_bas_path = work_dir.join(format!("{}.bas", tool_name));

        // Reform #1501/#1502 — a git-owned bot's `.gbdialog` lives in its
        // repository: the git monitor materializes the committed sources into
        // this work dir and queues the file for compile. Downloading the Drive
        // object here overwrote that materialization with a stale copy — the
        // split brain that froze a media-filing bot for hours — so the
        // materialized work copy is compiled as-is.
        let git_owned = {
            let mut conn = self.state.conn.get()?;
            git_owned_bots(&mut conn).contains(&(branch_name.clone(), bot_name.clone()))
        };
        let use_work_copy = git_owned && work_bas_path.exists();

        info!("Downloading {} from S3 to work dir", fp);
        let download_result = if use_work_copy {
            info!(
                "git-owned bot '{bot_name}': compiling the source materialized from git, Drive copy bypassed"
            );
            Err(format!("git-owned bot '{bot_name}': Drive copy bypassed").into())
        } else {
            download_from_s3(fp, &self.state).await
        };
        
        match download_result {
            Ok(content) => {
                if let Err(e) = std::fs::write(&work_bas_path, content) {
                    warn!("Failed to write {} to work dir: {}", work_bas_path.display(), e);
                    return Err(format!("Failed to write file: {}", e).into());
                }
                info!("Downloaded {} to {}", fp, work_bas_path.display());
                // #1279 — the object is back; clear the suppression so a
                // future failure logs normally again.
                Self::clear_missing_suppression(fp);
            }
            Err(e) => {
                // #1279 — a missing bucket/object repeats on every drive scan
                // (NoSuchBucket): logging the full S3 error body each time
                // flooded prod logs with thousands of duplicate blocks. The
                // error text is compacted and, once a key is known-missing,
                // demoted to debug so it logs at most once per hour (the
                // entry is dropped when the object reappears via Ok).
                let error_text = e.to_string();
                let missing = error_text.contains("NoSuchBucket") || error_text.contains("NoSuchKey");
                if missing && Self::suppress_missing_log(fp) {
                    debug!("S3 object still missing (suppressed): {}", fp);
                } else if missing {
                    warn!("S3 object missing: {} (compacted; details suppressed until it reappears)", fp);
                } else if use_work_copy {
                    debug!("Git-owned source compiled from the work copy: {}", fp);
                } else {
                    info!("No Drive copy of {} ({}); using the work copy", fp, e);
                }
                if !work_bas_path.exists() {
                    // #1264 — a permanently-absent object with no work copy used
                    // to return Err every scan, so the monitor retried the same
                    // file 2×/second and burned a full core on a hopeless loop
                    // (signup capacity gate saw the resulting CPU and vetoed
                    // every free signup). Suppress like NoSuchBucket: log once,
                    // then debug until the object reappears, and return Ok so
                    // the scanner moves on without spinning.
                if missing || Self::suppress_missing_log(fp) {
                    debug!("S3 object absent, compile skipped (suppressed): {}", fp);
                } else {
                    warn!("S3 object absent, compile skipped: {} (suppressed until it reappears)", fp);
                }
                self.mark_missing(fp).await;
                return Ok(());
                }
                info!("Failed to download {} from S3, using existing work copy", fp);
            }
        }

        // Verify file exists now
        if !work_bas_path.exists() {
            info!("File {} still not found after download attempt", work_bas_path.display());
            return Ok(());
        }

        // Ler conteúdo
        let _content = std::fs::read_to_string(&work_bas_path)?;

        // Compilar com BasicCompiler (já está no work dir, então compila in-place)
        let mut callbacks = CompilerCallbacks::new();
        #[cfg(feature = "tasks")]
        {
            let schedule_fn = crate::basic::keywords::set_schedule::execute_set_schedule;
            callbacks.execute_set_schedule = Some(Box::new(move |conn, cron, script, bot_id| {
                schedule_fn(conn, cron, script, bot_id)
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            }));
        }
        // ON UPDATE OF callback is always available (not behind tasks feature)
        callbacks.execute_on_update = Some(Box::new(|conn, table_name, script_name, bot_id, kind| {
            crate::basic::keywords::on_update::execute_on_update_registration(conn, table_name, script_name, bot_id, kind)
                .map(|_| ())
                .map_err(|e| e.to_string())
        }));
        // `ON EVENT "<event>"` in a tool: the tool subscribes itself to a
        // channel event at design time. Registered here (not at run time)
        // because the tool only ever runs *because* of the event.
        callbacks.execute_on_event = Some(Box::new(|conn, event, script, bot_id| {
            botcore::shared::basic_events::register_handler_on(
                conn,
                bot_id,
                event,
                script,
            )
            .map_err(|e| e.to_string())
        }));
        callbacks.execute_webhook = Some(Box::new(|conn, endpoint, script, bot_id| {
            crate::basic::keywords::webhook::execute_webhook_registration(conn, endpoint, script, bot_id)
                .map(|_| ())
                .map_err(|e| e.to_string())
        }));
        callbacks.execute_use_website = Some(Box::new(|conn, url, bot_id, refresh| {
            crate::basic::keywords::use_website::execute_use_website_preprocessing_with_refresh(conn, url, bot_id, refresh)
                .map(|_| ())
                .map_err(|e| e.to_string())
        }));
        callbacks.process_table_definitions = Some(Box::new(|runtime, bot_id, content| {
            crate::basic::keywords::table_definition::process_table_definitions(runtime, bot_id, content)
                .map(|_| ())
                .map_err(|e| e.to_string())
        }));
        callbacks.create_runtime = Some(Box::new(|state| {
            Arc::new(crate::basic::AppStateBasicRuntime(state))
        }));
        let mut compiler = BasicCompiler::with_callbacks(self.state.clone(), real_bot_id, callbacks);
        compiler.compile_file(
            work_bas_path.to_str().ok_or("Invalid path")?,
            work_dir.to_str().ok_or("Invalid path")?
        )?;

        // A tool's MCP manifest in Drive is the source of truth for its
        // argument schema. compile_file() always regenerates a manifest from
        // the .bas (with an empty schema when the script declares no
        // parameters), which clobbers the richer manifest persisted by
        // AutoTask shipped templates; tool_exec relies on
        // input_schema.properties to default arguments the LLM omitted
        // (otherwise optional params reach the script as undeclared
        // variables and abort it). Sync the Drive copy over the generated
        // one when it exists; missing manifests keep the generated fallback.
        let manifest_fp = match fp.strip_suffix(".bas") {
            Some(base) => format!("{base}.mcp.json"),
            None => fp.to_string(),
        };
        let work_manifest_path = work_dir.join(format!("{}.mcp.json", tool_name));
        // The repository's manifest is materialized next to the source, so a
        // git-owned bot keeps it (same reason as the source above).
        let manifest_sync = if use_work_copy && work_manifest_path.exists() {
            debug!("git-owned bot '{bot_name}': keeping the repository manifest for {manifest_fp}");
            Err(format!("git-owned bot '{bot_name}': Drive manifest bypassed").into())
        } else {
            download_from_s3(&manifest_fp, &self.state).await
        };
        match manifest_sync {
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(text) => {
                    if let Err(e) = std::fs::write(&work_manifest_path, text) {
                        warn!("Failed to write MCP manifest to work dir: {e}");
                    } else {
                        info!("Synced MCP manifest from Drive: {manifest_fp}");
                    }
                }
                Err(e) => warn!("MCP manifest from Drive is not UTF-8, keeping generated one: {e}"),
            },
            Err(_) => debug!("No MCP manifest in Drive for {manifest_fp}; keeping generated one"),
        }

        let work_ast_path = work_dir.join(format!("{}.ast", tool_name));
        let ast_path_str = work_ast_path.to_str().unwrap_or("").to_string();

        let branch_id = Self::resolve_branch_id(&self.state, &branch_name);

        let bot_id_str = real_bot_id.to_string();
        let branch_id_str = branch_id.to_string();
        let upsert_sql = diesel::sql_query(
            "INSERT INTO basic_tools (bot_id, tool_name, file_path, ast_path, compiled_at, is_active, branch_id) \
             VALUES ($1::uuid, $2, $3, $4, $5, true, $6::uuid) \
             ON CONFLICT (bot_id, tool_name) DO UPDATE SET \
             file_path = EXCLUDED.file_path, ast_path = EXCLUDED.ast_path, \
             compiled_at = EXCLUDED.compiled_at, is_active = true"
        )
        .bind::<diesel::sql_types::Text, _>(&bot_id_str)
        .bind::<diesel::sql_types::Text, _>(tool_name)
        .bind::<diesel::sql_types::Text, _>(fp)
        .bind::<diesel::sql_types::Text, _>(&ast_path_str)
        .bind::<diesel::sql_types::Timestamptz, _>(chrono::Utc::now())
        .bind::<diesel::sql_types::Text, _>(&branch_id_str);
        match upsert_sql.execute(&mut *self.state.conn.get()?)
        {
            Ok(_) => info!("Registered tool '{}' in database", tool_name),
            Err(e) => warn!("Failed to register tool '{}' in database: {}", tool_name, e),
        }

        info!("Compiled {} to {}.ast", fp, tool_name);
        Ok(())
    }

    /// Resolve the expected .ast path for a given file path, to check if it exists.
    /// Returns PathBuf without verifying existence — caller checks .exists().
    fn resolve_ast_path(&self, fp: &str) -> PathBuf {
        let parts: Vec<&str> = fp.split('/').collect();
        if parts.len() < 2 || parts.iter().any(|p| p.ends_with(".gbkb")) {
            return PathBuf::new();
        }

        let (branch_name, bot_name) = if parts[0].ends_with(".gbai") {
            let branch = parts[0].strip_suffix(".gbai").unwrap_or(parts[0]);
            let bot = if parts.len() >= 2 {
                parts[1].strip_suffix(".gbdialog").unwrap_or(parts[1])
            } else {
                branch
            };
            (branch.to_string(), bot.to_string())
        } else if parts.len() >= 2 && parts[0].ends_with(".gbdialog") {
            let bot = parts[0].strip_suffix(".gbdialog").unwrap_or(parts[0]);
            (bot.to_string(), bot.to_string())
        } else {
            return PathBuf::new();
        };

        let tool_name = parts.last()
            .unwrap_or(&"unknown")
            .strip_suffix(".bas")
            .unwrap_or(parts.last().unwrap_or(&"unknown"));
        let work_dir = self.work_root.join(format!("{branch_name}.gborg/{branch_name}.gbai/{bot_name}.gbdialog"));
        work_dir.join(format!("{}.ast", tool_name))
    }

    /// Resolve the branch UUID from the branch slug/name using the database.
    /// Falls back to Uuid::nil() if the branch is not found.
    fn resolve_branch_id(state: &Arc<AppState>, branch_name: &str) -> Uuid {
        use botcore::shared::models::schema::branches::dsl::*;

        let mut conn = match state.conn.get() {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to get DB connection for branch name lookup: {}", e);
                return Uuid::nil();
            }
        };

        match branches
            .filter(slug.eq(branch_name))
            .select(id)
            .first::<Uuid>(&mut *conn)
        {
            Ok(branch_id) => branch_id,
            Err(e) => {
                warn!("Branch '{}' not found in database ({}), using nil UUID", branch_name, e);
                Uuid::nil()
            }
        }
    }

    /// Resolve the real bot_id from the bot name using the database.
    /// Falls back to Uuid::nil() if the bot is not found (backward compatibility).
    fn resolve_bot_id(state: &Arc<AppState>, bot_name: &str) -> Uuid {
        use botcore::shared::models::schema::bots::dsl::*;

        let mut conn = match state.conn.get() {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to get DB connection for bot name lookup: {}", e);
                return Uuid::nil();
            }
        };

        match bots
            .filter(name.eq(bot_name))
            .select(id)
            .first::<Uuid>(&mut *conn)
        {
            Ok(bot_id) => bot_id,
            Err(e) => {
                warn!("Bot '{}' not found in database ({}), using nil UUID", bot_name, e);
                Uuid::nil()
            }
        }
    }
}
