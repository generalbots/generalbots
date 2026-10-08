#[cfg(test)]
mod signup_password_tests {
    use crate::api::password_policy_violation;

    #[test]
    fn accepts_a_policy_compliant_password() {
        assert_eq!(password_policy_violation("Campos10!"), None);
        assert_eq!(password_policy_violation("Abcdef1#xyz"), None);
    }

    #[test]
    fn rejects_the_signup_that_created_an_unloginnable_account() {
        // marcelbeiner@gmail.com signed up with `campos10`: Zitadel refused the
        // password set (no symbol), the failure was swallowed, and the account
        // could never log in. The pre-check must catch every variant.
        assert!(password_policy_violation("campos10").is_some());
        assert!(password_policy_violation("CAMPOS10!").is_some()); // no lower
        assert!(password_policy_violation("Campos!").is_some()); // no digit
        assert!(password_policy_violation("Campos10").is_some()); // no symbol
        assert!(password_policy_violation("Ca1!").is_some()); // too short
    }

    #[test]
    fn rejection_messages_name_the_missing_class() {
        let msg = password_policy_violation("campos10").unwrap_or_default();
        assert!(msg.contains("uppercase"), "got: {msg}");
        let msg = password_policy_violation("Campos10").unwrap_or_default();
        assert!(msg.contains("symbol"), "got: {msg}");
    }
}
