//! The strength a new password needs. It mirrors the directory's default
//! complexity policy, so a password the local record accepts is one the
//! directory accepts too and the two never drift apart.

pub const MIN_PASSWORD_LEN: usize = 8;
pub const MAX_PASSWORD_LEN: usize = 128;

/// What is wrong with a proposed password; `code` is stable for the interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasswordProblem {
    TooShort,
    TooLong,
    MissingLowercase,
    MissingUppercase,
    MissingDigit,
    MissingSymbol,
    SameAsCurrent,
}

impl PasswordProblem {
    pub fn code(self) -> &'static str {
        match self {
            Self::TooShort => "password_too_short",
            Self::TooLong => "password_too_long",
            Self::MissingLowercase => "password_needs_lowercase",
            Self::MissingUppercase => "password_needs_uppercase",
            Self::MissingDigit => "password_needs_digit",
            Self::MissingSymbol => "password_needs_symbol",
            Self::SameAsCurrent => "password_unchanged",
        }
    }

    pub fn message(self) -> &'static str {
        match self {
            Self::TooShort => "The new password needs at least 8 characters.",
            Self::TooLong => "The new password can have at most 128 characters.",
            Self::MissingLowercase => "The new password needs a lowercase letter.",
            Self::MissingUppercase => "The new password needs an uppercase letter.",
            Self::MissingDigit => "The new password needs a number.",
            Self::MissingSymbol => "The new password needs a symbol, such as ! or #.",
            Self::SameAsCurrent => "The new password must be different from the current one.",
        }
    }
}

fn is_symbol(c: char) -> bool {
    !c.is_alphanumeric() && !c.is_whitespace() && !c.is_control()
}

/// Checks a new password; `current`, when known, must differ from it.
pub fn check_new_password(new_password: &str, current: Option<&str>) -> Result<(), PasswordProblem> {
    let length = new_password.chars().count();
    if length < MIN_PASSWORD_LEN {
        return Err(PasswordProblem::TooShort);
    }
    if length > MAX_PASSWORD_LEN {
        return Err(PasswordProblem::TooLong);
    }
    if !new_password.chars().any(char::is_lowercase) {
        return Err(PasswordProblem::MissingLowercase);
    }
    if !new_password.chars().any(char::is_uppercase) {
        return Err(PasswordProblem::MissingUppercase);
    }
    if !new_password.chars().any(|c| c.is_ascii_digit()) {
        return Err(PasswordProblem::MissingDigit);
    }
    if !new_password.chars().any(is_symbol) {
        return Err(PasswordProblem::MissingSymbol);
    }
    if current == Some(new_password) {
        return Err(PasswordProblem::SameAsCurrent);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_password_needs_every_character_class() {
        assert_eq!(check_new_password("Ab1!", None), Err(PasswordProblem::TooShort));
        assert_eq!(check_new_password("ABCDEFG1!", None), Err(PasswordProblem::MissingLowercase));
        assert_eq!(check_new_password("abcdefg1!", None), Err(PasswordProblem::MissingUppercase));
        assert_eq!(check_new_password("Abcdefgh!", None), Err(PasswordProblem::MissingDigit));
        assert_eq!(check_new_password("Abcdefgh1", None), Err(PasswordProblem::MissingSymbol));
        assert_eq!(check_new_password("Coração-2026", None), Ok(()));
        let long = format!("Aa1!{}", "x".repeat(MAX_PASSWORD_LEN));
        assert_eq!(check_new_password(&long, None), Err(PasswordProblem::TooLong));
    }

    #[test]
    fn the_new_password_must_differ_from_the_current_one() {
        assert_eq!(
            check_new_password("Senha#Forte9", Some("Senha#Forte9")),
            Err(PasswordProblem::SameAsCurrent)
        );
        assert_eq!(check_new_password("Senha#Forte9", Some("Outra#Senha1")), Ok(()));
    }

    #[test]
    fn every_problem_has_a_stable_code() {
        assert_eq!(PasswordProblem::TooShort.code(), "password_too_short");
        assert_eq!(PasswordProblem::SameAsCurrent.code(), "password_unchanged");
    }
}
