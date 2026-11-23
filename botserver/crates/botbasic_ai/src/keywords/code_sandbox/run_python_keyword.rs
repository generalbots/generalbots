use super::*;

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
                file_path, user_clone.id
            );

            let state_for_task = Arc::clone(&state_clone);
            let session_id = user_clone.id;
            let bot_id = user_clone.bot_id;

            run_sandbox_blocking(move || async move {
                let sandbox = clamped_sandbox(&*state_for_task, bot_id, session_id);
                sandbox.execute_file(&file_path, CodeLanguage::Python).await
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
            let session_id = user.id;
            let bot_id = user.bot_id;

            run_sandbox_blocking(move || async move {
                let sandbox = clamped_sandbox(&*state_for_task, bot_id, session_id);
                sandbox
                    .execute_file(&file_path, CodeLanguage::JavaScript)
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
