use argon2::PasswordVerifier;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{DateTime, Duration, Utc};
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Bytea, Nullable, Text, Timestamptz, Uuid as DieselUuid};
use log::{error, info, warn};
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::shared::state::AppState;
use crate::shared::utils::DbPool;

const CHALLENGE_TIMEOUT_SECONDS: i64 = 300;
const PASSKEY_NAME_MAX_LENGTH: usize = 64;

#[derive(Debug, Clone)]
struct FallbackAttemptTracker {
    attempts: u32,
    locked_until: Option<DateTime<Utc>>,
}

pub struct PasskeyCredential {
    pub id: String,
    pub user_id: Uuid,
    pub credential_id: Vec<u8>,
    pub public_key: Vec<u8>,
    pub counter: u32,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub aaguid: Option<Vec<u8>>,
    pub transports: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasskeyChallenge {
    pub challenge: Vec<u8>,
    pub user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub operation: ChallengeOperation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChallengeOperation {
    Registration,
    Authentication,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationOptionsRequest {
    pub user_id: Uuid,
    pub username: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistrationOptions {
    pub challenge: String,
    pub rp: RelyingParty,
    pub user: UserEntity,
    pub pub_key_cred_params: Vec<PubKeyCredParam>,
    pub timeout: u32,
    pub attestation: String,
    pub authenticator_selection: AuthenticatorSelection,
    pub exclude_credentials: Vec<CredentialDescriptor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelyingParty {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserEntity {
    pub id: String,
    pub name: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PubKeyCredParam {
    #[serde(rename = "type")]
    pub cred_type: String,
    pub alg: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticatorSelection {
    pub authenticator_attachment: Option<String>,
    pub resident_key: String,
    pub require_resident_key: bool,
    pub user_verification: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialDescriptor {
    #[serde(rename = "type")]
    pub cred_type: String,
    pub id: String,
    pub transports: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistrationResponse {
    pub id: String,
    pub raw_id: String,
    pub response: AuthenticatorAttestationResponse,
    #[serde(rename = "type")]
    pub cred_type: String,
    pub client_extension_results: Option<HashMap<String, serde_json::Value>>,
    pub authenticator_attachment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticatorAttestationResponse {
    pub client_data_json: String,
    pub attestation_object: String,
    pub transports: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticationOptionsRequest {
    pub username: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticationOptions {
    pub challenge: String,
    pub timeout: u32,
    pub rp_id: String,
    pub allow_credentials: Vec<CredentialDescriptor>,
    pub user_verification: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticationResponse {
    pub id: String,
    pub raw_id: String,
    pub response: AuthenticatorAssertionResponse,
    #[serde(rename = "type")]
    pub cred_type: String,
    pub client_extension_results: Option<HashMap<String, serde_json::Value>>,
    pub authenticator_attachment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticatorAssertionResponse {
    pub client_data_json: String,
    pub authenticator_data: String,
    pub signature: String,
    pub user_handle: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasskeyInfo {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenamePasskeyRequest {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub success: bool,
    pub user_id: Option<Uuid>,
    pub credential_id: Option<String>,
    pub error: Option<String>,
    pub used_fallback: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationResult {
    pub success: bool,
    pub credential_id: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordFallbackRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordFallbackResponse {
    pub success: bool,
    pub user_id: Option<Uuid>,
    pub token: Option<String>,
    pub error: Option<String>,
    pub passkey_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackConfig {
    pub enabled: bool,
    pub require_additional_verification: bool,
    pub max_fallback_attempts: u32,
    pub lockout_duration_seconds: u64,
    pub prompt_passkey_setup: bool,
}

impl Default for FallbackConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            require_additional_verification: false,
            max_fallback_attempts: 5,
            lockout_duration_seconds: 900, // 15 minutes
            prompt_passkey_setup: true,
        }
    }
}

#[derive(QueryableByName)]
struct PasskeyRow {
    #[diesel(sql_type = Text)]
    id: String,
    #[diesel(sql_type = DieselUuid)]
    user_id: Uuid,
    #[diesel(sql_type = Bytea)]
    credential_id: Vec<u8>,
    #[diesel(sql_type = Bytea)]
    public_key: Vec<u8>,
    #[diesel(sql_type = BigInt)]
    counter: i64,
    #[diesel(sql_type = Text)]
    name: String,
    #[diesel(sql_type = Timestamptz)]
    created_at: DateTime<Utc>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    last_used_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = Nullable<Bytea>)]
    aaguid: Option<Vec<u8>>,
    #[diesel(sql_type = Nullable<Text>)]
    transports: Option<String>,
}

pub struct PasskeyService {
    pool: Arc<diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>>,
    rp_id: String,
    rp_name: String,
    rp_origin: String,
    challenges: Arc<RwLock<HashMap<String, PasskeyChallenge>>>,
    rng: SystemRandom,
    fallback_config: FallbackConfig,
    fallback_attempts: Arc<RwLock<HashMap<String, FallbackAttemptTracker>>>,
}

impl PasskeyService {
    pub fn new(
        pool: DbPool,
        rp_id: String,
        rp_name: String,
        rp_origin: String,
    ) -> Self {
        Self {
            pool: Arc::new(pool),
            rp_id,
            rp_name,
            rp_origin,
            challenges: Arc::new(RwLock::new(HashMap::new())),
            rng: SystemRandom::new(),
            fallback_config: FallbackConfig::default(),
            fallback_attempts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_fallback_config(
        pool: DbPool,
        rp_id: String,
        rp_name: String,
        rp_origin: String,
        fallback_config: FallbackConfig,
    ) -> Self {
        Self {
            pool: Arc::new(pool),
            rp_id,
            rp_name,
            rp_origin,
            challenges: Arc::new(RwLock::new(HashMap::new())),
            rng: SystemRandom::new(),
            fallback_config,
            fallback_attempts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn user_has_passkeys(&self, username: &str) -> Result<bool, PasskeyError> {
        let passkeys = self.get_passkeys_by_username(username)?;
        Ok(!passkeys.is_empty())
    }

    pub async fn authenticate_with_password_fallback(
        &self,
        request: &PasswordFallbackRequest,
    ) -> Result<PasswordFallbackResponse, PasskeyError> {
        if !self.fallback_config.enabled {
            return Ok(PasswordFallbackResponse {
                success: false,
                user_id: None,
                token: None,
                error: Some("Password fallback is disabled".to_string()),
                passkey_available: false,
            });
        }

        // Check if user is locked out
        if self.is_user_locked_out(&request.username).await {
            return Ok(PasswordFallbackResponse {
                success: false,
                user_id: None,
                token: None,
                error: Some("Account temporarily locked due to too many failed attempts".to_string()),
                passkey_available: false,
            });
        }

        // Verify password against database
        let verification_result = self.verify_password(&request.username, &request.password).await;

        match verification_result {
            Ok(user_id) => {
                // Clear failed attempts on successful login
                self.clear_fallback_attempts(&request.username).await;

                // Check if user has passkeys available
                let passkey_available = self.user_has_passkeys(&request.username).unwrap_or(false);

                // Generate session token
                let token = self.generate_session_token(&user_id);

                Ok(PasswordFallbackResponse {
                    success: true,
                    user_id: Some(user_id),
                    token: Some(token),
                    error: None,
                    passkey_available,
                })
            }
            Err(e) => {
                // Track failed attempt
                self.track_fallback_attempt(&request.username).await;

                Ok(PasswordFallbackResponse {
                    success: false,
                    user_id: None,
                    token: None,
                    error: Some(e.to_string()),
                    passkey_available: false,
                })
            }
        }
    }

    async fn is_user_locked_out(&self, username: &str) -> bool {
        let attempts = self.fallback_attempts.read().await;
        if let Some(tracker) = attempts.get(username) {
            if let Some(locked_until) = tracker.locked_until {
                return Utc::now() < locked_until;
            }
        }
        false
    }

    async fn track_fallback_attempt(&self, username: &str) {
        let mut attempts = self.fallback_attempts.write().await;
        let now = Utc::now();

        let tracker = attempts.entry(username.to_string()).or_insert(FallbackAttemptTracker {
            attempts: 0,
            locked_until: None,
        });

        tracker.attempts += 1;

        // Check if we should lock out the user
        if tracker.attempts >= self.fallback_config.max_fallback_attempts {
            tracker.locked_until = Some(
                now + chrono::Duration::seconds(self.fallback_config.lockout_duration_seconds as i64)
            );
        }
    }

    async fn clear_fallback_attempts(&self, username: &str) {
        let mut attempts = self.fallback_attempts.write().await;
        attempts.remove(username);
    }

    async fn verify_password(&self, username: &str, password: &str) -> Result<Uuid, PasskeyError> {
        let mut conn = self.pool.get().map_err(|_| PasskeyError::DatabaseError)?;

        #[derive(QueryableByName)]
        struct UserPasswordRow {
            #[diesel(sql_type = DieselUuid)]
            id: Uuid,
            #[diesel(sql_type = Nullable<Text>)]
            password_hash: Option<String>,
        }

        let result: Option<UserPasswordRow> = diesel::sql_query(
            "SELECT id, password_hash FROM users WHERE username = $1 OR email = $1"
        )
        .bind::<Text, _>(username)
        .get_result::<UserPasswordRow>(&mut conn)
        .optional()
        .map_err(|_| PasskeyError::DatabaseError)?;

        match result {
            Some(row) => {
                if let Some(hash) = row.password_hash {
                    let parsed_hash = argon2::PasswordHash::new(&hash)
                        .map_err(|_| PasskeyError::InvalidCredentialId)?;

                    if argon2::Argon2::default()
                        .verify_password(password.as_bytes(), &parsed_hash)
                        .is_ok()
                    {
                        return Ok(row.id);
                    }
                }
                Err(PasskeyError::InvalidCredentialId)
            }
            None => Err(PasskeyError::InvalidCredentialId),
        }
    }

    fn generate_session_token(&self, user_id: &Uuid) -> String {
        let random_bytes: [u8; 32] = rand::random();
        let token = base64::Engine::encode(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
            random_bytes
        );
        format!("{}:{}", user_id, token)
    }

    pub fn should_offer_password_fallback(&self, username: &str) -> Result<bool, PasskeyError> {
        if !self.fallback_config.enabled {
            return Ok(false);
        }

        let has_passkeys = self.user_has_passkeys(username)?;
        Ok(!has_passkeys || self.fallback_config.enabled)
    }

    pub fn get_fallback_config(&self) -> &FallbackConfig {
        &self.fallback_config
    }

    pub fn set_fallback_config(&mut self, config: FallbackConfig) {
        self.fallback_config = config;
    }

    pub async fn generate_registration_options(
        &self,
        request: RegistrationOptionsRequest,
    ) -> Result<RegistrationOptions, PasskeyError> {
        let challenge = self.generate_challenge()?;
        let challenge_b64 = URL_SAFE_NO_PAD.encode(&challenge);

        let passkey_challenge = PasskeyChallenge {
            challenge: challenge.clone(),
            user_id: Some(request.user_id),
            created_at: Utc::now(),
            operation: ChallengeOperation::Registration,
        };

        {
            let mut challenges = self.challenges.write().await;
            challenges.insert(challenge_b64.clone(), passkey_challenge);
        }

        let existing_credentials = self.get_user_passkeys(request.user_id)?;
        let exclude_credentials: Vec<CredentialDescriptor> = existing_credentials
            .into_iter()
            .map(|pk| CredentialDescriptor {
                cred_type: "public-key".to_string(),
                id: URL_SAFE_NO_PAD.encode(&pk.credential_id),
                transports: Some(pk.transports),
            })
            .collect();

        let user_id_b64 = URL_SAFE_NO_PAD.encode(request.user_id.as_bytes());

        Ok(RegistrationOptions {
            challenge: challenge_b64,
            rp: RelyingParty {
                id: self.rp_id.clone(),
                name: self.rp_name.clone(),
            },
            user: UserEntity {
                id: user_id_b64,
                name: request.username,
                display_name: request.display_name,
            },
            pub_key_cred_params: vec![
                PubKeyCredParam {
                    cred_type: "public-key".to_string(),
                    alg: -7,
                },
                PubKeyCredParam {
                    cred_type: "public-key".to_string(),
                    alg: -257,
                },
            ],
            timeout: 60000,
            attestation: "none".to_string(),
            authenticator_selection: AuthenticatorSelection {
                authenticator_attachment: None,
                resident_key: "preferred".to_string(),
                require_resident_key: false,
                user_verification: "preferred".to_string(),
            },
            exclude_credentials,
        })
    }

    pub async fn verify_registration(
        &self,
        response: RegistrationResponse,
        passkey_name: Option<String>,
    ) -> Result<RegistrationResult, PasskeyError> {
        let client_data_json = URL_SAFE_NO_PAD
            .decode(&response.response.client_data_json)
            .map_err(|_| PasskeyError::InvalidClientData)?;

        let client_data: ClientData = serde_json::from_slice(&client_data_json)
            .map_err(|_| PasskeyError::InvalidClientData)?;

        if client_data.r#type != "webauthn.create" {
            return Err(PasskeyError::InvalidCeremonyType);
        }

        if !self.verify_origin(&client_data.origin) {
            return Err(PasskeyError::InvalidOrigin);
        }

        let challenge_bytes = URL_SAFE_NO_PAD
            .decode(&client_data.challenge)
            .map_err(|_| PasskeyError::InvalidChallenge)?;
        log::debug!("Decoded challenge bytes, length: {}", challenge_bytes.len());

        let stored_challenge = self.get_and_remove_challenge(&client_data.challenge).await?;

        if stored_challenge.operation != ChallengeOperation::Registration {
            return Err(PasskeyError::InvalidCeremonyType);
        }

        let user_id = stored_challenge.user_id.ok_or(PasskeyError::MissingUserId)?;

        let attestation_object = URL_SAFE_NO_PAD
            .decode(&response.response.attestation_object)
            .map_err(|_| PasskeyError::InvalidAttestationObject)?;

        let (auth_data, public_key, aaguid) = self.parse_attestation_object(&attestation_object)?;
        log::debug!("Parsed attestation object, auth_data length: {}", auth_data.len());

        let credential_id = URL_SAFE_NO_PAD
            .decode(&response.raw_id)
            .map_err(|_| PasskeyError::InvalidCredentialId)?;

        let name = passkey_name.unwrap_or_else(|| {
            format!("Passkey {}", Utc::now().format("%Y-%m-%d %H:%M"))
        });

        let sanitized_name: String = name
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '-' || *c == '_')
            .take(PASSKEY_NAME_MAX_LENGTH)
            .collect();

        let transports = response
            .response
            .transports
            .unwrap_or_default()
            .join(",");

        self.store_passkey(
            user_id,
            &credential_id,
            &public_key,
            0,
            &sanitized_name,
            aaguid.as_deref(),
            &transports,
        )?;

        info!("Passkey registered for user {}", user_id);

        Ok(RegistrationResult {
            success: true,
            credential_id: Some(URL_SAFE_NO_PAD.encode(&credential_id)),
            error: None,
        })
    }

    pub async fn generate_authentication_options(
        &self,
        request: AuthenticationOptionsRequest,
    ) -> Result<AuthenticationOptions, PasskeyError> {
        let challenge = self.generate_challenge()?;
        let challenge_b64 = URL_SAFE_NO_PAD.encode(&challenge);

        let passkey_challenge = PasskeyChallenge {
            challenge: challenge.clone(),
            user_id: None,
            created_at: Utc::now(),
            operation: ChallengeOperation::Authentication,
        };

        {
            let mut challenges = self.challenges.write().await;
            challenges.insert(challenge_b64.clone(), passkey_challenge);
        }

        let allow_credentials = if let Some(username) = request.username {
            let credentials = self.get_passkeys_by_username(&username)?;
            credentials
                .into_iter()
                .map(|pk| CredentialDescriptor {
                    cred_type: "public-key".to_string(),
                    id: URL_SAFE_NO_PAD.encode(&pk.credential_id),
                    transports: Some(pk.transports),
                })
                .collect()
        } else {
            Vec::new()
        };

        Ok(AuthenticationOptions {
            challenge: challenge_b64,
            timeout: 60000,
            rp_id: self.rp_id.clone(),
            allow_credentials,
            user_verification: "preferred".to_string(),
        })
    }

    pub async fn verify_authentication(
        &self,
        response: AuthenticationResponse,
    ) -> Result<VerificationResult, PasskeyError> {
        let client_data_json = URL_SAFE_NO_PAD
            .decode(&response.response.client_data_json)
            .map_err(|_| PasskeyError::InvalidClientData)?;

        let client_data: ClientData = serde_json::from_slice(&client_data_json)
            .map_err(|_| PasskeyError::InvalidClientData)?;

        if client_data.r#type != "webauthn.get" {
            return Err(PasskeyError::InvalidCeremonyType);
        }

        if !self.verify_origin(&client_data.origin) {
            return Err(PasskeyError::InvalidOrigin);
        }

        let _stored_challenge = self.get_and_remove_challenge(&client_data.challenge).await?;

        let credential_id = URL_SAFE_NO_PAD
            .decode(&response.raw_id)
            .map_err(|_| PasskeyError::InvalidCredentialId)?;

        let passkey = self.get_passkey_by_credential_id(&credential_id)?;

        let authenticator_data = URL_SAFE_NO_PAD
            .decode(&response.response.authenticator_data)
            .map_err(|_| PasskeyError::InvalidAuthenticatorData)?;

        let signature = URL_SAFE_NO_PAD
            .decode(&response.response.signature)
            .map_err(|_| PasskeyError::InvalidSignature)?;

        let rp_id_hash = Sha256::digest(self.rp_id.as_bytes());
        if authenticator_data.len() < 37 || &authenticator_data[..32] != rp_id_hash.as_slice() {
            return Err(PasskeyError::RpIdMismatch);
        }

        let flags = authenticator_data[32];
        let user_present = (flags & 0x01) != 0;
        if !user_present {
            return Err(PasskeyError::UserNotPresent);
        }

        let counter_bytes: [u8; 4] = authenticator_data[33..37]
            .try_into()
            .map_err(|_| PasskeyError::InvalidAuthenticatorData)?;
        let counter = u32::from_be_bytes(counter_bytes);

        if counter > 0 && counter <= passkey.counter {
            warn!(
                "Possible credential cloning detected for user {}",
                passkey.user_id
            );
            return Err(PasskeyError::CounterMismatch);
        }

        let mut verification_data = Vec::new();
        verification_data.extend_from_slice(&authenticator_data);
        verification_data.extend_from_slice(&Sha256::digest(&client_data_json));

        let signature_valid = self.verify_signature(
            &passkey.public_key,
            &verification_data,
            &signature,
        )?;

        if !signature_valid {
            return Err(PasskeyError::SignatureVerificationFailed);
        }

        self.update_passkey_counter(&credential_id, counter)?;

        info!("Passkey authentication successful for user {}", passkey.user_id);

        Ok(VerificationResult {
            success: true,
            user_id: Some(passkey.user_id),
            credential_id: Some(URL_SAFE_NO_PAD.encode(&credential_id)),
            error: None,
            used_fallback: false,
        })
    }

    pub fn get_user_passkeys(&self, user_id: Uuid) -> Result<Vec<PasskeyCredential>, PasskeyError> {
        let mut conn = self.pool.get().map_err(|e| {
            error!("Failed to get database connection: {e}");
            PasskeyError::DatabaseError
        })?;

        let rows: Vec<PasskeyRow> = diesel::sql_query(
            "SELECT id, user_id, credential_id, public_key, counter, name, created_at, last_used_at, aaguid, transports FROM passkeys WHERE user_id = $1 ORDER BY created_at DESC"
        )
            .bind::<DieselUuid, _>(user_id)
            .load(&mut conn)
            .map_err(|e| {
                error!("Failed to query passkeys: {e}");
                PasskeyError::DatabaseError
            })?;

        let credentials = rows
            .into_iter()
            .map(|row| PasskeyCredential {
                id: row.id,
                user_id: row.user_id,
                credential_id: row.credential_id,
                public_key: row.public_key,
                counter: row.counter as u32,
                name: row.name,
                created_at: row.created_at,
                last_used_at: row.last_used_at,
                aaguid: row.aaguid,
                transports: row
                    .transports
                    .map(|t| t.split(',').map(String::from).collect())
                    .unwrap_or_default(),
            })
            .collect();

        Ok(credentials)
    }

    pub fn list_passkeys(&self, user_id: Uuid) -> Result<Vec<PasskeyInfo>, PasskeyError> {
        let passkeys = self.get_user_passkeys(user_id)?;
        let info = passkeys
            .into_iter()
            .map(|pk| PasskeyInfo {
                id: pk.id,
                name: pk.name,
                created_at: pk.created_at,
                last_used_at: pk.last_used_at,
            })
            .collect();
        Ok(info)
    }

    pub fn rename_passkey(
        &self,
        user_id: Uuid,
        passkey_id: &str,
        new_name: &str,
    ) -> Result<(), PasskeyError> {
        let sanitized_name: String = new_name
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '-' || *c == '_')
            .take(PASSKEY_NAME_MAX_LENGTH)
            .collect();

        let mut conn = self.pool.get().map_err(|_| PasskeyError::DatabaseError)?;

        let result = diesel::sql_query(
            "UPDATE passkeys SET name = $1 WHERE id = $2 AND user_id = $3",
        )
        .bind::<Text, _>(&sanitized_name)
        .bind::<Text, _>(passkey_id)
        .bind::<DieselUuid, _>(user_id)
        .execute(&mut conn)
        .map_err(|e| {
            error!("Failed to rename passkey: {e}");
            PasskeyError::DatabaseError
        })?;

        if result == 0 {
            return Err(PasskeyError::PasskeyNotFound);
        }

        Ok(())
    }

    pub fn delete_passkey(&self, user_id: Uuid, passkey_id: &str) -> Result<(), PasskeyError> {
        let mut conn = self.pool.get().map_err(|_| PasskeyError::DatabaseError)?;

        let result = diesel::sql_query(
            "DELETE FROM passkeys WHERE id = $1 AND user_id = $2",
        )
        .bind::<Text, _>(passkey_id)
        .bind::<DieselUuid, _>(user_id)
        .execute(&mut conn)
        .map_err(|e| {
            error!("Failed to delete passkey: {e}");
            PasskeyError::DatabaseError
        })?;

        if result == 0 {
            return Err(PasskeyError::PasskeyNotFound);
        }

        info!("Passkey {} deleted for user {}", passkey_id, user_id);
        Ok(())
    }

    fn generate_challenge(&self) -> Result<Vec<u8>, PasskeyError> {
        let mut challenge = vec![0u8; 32];
        self.rng
            .fill(&mut challenge)
            .map_err(|_| PasskeyError::ChallengeGenerationFailed)?;
        Ok(challenge)
    }

    async fn get_and_remove_challenge(&self, challenge_b64: &str) -> Result<PasskeyChallenge, PasskeyError> {
        let mut challenges = self.challenges.write().await;

        let challenge = challenges
            .remove(challenge_b64)
            .ok_or(PasskeyError::ChallengeNotFound)?;

        let age = Utc::now() - challenge.created_at;
        if age.num_seconds() > CHALLENGE_TIMEOUT_SECONDS {
            return Err(PasskeyError::ChallengeExpired);
        }

        Ok(challenge)
    }

    fn verify_origin(&self, origin: &str) -> bool {
        origin == self.rp_origin
    }

    fn parse_attestation_object(
        &self,
        attestation_object: &[u8],
    ) -> Result<(Vec<u8>, Vec<u8>, Option<Vec<u8>>), PasskeyError> {
        let value: ciborium::Value = ciborium::from_reader(attestation_object)
            .map_err(|_| PasskeyError::InvalidAttestationObject)?;

        let map = value
            .as_map()
            .ok_or(PasskeyError::InvalidAttestationObject)?;

        let auth_data = map
            .iter()
            .find(|(k, _)| k.as_text() == Some("authData"))
            .and_then(|(_, v)| v.as_bytes())
            .ok_or(PasskeyError::InvalidAttestationObject)?
            .to_vec();

        if auth_data.len() < 37 {
            return Err(PasskeyError::InvalidAuthenticatorData);
        }

        let rp_id_hash = Sha256::digest(self.rp_id.as_bytes());
        if &auth_data[..32] != rp_id_hash.as_slice() {
            return Err(PasskeyError::RpIdMismatch);
        }

        let flags = auth_data[32];
        let has_attested_credential = (flags & 0x40) != 0;

        if !has_attested_credential {
            return Err(PasskeyError::NoAttestedCredential);
        }

        let aaguid = auth_data[37..53].to_vec();

        let cred_id_len = u16::from_be_bytes([auth_data[53], auth_data[54]]) as usize;
        let cred_id_end = 55 + cred_id_len;

        if auth_data.len() < cred_id_end {
            return Err(PasskeyError::InvalidAuthenticatorData);
        }

        let public_key_cbor = &auth_data[cred_id_end..];
        let public_key = public_key_cbor.to_vec();

        Ok((auth_data, public_key, Some(aaguid)))
    }

    fn verify_signature(
        &self,
        public_key_cbor: &[u8],
        data: &[u8],
        signature: &[u8],
    ) -> Result<bool, PasskeyError> {
        let pk_value: ciborium::Value = ciborium::from_reader(public_key_cbor)
            .map_err(|_| PasskeyError::InvalidPublicKey)?;

        let pk_map = pk_value
            .as_map()
            .ok_or(PasskeyError::InvalidPublicKey)?;

        let kty = pk_map
            .iter()
            .find(|(k, _)| k.as_integer() == Some(1.into()))
            .and_then(|(_, v)| v.as_integer())
            .ok_or(PasskeyError::InvalidPublicKey)?;

        let alg = pk_map
            .iter()
            .find(|(k, _)| k.as_integer() == Some(3.into()))
            .and_then(|(_, v)| v.as_integer())
            .ok_or(PasskeyError::InvalidPublicKey)?;

        match (i128::from(kty), i128::from(alg)) {
            (2, -7) => self.verify_es256_signature(pk_map, data, signature),
            (3, -257) => self.verify_rs256_signature(pk_map, data, signature),
            _ => Err(PasskeyError::UnsupportedAlgorithm),
        }
    }

    fn verify_es256_signature(
        &self,
        pk_map: &[(ciborium::Value, ciborium::Value)],
        data: &[u8],
        signature: &[u8],
    ) -> Result<bool, PasskeyError> {
        let x = pk_map
            .iter()
            .find(|(k, _)| k.as_integer() == Some((-2).into()))
            .and_then(|(_, v)| v.as_bytes())
            .ok_or(PasskeyError::InvalidPublicKey)?;

        let y = pk_map
            .iter()
            .find(|(k, _)| k.as_integer() == Some((-3).into()))
            .and_then(|(_, v)| v.as_bytes())
            .ok_or(PasskeyError::InvalidPublicKey)?;

        if x.len() != 32 || y.len() != 32 {
            return Err(PasskeyError::InvalidPublicKey);
        }

        let mut public_key_bytes = vec![0x04];
        public_key_bytes.extend_from_slice(x);
        public_key_bytes.extend_from_slice(y);

        let public_key = ring::signature::UnparsedPublicKey::new(
            &ring::signature::ECDSA_P256_SHA256_ASN1,
            &public_key_bytes,
        );

        match public_key.verify(data, signature) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    fn verify_rs256_signature(
        &self,
        pk_map: &[(ciborium::Value, ciborium::Value)],
        data: &[u8],
        signature: &[u8],
    ) -> Result<bool, PasskeyError> {
        let n = pk_map
            .iter()
            .find(|(k, _)| k.as_integer() == Some((-1).into()))
            .and_then(|(_, v)| v.as_bytes())
            .ok_or(PasskeyError::InvalidPublicKey)?;

        let e = pk_map
            .iter()
            .find(|(k, _)| k.as_integer() == Some((-2).into()))
            .and_then(|(_, v)| v.as_bytes())
            .ok_or(PasskeyError::InvalidPublicKey)?;

        let public_key = ring::signature::RsaPublicKeyComponents { n, e };

        match public_key.verify(
            &ring::signature::RSA_PKCS1_2048_8192_SHA256,
            data,
            signature,
        ) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    fn store_passkey(
        &self,
        user_id: Uuid,
        credential_id: &[u8],
        public_key: &[u8],
        counter: u32,
        name: &str,
        aaguid: Option<&[u8]>,
        transports: &str,
    ) -> Result<(), PasskeyError> {
        let mut conn = self.pool.get().map_err(|_| PasskeyError::DatabaseError)?;

        let id = Uuid::new_v4().to_string();

        diesel::sql_query(
            r#"
            INSERT INTO passkeys (id, user_id, credential_id, public_key, counter, name, aaguid, transports, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())
            "#,
        )
        .bind::<Text, _>(&id)
        .bind::<DieselUuid, _>(user_id)
        .bind::<Bytea, _>(credential_id)
        .bind::<Bytea, _>(public_key)
        .bind::<BigInt, _>(counter as i64)
        .bind::<Text, _>(name)
        .bind::<Nullable<Bytea>, _>(aaguid)
        .bind::<Text, _>(transports)
        .execute(&mut conn)
        .map_err(|e| {
            error!("Failed to store passkey: {e}");
            PasskeyError::DatabaseError
        })?;

        Ok(())
    }

    fn get_passkey_by_credential_id(
        &self,
        credential_id: &[u8],
    ) -> Result<PasskeyCredential, PasskeyError> {
        let mut conn = self.pool.get().map_err(|_| PasskeyError::DatabaseError)?;

        let rows: Vec<PasskeyRow> = diesel::sql_query(
            "SELECT id, user_id, credential_id, public_key, counter, name, created_at, last_used_at, aaguid, transports FROM passkeys WHERE credential_id = $1",
        )
        .bind::<Bytea, _>(credential_id)
        .load(&mut conn)
        .map_err(|e| {
            error!("Failed to query passkey: {e}");
            PasskeyError::DatabaseError
        })?;

        let row = rows.into_iter().next().ok_or(PasskeyError::PasskeyNotFound)?;

        Ok(PasskeyCredential {
            id: row.id,
            user_id: row.user_id,
            credential_id: row.credential_id,
            public_key: row.public_key,
            counter: row.counter as u32,
            name: row.name,
            created_at: row.created_at,
            last_used_at: row.last_used_at,
            aaguid: row.aaguid,
            transports: row
                .transports
                .map(|t| t.split(',').map(String::from).collect())
                .unwrap_or_default(),
        })
    }

    fn get_passkeys_by_username(
        &self,
        username: &str,
    ) -> Result<Vec<PasskeyCredential>, PasskeyError> {
        let mut conn = self.pool.get().map_err(|_| PasskeyError::DatabaseError)?;

        let rows: Vec<PasskeyRow> = diesel::sql_query(
            r#"
            SELECT p.id, p.user_id, p.credential_id, p.public_key, p.counter, p.name, p.created_at, p.last_used_at, p.aaguid, p.transports
            FROM passkeys p
            JOIN users u ON u.id = p.user_id
            WHERE u.username = $1 OR u.email = $1
            ORDER BY p.created_at DESC
            "#,
        )
        .bind::<Text, _>(username)
        .load(&mut conn)
        .map_err(|e| {
            error!("Failed to query passkeys by username: {e}");
            PasskeyError::DatabaseError
        })?;

        let credentials = rows
            .into_iter()
            .map(|row| PasskeyCredential {
                id: row.id,
                user_id: row.user_id,
                credential_id: row.credential_id,
                public_key: row.public_key,
                counter: row.counter as u32,
                name: row.name,
                created_at: row.created_at,
                last_used_at: row.last_used_at,
                aaguid: row.aaguid,
                transports: row
                    .transports
                    .map(|t| t.split(',').map(String::from).collect())
                    .unwrap_or_default(),
            })
            .collect();

        Ok(credentials)
    }

    fn update_passkey_counter(
        &self,
        credential_id: &[u8],
        new_counter: u32,
    ) -> Result<(), PasskeyError> {
        let mut conn = self.pool.get().map_err(|_| PasskeyError::DatabaseError)?;

        diesel::sql_query(
            "UPDATE passkeys SET counter = $1, last_used_at = NOW() WHERE credential_id = $2",
        )
        .bind::<BigInt, _>(new_counter as i64)
        .bind::<Bytea, _>(credential_id)
        .execute(&mut conn)
        .map_err(|e| {
            error!("Failed to update passkey counter: {e}");
            PasskeyError::DatabaseError
        })?;

        Ok(())
    }

    pub async fn cleanup_expired_challenges(&self) {
        let mut challenges = self.challenges.write().await;
        let cutoff = Utc::now() - Duration::seconds(CHALLENGE_TIMEOUT_SECONDS);
        challenges.retain(|_, c| c.created_at > cutoff);
    }
}

#[derive(Debug, Deserialize)]
struct ClientData {
    #[serde(rename = "type")]
    r#type: String,
    challenge: String,
    origin: String,
}

#[derive(Debug, Clone)]
pub enum PasskeyError {
    DatabaseError,
    ChallengeGenerationFailed,
    ChallengeStorageError,
    ChallengeNotFound,
    ChallengeExpired,
    InvalidClientData,
    InvalidCeremonyType,
    InvalidOrigin,
    InvalidChallenge,
    InvalidAttestationObject,
    InvalidAuthenticatorData,
    InvalidCredentialId,
    InvalidSignature,
    InvalidPublicKey,
    MissingUserId,
    NoAttestedCredential,
    RpIdMismatch,
    UserNotPresent,
    CounterMismatch,
    SignatureVerificationFailed,
    UnsupportedAlgorithm,
    PasskeyNotFound,
}

impl std::fmt::Display for PasskeyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DatabaseError => write!(f, "Database error"),
            Self::ChallengeGenerationFailed => write!(f, "Challenge generation failed"),
            Self::ChallengeStorageError => write!(f, "Challenge storage error"),
            Self::ChallengeNotFound => write!(f, "Challenge not found"),
            Self::ChallengeExpired => write!(f, "Challenge expired"),
            Self::InvalidClientData => write!(f, "Invalid client data"),
            Self::InvalidCeremonyType => write!(f, "Invalid ceremony type"),
            Self::InvalidOrigin => write!(f, "Invalid origin"),
            Self::InvalidChallenge => write!(f, "Invalid challenge"),
            Self::InvalidAttestationObject => write!(f, "Invalid attestation object"),
            Self::InvalidAuthenticatorData => write!(f, "Invalid authenticator data"),
            Self::InvalidCredentialId => write!(f, "Invalid credential ID"),
            Self::InvalidSignature => write!(f, "Invalid signature"),
            Self::InvalidPublicKey => write!(f, "Invalid public key"),
            Self::MissingUserId => write!(f, "Missing user ID"),
            Self::NoAttestedCredential => write!(f, "No attested credential"),
            Self::RpIdMismatch => write!(f, "RP ID mismatch"),
            Self::UserNotPresent => write!(f, "User not present"),
            Self::CounterMismatch => write!(f, "Counter mismatch - possible cloning"),
            Self::SignatureVerificationFailed => write!(f, "Signature verification failed"),
            Self::UnsupportedAlgorithm => write!(f, "Unsupported algorithm"),
            Self::PasskeyNotFound => write!(f, "Passkey not found"),
        }
    }
}

impl std::error::Error for PasskeyError {}

impl IntoResponse for PasskeyError {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            Self::PasskeyNotFound => StatusCode::NOT_FOUND,
            Self::ChallengeExpired | Self::ChallengeNotFound => StatusCode::GONE,
            Self::InvalidOrigin | Self::RpIdMismatch => StatusCode::FORBIDDEN,
            Self::CounterMismatch | Self::SignatureVerificationFailed => StatusCode::UNAUTHORIZED,
            _ => StatusCode::BAD_REQUEST,
        };
        (status, self.to_string()).into_response()
    }
}

pub fn create_passkey_tables_migration() -> &'static str {
    r#"
    CREATE TABLE IF NOT EXISTS passkeys (
        id TEXT PRIMARY KEY,
        user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
        credential_id BYTEA NOT NULL UNIQUE,
        public_key BYTEA NOT NULL,
        counter BIGINT NOT NULL DEFAULT 0,
        name TEXT NOT NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        last_used_at TIMESTAMPTZ,
        aaguid BYTEA,
        transports TEXT
    );

    CREATE INDEX IF NOT EXISTS idx_passkeys_user_id ON passkeys(user_id);
    CREATE INDEX IF NOT EXISTS idx_passkeys_credential_id ON passkeys(credential_id);
    "#
}

pub fn passkey_routes(_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/registration/options", post(registration_options_handler))
        .route("/registration/verify", post(registration_verify_handler))
        .route("/authentication/options", post(authentication_options_handler))
        .route("/authentication/verify", post(authentication_verify_handler))
        .route("/list/:user_id", get(list_passkeys_handler))
        .route("/:user_id/:passkey_id", delete(delete_passkey_handler))
        .route("/:user_id/:passkey_id/rename", post(rename_passkey_handler))
        // Password fallback routes
        .route("/fallback/authenticate", post(password_fallback_handler))
        .route("/fallback/check/:username", get(check_fallback_available_handler))
        .route("/fallback/config", get(get_fallback_config_handler))
}

    async fn password_fallback_handler(
        State(state): State<Arc<AppState>>,
        Json(request): Json<PasswordFallbackRequest>,
    ) -> impl IntoResponse {
        let service = match get_passkey_service(&state) {
            Ok(s) => s,
            Err(e) => return e.into_response(),
        };
        match service.authenticate_with_password_fallback(&request).await {
            Ok(response) => Json(response).into_response(),
            Err(e) => e.into_response(),
        }
    }

    async fn check_fallback_available_handler(
        State(state): State<Arc<AppState>>,
        Path(username): Path<String>,
    ) -> impl IntoResponse {
        let service = match get_passkey_service(&state) {
            Ok(s) => s,
            Err(e) => return e.into_response(),
        };

        #[derive(Serialize)]
        struct FallbackAvailableResponse {
            available: bool,
            has_passkeys: bool,
            reason: Option<String>,
        }

        match service.should_offer_password_fallback(&username) {
            Ok(available) => {
                let has_passkeys = service.user_has_passkeys(&username).unwrap_or(false);
                Json(FallbackAvailableResponse {
                    available,
                    has_passkeys,
                    reason: if !available {
                        Some("Password fallback is disabled".to_string())
                    } else {
                        None
                    },
                }).into_response()
            }
            Err(e) => e.into_response(),
        }
    }

    async fn get_fallback_config_handler(
        State(state): State<Arc<AppState>>,
    ) -> impl IntoResponse {
        let service = match get_passkey_service(&state) {
            Ok(s) => s,
            Err(e) => return e.into_response(),
        };
        let config = service.get_fallback_config();

        #[derive(Serialize)]
        struct PublicFallbackConfig {
            enabled: bool,
            prompt_passkey_setup: bool,
        }

        Json(PublicFallbackConfig {
            enabled: config.enabled,
            prompt_passkey_setup: config.prompt_passkey_setup,
        }).into_response()
    }

async fn registration_options_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<RegistrationOptionsRequest>,
) -> Result<Json<RegistrationOptions>, PasskeyError> {
    let service = get_passkey_service(&state)?;
    let options = service.generate_registration_options(request).await?;
    Ok(Json(options))
}

async fn registration_verify_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<RegistrationVerifyRequest>,
) -> Result<Json<RegistrationResult>, PasskeyError> {
    let service = get_passkey_service(&state)?;
    let result = service.verify_registration(request.response, request.name).await?;
    Ok(Json(result))
}

async fn authentication_options_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<AuthenticationOptionsRequest>,
) -> Result<Json<AuthenticationOptions>, PasskeyError> {
    let service = get_passkey_service(&state)?;
    let options = service.generate_authentication_options(request).await?;
    Ok(Json(options))
}

async fn authentication_verify_handler(
    State(state): State<Arc<AppState>>,
    Json(response): Json<AuthenticationResponse>,
) -> Result<Json<VerificationResult>, PasskeyError> {
    let service = get_passkey_service(&state)?;
    let result = service.verify_authentication(response).await?;
    Ok(Json(result))
}

async fn list_passkeys_handler(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<Vec<PasskeyInfo>>, PasskeyError> {
    let service = get_passkey_service(&state)?;
    let passkeys = service.list_passkeys(user_id)?;
    Ok(Json(passkeys))
}

async fn delete_passkey_handler(
    State(state): State<Arc<AppState>>,
    Path((user_id, passkey_id)): Path<(Uuid, String)>,
) -> Result<StatusCode, PasskeyError> {
    let service = get_passkey_service(&state)?;
    service.delete_passkey(user_id, &passkey_id)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn rename_passkey_handler(
    State(state): State<Arc<AppState>>,
    Path((user_id, passkey_id)): Path<(Uuid, String)>,
    Json(request): Json<RenamePasskeyRequest>,
) -> Result<StatusCode, PasskeyError> {
    let service = get_passkey_service(&state)?;
    service.rename_passkey(user_id, &passkey_id, &request.name)?;
    Ok(StatusCode::OK)
}

#[derive(Debug, Deserialize)]
struct RegistrationVerifyRequest {
    response: RegistrationResponse,
    name: Option<String>,
}

fn get_passkey_service(state: &AppState) -> Result<PasskeyService, PasskeyError> {
    let pool = state.conn.clone();
    let rp_id = std::env::var("PASSKEY_RP_ID").unwrap_or_else(|_| "localhost".to_string());
    let rp_name = std::env::var("PASSKEY_RP_NAME").unwrap_or_else(|_| "General Bots".to_string());
    let rp_origin = std::env::var("PASSKEY_RP_ORIGIN").unwrap_or_else(|_| "http://localhost:8081".to_string());

    Ok(PasskeyService::new(pool, rp_id, rp_name, rp_origin))
}

#[cfg(test)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fallback_config_default() {
        let config = FallbackConfig::default();
        assert!(config.enabled);
        assert!(!config.require_additional_verification);
        assert_eq!(config.max_fallback_attempts, 5);
        assert_eq!(config.lockout_duration_seconds, 900);
        assert!(config.prompt_passkey_setup);
    }

    #[test]
    fn test_password_fallback_request_serialization() {
        let request = PasswordFallbackRequest {
            username: "testuser".to_string(),
            password: "testpass".to_string(),
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("testuser"));
    }

    #[test]
    fn test_password_fallback_response_structure() {
        let response = PasswordFallbackResponse {
            success: true,
            user_id: Some(Uuid::new_v4()),
            token: Some("test-token".to_string()),
            error: None,
            passkey_available: true,
        };
        assert!(response.success);
        assert!(response.user_id.is_some());
        assert!(response.passkey_available);
    }

    #[test]
    fn test_verification_result_with_fallback() {
        let result = VerificationResult {
            success: true,
            user_id: Some(Uuid::new_v4()),
            credential_id: None,
            error: None,
            used_fallback: true,
        };
        assert!(result.used_fallback);
    }

    use super::*;

    #[test]
    fn test_passkey_error_display() {
        assert_eq!(PasskeyError::DatabaseError.to_string(), "Database error");
        assert_eq!(PasskeyError::ChallengeExpired.to_string(), "Challenge expired");
        assert_eq!(PasskeyError::PasskeyNotFound.to_string(), "Passkey not found");
    }

    #[test]
    fn test_challenge_operation_serialization() {
        let op = ChallengeOperation::Registration;
        let json = serde_json::to_string(&op).unwrap_or_default();
        assert!(json.contains("registration"));
    }

    #[test]
    fn test_registration_options_structure() {
        let options = RegistrationOptions {
            challenge: "test_challenge".to_string(),
            rp: RelyingParty {
                id: "example.com".to_string(),
                name: "Example".to_string(),
            },
            user: UserEntity {
                id: "user_id".to_string(),
                name: "user@example.com".to_string(),
                display_name: "User".to_string(),
            },
            pub_key_cred_params: vec![
                PubKeyCredParam {
                    cred_type: "public-key".to_string(),
                    alg: -7,
                },
            ],
            timeout: 60000,
            attestation: "none".to_string(),
            authenticator_selection: AuthenticatorSelection {
                authenticator_attachment: None,
                resident_key: "preferred".to_string(),
                require_resident_key: false,
                user_verification: "preferred".to_string(),
            },
            exclude_credentials: vec![],
        };

        assert_eq!(options.rp.id, "example.com");
        assert_eq!(options.timeout, 60000);
    }

    #[test]
    fn test_passkey_info_creation() {
        let info = PasskeyInfo {
            id: "pk_123".to_string(),
            name: "My Passkey".to_string(),
            created_at: Utc::now(),
            last_used_at: None,
        };

        assert_eq!(info.id, "pk_123");
        assert_eq!(info.name, "My Passkey");
        assert!(info.last_used_at.is_none());
    }
}
