#![no_main]
//! JWT / claims decoding (#2119).
//!
//! The token string and its JSON payload are both attacker-controlled. The
//! invariant: `JwtManager` never panics on garbage, and the only decoding path
//! verifies the signature first (#2125), so arbitrary input must come back as
//! `Err` — never as claims.

use libfuzzer_sys::fuzz_target;

const SECRET: &str = "fuzz-secret-key-that-is-32-chars!!";

fuzz_target!(|data: &[u8]| {
    let Ok(token) = std::str::from_utf8(data) else {
        return;
    };
    let Ok(manager) = botsecurity_auth::jwt::JwtManager::from_secret(SECRET) else {
        return;
    };

    assert!(
        manager.validate_access_token(token).is_err(),
        "an unsigned token must never validate"
    );
    assert!(
        manager.validate_token(token).is_err(),
        "an unsigned token must never decode"
    );
    // A JWT payload is JSON: deserializing attacker-controlled claims must
    // fail closed instead of panicking on a type mismatch.
    let _ = serde_json::from_str::<botsecurity_auth::jwt::Claims>(token);
    let _ = serde_json::from_str::<serde_json::Value>(token);
});
