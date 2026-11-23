use super::*;

impl std::fmt::Debug for CodeSandbox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CodeSandbox")
            .field("config", &self.config)
            .field("session_id", &self.session_id)
            .finish()
    }
}
impl CodeSandbox {
    pub fn new(config: SandboxConfig, session_id: Uuid) -> Self {
        Self { config, session_id }
    }

    pub async fn execute(&self, code: &str, language: CodeLanguage) -> ExecutionResult {
        let allow_unisolated = std::env::var(UNISOLATED_PROCESS_OVERRIDE_ENV)
            .is_ok_and(|value| value == "1" || value.eq_ignore_ascii_case("true"));

        let selection = match select_runtime(&self.config, allow_unisolated) {
            Ok(selection) => selection,
            Err(e) => {
                warn!(
                    "Refusing sandbox execution for session {}: {}",
                    self.session_id, e
                );
                return ExecutionResult::error(&format!("Sandbox execution refused: {e}"));
            }
        };

        if selection.runtime == SandboxRuntime::Process {
            warn!(
                "UNSAFE: session {} is running untrusted code directly on the botserver host via the 'process' runtime ({} enabled)",
                self.session_id, UNISOLATED_PROCESS_OVERRIDE_ENV
            );
        }

        let start_time = std::time::Instant::now();

        let result = match selection.runtime {
            SandboxRuntime::LXC => self.execute_lxc(code, &language).await,
            SandboxRuntime::Docker => self.execute_docker(code, &language, &selection).await,
            SandboxRuntime::Firecracker => self.execute_firecracker().await,
            SandboxRuntime::Process => self.execute_process(code, &language).await,
            SandboxRuntime::Unknown => Err("Unknown sandbox runtime".to_string()),
        };

        let execution_time_ms = start_time.elapsed().as_millis() as u64;

        match result {
            Ok(mut exec_result) => {
                exec_result.execution_time_ms = execution_time_ms;
                exec_result
            }
            Err(e) => {
                let mut err_result = ExecutionResult::error(&e);
                err_result.execution_time_ms = execution_time_ms;
                err_result
            }
        }
    }

    async fn execute_lxc(
        &self,
        code: &str,
        language: &CodeLanguage,
    ) -> Result<ExecutionResult, String> {
        let container_name = format!("gb-sandbox-{}", Uuid::new_v4());

        std::fs::create_dir_all(&self.config.work_dir)
            .map_err(|e| format!("Failed to create work dir: {}", e))?;

        let code_file = format!(
            "{}/{}.{}",
            self.config.work_dir,
            self.session_id,
            language.file_extension()
        );
        std::fs::write(&code_file, code)
            .map_err(|e| format!("Failed to write code file: {}", e))?;

        let lxc_conf = format!("/etc/lxc/{}.conf", language.lxc_image());
        let interpreter = language.interpreter();

        let timeout_duration = Duration::from_secs(self.config.timeout_seconds);
        let cmd = match SafeCommand::new("lxc-execute")
            .and_then(|c| c.arg("-n"))
            .and_then(|c| c.arg(&container_name))
            .and_then(|c| c.arg("-f"))
            .and_then(|c| c.arg(&lxc_conf))
            .and_then(|c| c.arg("--"))
            .and_then(|c| c.arg(interpreter))
            .and_then(|c| c.arg(&code_file))
        {
            Ok(cmd) => cmd,
            Err(e) => return Err(format!("LXC execution failed: {e}")),
        };
        // The deadline kills the child's whole process group before returning
        // (#2660) — a bare `tokio::time::timeout` around a synthetic async call
        // could never fire, and never reaped the child.
        let outcome = cmd.execute_async_with_timeout(timeout_duration).await;

        let _ = std::fs::remove_file(&code_file);

        match outcome {
            Ok(Some(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let exit_code = output.status.code().unwrap_or(-1);

                Ok(ExecutionResult {
                    stdout,
                    stderr,
                    exit_code,
                    execution_time_ms: 0,
                    timed_out: false,
                    killed: !output.status.success(),
                    error: None,
                })
            }
            Ok(None) => Ok(ExecutionResult::timeout()),
            Err(e) => Err(format!("LXC execution failed: {e}")),
        }
    }

    async fn execute_docker(
        &self,
        code: &str,
        language: &CodeLanguage,
        policy: &RuntimeSelection,
    ) -> Result<ExecutionResult, String> {
        let image = match language {
            CodeLanguage::Python => "python:3.11-slim",
            CodeLanguage::JavaScript => "node:20-slim",
            CodeLanguage::Bash => "alpine:latest",
        };

        let mut args = vec![
            "run".to_string(),
            "--rm".to_string(),
            "--network".to_string(),
            if policy.network_enabled {
                "bridge"
            } else {
                "none"
            }
            .to_string(),
            "--memory".to_string(),
            format!("{}m", policy.memory_limit_mb),
            "--cpus".to_string(),
            format!("{:.2}", f64::from(policy.cpu_limit_percent) / 100.0),
            "--read-only".to_string(),
            "--security-opt".to_string(),
            "no-new-privileges".to_string(),
        ];
        // Only configured allowed paths are visible, and only read-only.
        for path in &policy.allowed_paths {
            args.push("--volume".to_string());
            args.push(format!("{path}:{path}:ro"));
        }
        args.extend([
            image.to_string(),
            language.interpreter().to_string(),
            "-c".to_string(),
            code.to_string(),
        ]);

        let timeout_duration = Duration::from_secs(self.config.timeout_seconds);
        let cmd = match SafeCommand::new("docker")
            .and_then(|c| c.args(&args.iter().map(|s| s.as_str()).collect::<Vec<_>>()))
        {
            Ok(cmd) => cmd,
            Err(e) => return Err(format!("Docker execution failed: {e}")),
        };
        // A timed-out container must be killed and reaped, not detached (#2660).
        let outcome = cmd.execute_async_with_timeout(timeout_duration).await;

        match outcome {
            Ok(Some(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let exit_code = output.status.code().unwrap_or(-1);

                Ok(ExecutionResult {
                    stdout,
                    stderr,
                    exit_code,
                    execution_time_ms: 0,
                    timed_out: false,
                    killed: !output.status.success(),
                    error: None,
                })
            }
            Ok(None) => Ok(ExecutionResult::timeout()),
            Err(e) => Err(format!("Docker execution failed: {e}")),
        }
    }

    async fn execute_firecracker(&self) -> Result<ExecutionResult, String> {
        warn!(
            "Firecracker runtime unavailable for session {}; refusing to run untrusted code on the host",
            self.session_id
        );
        Err(
            "Firecracker runtime is not available; refusing unsafe fallback to host process execution"
                .to_string(),
        )
    }

    async fn execute_process(
        &self,
        code: &str,
        language: &CodeLanguage,
    ) -> Result<ExecutionResult, String> {
        let temp_dir = format!("{}/{}", self.config.work_dir, Uuid::new_v4());
        std::fs::create_dir_all(&temp_dir)
            .map_err(|e| format!("Failed to create temp dir: {}", e))?;

        let code_file = format!("{}/code.{}", temp_dir, language.file_extension());
        std::fs::write(&code_file, code)
            .map_err(|e| format!("Failed to write code file: {}", e))?;

        let (cmd_name, cmd_args): (&str, Vec<&str>) = match *language {
            CodeLanguage::Python => ("python3", vec![code_file.as_str()]),
            CodeLanguage::JavaScript => ("node", vec![code_file.as_str()]),
            CodeLanguage::Bash => ("bash", vec![code_file.as_str()]),
        };

        let timeout_duration = Duration::from_secs(self.config.timeout_seconds);
        let cmd = match SafeCommand::new(cmd_name)
            .and_then(|c| c.args(&cmd_args))
            .and_then(|c| c.working_dir(std::path::Path::new(&temp_dir)))
        {
            Ok(cmd) => cmd,
            Err(e) => return Err(format!("Process execution failed: {e}")),
        };
        // Kills the child's process group on expiry, so `remove_dir_all` below
        // no longer races a still-running interpreter holding the code file.
        let outcome = cmd.execute_async_with_timeout(timeout_duration).await;

        let _ = std::fs::remove_dir_all(&temp_dir);

        match outcome {
            Ok(Some(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let exit_code = output.status.code().unwrap_or(-1);

                Ok(ExecutionResult {
                    stdout,
                    stderr,
                    exit_code,
                    execution_time_ms: 0,
                    timed_out: false,
                    killed: false,
                    error: None,
                })
            }
            Ok(None) => Ok(ExecutionResult::timeout()),
            Err(e) => Err(format!("Process execution failed: {e}")),
        }
    }

    pub async fn execute_file(&self, file_path: &str, language: CodeLanguage) -> ExecutionResult {
        match std::fs::read_to_string(file_path) {
            Ok(code) => self.execute(&code, language).await,
            Err(e) => ExecutionResult::error(&format!("Failed to read file: {}", e)),
        }
    }
}
pub fn register_sandbox_keywords(
    state: Arc<dyn BasicRuntime>,
    user: UserSession,
    engine: &mut Engine,
) {
    run_python_keyword(Arc::clone(&state), user.clone(), engine);
    run_javascript_keyword(Arc::clone(&state), user.clone(), engine);
    run_bash_keyword(Arc::clone(&state), user.clone(), engine);
    run_file_keyword(state, user, engine);
}
