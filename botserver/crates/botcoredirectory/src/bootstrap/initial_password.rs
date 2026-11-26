//! What happens to the random password the directory bootstrap gives the
//! administrator it creates.
//!
//! On a server install that password is the only way in, so it is shown once
//! and saved for the operator. In the desktop edition the person signs in with
//! the password the first-run wizard showed them; the directory account gets
//! that same password at the first sign-in, so the random one is a throwaway
//! that is never written to disk or printed.

use super::*;

/// Where the bootstrap's random administrator password goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitialPasswordDisclosure {
    /// Save it to `~/.gb-setup-credentials` and print the setup banner.
    SaveAndShow,
    /// Keep it in memory only; the wizard's password replaces it.
    Withhold,
}

pub fn initial_password_disclosure(desktop_edition: bool) -> InitialPasswordDisclosure {
    if desktop_edition {
        InitialPasswordDisclosure::Withhold
    } else {
        InitialPasswordDisclosure::SaveAndShow
    }
}

/// The current deployment's choice.
pub(super) fn current_disclosure() -> InitialPasswordDisclosure {
    initial_password_disclosure(botcore::product::is_shell_mode())
}

/// The desktop edition's console notice: no password, no file.
pub(super) fn print_desktop_bootstrap_notice(result: &BootstrapResult) {
    botlog::safe_info!(
        "Directory administrator '{}' created; it signs in with the password chosen in the first-run wizard",
        result.username
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_server_installs_save_and_show_the_random_password() {
        assert_eq!(initial_password_disclosure(true), InitialPasswordDisclosure::Withhold);
        assert_eq!(initial_password_disclosure(false), InitialPasswordDisclosure::SaveAndShow);
    }
}
