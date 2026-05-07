use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

const NONCE_SIZE: usize = 12;
const KEY_SIZE: usize = 32;
const SALT_SIZE: usize = 32;
const PBKDF2_ITERATIONS: u32 = 100_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    pub enabled: bool,
    pub algorithm: EncryptionAlgorithm,
    pub key_rotation_days: u32,
    pub envelope_encryption: bool,
    pub compress_before_encrypt: bool,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            key_rotation_days: 90,
            envelope_encryption: true,
            compress_before_encrypt: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    Aes256Gcm,
}

impl EncryptionAlgorithm {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Aes256Gcm => "AES-256-GCM",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    pub algorithm: String,
    pub nonce: String,
    pub ciphertext: String,
    pub key_id: Option<String>,
    pub version: u32,
}

impl EncryptedData {
    pub fn to_string_compact(&self) -> String {
        format!(
            "{}:{}:{}:{}:{}",
            self.version,
            self.algorithm,
            self.key_id.as_deref().unwrap_or("default"),
            self.nonce,
            self.ciphertext
        )
    }

    pub fn from_string_compact(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() < 5 {
            return Err(anyhow!("Invalid encrypted data format"));
        }

        let version: u32 = parts[0].parse().map_err(|_| anyhow!("Invalid version"))?;
        let algorithm = parts[1].to_string();
        let key_id = if parts[2] == "default" {
            None
        } else {
            Some(parts[2].to_string())
        };
        let nonce = parts[3].to_string();
        let ciphertext = parts[4..].join(":");

        Ok(Self {
            algorithm,
            nonce,
            ciphertext,
            key_id,
            version,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionKey {
    pub id: String,
    pub key_data: Vec<u8>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub is_primary: bool,
    pub purpose: KeyPurpose,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyPurpose {
    DataEncryption,
    KeyEncryption,
    Signing,
}

impl EncryptionKey {
    pub fn generate(purpose: KeyPurpose) -> Self {
        let mut rng = rand::rng();
        let key_data: Vec<u8> = (0..KEY_SIZE).map(|_| rng.random()).collect();

        Self {
            id: Uuid::new_v4().to_string(),
            key_data,
            created_at: chrono::Utc::now(),
            expires_at: None,
            is_primary: false,
            purpose,
        }
    }

    pub fn generate_primary(purpose: KeyPurpose, expiry_days: u32) -> Self {
        let mut key = Self::generate(purpose);
        key.is_primary = true;
        key.expires_at = Some(
            chrono::Utc::now() + chrono::Duration::days(expiry_days as i64),
        );
        key
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expires) = self.expires_at {
            chrono::Utc::now() > expires
        } else {
            false
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.is_expired() && self.key_data.len() == KEY_SIZE
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvelopeEncryptedData {
    pub encrypted_dek: EncryptedData,
    pub encrypted_data: EncryptedData,
    pub kek_id: String,
}

pub struct EncryptionManager {
    config: EncryptionConfig,
    keys: Arc<RwLock<HashMap<String, EncryptionKey>>>,
    primary_dek_id: Arc<RwLock<Option<String>>>,
    primary_kek_id: Arc<RwLock<Option<String>>>,
}

impl EncryptionManager {
    pub fn new(config: EncryptionConfig) -> Result<Self> {
        let manager = Self {
            config,
            keys: Arc::new(RwLock::new(HashMap::new())),
            primary_dek_id: Arc::new(RwLock::new(None)),
            primary_kek_id: Arc::new(RwLock::new(None)),
        };

        Ok(manager)
    }

    pub async fn initialize(&self) -> Result<()> {
        let dek = EncryptionKey::generate_primary(
            KeyPurpose::DataEncryption,
            self.config.key_rotation_days,
        );
        let dek_id = dek.id.clone();

        let kek = EncryptionKey::generate_primary(
            KeyPurpose::KeyEncryption,
            self.config.key_rotation_days,
        );
        let kek_id = kek.id.clone();

        {
            let mut keys = self.keys.write().await;
            keys.insert(dek.id.clone(), dek);
            keys.insert(kek.id.clone(), kek);
        }

        *self.primary_dek_id.write().await = Some(dek_id.clone());
        *self.primary_kek_id.write().await = Some(kek_id.clone());

        info!("Encryption manager initialized with DEK {} and KEK {}", dek_id, kek_id);
        Ok(())
    }

    pub async fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptedData> {
        if !self.config.enabled {
            return Err(anyhow!("Encryption is disabled"));
        }

        let key_id = self
            .primary_dek_id
            .read()
            .await
            .clone()
            .ok_or_else(|| anyhow!("No primary DEK available"))?;

        let keys = self.keys.read().await;
        let key = keys
            .get(&key_id)
            .ok_or_else(|| anyhow!("DEK not found"))?;

        if !key.is_valid() {
            return Err(anyhow!("DEK is invalid or expired"));
        }

        let encrypted = encrypt_aes_gcm(plaintext, &key.key_data)?;

        Ok(EncryptedData {
            algorithm: self.config.algorithm.as_str().to_string(),
            nonce: encrypted.0,
            ciphertext: encrypted.1,
            key_id: Some(key_id),
            version: 1,
        })
    }

    pub async fn decrypt(&self, encrypted: &EncryptedData) -> Result<Vec<u8>> {
        if !self.config.enabled {
            return Err(anyhow!("Encryption is disabled"));
        }

        let key_id = encrypted
            .key_id
            .as_ref()
            .ok_or_else(|| anyhow!("No key ID in encrypted data"))?;

        let keys = self.keys.read().await;
        let key = keys
            .get(key_id)
            .ok_or_else(|| anyhow!("DEK not found: {key_id}"))?;

        decrypt_aes_gcm(&encrypted.nonce, &encrypted.ciphertext, &key.key_data)
    }

    pub async fn encrypt_string(&self, plaintext: &str) -> Result<String> {
        let encrypted = self.encrypt(plaintext.as_bytes()).await?;
        Ok(encrypted.to_string_compact())
    }

    pub async fn decrypt_string(&self, encrypted: &str) -> Result<String> {
        let encrypted_data = EncryptedData::from_string_compact(encrypted)?;
        let decrypted = self.decrypt(&encrypted_data).await?;
        String::from_utf8(decrypted).map_err(|e| anyhow!("Invalid UTF-8: {e}"))
    }

    pub async fn encrypt_with_envelope(&self, plaintext: &[u8]) -> Result<EnvelopeEncryptedData> {
        if !self.config.envelope_encryption {
            return Err(anyhow!("Envelope encryption is disabled"));
        }

        let dek = EncryptionKey::generate(KeyPurpose::DataEncryption);
        let encrypted_data = encrypt_aes_gcm(plaintext, &dek.key_data)?;

        let kek_id = self
            .primary_kek_id
            .read()
            .await
            .clone()
            .ok_or_else(|| anyhow!("No primary KEK available"))?;

        let keys = self.keys.read().await;
        let kek = keys
            .get(&kek_id)
            .ok_or_else(|| anyhow!("KEK not found"))?;

        if !kek.is_valid() {
            return Err(anyhow!("KEK is invalid or expired"));
        }

        let encrypted_dek = encrypt_aes_gcm(&dek.key_data, &kek.key_data)?;

        Ok(EnvelopeEncryptedData {
            encrypted_dek: EncryptedData {
                algorithm: self.config.algorithm.as_str().to_string(),
                nonce: encrypted_dek.0,
                ciphertext: encrypted_dek.1,
                key_id: Some(kek_id.clone()),
                version: 1,
            },
            encrypted_data: EncryptedData {
                algorithm: self.config.algorithm.as_str().to_string(),
                nonce: encrypted_data.0,
                ciphertext: encrypted_data.1,
                key_id: None,
                version: 1,
            },
            kek_id,
        })
    }

    pub async fn decrypt_with_envelope(&self, envelope: &EnvelopeEncryptedData) -> Result<Vec<u8>> {
        if !self.config.envelope_encryption {
            return Err(anyhow!("Envelope encryption is disabled"));
        }

        let keys = self.keys.read().await;
        let kek = keys
            .get(&envelope.kek_id)
            .ok_or_else(|| anyhow!("KEK not found: {}", envelope.kek_id))?;

        let dek_bytes = decrypt_aes_gcm(
            &envelope.encrypted_dek.nonce,
            &envelope.encrypted_dek.ciphertext,
            &kek.key_data,
        )?;

        decrypt_aes_gcm(
            &envelope.encrypted_data.nonce,
            &envelope.encrypted_data.ciphertext,
            &dek_bytes,
        )
    }

    pub async fn rotate_keys(&self) -> Result<(String, String)> {
        let new_dek = EncryptionKey::generate_primary(
            KeyPurpose::DataEncryption,
            self.config.key_rotation_days,
        );
        let new_dek_id = new_dek.id.clone();

        let new_kek = EncryptionKey::generate_primary(
            KeyPurpose::KeyEncryption,
            self.config.key_rotation_days,
        );
        let new_kek_id = new_kek.id.clone();

        {
            let mut keys = self.keys.write().await;

            if let Some(old_dek_id) = self.primary_dek_id.read().await.as_ref() {
                if let Some(old_key) = keys.get_mut(old_dek_id) {
                    old_key.is_primary = false;
                }
            }

            if let Some(old_kek_id) = self.primary_kek_id.read().await.as_ref() {
                if let Some(old_key) = keys.get_mut(old_kek_id) {
                    old_key.is_primary = false;
                }
            }

            keys.insert(new_dek.id.clone(), new_dek);
            keys.insert(new_kek.id.clone(), new_kek);
        }

        *self.primary_dek_id.write().await = Some(new_dek_id.clone());
        *self.primary_kek_id.write().await = Some(new_kek_id.clone());

        info!("Rotated encryption keys: DEK={new_dek_id}, KEK={new_kek_id}");

        Ok((new_dek_id, new_kek_id))
    }

    pub async fn get_key(&self, key_id: &str) -> Option<EncryptionKey> {
        let keys = self.keys.read().await;
        keys.get(key_id).cloned()
    }

    pub async fn add_key(&self, key: EncryptionKey) {
        let mut keys = self.keys.write().await;
        keys.insert(key.id.clone(), key);
    }

    pub async fn remove_expired_keys(&self) -> usize {
        let mut keys = self.keys.write().await;
        let initial_count = keys.len();

        let primary_dek = self.primary_dek_id.read().await.clone();
        let primary_kek = self.primary_kek_id.read().await.clone();

        keys.retain(|id, key| {
            if Some(id.clone()) == primary_dek || Some(id.clone()) == primary_kek {
                return true;
            }
            !key.is_expired()
        });

        let removed = initial_count - keys.len();
        if removed > 0 {
            info!("Removed {removed} expired encryption keys");
        }
        removed
    }

    pub fn config(&self) -> &EncryptionConfig {
        &self.config
    }
}

fn encrypt_aes_gcm(plaintext: &[u8], key: &[u8]) -> Result<(String, String)> {
    if key.len() != KEY_SIZE {
        return Err(anyhow!("Invalid key size: expected {KEY_SIZE}, got {}", key.len()));
    }

    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);

    let mut rng = rand::rng();
    let nonce_bytes: [u8; NONCE_SIZE] = std::array::from_fn(|_| rng.random());
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| anyhow!("Encryption failed: {e}"))?;

    let nonce_b64 = BASE64.encode(nonce_bytes);
    let ciphertext_b64 = BASE64.encode(ciphertext);

    Ok((nonce_b64, ciphertext_b64))
}

fn decrypt_aes_gcm(nonce_b64: &str, ciphertext_b64: &str, key: &[u8]) -> Result<Vec<u8>> {
    if key.len() != KEY_SIZE {
        return Err(anyhow!("Invalid key size: expected {KEY_SIZE}, got {}", key.len()));
    }

    let nonce_bytes = BASE64
        .decode(nonce_b64)
        .map_err(|e| anyhow!("Invalid nonce encoding: {e}"))?;

    if nonce_bytes.len() != NONCE_SIZE {
        return Err(anyhow!("Invalid nonce size"));
    }

    let ciphertext = BASE64
        .decode(ciphertext_b64)
        .map_err(|e| anyhow!("Invalid ciphertext encoding: {e}"))?;

    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&nonce_bytes);

    cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| anyhow!("Decryption failed: {e}"))
}

pub fn derive_key_from_password(password: &str, salt: &[u8]) -> Result<Vec<u8>> {
    if salt.len() < SALT_SIZE {
        return Err(anyhow!("Salt too short"));
    }

    let mut key = vec![0u8; KEY_SIZE];

    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(salt);

    let mut result = hasher.finalize_reset();

    for _ in 0..PBKDF2_ITERATIONS {
        hasher.update(result);
        hasher.update(salt);
        result = hasher.finalize_reset();
    }

    key.copy_from_slice(&result[..KEY_SIZE]);
    Ok(key)
}

pub fn generate_salt() -> Vec<u8> {
    let mut rng = rand::rng();
    (0..SALT_SIZE).map(|_| rng.random()).collect()
}

pub fn encrypt_field(plaintext: &str, key: &[u8]) -> Result<String> {
    let (nonce, ciphertext) = encrypt_aes_gcm(plaintext.as_bytes(), key)?;
    Ok(format!("1:{}:{}", nonce, ciphertext))
}

pub fn decrypt_field(encrypted: &str, key: &[u8]) -> Result<String> {
    let parts: Vec<&str> = encrypted.split(':').collect();
    if parts.len() != 3 {
        return Err(anyhow!("Invalid encrypted field format"));
    }

    let version: u32 = parts[0].parse().map_err(|_| anyhow!("Invalid version"))?;
    if version != 1 {
        return Err(anyhow!("Unsupported encryption version: {version}"));
    }

    let decrypted = decrypt_aes_gcm(parts[1], parts[2], key)?;
    String::from_utf8(decrypted).map_err(|e| anyhow!("Invalid UTF-8: {e}"))
}

pub fn hash_for_search(plaintext: &str, pepper: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(plaintext.as_bytes());
    hasher.update(pepper);
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_aes_gcm() {
        let key = vec![0u8; KEY_SIZE];
        let plaintext = b"Hello, World!";

        let (nonce, ciphertext) = encrypt_aes_gcm(plaintext, &key).expect("Encrypt failed");
        let decrypted = decrypt_aes_gcm(&nonce, &ciphertext, &key).expect("Decrypt failed");

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encryption_key_generation() {
        let key = EncryptionKey::generate(KeyPurpose::DataEncryption);

        assert_eq!(key.key_data.len(), KEY_SIZE);
        assert!(!key.is_primary);
        assert!(!key.is_expired());
        assert!(key.is_valid());
    }

    #[test]
    fn test_encryption_key_primary() {
        let key = EncryptionKey::generate_primary(KeyPurpose::DataEncryption, 30);

        assert!(key.is_primary);
        assert!(key.expires_at.is_some());
        assert!(!key.is_expired());
    }

    #[test]
    fn test_encrypted_data_compact_format() {
        let data = EncryptedData {
            algorithm: "AES-256-GCM".into(),
            nonce: "abc123".into(),
            ciphertext: "xyz789".into(),
            key_id: Some("key-1".into()),
            version: 1,
        };

        let compact = data.to_string_compact();
        let parsed = EncryptedData::from_string_compact(&compact).expect("Parse failed");

        assert_eq!(parsed.version, data.version);
        assert_eq!(parsed.algorithm, data.algorithm);
        assert_eq!(parsed.nonce, data.nonce);
        assert_eq!(parsed.ciphertext, data.ciphertext);
        assert_eq!(parsed.key_id, data.key_id);
    }

    #[test]
    fn test_derive_key_from_password() {
        let password = "test_password";
        let salt = generate_salt();

        let key1 = derive_key_from_password(password, &salt).expect("Derive failed");
        let key2 = derive_key_from_password(password, &salt).expect("Derive failed");

        assert_eq!(key1, key2);
        assert_eq!(key1.len(), KEY_SIZE);
    }

    #[test]
    fn test_field_encryption() {
        let key = vec![0u8; KEY_SIZE];
        let plaintext = "sensitive_data";

        let encrypted = encrypt_field(plaintext, &key).expect("Encrypt failed");
        let decrypted = decrypt_field(&encrypted, &key).expect("Decrypt failed");

        assert_eq!(decrypted, plaintext);
        assert!(encrypted.starts_with("1:"));
    }

    #[test]
    fn test_hash_for_search() {
        let pepper = b"secret_pepper";
        let value = "searchable_value";

        let hash1 = hash_for_search(value, pepper);
        let hash2 = hash_for_search(value, pepper);

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64);
    }

    #[tokio::test]
    async fn test_encryption_manager() {
        let config = EncryptionConfig::default();
        let manager = EncryptionManager::new(config).expect("Manager creation failed");
        manager.initialize().await.expect("Init failed");

        let plaintext = b"Test data to encrypt";
        let encrypted = manager.encrypt(plaintext).await.expect("Encrypt failed");
        let decrypted = manager.decrypt(&encrypted).await.expect("Decrypt failed");

        assert_eq!(decrypted, plaintext);
    }

    #[tokio::test]
    async fn test_envelope_encryption() {
        let mut config = EncryptionConfig::default();
        config.envelope_encryption = true;
        let manager = EncryptionManager::new(config).expect("Manager creation failed");
        manager.initialize().await.expect("Init failed");

        let plaintext = b"Sensitive data with envelope encryption";
        let envelope = manager
            .encrypt_with_envelope(plaintext)
            .await
            .expect("Envelope encrypt failed");
        let decrypted = manager
            .decrypt_with_envelope(&envelope)
            .await
            .expect("Envelope decrypt failed");

        assert_eq!(decrypted, plaintext);
    }

    #[tokio::test]
    async fn test_string_encryption() {
        let config = EncryptionConfig::default();
        let manager = EncryptionManager::new(config).expect("Manager creation failed");
        manager.initialize().await.expect("Init failed");

        let plaintext = "Hello, encrypted world!";
        let encrypted = manager
            .encrypt_string(plaintext)
            .await
            .expect("Encrypt failed");
        let decrypted = manager
            .decrypt_string(&encrypted)
            .await
            .expect("Decrypt failed");

        assert_eq!(decrypted, plaintext);
    }

    #[tokio::test]
    async fn test_key_rotation() {
        let config = EncryptionConfig::default();
        let manager = EncryptionManager::new(config).expect("Manager creation failed");
        manager.initialize().await.expect("Init failed");

        let old_dek = manager.primary_dek_id.read().await.clone();

        let (new_dek, _new_kek) = manager.rotate_keys().await.expect("Rotation failed");

        assert_ne!(old_dek, Some(new_dek));
    }
}
