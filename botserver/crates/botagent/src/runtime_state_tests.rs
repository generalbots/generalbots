use super::{evaluate, AgentRuntimeState};
use crate::runtime_host::RuntimeHost;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

const WRAPPER: &str = r"C:\App\installers\incus.exe";
const MARKER: &str = r"C:\Stack\agent-runtime\ready.json";

/// A host whose every answer is injected.
struct FakeHost {
    platform: &'static str,
    env: HashMap<&'static str, String>,
    programs: HashSet<String>,
    files: HashSet<PathBuf>,
    wsl: bool,
    distros: Vec<&'static str>,
}

impl FakeHost {
    fn windows() -> Self {
        Self {
            platform: "windows",
            env: HashMap::new(),
            programs: HashSet::new(),
            files: HashSet::new(),
            wsl: true,
            distros: Vec::new(),
        }
    }

    fn on(platform: &'static str) -> Self {
        Self {
            platform,
            ..Self::windows()
        }
    }

    fn env(mut self, name: &'static str, value: &str) -> Self {
        self.env.insert(name, value.to_string());
        self
    }

    fn program(mut self, name: &str) -> Self {
        self.programs.insert(name.to_string());
        self
    }

    fn file(mut self, path: &str) -> Self {
        self.files.insert(PathBuf::from(path));
        self
    }

    fn distro(mut self, name: &'static str) -> Self {
        self.distros.push(name);
        self
    }

    fn without_wsl(mut self) -> Self {
        self.wsl = false;
        self
    }

    /// What the desktop shell sets when it ships the wrapper.
    fn product_setup(self) -> Self {
        self.env("AGENT_INCUS_BIN", WRAPPER)
            .env("GBO_WSL_DISTRO", "GeneralBotsLite")
            .env("GBO_AGENT_RUNTIME_MARKER", MARKER)
            .program(WRAPPER)
    }
}

impl RuntimeHost for FakeHost {
    fn platform(&self) -> &str {
        self.platform
    }
    fn env(&self, name: &str) -> Option<String> {
        self.env.get(name).cloned()
    }
    fn program_exists(&self, program: &str) -> bool {
        self.programs.contains(program)
    }
    fn file_exists(&self, path: &Path) -> bool {
        self.files.contains(path)
    }
    fn wsl_installed(&self) -> bool {
        self.wsl
    }
    fn distro_registered(&self, name: &str) -> bool {
        self.distros.iter().any(|d| d.eq_ignore_ascii_case(name))
    }}

fn state(host: &FakeHost) -> AgentRuntimeState {
    evaluate(host).state
}

#[test]
fn codes_are_the_ones_the_ui_reads() {
    let codes: Vec<&str> = [
        AgentRuntimeState::Unsupported,
        AgentRuntimeState::WslMissing,
        AgentRuntimeState::DistroMissing,
        AgentRuntimeState::RuntimeNotInstalled,
        AgentRuntimeState::Ready,
    ]
    .iter()
    .map(|s| s.code())
    .collect();
    assert_eq!(
        codes,
        ["unsupported", "wsl_missing", "distro_missing", "runtime_not_installed", "ready"]
    );
}

#[test]
fn windows_without_wsl_is_wsl_missing_even_with_the_wrapper() {
    let host = FakeHost::windows().product_setup().without_wsl();
    let runtime = evaluate(&host);
    assert_eq!(runtime.state, AgentRuntimeState::WslMissing);
    assert!(runtime.reason.is_some_and(|r| r.contains("WSL2")));
    assert_eq!(state(&FakeHost::windows().without_wsl()), AgentRuntimeState::WslMissing);
}

#[test]
fn windows_with_wsl_but_no_product_distro_is_distro_missing() {
    let host = FakeHost::windows().product_setup().distro("Ubuntu-24.04");
    assert_eq!(state(&host), AgentRuntimeState::DistroMissing);
}

#[test]
fn wsl_and_distro_alone_are_not_a_backend() {
    let host = FakeHost::windows().distro("GeneralBotsLite").program("wsl");
    assert_eq!(state(&host), AgentRuntimeState::RuntimeNotInstalled);
    let bare = host.env("AGENT_INCUS_BIN", "incus").program("incus");
    assert_eq!(state(&bare), AgentRuntimeState::RuntimeNotInstalled);
}

#[test]
fn product_setup_without_the_marker_is_not_installed() {
    let host = FakeHost::windows().product_setup().distro("GeneralBotsLite");
    let runtime = evaluate(&host);
    assert_eq!(runtime.state, AgentRuntimeState::RuntimeNotInstalled);
    assert!(runtime.reason.is_some_and(|r| r.contains("ready.json")));
}

#[test]
fn product_setup_with_a_missing_wrapper_is_not_installed() {
    let mut host = FakeHost::windows().product_setup().distro("GeneralBotsLite").file(MARKER);
    host.programs.clear();
    assert_eq!(state(&host), AgentRuntimeState::RuntimeNotInstalled);
}

#[test]
fn product_setup_complete_is_ready() {
    let host = FakeHost::windows().product_setup().distro("generalbotslite").file(MARKER);
    assert_eq!(evaluate(&host).reason, None);
    assert_eq!(state(&host), AgentRuntimeState::Ready);
}

#[test]
fn a_blank_marker_variable_counts_as_unset() {
    let host = FakeHost::windows()
        .env("AGENT_INCUS_BIN", WRAPPER)
        .program(WRAPPER)
        .distro("GeneralBotsLite")
        .env("GBO_AGENT_RUNTIME_MARKER", "  ");
    assert_eq!(state(&host), AgentRuntimeState::Ready, "blank marker variable = manual setup");
    let product = FakeHost::windows().distro("GeneralBotsLite").file(MARKER);
    assert_eq!(state(&product), AgentRuntimeState::RuntimeNotInstalled, "no wrapper configured");
}

#[test]
fn owner_manual_setup_needs_no_marker() {
    let wrapper = r"C:\Users\me\Desktop\App\agent\incus.exe";
    let host = FakeHost::windows()
        .env("AGENT_INCUS_BIN", wrapper)
        .env("GBO_WSL_DISTRO", "Ubuntu-24.04")
        .program(wrapper)
        .distro("Ubuntu-24.04")
        .distro("docker-desktop");
    assert_eq!(state(&host), AgentRuntimeState::Ready);
}

#[test]
fn manual_setup_still_needs_its_distro_and_wrapper() {
    let wrapper = r"D:\tools\incus.exe";
    let no_distro = FakeHost::windows()
        .env("AGENT_INCUS_BIN", wrapper)
        .env("GBO_WSL_DISTRO", "Ubuntu-24.04")
        .program(wrapper);
    assert_eq!(state(&no_distro), AgentRuntimeState::DistroMissing);
    let no_wrapper = FakeHost::windows()
        .env("AGENT_INCUS_BIN", wrapper)
        .env("GBO_WSL_DISTRO", "Ubuntu-24.04")
        .distro("Ubuntu-24.04");
    assert_eq!(state(&no_wrapper), AgentRuntimeState::RuntimeNotInstalled);
}

#[test]
fn an_invalid_distro_name_is_distro_missing() {
    let host = FakeHost::windows()
        .product_setup()
        .env("GBO_WSL_DISTRO", "a b;c")
        .distro("a b;c")
        .file(MARKER);
    assert_eq!(state(&host), AgentRuntimeState::DistroMissing);
}

#[test]
fn remote_opt_out_skips_wsl_but_needs_the_client() {
    let host = FakeHost::windows()
        .without_wsl()
        .env("AGENT_INCUS_REMOTE", "1")
        .env("AGENT_INCUS_BIN", "incus-remote.cmd")
        .program("incus-remote.cmd");
    assert_eq!(state(&host), AgentRuntimeState::Ready);
    let missing = FakeHost::windows()
        .without_wsl()
        .env("AGENT_INCUS_REMOTE", "1")
        .env("AGENT_INCUS_BIN", "incus-remote.cmd");
    assert_eq!(state(&missing), AgentRuntimeState::RuntimeNotInstalled);
}

#[test]
fn a_remote_client_without_the_opt_out_is_checked_like_a_wrapper() {
    let host = FakeHost::windows()
        .without_wsl()
        .env("AGENT_INCUS_BIN", "incus-remote.cmd")
        .program("incus-remote.cmd");
    assert_eq!(state(&host), AgentRuntimeState::WslMissing);
}

#[test]
fn linux_needs_incus_on_path() {
    let missing = evaluate(&FakeHost::on("linux"));
    assert_eq!(missing.state, AgentRuntimeState::RuntimeNotInstalled);
    assert!(missing.reason.is_some_and(|r| r.contains("AGENT_INCUS_BIN")));
    assert_eq!(state(&FakeHost::on("linux").program("incus")), AgentRuntimeState::Ready);
}

#[test]
fn macos_without_incus_is_unsupported() {
    assert_eq!(state(&FakeHost::on("macos")), AgentRuntimeState::Unsupported);
    assert_eq!(state(&FakeHost::on("macos").program("incus")), AgentRuntimeState::Ready);
}

#[test]
fn a_configured_absolute_path_must_exist() {
    assert!(!crate::runtime_host::binary_on_path("/definitely/not/a/real/incus"));
}

#[test]
fn distro_names_follow_the_wrapper_rule() {
    use crate::runtime_host::valid_distro_name;
    assert!(valid_distro_name("GeneralBotsLite"));
    assert!(valid_distro_name("Ubuntu-24.04"));
    for bad in ["", "-d", "a b", "a/b", "x;y"] {
        assert!(!valid_distro_name(bad), "{bad}");
    }
}

#[test]
fn the_real_host_answers_without_starting_anything() {
    use crate::runtime_host::SystemHost;
    let host = SystemHost;
    assert!(!host.distro_registered("no-such-distro-for-this-test"));
    let runtime = evaluate(&host);
    println!("agent_runtime_state on this host: {}", runtime.state.code());
    assert_eq!(runtime.reason.is_none(), runtime.state == AgentRuntimeState::Ready);
}
