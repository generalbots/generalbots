use super::mfa_types::*;
use anyhow::{anyhow, Result};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::Utc;
use std::collections::HashMap;
use tracing::{info, debug};

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

        let valid = super::mfa_helpers::verify_totp(&enrollment.secret, code, enrollment.period);

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

        let valid = super::mfa_helpers::verify_totp(&enrollment.secret, code, enrollment.period);
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
