//! Points the sidecar at the bundled agent-workspace wrapper.
//!
//! When the installer ships `installers/incus.exe`, the server gets
//! `AGENT_INCUS_BIN` (that file), `GBO_WSL_DISTRO` (the product distro) and
//! `GBO_AGENT_RUNTIME_MARKER` (`<stack>/agent-runtime/ready.json`, written
//! when the agent runtime install ends). A person who already set
//! `AGENT_INCUS_BIN` keeps a manual setup: nothing is added then, so the
//! server judges it without the marker. Otherwise each value already in the
//! environment wins over the default.

use std::ffi::OsString;
use std::path::Path;

const WRAPPER_FILE: &str = "incus.exe";
const PRODUCT_DISTRO: &str = "GeneralBotsLite";

/// Variables to set on the sidecar, given whether the wrapper file exists
/// and how the current environment answers `lookup`.
pub(super) fn agent_env_for(
    installers: &Path,
    stack: &Path,
    wrapper_exists: bool,
    lookup: impl Fn(&str) -> Option<OsString>,
) -> Vec<(&'static str, OsString)> {
    let present = |name: &str| {
        lookup(name).is_some_and(|value| !value.to_string_lossy().trim().is_empty())
    };
    if !wrapper_exists || present("AGENT_INCUS_BIN") {
        return Vec::new();
    }
    let marker = stack.join("agent-runtime").join("ready.json");
    [
        ("AGENT_INCUS_BIN", installers.join(WRAPPER_FILE).into_os_string()),
        ("GBO_WSL_DISTRO", OsString::from(PRODUCT_DISTRO)),
        ("GBO_AGENT_RUNTIME_MARKER", marker.into_os_string()),
    ]
    .into_iter()
    .filter(|(name, _)| !present(name))
    .collect()
}

/// [`agent_env_for`] with the real file system and process environment.
pub(super) fn agent_env(installers: &Path, stack: &Path) -> Vec<(&'static str, OsString)> {
    let wrapper_exists = installers.join(WRAPPER_FILE).is_file();
    agent_env_for(installers, stack, wrapper_exists, |name| std::env::var_os(name))
}

#[cfg(test)]
mod tests {
    use super::agent_env_for;
    use std::ffi::OsString;
    use std::path::Path;

    const INSTALLERS: &str = r"C:\App\installers";
    const STACK: &str = r"C:\Data\stack";

    fn run(wrapper: bool, env: &[(&str, &str)]) -> Vec<(&'static str, OsString)> {
        let env: Vec<(String, OsString)> = env
            .iter()
            .map(|(k, v)| (k.to_string(), OsString::from(v)))
            .collect();
        agent_env_for(Path::new(INSTALLERS), Path::new(STACK), wrapper, |name| {
            env.iter().find(|(k, _)| k == name).map(|(_, v)| v.clone())
        })
    }

    #[test]
    fn bundled_wrapper_sets_the_product_trio() {
        let env = run(true, &[]);
        assert_eq!(
            env,
            vec![
                ("AGENT_INCUS_BIN", Path::new(INSTALLERS).join("incus.exe").into_os_string()),
                ("GBO_WSL_DISTRO", OsString::from("GeneralBotsLite")),
                (
                    "GBO_AGENT_RUNTIME_MARKER",
                    Path::new(STACK).join("agent-runtime").join("ready.json").into_os_string()
                ),
            ]
        );
    }

    #[test]
    fn no_wrapper_sets_nothing() {
        assert!(run(false, &[]).is_empty());
    }

    #[test]
    fn a_manual_wrapper_keeps_the_whole_setup_untouched() {
        let env = run(
            true,
            &[("AGENT_INCUS_BIN", r"C:\Users\me\agent\incus.exe"), ("GBO_WSL_DISTRO", "Ubuntu-24.04")],
        );
        assert!(env.is_empty());
        assert!(run(true, &[("AGENT_INCUS_BIN", r"C:\tools\incus.exe")]).is_empty());
    }

    #[test]
    fn existing_distro_or_marker_values_win() {
        let env = run(true, &[("GBO_WSL_DISTRO", "Other"), ("GBO_AGENT_RUNTIME_MARKER", r"D:\m.json")]);
        assert_eq!(env.len(), 1);
        assert_eq!(env[0].0, "AGENT_INCUS_BIN");
    }

    #[test]
    fn a_blank_wrapper_variable_counts_as_unset() {
        assert_eq!(run(true, &[("AGENT_INCUS_BIN", "  ")]).len(), 3);
    }
}
