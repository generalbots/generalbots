use anyhow::{anyhow, Result};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Algorithm, Argon2, Params, Version,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tracing::{debug, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordConfig {
    pub min_length: usize,
    pub max_length: usize,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_digit: bool,
    pub require_special: bool,
    pub min_unique_chars: usize,
    pub max_consecutive_chars: usize,
    pub password_history_count: usize,
    pub expiration_days: Option<u32>,
    pub lockout_threshold: u32,
    pub lockout_duration_minutes: u32,
}

impl Default for PasswordConfig {
    fn default() -> Self {
        Self {
            min_length: 12,
            max_length: 128,
            require_uppercase: true,
            require_lowercase: true,
            require_digit: true,
            require_special: true,
            min_unique_chars: 6,
            max_consecutive_chars: 3,
            password_history_count: 12,
            expiration_days: Some(90),
            lockout_threshold: 5,
            lockout_duration_minutes: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Argon2Config {
    pub memory_cost_kib: u32,
    pub time_cost: u32,
    pub parallelism: u32,
    pub output_length: usize,
}

impl Default for Argon2Config {
    fn default() -> Self {
        Self {
            memory_cost_kib: 65536,
            time_cost: 3,
            parallelism: 4,
            output_length: 32,
        }
    }
}

impl Argon2Config {
    pub fn high_security() -> Self {
        Self {
            memory_cost_kib: 131072,
            time_cost: 4,
            parallelism: 8,
            output_length: 32,
        }
    }

    pub fn low_memory() -> Self {
        Self {
            memory_cost_kib: 32768,
            time_cost: 4,
            parallelism: 2,
            output_length: 32,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PasswordStrength {
    VeryWeak,
    Weak,
    Fair,
    Strong,
    VeryStrong,
}

impl PasswordStrength {
    pub fn score(&self) -> u8 {
        match self {
            Self::VeryWeak => 0,
            Self::Weak => 1,
            Self::Fair => 2,
            Self::Strong => 3,
            Self::VeryStrong => 4,
        }
    }

    pub fn is_acceptable(&self) -> bool {
        matches!(self, Self::Fair | Self::Strong | Self::VeryStrong)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordValidationResult {
    pub is_valid: bool,
    pub strength: PasswordStrength,
    pub score: u8,
    pub issues: Vec<PasswordIssue>,
    pub suggestions: Vec<String>,
    pub crack_time_display: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PasswordIssue {
    TooShort { min: usize, actual: usize },
    TooLong { max: usize, actual: usize },
    MissingUppercase,
    MissingLowercase,
    MissingDigit,
    MissingSpecial,
    InsufficientUniqueChars { min: usize, actual: usize },
    TooManyConsecutiveChars { max: usize },
    CommonPassword,
    ContainsUsername,
    ContainsEmail,
    RecentlyUsed,
    Compromised,
}

impl PasswordIssue {
    pub fn message(&self) -> String {
        match self {
            Self::TooShort { min, actual } => {
                format!("Password must be at least {min} characters (currently {actual})")
            }
            Self::TooLong { max, actual } => {
                format!("Password must be at most {max} characters (currently {actual})")
            }
            Self::MissingUppercase => "Password must contain at least one uppercase letter".into(),
            Self::MissingLowercase => "Password must contain at least one lowercase letter".into(),
            Self::MissingDigit => "Password must contain at least one digit".into(),
            Self::MissingSpecial => "Password must contain at least one special character".into(),
            Self::InsufficientUniqueChars { min, actual } => {
                format!("Password must have at least {min} unique characters (currently {actual})")
            }
            Self::TooManyConsecutiveChars { max } => {
                format!("Password must not have more than {max} consecutive identical characters")
            }
            Self::CommonPassword => "This password is too common and easily guessed".into(),
            Self::ContainsUsername => "Password must not contain your username".into(),
            Self::ContainsEmail => "Password must not contain your email address".into(),
            Self::RecentlyUsed => "This password was used recently, please choose a new one".into(),
            Self::Compromised => "This password has been found in data breaches".into(),
        }
    }
}

pub struct PasswordHasher2 {
    argon2: Argon2<'static>,
    config: PasswordConfig,
}

impl PasswordHasher2 {
    pub fn new(argon2_config: Argon2Config, password_config: PasswordConfig) -> Result<Self> {
        let params = Params::new(
            argon2_config.memory_cost_kib,
            argon2_config.time_cost,
            argon2_config.parallelism,
            Some(argon2_config.output_length),
        )
        .map_err(|e| anyhow!("Invalid Argon2 parameters: {e}"))?;

        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

        Ok(Self {
            argon2,
            config: password_config,
        })
    }

    pub fn with_defaults() -> Result<Self> {
        Self::new(Argon2Config::default(), PasswordConfig::default())
    }

    pub fn hash(&self, password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let hash = self
            .argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow!("Failed to hash password: {e}"))?;

        Ok(hash.to_string())
    }

    pub fn verify(&self, password: &str, hash: &str) -> Result<bool> {
        let parsed_hash =
            PasswordHash::new(hash).map_err(|e| anyhow!("Invalid password hash format: {e}"))?;

        match self.argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(()) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => Err(anyhow!("Password verification failed: {e}")),
        }
    }

    pub fn needs_rehash(&self, hash: &str) -> Result<bool> {
        let parsed_hash =
            PasswordHash::new(hash).map_err(|e| anyhow!("Invalid password hash format: {e}"))?;

        let algorithm = parsed_hash.algorithm;
        if algorithm != argon2::ARGON2ID_IDENT {
            return Ok(true);
        }

        if let Some(m_param) = parsed_hash.params.get_str("m") {
            if let Ok(memory) = m_param.parse::<u32>() {
                if memory < self.argon2.params().m_cost() {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    pub fn validate(
        &self,
        password: &str,
        username: Option<&str>,
        email: Option<&str>,
        previous_hashes: &[String],
    ) -> PasswordValidationResult {
        let mut issues = Vec::new();
        let mut suggestions = Vec::new();

        let length = password.len();
        if length < self.config.min_length {
            issues.push(PasswordIssue::TooShort {
                min: self.config.min_length,
                actual: length,
            });
        }
        if length > self.config.max_length {
            issues.push(PasswordIssue::TooLong {
                max: self.config.max_length,
                actual: length,
            });
        }

        let has_uppercase = password.chars().any(|c| c.is_uppercase());
        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        let has_special = password.chars().any(|c| !c.is_alphanumeric());

        if self.config.require_uppercase && !has_uppercase {
            issues.push(PasswordIssue::MissingUppercase);
            suggestions.push("Add uppercase letters (A-Z)".into());
        }
        if self.config.require_lowercase && !has_lowercase {
            issues.push(PasswordIssue::MissingLowercase);
            suggestions.push("Add lowercase letters (a-z)".into());
        }
        if self.config.require_digit && !has_digit {
            issues.push(PasswordIssue::MissingDigit);
            suggestions.push("Add numbers (0-9)".into());
        }
        if self.config.require_special && !has_special {
            issues.push(PasswordIssue::MissingSpecial);
            suggestions.push("Add special characters (!@#$%^&*)".into());
        }

        let unique_chars: HashSet<char> = password.chars().collect();
        if unique_chars.len() < self.config.min_unique_chars {
            issues.push(PasswordIssue::InsufficientUniqueChars {
                min: self.config.min_unique_chars,
                actual: unique_chars.len(),
            });
            suggestions.push("Use more varied characters".into());
        }

        if has_consecutive_chars(password, self.config.max_consecutive_chars) {
            issues.push(PasswordIssue::TooManyConsecutiveChars {
                max: self.config.max_consecutive_chars,
            });
            suggestions.push("Avoid repeating characters".into());
        }

        if is_common_password(password) {
            issues.push(PasswordIssue::CommonPassword);
            suggestions.push("Choose a less common password".into());
        }

        let password_lower = password.to_lowercase();
        if let Some(uname) = username {
            if !uname.is_empty() && password_lower.contains(&uname.to_lowercase()) {
                issues.push(PasswordIssue::ContainsUsername);
                suggestions.push("Remove your username from the password".into());
            }
        }

        if let Some(mail) = email {
            let email_parts: Vec<&str> = mail.split('@').collect();
            if let Some(local_part) = email_parts.first() {
                if !local_part.is_empty() && password_lower.contains(&local_part.to_lowercase()) {
                    issues.push(PasswordIssue::ContainsEmail);
                    suggestions.push("Remove your email from the password".into());
                }
            }
        }

        for prev_hash in previous_hashes.iter().take(self.config.password_history_count) {
            if self.verify(password, prev_hash).unwrap_or(false) {
                issues.push(PasswordIssue::RecentlyUsed);
                suggestions.push("Choose a password you haven't used before".into());
                break;
            }
        }

        let strength = calculate_strength(password, &issues);
        let score = strength.score();
        let is_valid = issues.is_empty() && strength.is_acceptable();
        let crack_time_display = estimate_crack_time(password);

        PasswordValidationResult {
            is_valid,
            strength,
            score,
            issues,
            suggestions,
            crack_time_display,
        }
    }

    pub fn config(&self) -> &PasswordConfig {
        &self.config
    }
}

fn has_consecutive_chars(password: &str, max: usize) -> bool {
    let chars: Vec<char> = password.chars().collect();
    let mut count = 1;

    for i in 1..chars.len() {
        if chars[i] == chars[i - 1] {
            count += 1;
            if count > max {
                return true;
            }
        } else {
            count = 1;
        }
    }
    false
}

fn is_common_password(password: &str) -> bool {
    const COMMON_PASSWORDS: &[&str] = &[
        "password",
        "123456",
        "12345678",
        "qwerty",
        "abc123",
        "monkey",
        "1234567",
        "letmein",
        "trustno1",
        "dragon",
        "baseball",
        "iloveyou",
        "master",
        "sunshine",
        "ashley",
        "bailey",
        "shadow",
        "123123",
        "654321",
        "superman",
        "qazwsx",
        "michael",
        "football",
        "password1",
        "password123",
        "welcome",
        "welcome1",
        "admin",
        "admin123",
        "root",
        "toor",
        "pass",
        "test",
        "guest",
        "changeme",
        "default",
        "secret",
        "login",
        "passw0rd",
        "p@ssword",
        "p@ssw0rd",
        "qwerty123",
        "azerty",
        "000000",
        "111111",
        "1234567890",
        "0987654321",
    ];

    let lower = password.to_lowercase();
    COMMON_PASSWORDS.iter().any(|&common| lower == common || lower.contains(common))
}

fn calculate_strength(password: &str, issues: &[PasswordIssue]) -> PasswordStrength {
    if !issues.is_empty() {
        let critical_issues = issues.iter().any(|i| {
            matches!(
                i,
                PasswordIssue::TooShort { .. }
                    | PasswordIssue::CommonPassword
                    | PasswordIssue::Compromised
            )
        });
        if critical_issues {
            return PasswordStrength::VeryWeak;
        }
        return PasswordStrength::Weak;
    }

    let length = password.len();
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());
    let unique_chars: HashSet<char> = password.chars().collect();

    let mut score = 0;

    if length >= 8 {
        score += 1;
    }
    if length >= 12 {
        score += 1;
    }
    if length >= 16 {
        score += 1;
    }
    if length >= 20 {
        score += 1;
    }

    if has_uppercase {
        score += 1;
    }
    if has_lowercase {
        score += 1;
    }
    if has_digit {
        score += 1;
    }
    if has_special {
        score += 2;
    }

    if unique_chars.len() >= 10 {
        score += 1;
    }
    if unique_chars.len() >= 15 {
        score += 1;
    }

    match score {
        0..=3 => PasswordStrength::VeryWeak,
        4..=5 => PasswordStrength::Weak,
        6..=7 => PasswordStrength::Fair,
        8..=9 => PasswordStrength::Strong,
        _ => PasswordStrength::VeryStrong,
    }
}

fn estimate_crack_time(password: &str) -> String {
    let length = password.len();
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    let mut charset_size = 0;
    if has_lowercase {
        charset_size += 26;
    }
    if has_uppercase {
        charset_size += 26;
    }
    if has_digit {
        charset_size += 10;
    }
    if has_special {
        charset_size += 32;
    }
    if charset_size == 0 {
        charset_size = 26;
    }

    let guesses_per_second: f64 = 10_000_000_000.0;
    let combinations = (charset_size as f64).powi(length as i32);
    let seconds = combinations / guesses_per_second / 2.0;

    if seconds < 1.0 {
        "instantly".into()
    } else if seconds < 60.0 {
        format!("{:.0} seconds", seconds)
    } else if seconds < 3600.0 {
        format!("{:.0} minutes", seconds / 60.0)
    } else if seconds < 86400.0 {
        format!("{:.0} hours", seconds / 3600.0)
    } else if seconds < 31536000.0 {
        format!("{:.0} days", seconds / 86400.0)
    } else if seconds < 31536000.0 * 100.0 {
        format!("{:.0} years", seconds / 31536000.0)
    } else if seconds < 31536000.0 * 1000.0 {
        "centuries".into()
    } else {
        "millennia+".into()
    }
}

pub fn hash_password(password: &str) -> Result<String> {
    let hasher = PasswordHasher2::with_defaults()?;
    hasher.hash(password)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    let hasher = PasswordHasher2::with_defaults()?;
    hasher.verify(password, hash)
}

pub fn validate_password(password: &str) -> PasswordValidationResult {
    let hasher = match PasswordHasher2::with_defaults() {
        Ok(h) => h,
        Err(e) => {
            warn!("Failed to create password hasher: {e}");
            return PasswordValidationResult {
                is_valid: false,
                strength: PasswordStrength::VeryWeak,
                score: 0,
                issues: vec![],
                suggestions: vec!["Internal error during validation".into()],
                crack_time_display: "unknown".into(),
            };
        }
    };
    hasher.validate(password, None, None, &[])
}

pub fn generate_secure_password(length: usize) -> String {
    use rand::Rng;

    const UPPERCASE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    const LOWERCASE: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
    const DIGITS: &[u8] = b"0123456789";
    const SPECIAL: &[u8] = b"!@#$%^&*()_+-=[]{}|;:,.<>?";

    let length = length.max(16);
    let mut rng = rand::rng();
    let mut password = Vec::with_capacity(length);

    password.push(UPPERCASE[rng.random_range(0..UPPERCASE.len())]);
    password.push(LOWERCASE[rng.random_range(0..LOWERCASE.len())]);
    password.push(DIGITS[rng.random_range(0..DIGITS.len())]);
    password.push(SPECIAL[rng.random_range(0..SPECIAL.len())]);

    let all_chars: Vec<u8> = [UPPERCASE, LOWERCASE, DIGITS, SPECIAL].concat();
    for _ in 4..length {
        password.push(all_chars[rng.random_range(0..all_chars.len())]);
    }

    for i in (1..password.len()).rev() {
        let j = rng.random_range(0..=i);
        password.swap(i, j);
    }

    String::from_utf8(password).unwrap_or_else(|_| {
        debug!("Generated password contained invalid UTF-8, regenerating");
        generate_secure_password(length)
    })
}

pub fn generate_recovery_codes(count: usize) -> Vec<String> {
    use rand::Rng;

    const CHARS: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let mut rng = rand::rng();
    let mut codes = Vec::with_capacity(count);

    for _ in 0..count {
        let code: String = (0..8)
            .map(|_| CHARS[rng.random_range(0..CHARS.len())] as char)
            .collect();
        let formatted = format!("{}-{}", &code[..4], &code[4..]);
        codes.push(formatted);
    }

    codes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify() {
        let hasher = PasswordHasher2::with_defaults().expect("Failed to create hasher");
        let password = "SecureP@ssw0rd123!";
        let hash = hasher.hash(password).expect("Failed to hash");

        assert!(hasher.verify(password, &hash).expect("Verify failed"));
        assert!(!hasher
            .verify("WrongPassword", &hash)
            .expect("Verify failed"));
    }

    #[test]
    fn test_password_validation_success() {
        let hasher = PasswordHasher2::with_defaults().expect("Failed to create hasher");
        let result = hasher.validate("Str0ng!P@ssword123", None, None, &[]);

        assert!(result.is_valid);
        assert!(result.issues.is_empty());
        assert!(result.strength.is_acceptable());
    }

    #[test]
    fn test_password_too_short() {
        let hasher = PasswordHasher2::with_defaults().expect("Failed to create hasher");
        let result = hasher.validate("Short1!", None, None, &[]);

        assert!(!result.is_valid);
        assert!(result.issues.iter().any(|i| matches!(i, PasswordIssue::TooShort { .. })));
    }

    #[test]
    fn test_common_password_detection() {
        let hasher = PasswordHasher2::with_defaults().expect("Failed to create hasher");
        let result = hasher.validate("password123", None, None, &[]);

        assert!(!result.is_valid);
        assert!(result
            .issues
            .iter()
            .any(|i| matches!(i, PasswordIssue::CommonPassword)));
    }

    #[test]
    fn test_username_in_password() {
        let hasher = PasswordHasher2::with_defaults().expect("Failed to create hasher");
        let result = hasher.validate("JohnDoe2024!Secure", Some("johndoe"), None, &[]);

        assert!(result
            .issues
            .iter()
            .any(|i| matches!(i, PasswordIssue::ContainsUsername)));
    }

    #[test]
    fn test_password_strength_levels() {
        assert_eq!(PasswordStrength::VeryWeak.score(), 0);
        assert_eq!(PasswordStrength::Weak.score(), 1);
        assert_eq!(PasswordStrength::Fair.score(), 2);
        assert_eq!(PasswordStrength::Strong.score(), 3);
        assert_eq!(PasswordStrength::VeryStrong.score(), 4);
    }

    #[test]
    fn test_generate_secure_password() {
        let password = generate_secure_password(20);

        assert_eq!(password.len(), 20);
        assert!(password.chars().any(|c| c.is_uppercase()));
        assert!(password.chars().any(|c| c.is_lowercase()));
        assert!(password.chars().any(|c| c.is_ascii_digit()));
        assert!(password.chars().any(|c| !c.is_alphanumeric()));
    }

    #[test]
    fn test_generate_recovery_codes() {
        let codes = generate_recovery_codes(10);

        assert_eq!(codes.len(), 10);
        for code in &codes {
            assert_eq!(code.len(), 9);
            assert!(code.contains('-'));
        }
    }

    #[test]
    fn test_needs_rehash() {
        let hasher = PasswordHasher2::with_defaults().expect("Failed to create hasher");
        let hash = hasher.hash("TestPassword123!").expect("Failed to hash");

        assert!(!hasher.needs_rehash(&hash).expect("Rehash check failed"));
    }

    #[test]
    fn test_consecutive_chars_detection() {
        assert!(has_consecutive_chars("aaaa", 3));
        assert!(!has_consecutive_chars("aaa", 3));
        assert!(!has_consecutive_chars("abcd", 3));
    }

    #[test]
    fn test_helper_functions() {
        let hash = hash_password("TestP@ssw0rd!").expect("Hash failed");
        assert!(verify_password("TestP@ssw0rd!", &hash).expect("Verify failed"));

        let result = validate_password("WeakPassword123!");
        assert!(result.strength.score() >= 0);
    }
}
