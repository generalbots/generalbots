use super::*;
use diesel::prelude::*;

/// Hard upper bound for a synchronous Rhai `RUN …` call. The worker thread is
/// bounded by this deadline too, so a hung child can never pin it forever.
const CALLER_DEADLINE: Duration = Duration::from_secs(60);

/// Headroom between the sandbox's own (group-killing) deadline and the caller's
/// deadline. The inner deadline must fire first so the whole process group is
/// killed, not just the direct child via `kill_on_drop`.
const CALLER_MARGIN_SECS: u64 = 5;

/// Pure timeout decision: keep the configured sandbox deadline strictly below
/// the caller's hard deadline. Unit-tested offline (#2660).
fn effective_sandbox_timeout(configured: u64, hard_limit_secs: u64, margin_secs: u64) -> u64 {
    configured.min(hard_limit_secs.saturating_sub(margin_secs))
}

/// Build a sandbox whose internal deadline is guaranteed to fire before the
/// caller gives up, so the child's process group is always killed (#2660).
fn clamped_sandbox(state: &dyn BasicRuntime, bot_id: Uuid, session_id: Uuid) -> CodeSandbox {
    let mut config = SandboxConfig::from_bot_config(state, bot_id);
    config.timeout_seconds = effective_sandbox_timeout(
        config.timeout_seconds,
        CALLER_DEADLINE.as_secs(),
        CALLER_MARGIN_SECS,
    );
    CodeSandbox::new(config, session_id)
}

/// #2659 — the only filesystem subtree a BASIC script may read through
/// `WITH FILE` is its own bot's materialized work tree. Both the current
/// org-scoped layout and the legacy bot-named layout are accepted so an
/// operator-configured path can never pull in another tenant's files.
fn file_work_roots(
    state: &dyn BasicRuntime,
    user: &UserSession,
) -> Result<Vec<std::path::PathBuf>, String> {
    use botschema::bots;

    let mut conn = state
        .db_pool()
        .get()
        .map_err(|e| format!("DB error: {e}"))?;
    let bot_name: String = bots::table
        .filter(bots::id.eq(user.bot_id))
        .select(bots::name)
        .first(&mut *conn)
        .map_err(|e| format!("Failed to resolve bot name: {e}"))?;
    if bot_name.is_empty()
        || bot_name.starts_with('.')
        || bot_name.contains('/')
        || bot_name.contains('\\')
        || bot_name.contains("..")
    {
        return Err("unsafe bot name for sandbox file access".to_string());
    }

    let work_root = std::path::PathBuf::from(state.data_dir());
    let org = user.org_id().unwrap_or_else(uuid::Uuid::nil);
    let current = work_root
        .join(format!("{org}.gborg"))
        .join(format!("{bot_name}.gbai"));
    if current.is_dir() {
        return Ok(vec![current]);
    }
    let legacy = work_root
        .join(format!("{bot_name}.gborg"))
        .join(format!("{bot_name}.gbai"));
    Ok(vec![legacy, current])
}

/// A `WITH FILE` sandbox also carries the bot work tree in its allowlist, so
/// the same roots are visible to a container runtime and to the path guard.
fn file_sandbox(
    state: &dyn BasicRuntime,
    user: &UserSession,
    work_roots: &[std::path::PathBuf],
) -> CodeSandbox {
    let mut config = SandboxConfig::from_bot_config(state, user.bot_id);
    config.timeout_seconds = effective_sandbox_timeout(
        config.timeout_seconds,
        CALLER_DEADLINE.as_secs(),
        CALLER_MARGIN_SECS,
    );
    for root in work_roots {
        let value = root.to_string_lossy().to_string();
        if !config.allowed_paths.contains(&value) {
            config.allowed_paths.push(value);
        }
    }
    CodeSandbox::new(config, user.id)
}

/// Bridge the synchronous Rhai engine to the async sandbox.
///
/// Rhai custom syntax is synchronous and runs inside a tokio runtime, so a
/// fresh thread must own the bridge runtime. Unlike the old pattern, the
/// worker is *not* left running after the caller times out: it wraps the
/// sandbox future in its own `tokio::time::timeout`, and every sandbox timeout
/// now kills and reaps the child (and its process group) via
/// `SafeCommand::execute_async_with_timeout`. The outer `recv_timeout` only
/// guards against a non-cooperative config/runtime setup; it is set above the
/// inner deadline so it normally never wins the race.
fn run_sandbox_blocking<F, Fut>(make_future: F) -> Result<Dynamic, Box<rhai::EvalAltResult>>
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ExecutionResult> + Send + 'static,
{
    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let runtime = match tokio::runtime::Runtime::new() {
            Ok(runtime) => runtime,
            Err(e) => {
                let _ = tx.send(ExecutionResult::error(&format!(
                    "Failed to create runtime: {e}"
                )));
                return;
            }
        };
        let result = runtime.block_on(async move {
            match tokio::time::timeout(CALLER_DEADLINE, make_future()).await {
                Ok(result) => result,
                Err(_) => ExecutionResult::timeout(),
            }
        });
        let _ = tx.send(result);
    });

    match rx.recv_timeout(CALLER_DEADLINE + Duration::from_secs(CALLER_MARGIN_SECS)) {
        Ok(result) => Ok(Dynamic::from(result.output())),
        Err(_) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
            "Sandbox execution timed out".into(),
            rhai::Position::NONE,
        ))),
    }
}

pub fn run_python_keyword(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user;

    if let Err(e) = engine.register_custom_syntax(
        ["RUN", "PYTHON", "$expr$"],
        false,
        move |context, inputs| {
            let code = context
                .eval_expression_tree(&inputs[0])?
                .to_string()
                .trim_matches('"')
                .to_string();

            trace!("RUN PYTHON for session: {}", user_clone.id);

            let state_for_task = Arc::clone(&state_clone);
            let session_id = user_clone.id;
            let bot_id = user_clone.bot_id;

            run_sandbox_blocking(move || async move {
                let sandbox = clamped_sandbox(&*state_for_task, bot_id, session_id);
                sandbox.execute(&code, CodeLanguage::Python).await
            })
        },
    ) {
        log::error!("Failed to register RUN PYTHON syntax failed: {e}");
    }
}

pub fn run_javascript_keyword(
    state: Arc<dyn BasicRuntime>,
    user: UserSession,
    engine: &mut Engine,
) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    if let Err(e) = engine.register_custom_syntax(
        ["RUN", "JAVASCRIPT", "$expr$"],
        false,
        move |context, inputs| {
            let code = context
                .eval_expression_tree(&inputs[0])?
                .to_string()
                .trim_matches('"')
                .to_string();

            trace!("RUN JAVASCRIPT for session: {}", user_clone.id);

            let state_for_task = Arc::clone(&state_clone);
            let session_id = user_clone.id;
            let bot_id = user_clone.bot_id;

            run_sandbox_blocking(move || async move {
                let sandbox = clamped_sandbox(&*state_for_task, bot_id, session_id);
                sandbox.execute(&code, CodeLanguage::JavaScript).await
            })
        },
    ) {
        log::error!("Failed to register RUN JAVASCRIPT syntax failed: {e}");
    }

    if let Err(e) =
        engine.register_custom_syntax(["RUN", "JS", "$expr$"], false, move |context, inputs| {
            let code = context
                .eval_expression_tree(&inputs[0])?
                .to_string()
                .trim_matches('"')
                .to_string();

            let state_for_task = Arc::clone(&state);
            let session_id = user.id;
            let bot_id = user.bot_id;

            run_sandbox_blocking(move || async move {
                let sandbox = clamped_sandbox(&*state_for_task, bot_id, session_id);
                sandbox.execute(&code, CodeLanguage::JavaScript).await
            })
        })
    {
        log::error!("Failed to register RUN JS syntax failed: {e}");
    }
}

pub fn run_bash_keyword(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    if let Err(e) =
        engine.register_custom_syntax(["RUN", "BASH", "$expr$"], false, move |context, inputs| {
            let code = context
                .eval_expression_tree(&inputs[0])?
                .to_string()
                .trim_matches('"')
                .to_string();

            trace!("RUN BASH for session: {}", user.id);

            let state_for_task = Arc::clone(&state);
            let session_id = user.id;
            let bot_id = user.bot_id;

            run_sandbox_blocking(move || async move {
                let sandbox = clamped_sandbox(&*state_for_task, bot_id, session_id);
                sandbox.execute(&code, CodeLanguage::Bash).await
            })
        })
    {
        log::error!("Failed to register RUN BASH syntax failed: {e}");
    }
}

pub fn run_file_keyword(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    if let Err(e) = engine.register_custom_syntax(
        ["RUN", "PYTHON", "WITH", "FILE", "$expr$"],
        false,
        move |context, inputs| {
            let file_path = context
                .eval_expression_tree(&inputs[0])?
                .to_string()
                .trim_matches('"')
                .to_string();

            trace!(
                "RUN PYTHON WITH FILE {} for session: {}",
                file_path,
                user_clone.id
            );

            let state_for_task = Arc::clone(&state_clone);
            let user_for_task = user_clone.clone();
            let work_roots = match file_work_roots(&*state_for_task, &user_for_task) {
                Ok(roots) => roots,
                Err(error) => {
                    warn!("Refusing sandbox file access: {error}");
                    return Ok(Dynamic::from(format!(
                        "Error: Sandbox file access refused: {error}"
                    )));
                }
            };

            run_sandbox_blocking(move || async move {
                let sandbox = file_sandbox(&*state_for_task, &user_for_task, &work_roots);
                sandbox
                    .execute_file(&file_path, CodeLanguage::Python, &work_roots)
                    .await
            })
        },
    ) {
        log::error!("Failed to register RUN PYTHON WITH FILE syntax failed: {e}");
    }

    if let Err(e) = engine.register_custom_syntax(
        ["RUN", "JAVASCRIPT", "WITH", "FILE", "$expr$"],
        false,
        move |context, inputs| {
            let file_path = context
                .eval_expression_tree(&inputs[0])?
                .to_string()
                .trim_matches('"')
                .to_string();

            let state_for_task = Arc::clone(&state);
            let user_for_task = user.clone();
            let work_roots = match file_work_roots(&*state_for_task, &user_for_task) {
                Ok(roots) => roots,
                Err(error) => {
                    warn!("Refusing sandbox file access: {error}");
                    return Ok(Dynamic::from(format!(
                        "Error: Sandbox file access refused: {error}"
                    )));
                }
            };

            run_sandbox_blocking(move || async move {
                let sandbox = file_sandbox(&*state_for_task, &user_for_task, &work_roots);
                sandbox
                    .execute_file(&file_path, CodeLanguage::JavaScript, &work_roots)
                    .await
            })
        },
    ) {
        log::error!("Failed to register RUN JAVASCRIPT WITH FILE syntax failed: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sandbox_deadline_stays_below_the_callers_hard_deadline() {
        let hard = CALLER_DEADLINE.as_secs();
        // Configured values larger than the budget are clamped with headroom.
        assert_eq!(
            effective_sandbox_timeout(600, hard, CALLER_MARGIN_SECS),
            hard - CALLER_MARGIN_SECS
        );
        // Smaller configured values are honoured unchanged.
        assert_eq!(effective_sandbox_timeout(3, hard, CALLER_MARGIN_SECS), 3);
        // A misconfigured zero margin still never exceeds the caller deadline.
        assert_eq!(effective_sandbox_timeout(600, hard, 0), hard);
        assert_eq!(effective_sandbox_timeout(600, 2, 5), 0);
    }
}
