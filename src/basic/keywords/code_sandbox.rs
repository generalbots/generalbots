use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use diesel::prelude::*;
use log::{trace, warn};
use rhai::{Dynamic, Engine};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SandboxRuntime {
    LXC,

    Docker,

    Firecracker,

    Process,
}

impl Default for SandboxRuntime {
    fn default() -> Self {
        Self::Process
    }
}

impl From<&str> for SandboxRuntime {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "lxc" => Self::LXC,
            "docker" => Self::Docker,
            "firecracker" => Self::Firecracker,
            _ => Self::Process,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CodeLanguage {
    Python,
    JavaScript,
    Bash,
}

impl From<&str> for CodeLanguage {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "python" | "py" | "javascript" | "js" | "node" | "bash" | "sh" | "shell" => {}
            _ => {}
        }
        match s.to_lowercase().as_str() {
            "python" | "py" => Self::Python,
            "javascript" | "js" | "node" => Self::JavaScript,
            "bash" | "sh" | "shell" | _ => Self::Bash,
        }
    }
}

impl CodeLanguage {
    pub fn file_extension(&self) -> &str {
        match self {
            Self::Python => "py",
            Self::JavaScript => "js",
            Self::Bash => "sh",
        }
    }

    pub fn interpreter(&self) -> &str {
        match self {
            Self::Python => "python3",
            Self::JavaScript => "node",
            Self::Bash => "bash",
        }
    }

    pub fn lxc_image(&self) -> &str {
        match self {
            Self::Python => "gb-sandbox-python",
            Self::JavaScript => "gb-sandbox-node",
            Self::Bash => "gb-sandbox-base",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub enabled: bool,

    pub runtime: SandboxRuntime,

    pub timeout_seconds: u64,

    pub memory_limit_mb: u64,

    pub cpu_limit_percent: u32,

    pub network_enabled: bool,

    pub work_dir: String,

    pub env_vars: HashMap<String, String>,

    pub allowed_paths: Vec<String>,

    pub python_packages: Vec<String>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            runtime: SandboxRuntime::Process,
            timeout_seconds: 30,
            memory_limit_mb: 256,
            cpu_limit_percent: 50,
            network_enabled: false,
            work_dir: "/tmp/gb-sandbox".to_string(),
            env_vars: HashMap::new(),
            allowed_paths: vec!["/data".to_string(), "/tmp".to_string()],
            python_packages: vec![],
        }
    }
}

impl SandboxConfig {
    pub fn from_bot_config(state: &AppState, bot_id: Uuid) -> Self {
        let mut config = Self::default();

        if let Ok(mut conn) = state.conn.get() {
            #[derive(QueryableByName)]
            struct ConfigRow {
                #[diesel(sql_type = diesel::sql_types::Text)]
                config_key: String,
                #[diesel(sql_type = diesel::sql_types::Text)]
                config_value: String,
            }

            let configs: Vec<ConfigRow> = diesel::sql_query(
                "SELECT config_key, config_value FROM bot_configuration \
                 WHERE bot_id = $1 AND config_key LIKE 'sandbox-%'",
            )
            .bind::<diesel::sql_types::Uuid, _>(bot_id)
            .load(&mut conn)
            .unwrap_or_default();

            for row in configs {
                match row.config_key.as_str() {
                    "sandbox-enabled" => {
                        config.enabled = row.config_value.to_lowercase() == "true";
                    }
                    "sandbox-runtime" => {
                        config.runtime = SandboxRuntime::from(row.config_value.as_str());
                    }
                    "sandbox-timeout" => {
                        config.timeout_seconds = row.config_value.parse().unwrap_or(30);
                    }

                    "sandbox-memory-mb" | "sandbox-memory-limit" => {
                        config.memory_limit_mb = row.config_value.parse().unwrap_or(256);
                    }
                    "sandbox-cpu-percent" | "sandbox-cpu-limit" => {
                        config.cpu_limit_percent = row.config_value.parse().unwrap_or(50);
                    }
                    "sandbox-network" | "sandbox-network-enabled" => {
                        config.network_enabled = row.config_value.to_lowercase() == "true";
                    }
                    "sandbox-python-packages" => {
                        config.python_packages = row
                            .config_value
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                    }
                    "sandbox-allowed-paths" => {
                        config.allowed_paths = row
                            .config_value
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                    }
                    _ => {}
                }
            }
        }

        config
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub stdout: String,

    pub stderr: String,

    pub exit_code: i32,

    pub execution_time_ms: u64,

    pub timed_out: bool,

    pub killed: bool,

    pub error: Option<String>,
}

impl ExecutionResult {
    pub fn success(stdout: String, stderr: String, execution_time_ms: u64) -> Self {
        Self {
            stdout,
            stderr,
            exit_code: 0,
            execution_time_ms,
            timed_out: false,
            killed: false,
            error: None,
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            stdout: String::new(),
            stderr: String::new(),
            exit_code: -1,
            execution_time_ms: 0,
            timed_out: false,
            killed: false,
            error: Some(message.to_string()),
        }
    }

    pub fn timeout() -> Self {
        Self {
            stdout: String::new(),
            stderr: "Execution timed out".to_string(),
            exit_code: -1,
            execution_time_ms: 0,
            timed_out: true,
            killed: true,
            error: Some("Execution exceeded time limit".to_string()),
        }
    }

    pub fn is_success(&self) -> bool {
        self.exit_code == 0 && self.error.is_none() && !self.timed_out
    }

    pub fn output(&self) -> String {
        if self.is_success() {
            self.stdout.clone()
        } else if let Some(err) = &self.error {
            format!("Error: {}\n{}", err, self.stderr)
        } else {
            format!("Error (exit code {}): {}", self.exit_code, self.stderr)
        }
    }
}

pub struct CodeSandbox {
    config: SandboxConfig,
    session_id: Uuid,
}

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
        if !self.config.enabled {
            return ExecutionResult::error("Sandbox execution is disabled");
        }

        let start_time = std::time::Instant::now();

        let result = match self.config.runtime {
            SandboxRuntime::LXC => self.execute_lxc(code, &language).await,
            SandboxRuntime::Docker => self.execute_docker(code, &language).await,
            SandboxRuntime::Firecracker => self.execute_firecracker(code, &language).await,
            SandboxRuntime::Process => self.execute_process(code, &language).await,
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

        let mut cmd = Command::new("lxc-execute");
        cmd.arg("-n")
            .arg(&container_name)
            .arg("-f")
            .arg(format!("/etc/lxc/{}.conf", language.lxc_image()))
            .arg("--")
            .arg(language.interpreter())
            .arg(&code_file);

        cmd.env(
            "LXC_CGROUP_MEMORY_LIMIT",
            format!("{}M", self.config.memory_limit_mb),
        )
        .env(
            "LXC_CGROUP_CPU_QUOTA",
            format!("{}", self.config.cpu_limit_percent * 1000),
        );

        let timeout_duration = Duration::from_secs(self.config.timeout_seconds);
        let output = timeout(timeout_duration, async {
            tokio::process::Command::new("lxc-execute")
                .arg("-n")
                .arg(&container_name)
                .arg("-f")
                .arg(format!("/etc/lxc/{}.conf", language.lxc_image()))
                .arg("--")
                .arg(language.interpreter())
                .arg(&code_file)
                .output()
                .await
        })
        .await;

        let _ = std::fs::remove_file(&code_file);

        match output {
            Ok(Ok(output)) => {
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
            Ok(Err(e)) => Err(format!("LXC execution failed: {}", e)),
            Err(_) => Ok(ExecutionResult::timeout()),
        }
    }

    async fn execute_docker(
        &self,
        code: &str,
        language: &CodeLanguage,
    ) -> Result<ExecutionResult, String> {
        let image = match language {
            CodeLanguage::Python => "python:3.11-slim",
            CodeLanguage::JavaScript => "node:20-slim",
            CodeLanguage::Bash => "alpine:latest",
        };

        let args = vec![
            "run".to_string(),
            "--rm".to_string(),
            "--network".to_string(),
            if self.config.network_enabled {
                "bridge"
            } else {
                "none"
            }
            .to_string(),
            "--memory".to_string(),
            format!("{}m", self.config.memory_limit_mb),
            "--cpus".to_string(),
            format!("{:.2}", f64::from(self.config.cpu_limit_percent) / 100.0),
            "--read-only".to_string(),
            "--security-opt".to_string(),
            "no-new-privileges".to_string(),
            image.to_string(),
            language.interpreter().to_string(),
            "-c".to_string(),
            code.to_string(),
        ];

        let timeout_duration = Duration::from_secs(self.config.timeout_seconds);
        let output = timeout(timeout_duration, async {
            tokio::process::Command::new("docker")
                .args(&args)
                .output()
                .await
        })
        .await;

        match output {
            Ok(Ok(output)) => {
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
            Ok(Err(e)) => Err(format!("Docker execution failed: {}", e)),
            Err(_) => Ok(ExecutionResult::timeout()),
        }
    }

    async fn execute_firecracker(
        &self,
        code: &str,
        language: &CodeLanguage,
    ) -> Result<ExecutionResult, String> {
        warn!("Firecracker runtime not yet implemented, falling back to process isolation");
        self.execute_process(code, language).await
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
        let output = timeout(timeout_duration, async {
            tokio::process::Command::new(cmd_name)
                .args(&cmd_args)
                .current_dir(&temp_dir)
                .env_clear()
                .env("PATH", "/usr/local/bin:/usr/bin:/bin")
                .env("HOME", &temp_dir)
                .env("TMPDIR", &temp_dir)
                .output()
                .await
        })
        .await;

        let _ = std::fs::remove_dir_all(&temp_dir);

        match output {
            Ok(Ok(output)) => {
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
            Ok(Err(e)) => Err(format!("Process execution failed: {}", e)),
            Err(_) => Ok(ExecutionResult::timeout()),
        }
    }

    pub async fn execute_file(&self, file_path: &str, language: CodeLanguage) -> ExecutionResult {
        match std::fs::read_to_string(file_path) {
            Ok(code) => self.execute(&code, language).await,
            Err(e) => ExecutionResult::error(&format!("Failed to read file: {}", e)),
        }
    }
}

pub fn register_sandbox_keywords(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    run_python_keyword(Arc::clone(&state), user.clone(), engine);
    run_javascript_keyword(Arc::clone(&state), user.clone(), engine);
    run_bash_keyword(Arc::clone(&state), user.clone(), engine);
    run_file_keyword(state, user, engine);
}

pub fn run_python_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user;

    engine
        .register_custom_syntax(
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

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
                    let result = rt.block_on(async {
                        let config = SandboxConfig::from_bot_config(&state_for_task, bot_id);
                        let sandbox = CodeSandbox::new(config, session_id);
                        sandbox.execute(&code, CodeLanguage::Python).await
                    });
                    let _ = tx.send(result);
                });

                match rx.recv_timeout(std::time::Duration::from_secs(60)) {
                    Ok(result) => Ok(Dynamic::from(result.output())),
                    Err(_) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        "RUN PYTHON timed out".into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .expect("Failed to register RUN PYTHON syntax");
}

pub fn run_javascript_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine
        .register_custom_syntax(
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

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
                    let result = rt.block_on(async {
                        let config = SandboxConfig::from_bot_config(&state_for_task, bot_id);
                        let sandbox = CodeSandbox::new(config, session_id);
                        sandbox.execute(&code, CodeLanguage::JavaScript).await
                    });
                    let _ = tx.send(result);
                });

                match rx.recv_timeout(std::time::Duration::from_secs(60)) {
                    Ok(result) => Ok(Dynamic::from(result.output())),
                    Err(_) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        "RUN JAVASCRIPT timed out".into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .expect("Failed to register RUN JAVASCRIPT syntax");

    engine
        .register_custom_syntax(["RUN", "JS", "$expr$"], false, move |context, inputs| {
            let code = context
                .eval_expression_tree(&inputs[0])?
                .to_string()
                .trim_matches('"')
                .to_string();

            let state_for_task = Arc::clone(&state);
            let session_id = user.id;
            let bot_id = user.bot_id;

            let (tx, rx) = std::sync::mpsc::channel();

            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
                let result = rt.block_on(async {
                    let config = SandboxConfig::from_bot_config(&state_for_task, bot_id);
                    let sandbox = CodeSandbox::new(config, session_id);
                    sandbox.execute(&code, CodeLanguage::JavaScript).await
                });
                let _ = tx.send(result);
            });

            match rx.recv_timeout(std::time::Duration::from_secs(60)) {
                Ok(result) => Ok(Dynamic::from(result.output())),
                Err(_) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    "RUN JS timed out".into(),
                    rhai::Position::NONE,
                ))),
            }
        })
        .expect("Failed to register RUN JS syntax");
}

pub fn run_bash_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(["RUN", "BASH", "$expr$"], false, move |context, inputs| {
            let code = context
                .eval_expression_tree(&inputs[0])?
                .to_string()
                .trim_matches('"')
                .to_string();

            trace!("RUN BASH for session: {}", user.id);

            let state_for_task = Arc::clone(&state);
            let session_id = user.id;
            let bot_id = user.bot_id;

            let (tx, rx) = std::sync::mpsc::channel();

            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
                let result = rt.block_on(async {
                    let config = SandboxConfig::from_bot_config(&state_for_task, bot_id);
                    let sandbox = CodeSandbox::new(config, session_id);
                    sandbox.execute(&code, CodeLanguage::Bash).await
                });
                let _ = tx.send(result);
            });

            match rx.recv_timeout(std::time::Duration::from_secs(60)) {
                Ok(result) => Ok(Dynamic::from(result.output())),
                Err(_) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    "RUN BASH timed out".into(),
                    rhai::Position::NONE,
                ))),
            }
        })
        .expect("Failed to register RUN BASH syntax");
}

pub fn run_file_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine
        .register_custom_syntax(
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
                let session_id = user_clone.id;
                let bot_id = user_clone.bot_id;

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
                    let result = rt.block_on(async {
                        let config = SandboxConfig::from_bot_config(&state_for_task, bot_id);
                        let sandbox = CodeSandbox::new(config, session_id);
                        sandbox.execute_file(&file_path, CodeLanguage::Python).await
                    });
                    let _ = tx.send(result);
                });

                match rx.recv_timeout(std::time::Duration::from_secs(60)) {
                    Ok(result) => Ok(Dynamic::from(result.output())),
                    Err(_) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        "RUN PYTHON WITH FILE timed out".into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .expect("Failed to register RUN PYTHON WITH FILE syntax");

    engine
        .register_custom_syntax(
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

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
                    let result = rt.block_on(async {
                        let config = SandboxConfig::from_bot_config(&state_for_task, bot_id);
                        let sandbox = CodeSandbox::new(config, session_id);
                        sandbox
                            .execute_file(&file_path, CodeLanguage::JavaScript)
                            .await
                    });
                    let _ = tx.send(result);
                });

                match rx.recv_timeout(std::time::Duration::from_secs(60)) {
                    Ok(result) => Ok(Dynamic::from(result.output())),
                    Err(_) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        "RUN JAVASCRIPT WITH FILE timed out".into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .expect("Failed to register RUN JAVASCRIPT WITH FILE syntax");
}

pub fn generate_python_lxc_config() -> String {
    r"
# LXC configuration for Python sandbox
lxc.include = /usr/share/lxc/config/common.conf
lxc.arch = linux64

# Container name template
lxc.uts.name = gb-sandbox-python

# Root filesystem
lxc.rootfs.path = dir:/var/lib/lxc/gb-sandbox-python/rootfs

# Network - isolated by default
lxc.net.0.type = empty

# Resource limits
lxc.cgroup2.memory.max = 256M
lxc.cgroup2.cpu.max = 50000 100000

# Security
lxc.cap.drop = sys_admin sys_boot sys_module sys_time
lxc.apparmor.profile = generated
lxc.seccomp.profile = /usr/share/lxc/config/common.seccomp

# Mount points - minimal
lxc.mount.auto = proc:mixed sys:ro
lxc.mount.entry = /usr/bin/python3 usr/bin/python3 none ro,bind 0 0
lxc.mount.entry = /usr/lib/python3 usr/lib/python3 none ro,bind 0 0
lxc.mount.entry = tmpfs tmp tmpfs defaults 0 0
"
    .to_string()
}

pub fn generate_node_lxc_config() -> String {
    r"
# LXC configuration for Node.js sandbox
lxc.include = /usr/share/lxc/config/common.conf
lxc.arch = linux64

# Container name template
lxc.uts.name = gb-sandbox-node

# Root filesystem
lxc.rootfs.path = dir:/var/lib/lxc/gb-sandbox-node/rootfs

# Network - isolated by default
lxc.net.0.type = empty

# Resource limits
lxc.cgroup2.memory.max = 256M
lxc.cgroup2.cpu.max = 50000 100000

# Security
lxc.cap.drop = sys_admin sys_boot sys_module sys_time
lxc.apparmor.profile = generated
lxc.seccomp.profile = /usr/share/lxc/config/common.seccomp

# Mount points - minimal
lxc.mount.auto = proc:mixed sys:ro
lxc.mount.entry = /usr/bin/node usr/bin/node none ro,bind 0 0
lxc.mount.entry = /usr/lib/node_modules usr/lib/node_modules none ro,bind 0 0
lxc.mount.entry = tmpfs tmp tmpfs defaults 0 0
"
    .to_string()
}
