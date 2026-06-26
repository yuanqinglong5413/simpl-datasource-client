use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

const SERVICE_NAME: &str = "com.simplsource.app";
const SECRETS_FILE: &str = "secrets.enc.json";

#[derive(Debug, Error)]
pub enum CredentialError {
    #[error("failed to read secrets: {0}")]
    Read(String),
    #[error("failed to write secrets: {0}")]
    Write(String),
    #[error("secret not found")]
    NotFound,
}

/// 凭证存储：优先 OS Keychain，不可用时降级为本地 AES-GCM 加密文件。
pub struct CredentialStore {
    data_dir: PathBuf,
    /// Keyring 是否可用（运行时探测一次）。
    keyring_available: bool,
}

#[derive(Serialize, Deserialize, Default)]
struct EncryptedSecrets {
    nonce: String,
    ciphertext: String,
}

#[derive(Serialize, Deserialize, Default)]
struct SecretsPayload {
    entries: HashMap<String, String>,
}

impl CredentialStore {
    pub fn new(data_dir: impl AsRef<Path>) -> Self {
        let data_dir = data_dir.as_ref().to_path_buf();
        let _ = fs::create_dir_all(&data_dir);
        let keyring_available = keyring::Entry::new(SERVICE_NAME, "probe").is_ok();

        Self {
            data_dir,
            keyring_available,
        }
    }

    fn keyring_key(connection_id: &Uuid) -> String {
        format!("conn-{connection_id}")
    }

    /// 保存连接密码等敏感字段。
    pub fn store_secret(&self, connection_id: &Uuid, secret: &str) -> Result<(), CredentialError> {
        if self.keyring_available {
            if let Ok(entry) = keyring::Entry::new(SERVICE_NAME, &Self::keyring_key(connection_id))
            {
                entry
                    .set_password(secret)
                    .map_err(|e| CredentialError::Write(e.to_string()))?;
                return Ok(());
            }
        }
        self.store_encrypted(connection_id, secret)
    }

    /// 读取敏感字段。
    pub fn get_secret(&self, connection_id: &Uuid) -> Result<Option<String>, CredentialError> {
        if self.keyring_available {
            if let Ok(entry) = keyring::Entry::new(SERVICE_NAME, &Self::keyring_key(connection_id))
            {
                match entry.get_password() {
                    Ok(value) => return Ok(Some(value)),
                    Err(keyring::Error::NoEntry) => {}
                    Err(_) => {}
                }
            }
        }
        self.load_encrypted(connection_id)
    }

    /// 删除敏感字段。
    pub fn delete_secret(&self, connection_id: &Uuid) -> Result<(), CredentialError> {
        if self.keyring_available {
            if let Ok(entry) = keyring::Entry::new(SERVICE_NAME, &Self::keyring_key(connection_id))
            {
                let _ = entry.delete_credential();
            }
        }
        let mut payload = self.load_all_encrypted()?;
        payload.entries.remove(&connection_id.to_string());
        self.save_all_encrypted(&payload)
    }

    pub fn uses_keyring(&self) -> bool {
        self.keyring_available
    }

    fn secrets_path(&self) -> PathBuf {
        self.data_dir.join(SECRETS_FILE)
    }

    fn derive_key(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(SERVICE_NAME.as_bytes());
        hasher.update(self.data_dir.to_string_lossy().as_bytes());
        hasher.finalize().into()
    }

    fn load_all_encrypted(&self) -> Result<SecretsPayload, CredentialError> {
        let path = self.secrets_path();
        if !path.exists() {
            return Ok(SecretsPayload::default());
        }
        let raw = fs::read_to_string(&path).map_err(|e| CredentialError::Read(e.to_string()))?;
        let wrapper: EncryptedSecrets =
            serde_json::from_str(&raw).map_err(|e| CredentialError::Read(e.to_string()))?;
        if wrapper.ciphertext.is_empty() {
            return Ok(SecretsPayload::default());
        }
        let nonce_bytes = B64
            .decode(&wrapper.nonce)
            .map_err(|e| CredentialError::Read(e.to_string()))?;
        let cipher_bytes = B64
            .decode(&wrapper.ciphertext)
            .map_err(|e| CredentialError::Read(e.to_string()))?;
        let cipher = Aes256Gcm::new_from_slice(&self.derive_key())
            .map_err(|e| CredentialError::Read(e.to_string()))?;
        let nonce = Nonce::from_slice(&nonce_bytes);
        let plain = cipher
            .decrypt(nonce, cipher_bytes.as_ref())
            .map_err(|e| CredentialError::Read(e.to_string()))?;
        serde_json::from_slice(&plain).map_err(|e| CredentialError::Read(e.to_string()))
    }

    fn save_all_encrypted(&self, payload: &SecretsPayload) -> Result<(), CredentialError> {
        let plain = serde_json::to_vec(payload).map_err(|e| CredentialError::Write(e.to_string()))?;
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let cipher = Aes256Gcm::new_from_slice(&self.derive_key())
            .map_err(|e| CredentialError::Write(e.to_string()))?;
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(nonce, plain.as_ref())
            .map_err(|e| CredentialError::Write(e.to_string()))?;
        let wrapper = EncryptedSecrets {
            nonce: B64.encode(nonce_bytes),
            ciphertext: B64.encode(ciphertext),
        };
        let json = serde_json::to_string_pretty(&wrapper)
            .map_err(|e| CredentialError::Write(e.to_string()))?;
        fs::write(self.secrets_path(), json).map_err(|e| CredentialError::Write(e.to_string()))
    }

    fn store_encrypted(&self, connection_id: &Uuid, secret: &str) -> Result<(), CredentialError> {
        let mut payload = self.load_all_encrypted()?;
        payload
            .entries
            .insert(connection_id.to_string(), secret.to_string());
        self.save_all_encrypted(&payload)
    }

    fn load_encrypted(&self, connection_id: &Uuid) -> Result<Option<String>, CredentialError> {
        let payload = self.load_all_encrypted()?;
        Ok(payload.entries.get(&connection_id.to_string()).cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_and_load_secret_without_keyring() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = CredentialStore {
            data_dir: dir.path().to_path_buf(),
            keyring_available: false,
        };
        let id = Uuid::new_v4();
        store.store_secret(&id, "s3cr3t").expect("store");
        let loaded = store.get_secret(&id).expect("get").expect("some");
        assert_eq!(loaded, "s3cr3t");
    }
}
