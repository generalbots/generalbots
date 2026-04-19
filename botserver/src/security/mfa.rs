use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

const TOTP_DIGITS: u32 = 6;
const TOTP_PERIOD: u64 = 30;
const TOTP_SECRET_LENGTH: usize = 20;
const RECOVERY_CODE_COUNT: usize = 10;
const RECOVERY_CODE_LENGTH: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MfaMethod {
    Totp,
    WebAuthn,
    EmailOtp,
    SmsOtp,
    RecoveryCode,
}

impl MfaMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Totp => "totp",
            Self::WebAuthn => "webauthn",
            Self::EmailOtp => "email_otp",
            Self::SmsOtp => "sms_otp",
            Self::RecoveryCode => "recovery_code",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Totp => "Authenticator App",
            Self::WebAuthn => "Security Key",
            Self::EmailOtp => "Email Code",
            Self::SmsOtp => "SMS Code",
            Self::RecoveryCode => "Recovery Code",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MfaStatus {
    NotEnrolled,
    Pending,
    Enabled,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaConfig {
    pub require_mfa: bool,
    pub allowed_methods: Vec<MfaMethod>,
    pub totp_issuer: String,
    pub totp_algorithm: TotpAlgorithm,
    pub totp_digits: u32,
    pub totp_period: u64,
    pub otp_expiry_seconds: u64,
    pub max_verification_attempts: u32,
    pub lockout_duration_minutes: u32,
    pub recovery_code_count: usize,
}

impl Default for MfaConfig {
    fn default() -> Self {
        Self {
            require_mfa: false,
            allowed_methods: vec![MfaMethod::Totp, MfaMethod::WebAuthn, MfaMethod::RecoveryCode],
            totp_issuer: "GeneralBots".into(),
            totp_algorithm: TotpAlgorithm::Sha1,
            totp_digits: TOTP_DIGITS,
            totp_period: TOTP_PERIOD,
            otp_expiry_seconds: 300,
            max_verification_attempts: 5,
            lockout_duration_minutes: 15,
            recovery_code_count: RECOVERY_CODE_COUNT,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TotpAlgorithm {
    Sha1,
    Sha256,
    Sha512,
}

impl TotpAlgorithm {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sha1 => "SHA1",
            Self::Sha256 => "SHA256",
            Self::Sha512 => "SHA512",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotpEnrollment {
    pub user_id: Uuid,
    pub secret: String,
    pub issuer: String,
    pub account_name: String,
    pub algorithm: TotpAlgorithm,
    pub digits: u32,
    pub period: u64,
    pub created_at: DateTime<Utc>,
    pub verified: bool,
}

impl TotpEnrollment {
    pub fn new(user_id: Uuid, account_name: &str, config: &MfaConfig) -> Self {
        Self {
            user_id,
            secret: generate_totp_secret(),
            issuer: config.totp_issuer.clone(),
            account_name: account_name.to_string(),
            algorithm: config.totp_algorithm,
            digits: config.totp_digits,
            period: config.totp_period,
            created_at: Utc::now(),
            verified: false,
        }
    }

    pub fn to_uri(&self) -> String {
        let encoded_issuer = urlencoding::encode(&self.issuer);
        let encoded_account = urlencoding::encode(&self.account_name);
        let encoded_secret = base32_encode(&self.secret);

        format!(
            "otpauth://totp/{encoded_issuer}:{encoded_account}?secret={encoded_secret}&issuer={encoded_issuer}&algorithm={}&digits={}&period={}",
            self.algorithm.as_str(),
            self.digits,
            self.period
        )
    }

    pub fn secret_base32(&self) -> String {
        base32_encode(&self.secret)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebAuthnCredential {
    pub id: String,
    pub user_id: Uuid,
    pub credential_id: Vec<u8>,
    pub public_key: Vec<u8>,
    pub counter: u32,
    pub device_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub transports: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebAuthnChallenge {
    pub challenge: Vec<u8>,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub is_registration: bool,
}

impl WebAuthnChallenge {
    pub fn new_registration(user_id: Uuid, expiry_seconds: u64) -> Self {
        let now = Utc::now();
        Self {
            challenge: generate_challenge(),
            user_id,
            created_at: now,
            expires_at: now + chrono::Duration::seconds(expiry_seconds as i64),
            is_registration: true,
        }
    }

    pub fn new_authentication(user_id: Uuid, expiry_seconds: u64) -> Self {
        let now = Utc::now();
        Self {
            challenge: generate_challenge(),
            user_id,
            created_at: now,
            expires_at: now + chrono::Duration::seconds(expiry_seconds as i64),
            is_registration: false,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn challenge_base64(&self) -> String {
        BASE64.encode(&self.challenge)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryCode {
    pub code_hash: String,
    pub created_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
}

impl RecoveryCode {
    pub fn generate_set(count: usize) -> (Vec<String>, Vec<Self>) {
        let mut codes = Vec::with_capacity(count);
        let mut recovery_codes = Vec::with_capacity(count);
        let now = Utc::now();

        for _ in 0..count {
            let code = generate_recovery_code();
            let hash = hash_recovery_code(&code);

            codes.push(code);
            recovery_codes.push(Self {
                code_hash: hash,
                created_at: now,
                used_at: None,
            });
        }

        (codes, recovery_codes)
    }

    pub fn verify(&self, code: &str) -> bool {
        if self.used_at.is_some() {
            return false;
        }
        verify_recovery_code(code, &self.code_hash)
    }

    pub fn mark_used(&mut self) {
        self.used_at = Some(Utc::now());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMfaState {
    pub user_id: Uuid,
    pub status: MfaStatus,
    pub enabled_methods: Vec<MfaMethod>,
    pub totp_enrollment: Option<TotpEnrollment>,
    pub webauthn_credentials: Vec<WebAuthnCredential>,
    pub recovery_codes: Vec<RecoveryCode>,
    pub verification_attempts: u32,
    pub locked_until: Option<DateTime<Utc>>,
    pub last_verified_at: Option<DateTime<Utc>>,
    pub preferred_method: Option<MfaMethod>,
}

impl UserMfaState {
    pub fn new(user_id: Uuid) -> Self {
        Self {
            user_id,
            status: MfaStatus::NotEnrolled,
            enabled_methods: Vec::new(),
            totp_enrollment: None,
            webauthn_credentials: Vec::new(),
            recovery_codes: Vec::new(),
            verification_attempts: 0,
            locked_until: None,
            last_verified_at: None,
            preferred_method: None,
        }
    }

    pub fn is_enrolled(&self) -> bool {
        !self.enabled_methods.is_empty()
    }

    pub fn is_locked(&self) -> bool {
        if let Some(locked_until) = self.locked_until {
            Utc::now() < locked_until
        } else {
            false
        }
    }

    pub fn has_method(&self, method: MfaMethod) -> bool {
        self.enabled_methods.contains(&method)
    }

    pub fn available_recovery_codes(&self) -> usize {
        self.recovery_codes.iter().filter(|c| c.used_at.is_none()).count()
    }

    pub fn record_attempt(&mut self, success: bool, lockout_threshold: u32, lockout_minutes: u32) {
        if success {
            self.verification_attempts = 0;
            self.locked_until = None;
            self.last_verified_at = Some(Utc::now());
        } else {
            self.verification_attempts += 1;
            if self.verification_attempts >= lockout_threshold {
                self.locked_until =
                    Some(Utc::now() + chrono::Duration::minutes(lockout_minutes as i64));
                warn!(
                    "User {} locked out due to {} failed MFA attempts",
                    self.user_id, self.verification_attempts
                );
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtpChallenge {
    pub id: String,
    pub user_id: Uuid,
    pub method: MfaMethod,
    pub code_hash: String,
    pub destination: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub verified: bool,
}

impl OtpChallenge {
    pub fn new(user_id: Uuid, method: MfaMethod, destination: &str, expiry_seconds: u64) -> (String, Self) {
        let code = generate_otp_code();
        let now = Utc::now();

        let challenge = Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            method,
            code_hash: hash_otp_code(&code),
            destination: mask_destination(destination, method),
            created_at: now,
            expires_at: now + chrono::Duration::seconds(expiry_seconds as i64),
            verified: false,
        };

        (code, challenge)
    }

    pub fn verify(&self, code: &str) -> bool {
        if self.verified || self.is_expired() {
            return false;
        }
        verify_otp_code(code, &self.code_hash)
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

pub struct MfaManager {
    config: MfaConfig,
    user_states: Arc<RwLock<HashMap<Uuid, UserMfaState>>>,
    pending_challenges: Arc<RwLock<HashMap<String, WebAuthnChallenge>>>,
    otp_challenges: Arc<RwLock<HashMap<String, OtpChallenge>>>,
}

impl MfaManager {
    pub fn new(config: MfaConfig) -> Self {
        Self {
            config,
            user_states: Arc::new(RwLock::new(HashMap::new())),
            pending_challenges: Arc::new(RwLock::new(HashMap::new())),
            otp_challenges: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_user_state(&self, user_id: Uuid) -> UserMfaState {
        let states = self.user_states.read().await;
        states.get(&user_id).cloned().unwrap_or_else(|| UserMfaState::new(user_id))
    }

    pub async fn start_totp_enrollment(&self, user_id: Uuid, account_name: &str) -> Result<TotpEnrollment> {
        if !self.config.allowed_methods.contains(&MfaMethod::Totp) {
            return Err(anyhow!("TOTP is not an allowed MFA method"));
        }

        let enrollment = TotpEnrollment::new(user_id, account_name, &self.config);

        let mut states = self.user_states.write().await;
        let state = states.entry(user_id).or_insert_with(|| UserMfaState::new(user_id));
        state.totp_enrollment = Some(enrollment.clone());
        state.status = MfaStatus::Pending;

        info!("Started TOTP enrollment for user {user_id}");
        Ok(enrollment)
    }

    pub async fn verify_totp_enrollment(&self, user_id: Uuid, code: &str) -> Result<bool> {
        let mut states = self.user_states.write().await;
        let state = states.get_mut(&user_id).ok_or_else(|| anyhow!("User not found"))?;

        if state.is_locked() {
            return Err(anyhow!("Account is temporarily locked"));
        }

        let enrollment = state
            .totp_enrollment
            .as_ref()
            .ok_or_else(|| anyhow!("No pending TOTP enrollment"))?;

        let valid = verify_totp(&enrollment.secret, code, enrollment.period);

        if valid {
            if let Some(ref mut e) = state.totp_enrollment {
                e.verified = true;
            }
            state.status = MfaStatus::Enabled;
            if !state.enabled_methods.contains(&MfaMethod::Totp) {
                state.enabled_methods.push(MfaMethod::Totp);
            }
            state.record_attempt(true, self.config.max_verification_attempts, self.config.lockout_duration_minutes);

            let (codes, recovery_codes) = RecoveryCode::generate_set(self.config.recovery_code_count);
            state.recovery_codes = recovery_codes;
            if !state.enabled_methods.contains(&MfaMethod::RecoveryCode) {
                state.enabled_methods.push(MfaMethod::RecoveryCode);
            }

            info!("TOTP enrollment verified for user {user_id}");
            debug!("Generated {} recovery codes for user {user_id}", codes.len());
        } else {
            state.record_attempt(false, self.config.max_verification_attempts, self.config.lockout_duration_minutes);
        }

        Ok(valid)
    }

    pub async fn verify_totp(&self, user_id: Uuid, code: &str) -> Result<bool> {
        let mut states = self.user_states.write().await;
        let state = states.get_mut(&user_id).ok_or_else(|| anyhow!("User not found"))?;

        if state.is_locked() {
            return Err(anyhow!("Account is temporarily locked"));
        }

        if !state.has_method(MfaMethod::Totp) {
            return Err(anyhow!("TOTP is not enabled for this user"));
        }

        let enrollment = state
            .totp_enrollment
            .as_ref()
            .ok_or_else(|| anyhow!("TOTP not configured"))?;

        if !enrollment.verified {
            return Err(anyhow!("TOTP enrollment not verified"));
        }

        let valid = verify_totp(&enrollment.secret, code, enrollment.period);
        state.record_attempt(valid, self.config.max_verification_attempts, self.config.lockout_duration_minutes);

        Ok(valid)
    }

    pub async fn start_webauthn_registration(
        &self,
        user_id: Uuid,
    ) -> Result<WebAuthnChallenge> {
        if !self.config.allowed_methods.contains(&MfaMethod::WebAuthn) {
            return Err(anyhow!("WebAuthn is not an allowed MFA method"));
        }

        let challenge = WebAuthnChallenge::new_registration(user_id, self.config.otp_expiry_seconds);

        let mut challenges = self.pending_challenges.write().await;
        challenges.insert(challenge.challenge_base64(), challenge.clone());

        info!("Started WebAuthn registration for user {user_id}");
        Ok(challenge)
    }

    pub async fn complete_webauthn_registration(
        &self,
        user_id: Uuid,
        challenge_response: &str,
        credential_id: Vec<u8>,
        public_key: Vec<u8>,
        device_name: Option<String>,
    ) -> Result<WebAuthnCredential> {
        let mut challenges = self.pending_challenges.write().await;
        let challenge = challenges
            .remove(challenge_response)
            .ok_or_else(|| anyhow!("Challenge not found or expired"))?;

        if challenge.is_expired() {
            return Err(anyhow!("Challenge expired"));
        }

        if challenge.user_id != user_id {
            return Err(anyhow!("Challenge user mismatch"));
        }

        if !challenge.is_registration {
            return Err(anyhow!("Not a registration challenge"));
        }

        let credential = WebAuthnCredential {
            id: Uuid::new_v4().to_string(),
            user_id,
            credential_id,
            public_key,
            counter: 0,
            device_name,
            created_at: Utc::now(),
            last_used_at: None,
            transports: Vec::new(),
        };

        let mut states = self.user_states.write().await;
        let state = states.entry(user_id).or_insert_with(|| UserMfaState::new(user_id));
        state.webauthn_credentials.push(credential.clone());
        state.status = MfaStatus::Enabled;
        if !state.enabled_methods.contains(&MfaMethod::WebAuthn) {
            state.enabled_methods.push(MfaMethod::WebAuthn);
        }

        if state.recovery_codes.is_empty() {
            let (_, recovery_codes) = RecoveryCode::generate_set(self.config.recovery_code_count);
            state.recovery_codes = recovery_codes;
            if !state.enabled_methods.contains(&MfaMethod::RecoveryCode) {
                state.enabled_methods.push(MfaMethod::RecoveryCode);
            }
        }

        info!("WebAuthn registration completed for user {user_id}");
        Ok(credential)
    }

    pub async fn start_webauthn_authentication(&self, user_id: Uuid) -> Result<WebAuthnChallenge> {
        let states = self.user_states.read().await;
        let state = states.get(&user_id).ok_or_else(|| anyhow!("User not found"))?;

        if !state.has_method(MfaMethod::WebAuthn) {
            return Err(anyhow!("WebAuthn is not enabled for this user"));
        }

        if state.webauthn_credentials.is_empty() {
            return Err(anyhow!("No WebAuthn credentials registered"));
        }

        let challenge = WebAuthnChallenge::new_authentication(user_id, self.config.otp_expiry_seconds);

        drop(states);

        let mut challenges = self.pending_challenges.write().await;
        challenges.insert(challenge.challenge_base64(), challenge.clone());

        Ok(challenge)
    }

    pub async fn verify_webauthn(
        &self,
        user_id: Uuid,
        challenge_response: &str,
        credential_id: &[u8],
        _authenticator_data: &[u8],
        _signature: &[u8],
        new_counter: u32,
    ) -> Result<bool> {
        let mut challenges = self.pending_challenges.write().await;
        let challenge = challenges
            .remove(challenge_response)
            .ok_or_else(|| anyhow!("Challenge not found or expired"))?;

        if challenge.is_expired() {
            return Err(anyhow!("Challenge expired"));
        }

        if challenge.user_id != user_id {
            return Err(anyhow!("Challenge user mismatch"));
        }

        if challenge.is_registration {
            return Err(anyhow!("Not an authentication challenge"));
        }

        drop(challenges);

        let mut states = self.user_states.write().await;
        let state = states.get_mut(&user_id).ok_or_else(|| anyhow!("User not found"))?;

        if state.is_locked() {
            return Err(anyhow!("Account is temporarily locked"));
        }

        let credential = state
            .webauthn_credentials
            .iter_mut()
            .find(|c| c.credential_id == credential_id)
            .ok_or_else(|| anyhow!("Credential not found"))?;

        if new_counter <= credential.counter {
            state.record_attempt(false, self.config.max_verification_attempts, self.config.lockout_duration_minutes);
            return Err(anyhow!("Invalid counter - possible replay attack"));
        }

        credential.counter = new_counter;
        credential.last_used_at = Some(Utc::now());
        state.record_attempt(true, self.config.max_verification_attempts, self.config.lockout_duration_minutes);

        Ok(true)
    }

    pub async fn send_email_otp(&self, user_id: Uuid, email: &str) -> Result<OtpChallenge> {
        if !self.config.allowed_methods.contains(&MfaMethod::EmailOtp) {
            return Err(anyhow!("Email OTP is not an allowed MFA method"));
        }

        let (code, challenge) = OtpChallenge::new(
            user_id,
            MfaMethod::EmailOtp,
            email,
            self.config.otp_expiry_seconds,
        );

        let mut otp_challenges = self.otp_challenges.write().await;
        otp_challenges.insert(challenge.id.clone(), challenge.clone());

        info!("Email OTP challenge created for user {user_id}, code: {code}");

        Ok(challenge)
    }

    pub async fn verify_email_otp(&self, user_id: Uuid, challenge_id: &str, code: &str) -> Result<bool> {
        let mut states = self.user_states.write().await;
        let state = states.entry(user_id).or_insert_with(|| UserMfaState::new(user_id));

        if state.is_locked() {
            return Err(anyhow!("Account is temporarily locked"));
        }

        drop(states);

        let mut otp_challenges = self.otp_challenges.write().await;
        let challenge = otp_challenges
            .get_mut(challenge_id)
            .ok_or_else(|| anyhow!("Challenge not found"))?;

        if challenge.user_id != user_id {
            return Err(anyhow!("Challenge user mismatch"));
        }

        let valid = challenge.verify(code);

        if valid {
            challenge.verified = true;
        }

        drop(otp_challenges);

        let mut states = self.user_states.write().await;
        let state = states.entry(user_id).or_insert_with(|| UserMfaState::new(user_id));
        state.record_attempt(valid, self.config.max_verification_attempts, self.config.lockout_duration_minutes);

        Ok(valid)
    }

    pub async fn verify_recovery_code(&self, user_id: Uuid, code: &str) -> Result<bool> {
        let mut states = self.user_states.write().await;
        let state = states.get_mut(&user_id).ok_or_else(|| anyhow!("User not found"))?;

        if state.is_locked() {
            return Err(anyhow!("Account is temporarily locked"));
        }

        if !state.has_method(MfaMethod::RecoveryCode) {
            return Err(anyhow!("Recovery codes not enabled"));
        }

        let normalized_code = code.replace('-', "").to_uppercase();

        for recovery_code in &mut state.recovery_codes {
            if recovery_code.verify(&normalized_code) {
                recovery_code.mark_used();
                state.record_attempt(true, self.config.max_verification_attempts, self.config.lockout_duration_minutes);
                info!("Recovery code used for user {user_id}");
                return Ok(true);
            }
        }

        state.record_attempt(false, self.config.max_verification_attempts, self.config.lockout_duration_minutes);
        Ok(false)
    }

    pub async fn regenerate_recovery_codes(&self, user_id: Uuid) -> Result<Vec<String>> {
        let mut states = self.user_states.write().await;
        let state = states.get_mut(&user_id).ok_or_else(|| anyhow!("User not found"))?;

        if !state.is_enrolled() {
            return Err(anyhow!("MFA not enrolled"));
        }

        let (codes, recovery_codes) = RecoveryCode::generate_set(self.config.recovery_code_count);
        state.recovery_codes = recovery_codes;

        if !state.enabled_methods.contains(&MfaMethod::RecoveryCode) {
            state.enabled_methods.push(MfaMethod::RecoveryCode);
        }

        info!("Regenerated recovery codes for user {user_id}");
        Ok(codes)
    }

    pub async fn disable_mfa(&self, user_id: Uuid, method: MfaMethod) -> Result<()> {
        let mut states = self.user_states.write().await;
        let state = states.get_mut(&user_id).ok_or_else(|| anyhow!("User not found"))?;

        match method {
            MfaMethod::Totp => {
                state.totp_enrollment = None;
            }
            MfaMethod::WebAuthn => {
                state.webauthn_credentials.clear();
            }
            MfaMethod::RecoveryCode => {
                state.recovery_codes.clear();
            }
            _ => {}
        }

        state.enabled_methods.retain(|m| *m != method);

        if state.enabled_methods.is_empty() || state.enabled_methods == vec![MfaMethod::RecoveryCode] {
            state.status = MfaStatus::Disabled;
            state.recovery_codes.clear();
            state.enabled_methods.clear();
        }

        info!("Disabled MFA method {:?} for user {user_id}", method);
        Ok(())
    }

    pub async fn disable_all_mfa(&self, user_id: Uuid) -> Result<()> {
        let mut states = self.user_states.write().await;
        let state = states.get_mut(&user_id).ok_or_else(|| anyhow!("User not found"))?;

        state.totp_enrollment = None;
        state.webauthn_credentials.clear();
        state.recovery_codes.clear();
        state.enabled_methods.clear();
        state.status = MfaStatus::Disabled;

        info!("Disabled all MFA for user {user_id}");
        Ok(())
    }

    pub fn config(&self) -> &MfaConfig {
        &self.config
    }

    pub fn is_mfa_required(&self) -> bool {
        self.config.require_mfa
    }
}

fn generate_totp_secret() -> String {
    let mut rng = rand::rng();
    let secret: Vec<u8> = (0..TOTP_SECRET_LENGTH).map(|_| rng.random()).collect();
    hex::encode(secret)
}

fn generate_challenge() -> Vec<u8> {
    let mut rng = rand::rng();
    let challenge: Vec<u8> = (0..32).map(|_| rng.random()).collect();
    challenge
}

fn generate_recovery_code() -> String {
    const CHARS: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let mut rng = rand::rng();

    let code: String = (0..RECOVERY_CODE_LENGTH)
        .map(|_| CHARS[rng.random_range(0..CHARS.len())] as char)
        .collect();

    format!("{}-{}", &code[..4], &code[4..])
}

fn generate_otp_code() -> String {
    let mut rng = rand::rng();
    format!("{:06}", rng.random_range(0..1_000_000u32))
}

fn hash_recovery_code(code: &str) -> String {
    use sha2::Digest;
    let normalized = code.replace('-', "").to_uppercase();
    let hash = Sha256::digest(normalized.as_bytes());
    hex::encode(hash)
}

fn verify_recovery_code(code: &str, hash: &str) -> bool {
    let code_hash = hash_recovery_code(code);
    constant_time_compare(&code_hash, hash)
}

fn hash_otp_code(code: &str) -> String {
    use sha2::Digest;
    let hash = Sha256::digest(code.as_bytes());
    hex::encode(hash)
}

fn verify_otp_code(code: &str, hash: &str) -> bool {
    let code_hash = hash_otp_code(code);
    constant_time_compare(&code_hash, hash)
}

fn constant_time_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        result |= x ^ y;
    }
    result == 0
}

fn base32_encode(data: &str) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let bytes = data.as_bytes();
    let mut result = String::new();

    let mut buffer: u64 = 0;
    let mut bits_left = 0;

    for &byte in bytes {
        buffer = (buffer << 8) | (byte as u64);
        bits_left += 8;

        while bits_left >= 5 {
            bits_left -= 5;
            let index = ((buffer >> bits_left) & 0x1F) as usize;
            result.push(ALPHABET[index] as char);
        }
    }

    if bits_left > 0 {
        let index = ((buffer << (5 - bits_left)) & 0x1F) as usize;
        result.push(ALPHABET[index] as char);
    }

    result
}

fn verify_totp(secret: &str, code: &str, period: u64) -> bool {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let counter = now / period;

    for offset in [-1i64, 0, 1] {
        let check_counter = (counter as i64 + offset) as u64;
        if let Ok(expected) = generate_totp_code(secret, check_counter) {
            if constant_time_compare(&expected, code) {
                return true;
            }
        }
    }

    false
}

fn generate_totp_code(secret: &str, counter: u64) -> Result<String> {
    let secret_bytes = hex::decode(secret).map_err(|e| anyhow!("Invalid secret: {e}"))?;

    let mut mac = HmacSha256::new_from_slice(&secret_bytes)
        .map_err(|e| anyhow!("HMAC error: {e}"))?;

    mac.update(&counter.to_be_bytes());
    let result = mac.finalize().into_bytes();

    let offset = (result[result.len() - 1] & 0x0F) as usize;
    if offset + 4 > result.len() {
        return Err(anyhow!("Invalid HMAC result length"));
    }
    let code = u32::from_be_bytes([
        result[offset] & 0x7F,
        result[offset + 1],
        result[offset + 2],
        result[offset + 3],
    ]);

    let otp = code % 10u32.pow(TOTP_DIGITS);
    Ok(format!("{:0width$}", otp, width = TOTP_DIGITS as usize))
}

fn mask_destination(destination: &str, method: MfaMethod) -> String {
    match method {
        MfaMethod::EmailOtp => {
            if let Some(at_pos) = destination.find('@') {
                let local = &destination[..at_pos];
                let domain = &destination[at_pos..];
                if local.len() <= 2 {
                    format!("{}***{}", &local[..1], domain)
                } else {
                    format!("{}***{}{}", &local[..2], &local[local.len()-1..], domain)
                }
            } else {
                "***@***".into()
            }
        }
        MfaMethod::SmsOtp => {
            let digits: String = destination.chars().filter(|c| c.is_ascii_digit()).collect();
            if digits.len() <= 4 {
                "***".into()
            } else {
                format!("***{}", &digits[digits.len()-4..])
            }
        }
        _ => "***".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_totp_enrollment_creation() {
        let config = MfaConfig::default();
        let user_id = Uuid::new_v4();
        let enrollment = TotpEnrollment::new(user_id, "test@example.com", &config);

        assert_eq!(enrollment.user_id, user_id);
        assert!(!enrollment.secret.is_empty());
        assert!(!enrollment.verified);
    }

    #[test]
    fn test_totp_uri_generation() {
        let config = MfaConfig::default();
        let user_id = Uuid::new_v4();
        let enrollment = TotpEnrollment::new(user_id, "test@example.com", &config);

        let uri = enrollment.to_uri();
        assert!(uri.starts_with("otpauth://totp/"));
        assert!(uri.contains("secret="));
        assert!(uri.contains("issuer="));
    }

    #[test]
    fn test_recovery_code_generation() {
        let (codes, recovery_codes) = RecoveryCode::generate_set(10);

        assert_eq!(codes.len(), 10);
        assert_eq!(recovery_codes.len(), 10);

        for code in &codes {
            assert!(code.contains('-'));
            assert_eq!(code.len(), 9);
        }
    }

    #[test]
    fn test_recovery_code_verification() {
        let (codes, mut recovery_codes) = RecoveryCode::generate_set(1);

        assert!(recovery_codes[0].verify(&codes[0]));
        assert!(!recovery_codes[0].verify("WRONG-CODE"));

        recovery_codes[0].mark_used();
        assert!(!recovery_codes[0].verify(&codes[0]));
    }

    #[test]
    fn test_user_mfa_state() {
        let user_id = Uuid::new_v4();
        let state = UserMfaState::new(user_id);

        assert_eq!(state.user_id, user_id);
        assert_eq!(state.status, MfaStatus::NotEnrolled);
        assert!(!state.is_enrolled());
        assert!(!state.is_locked());
    }

    #[test]
    fn test_otp_challenge_creation() {
        let user_id = Uuid::new_v4();
        let (code, challenge) = OtpChallenge::new(user_id, MfaMethod::EmailOtp, "test@example.com", 300);

        assert_eq!(code.len(), 6);
        assert_eq!(challenge.user_id, user_id);
        assert!(!challenge.is_expired());
    }

    #[test]
    fn test_webauthn_challenge() {
        let user_id = Uuid::new_v4();
        let challenge = WebAuthnChallenge::new_registration(user_id, 300);

        assert_eq!(challenge.user_id, user_id);
        assert!(challenge.is_registration);
        assert!(!challenge.is_expired());
        assert!(!challenge.challenge_base64().is_empty());
    }

    #[test]
    fn test_mfa_method_names() {
        assert_eq!(MfaMethod::Totp.as_str(), "totp");
        assert_eq!(MfaMethod::WebAuthn.as_str(), "webauthn");
        assert_eq!(MfaMethod::Totp.display_name(), "Authenticator App");
        assert_eq!(MfaMethod::WebAuthn.display_name(), "Security Key");
    }

    #[test]
    fn test_mask_email() {
        let masked = mask_destination("test@example.com", MfaMethod::EmailOtp);
        assert!(masked.contains("***"));
        assert!(masked.contains("@example.com"));
    }

    #[test]
    fn test_mask_phone() {
        let masked = mask_destination("+1234567890", MfaMethod::SmsOtp);
        assert!(masked.contains("***"));
        assert!(masked.ends_with("7890"));
    }

    #[tokio::test]
    async fn test_mfa_manager_creation() {
        let config = MfaConfig::default();
        let manager = MfaManager::new(config);

        let user_id = Uuid::new_v4();
        let state = manager.get_user_state(user_id).await;

        assert_eq!(state.user_id, user_id);
        assert!(!state.is_enrolled());
    }

    #[tokio::test]
    async fn test_totp_enrollment_flow() {
        let config = MfaConfig::default();
        let manager = MfaManager::new(config);
        let user_id = Uuid::new_v4();

        let enrollment = manager
            .start_totp_enrollment(user_id, "test@example.com")
            .await
            .expect("Enrollment failed");

        assert_eq!(enrollment.user_id, user_id);
        assert!(!enrollment.verified);

        let state = manager.get_user_state(user_id).await;
        assert_eq!(state.status, MfaStatus::Pending);
    }

    #[test]
    fn test_constant_time_compare() {
        assert!(constant_time_compare("abc123", "abc123"));
        assert!(!constant_time_compare("abc123", "abc124"));
        assert!(!constant_time_compare("abc", "abcd"));
    }

    #[test]
    fn test_base32_encode() {
        let encoded = base32_encode("test");
        assert!(!encoded.is_empty());
        assert!(encoded.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()));
    }
}
