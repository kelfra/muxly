use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use rand::Rng;
use sha2::{Digest, Sha256};

/// Credentials store
pub struct Credentials {
    /// Path to the credentials file
    path: PathBuf,
    /// Key for encrypting/decrypting credentials
    key: [u8; 32],
    /// In-memory credentials cache
    credentials: HashMap<String, EncryptedCredential>,
}

/// Encrypted credential
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EncryptedCredential {
    /// Base64-encoded encrypted value
    value: String,
    /// Base64-encoded nonce
    nonce: String,
}

impl Credentials {
    /// Create a new credentials store
    pub fn new(path: impl AsRef<Path>, master_key: &str) -> Result<Self> {
        let path = path.as_ref().to_owned();
        
        // Derive a key from the master key
        let key = derive_key(master_key);
        
        // Load existing credentials if the file exists
        let credentials = if path.exists() {
            Self::load_credentials(&path)?
        } else {
            HashMap::new()
        };
        
        Ok(Self {
            path,
            key,
            credentials,
        })
    }
    
    /// Load credentials from a file
    fn load_credentials(path: &Path) -> Result<HashMap<String, EncryptedCredential>> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        
        let credentials: HashMap<String, EncryptedCredential> = serde_json::from_str(&contents)?;
        Ok(credentials)
    }
    
    /// Save credentials to a file
    fn save_credentials(&self) -> Result<()> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Serialize and write to file
        let contents = serde_json::to_string_pretty(&self.credentials)?;
        let mut file = File::create(&self.path)?;
        file.write_all(contents.as_bytes())?;
        
        Ok(())
    }
    
    /// Set a credential
    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        // Encrypt the value
        let cipher = Aes256Gcm::new_from_slice(&self.key)?;
        
        // Generate a random nonce
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // Encrypt the value
        let ciphertext = cipher.encrypt(nonce, value.as_bytes())?;
        
        // Store the encrypted credential
        self.credentials.insert(
            key.to_string(),
            EncryptedCredential {
                value: STANDARD.encode(ciphertext),
                nonce: STANDARD.encode(nonce),
            },
        );
        
        // Save the updated credentials
        self.save_credentials()?;
        
        Ok(())
    }
    
    /// Get a credential
    pub fn get(&self, key: &str) -> Result<String> {
        // Get the encrypted credential
        let credential = self.credentials.get(key)
            .ok_or_else(|| anyhow::anyhow!("Credential not found: {}", key))?;
        
        // Decrypt the value
        let cipher = Aes256Gcm::new_from_slice(&self.key)?;
        
        // Decode base64 values
        let ciphertext = STANDARD.decode(&credential.value)?;
        let nonce_bytes = STANDARD.decode(&credential.nonce)?;
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // Decrypt the value
        let plaintext = cipher.decrypt(nonce, ciphertext.as_ref())?;
        let value = String::from_utf8(plaintext)?;
        
        Ok(value)
    }
    
    /// Delete a credential
    pub fn delete(&mut self, key: &str) -> Result<()> {
        // Remove the credential
        self.credentials.remove(key);
        
        // Save the updated credentials
        self.save_credentials()?;
        
        Ok(())
    }
    
    /// List all credential keys
    pub fn list(&self) -> Vec<String> {
        self.credentials.keys().cloned().collect()
    }
}

/// Derive a key from a master key
fn derive_key(master_key: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(master_key.as_bytes());
    let result = hasher.finalize();
    
    // Convert the result to a 32-byte array
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    
    key
} 