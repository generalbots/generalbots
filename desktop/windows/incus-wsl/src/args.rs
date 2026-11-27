//! Pure argument building for the wrapper: no process, no environment.

use std::ffi::OsString;

/// The product's own WSL distro. Other distros are used only when
/// `GBO_WSL_DISTRO` names them explicitly.
pub const DEFAULT_DISTRO: &str = "GeneralBotsLite";

/// First argument that selects a fixed service command instead of `incus`.
pub const SERVICE_VERB: &str = "__gb-service";

/// Why the wrapper refused to run; every case exits with code 2.
#[derive(Debug, PartialEq, Eq)]
pub enum ArgsError {
    InvalidDistro(String),
    UnknownServiceAction,
}

impl std::fmt::Display for ArgsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidDistro(name) => write!(
                f,
                "GBO_WSL_DISTRO `{name}` is not a valid distro name (letters, digits, `.`, `_` and `-` only)"
            ),
            Self::UnknownServiceAction => write!(f, "{SERVICE_VERB} takes `state` or `restart`"),
        }
    }
}

/// A distro name is 1 to 64 characters from `[A-Za-z0-9._-]` and does not
/// start with `-`, so it can never be read as a `wsl.exe` option.
pub fn valid_distro_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && !name.starts_with('-')
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

/// The distro to use: the trimmed `GBO_WSL_DISTRO` value, or the default
/// when it is unset or blank.
pub fn resolve_distro(configured: Option<&str>) -> Result<String, ArgsError> {
    let name = configured
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_DISTRO);
    if valid_distro_name(name) {
        Ok(name.to_string())
    } else {
        Err(ArgsError::InvalidDistro(name.to_string()))
    }
}

/// Arguments for `wsl.exe`: `-d <distro> -u root --exec` followed by either
/// `incus <args...>` (unchanged) or one of the fixed service commands.
pub fn wsl_args(distro: &str, args: &[OsString]) -> Result<Vec<OsString>, ArgsError> {
    if !valid_distro_name(distro) {
        return Err(ArgsError::InvalidDistro(distro.to_string()));
    }
    let mut out: Vec<OsString> = ["-d", distro, "-u", "root", "--exec"]
        .iter()
        .map(OsString::from)
        .collect();
    if args.first().is_some_and(|verb| verb == SERVICE_VERB) {
        let fixed: &[&str] = match args.get(1).and_then(|action| action.to_str()) {
            Some("state") if args.len() == 2 => &[
                "systemctl",
                "show",
                "incus.service",
                "--property=ActiveState,SubState,MainPID",
            ],
            Some("restart") if args.len() == 2 => &["systemctl", "restart", "incus.service"],
            _ => return Err(ArgsError::UnknownServiceAction),
        };
        out.extend(fixed.iter().map(OsString::from));
    } else {
        out.push(OsString::from("incus"));
        out.extend(args.iter().cloned());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn os(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn default_distro_is_the_product_distro() {
        assert_eq!(resolve_distro(None), Ok("GeneralBotsLite".to_string()));
        assert_eq!(resolve_distro(Some("   ")), Ok("GeneralBotsLite".to_string()));
    }

    #[test]
    fn configured_distro_is_trimmed_and_kept() {
        assert_eq!(resolve_distro(Some(" Ubuntu-24.04 ")), Ok("Ubuntu-24.04".to_string()));
    }

    #[test]
    fn distro_names_outside_the_charset_are_rejected() {
        for bad in ["a b", "x;y", "a/b", "a\\b", "-d", "é", "a\"b", &"x".repeat(65)] {
            assert!(resolve_distro(Some(bad)).is_err(), "{bad} must be rejected");
        }
        assert!(valid_distro_name("Debian_13.x-1"));
    }

    #[test]
    fn incus_arguments_pass_through_unchanged() {
        let args = wsl_args("GeneralBotsLite", &os(&["exec", "c1", "--", "sh", "-c", "echo a; b"]));
        assert_eq!(
            args,
            Ok(os(&[
                "-d", "GeneralBotsLite", "-u", "root", "--exec", "incus", "exec", "c1", "--", "sh",
                "-c", "echo a; b"
            ]))
        );
    }

    #[test]
    fn no_arguments_runs_bare_incus() {
        assert_eq!(
            wsl_args("GeneralBotsLite", &[]),
            Ok(os(&["-d", "GeneralBotsLite", "-u", "root", "--exec", "incus"]))
        );
    }

    #[test]
    fn service_state_and_restart_are_fixed_commands() {
        assert_eq!(
            wsl_args("D", &os(&["__gb-service", "state"])),
            Ok(os(&[
                "-d",
                "D",
                "-u",
                "root",
                "--exec",
                "systemctl",
                "show",
                "incus.service",
                "--property=ActiveState,SubState,MainPID"
            ]))
        );
        assert_eq!(
            wsl_args("D", &os(&["__gb-service", "restart"])),
            Ok(os(&["-d", "D", "-u", "root", "--exec", "systemctl", "restart", "incus.service"]))
        );
    }

    #[test]
    fn unknown_or_extended_service_actions_are_rejected() {
        for args in [
            os(&["__gb-service"]),
            os(&["__gb-service", "stop"]),
            os(&["__gb-service", "restart", "sshd.service"]),
        ] {
            assert_eq!(wsl_args("D", &args), Err(ArgsError::UnknownServiceAction));
        }
    }

    #[test]
    fn an_invalid_distro_is_rejected_by_the_builder_too() {
        assert!(matches!(
            wsl_args("bad name", &[]),
            Err(ArgsError::InvalidDistro(_))
        ));
    }
}
