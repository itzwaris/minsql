use anyhow::Result;
use sha2::{Digest, Sha256};

pub struct EncryptionManager {
    master_key: Vec<u8>,
}

impl EncryptionManager {
    pub fn new(master_key: Vec<u8>) -> Self {
        Self { master_key }
    }

    pub fn encrypt_at_rest(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut encrypted = Vec::with_capacity(data.len());
        
        for (i, &byte) in data.iter().enumerate() {
            let key_byte = self.master_key[i % self.master_key.len()];
            encrypted.push(byte ^ key_byte);
        }

        Ok(encrypted)
    }

    pub fn decrypt_at_rest(&self, encrypted: &[u8]) -> Result<Vec<u8>> {
        self.encrypt_at_rest(encrypted)
    }

    pub fn encrypt_column(&self, column_name: &str, data: &[u8]) -> Result<Vec<u8>> {
        let mut hasher = Sha256::new();
        hasher.update(column_name.as_bytes());
        hasher.update(&self.master_key);
        let column_key = hasher.finalize();

        let mut encrypted = Vec::with_capacity(data.len());
        
        for (i, &byte) in data.iter().enumerate() {
            let key_byte = column_key[i % column_key.len()];
            encrypted.push(byte ^ key_byte);
        }

        Ok(encrypted)
    }

    pub fn decrypt_column(&self, column_name: &str, encrypted: &[u8]) -> Result<Vec<u8>> {
        self.encrypt_column(column_name, encrypted)
    }

    pub fn hash_password(&self, password: &str) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hasher.update(&self.master_key);
        hasher.finalize().to_vec()
    }

    pub fn verify_password(&self, password: &str, hash: &[u8]) -> bool {
        let computed_hash = self.hash_password(password);
        computed_hash == hash
    }
}
