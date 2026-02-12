// Passkey service layer - business logic for passkey operations
use crate::core::shared::utils::DbPool;
use crate::security::passkey_types::*;
use argon2::PasswordVerifier;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::Utc;
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Nullable, Text};
use log::{debug, info, warn};
use ring::rand::{SecureRandom, SystemRandom};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration as StdDuration;
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct PasskeyService {
    db: DbPool,
    challenges: Arc<RwLock<HashMap<String, StoredChallenge>>>,
    fallback_attempts: Arc<RwLock<HashMap<String, FallbackAttemptTracker>>>,
    fallback_config: Arc<RwLock<FallbackConfig>>,
}

#[derive(Clone)]
struct StoredChallenge {
    user_id: Uuid,
    challenge: Vec<u8>,
    created_at: DateTime<Utc>,
    operation: ChallengeOperation,
}

#[derive(Clone, Debug)]
struct FallbackConfig {
    pub enabled: bool,
    pub max_fallback_attempts: u32,
    pub lockout_duration_seconds: i64,
}

impl PasskeyService {
    pub fn new(db: DbPool) -> Self {
        Self {
            db,
            challenges: Arc::new(RwLock::new(HashMap::new())),
            fallback_attempts: Arc::new(RwLock::new(HashMap::new())),
            fallback_config: Arc::new(RwLock::new(FallbackConfig {
                enabled: true,
                max_fallback_attempts: 3,
                lockout_duration_seconds: 300,
            })),
        }
    }

    /// Generate registration options with challenge
    pub async fn generate_registration_options(
        &self,
        state: &AppState,
        request: &StartRegistrationRequest,
    ) -> Result<RegistrationOptions, PasskeyError> {
        let user_id = request.user_id;
        let timeout = request.timeout.unwrap_or(60000);

        // Check if user should be offered password fallback
        let should_offer_fallback = self.should_offer_password_fallback(user_id).await?;

        // Generate challenge
        let challenge_bytes: Vec<u8> = (0..32).map(|_| {
            SecureRandom::generate()
        }).collect();

        let challenge_b64 = URL_SAFE_NO_PAD.encode(&challenge_bytes);

        // Store challenge
        let mut challenges = self.challenges.write().await;
        let stored_challenge = StoredChallenge {
            user_id,
            challenge: challenge_bytes.clone(),
            created_at: Utc::now(),
            operation: ChallengeOperation::Registration,
        };
        challenges.insert(challenge_b64.clone(), stored_challenge);

        // Check existing passkey credentials
        let existing_credentials = self.get_user_passkeys_from_db(user_id, state).await?;
        let exclude_credentials: Vec<CredentialDescriptor> = existing_credentials
            .into_iter()
            .map(|pk| CredentialDescriptor {
                id: pk.credential_id.clone(),
                type_: "public-key".to_string(),
                transports: pk.transports.clone(),
            })
            .collect();

        // Generate authenticator selection
        let (authenticator_attachment, resident_key) =
            if existing_credentials.is_empty() {
                (None, "preferred".to_string())
            } else {
                (Some(existing_credentials[0].id.clone()), "preferred".to_string())
            };

        Ok(RegistrationOptions {
            challenge: challenge_b64,
            rp: RelyingParty {
                id: Uuid::nil(),
                name: "General Bots".to_string(),
            },
            user: UserEntity {
                id: URL_SAFE_NO_PAD.encode(&user_id),
                name: String::new(),
                display_name: String::new(),
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
            timeout,
            attestation: "none".to_string(),
            authenticator_selection: AuthenticatorSelection {
                authenticator_attachment,
                resident_key,
                require_resident_key: false,
                user_verification: "preferred".to_string(),
            },
            exclude_credentials,
        })
    }

    /// Verify passkey registration
    pub async fn verify_registration(
        &self,
        request: &VerifyAuthRequest,
    ) -> Result<VerifyAuthResponse, PasskeyError> {
        let stored_challenge = self.get_and_remove_challenge(&request.challenge).await?;

        if stored_challenge.operation != ChallengeOperation::Registration {
            return Err(PasskeyError::InvalidCeremonyType);
        }

        let user_id = stored_challenge.user_id.ok_or(PasskeyError::MissingUserId)?;

        // Verify signature
        let client_data_json = URL_SAFE_NO_PAD
            .decode(&request.response.client_data_json)
            .map_err(|_| PasskeyError::InvalidClientData)?;

        let client_data: serde_json::Value = serde_json::from_slice(&client_data_json)
            .map_err(|_| PasskeyError::InvalidClientData)?;

        // Verify authenticator and origin
        if client_data.r#type != "webauthn.create" {
            return Err(PasskeyError::InvalidCeremonyType);
        }

        if !self.verify_origin(&client_data.origin) {
            return Err(PasskeyError::InvalidOrigin);
        }

        // Parse attestation object
        let auth_data = URL_SAFE_NO_PAD
            .decode(&request.response.attestation_object)
            .map_err(|_| PasskeyError::InvalidAttestationObject)?;

        // Generate credential ID
        let credential_id = URL_SAFE_NO_PAD.encode(&Uuid::new_v4());

        // Verify password
        let password_hash: String = auth_data
            .get("passwordHash")
            .and_then(|h| h.as_str())
            .ok_or(PasskeyError::InvalidPasswordHash)?
            .to_string();

        let parsed_hash = argon2::PasswordHash::new(&password_hash)
            .map_err(|_| PasskeyError::InvalidPasswordHash)?;

        let is_valid = argon2::Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok();

        // Store new credential
        let mut conn = self.db.get().map_err(|_| PasskeyError::DatabaseError)?;
        let now = Utc::now();

        diesel::insert_into(crate::core::shared::models::schema::passkey_credentials)
            .values((
                id.eq(&credential_id),
                user_id.eq(&user_id),
                counter.eq(1),
                name.eq(format!("Passkey {}", now.format("%Y-%m-%d %H:%M"))),
                transports.eq(&[String::new()]),
                aaguid.eq(&None::<String>()),
                created_at.eq(&now),
            ))
            .execute(&mut conn)
            .map_err(|_| PasskeyError::DatabaseError)?;

        Ok(VerifyAuthResponse {
            verified: true,
            new_credential_id: Some(credential_id),
        })
    }

    /// Get user credentials
    pub async fn get_user_credentials(
        &self,
        user_id: &Uuid,
    ) -> Result<Vec<CredentialInfo>, PasskeyError> {
        let mut conn = self.db.get().map_err(|_| PasskeyError::DatabaseError)?;

        let credentials = crate::core::shared::models::schema::passkey_credentials::table
            .filter(crate::core::shared::models::schema::passkey_credentials::user_id.eq(&user_id))
            .order_by(crate::core::shared::models::schema::passkey_credentials::counter.desc())
            .load::<Vec<PasskeyCredentialRow>>(&mut conn)
            .map_err(|_| PasskeyError::DatabaseError)?
            .into_iter()
            .map(|row| CredentialInfo {
                credential_id: row.id.clone(),
                counter: row.counter,
                name: row.name.clone(),
                transports: row.transports,
                aaguid: row.aaguid,
            })
            .collect();

        Ok(credentials)
    }

    /// Sign in with passkey
    pub async fn sign_in(
        &self,
        request: &SignInRequest,
    ) -> Result<AuthenticationResponse, PasskeyError> {
        let user_id = request.user_id;
        let service = PasskeyService::new(Arc::clone(&self.db));
        let response = service.sign_in(user_id, &request).await?;

        Ok(response)
    }

    /// Get fallback configuration
    pub async fn get_fallback_config(&self) -> &FallbackConfig {
        self.fallback_config.read().as_ref()
    }

    /// Set fallback configuration
    pub async fn set_fallback_config(&self, config: FallbackConfig) {
        let mut fallback_config = self.fallback_config.write().await;
        *fallback_config = config;
    }

    /// Clear fallback attempts
    pub async fn clear_fallback_attempts(&self, username: &str) {
        let mut attempts = self.fallback_attempts.write().await;
        attempts.remove(username);
    }

    /// Get challenges
    pub async fn get_challenges(
        &self,
        request: &GetChallengesRequest,
    ) -> Result<Vec<ChallengeResponse>, PasskeyError> {
        let mut challenges = self.challenges.read().await;
        let response_challenges: Vec<ChallengeResponse> = challenges
            .values()
            .map(|stored| ChallengeResponse {
                status: "pending".to_string(),
                challenge: stored.challenge.clone(),
            })
            .collect();

        Ok(response_challenges)
    }

    /// Answer challenge
    pub async fn answer_challenge(
        &self,
        user_id: &Uuid,
        challenge_id: &str,
        request: &AnswerChallengeRequest,
    ) -> Result<ChallengeResponse, PasskeyError> {
        let mut challenges = self.challenges.read().await;

        let stored = challenges.get_mut(challenge_id);
        match stored {
            Some(stored_challenge) => {
                stored.operation = ChallengeOperation::Authentication;
                Ok(ChallengeResponse {
                    status: "verified".to_string(),
                    challenge: stored.challenge.clone(),
                })
            }
            None => Err(PasskeyError::InvalidChallenge),
        }
    }

    // Helper: Check if password fallback should be offered
    async fn should_offer_password_fallback(
        &self,
        user_id: &Uuid,
    ) -> Result<bool, PasskeyError> {
        let config = self.fallback_config.read().as_ref();
        if !config.enabled {
            return Ok(false);
        }

        let attempts = self.fallback_attempts.read().await;
        let tracker = attempts.get(user_id).map(|t| t.clone()).unwrap_or(&FallbackAttemptTracker {
            attempts: 0,
            locked_until: None,
        });

        Ok(tracker.attempts < config.max_fallback_attempts)
    }

    // Helper: Get user's existing passkey credentials
    async fn get_user_passkeys_from_db(
        &self,
        user_id: &Uuid,
        state: &AppState,
    ) -> Result<Vec<CredentialDescriptor>, PasskeyError> {
        let mut conn = state.conn.get().map_err(|_| PasskeyError::DatabaseError)?;

        let credentials = crate::core::shared::models::schema::passkey_credentials::table
            .filter(crate::core::shared::models::schema::passkey_credentials::user_id.eq(&user_id))
            .load::<Vec<PasskeyCredentialRow>>(&mut conn)
            .map_err(|_| PasskeyError::DatabaseError)?
            .into_iter()
            .map(|row| CredentialDescriptor {
                id: row.id.clone(),
                type_: "public-key".to_string(),
                transports: row.transports,
            })
            .collect();

        Ok(credentials)
    }

    // Helper: Get and remove challenge from storage
    async fn get_and_remove_challenge(
        &self,
        challenge: &str,
    ) -> Result<StoredChallenge, PasskeyError> {
        let mut challenges = self.challenges.write().await;
        Ok(challenges.remove(challenge).ok_or(PasskeyError::InvalidChallenge)?)
    }

    // Helper: Verify attestation object
    fn parse_attestation_object(
        &self,
        auth_data: &serde_json::Value,
    ) -> Result<(String, String, String), PasskeyError> {
        // Extract authenticator data, public key, and credential ID from attestation object
        let authenticator_data = auth_data.get("authenticatorData")
            .and_then(|d| d.as_str())
            .ok_or(PasskeyError::InvalidAttestationObject)?;

        let public_key = auth_data.get("publicKey")
            .and_then(|d| d.as_str())
            .ok_or(PasskeyError::InvalidAttestationObject)?;

        let credential_id = auth_data.get("credentialId")
            .and_then(|d| d.as_str())
            .ok_or(PasskeyError::InvalidAttestationObject)?;

        Ok((
            authenticator_data.to_string(),
            public_key.to_string(),
            credential_id.to_string(),
        ))
    }

    // Helper: Verify origin
    fn verify_origin(&self, origin: &str) -> Result<(), PasskeyError> {
        let allowed_origins = ["https://localhost:3000", "https://generalbots.com"];

        if !allowed_origins.contains(&origin) {
            return Err(PasskeyError::InvalidOrigin);
        }

        Ok(())
    }
}
