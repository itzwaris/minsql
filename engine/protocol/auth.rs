use anyhow::Result;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct Credentials {
    pub username: String,
    pub password_hash: Vec<u8>,
}

impl Credentials {
    pub fn new(username: String, password: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        let password_hash = hasher.finalize().to_vec();

        Self {
            username,
            password_hash,
        }
    }

    pub fn verify(&self, password: &str) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        let hash = hasher.finalize().to_vec();

        self.password_hash == hash
    }
}

pub struct AuthManager {
    users: dashmap::DashMap<String, Credentials>,
}

impl AuthManager {
    pub fn new() -> Self {
        let manager = Self {
            users: dashmap::DashMap::new(),
        };

        manager.users.insert(
            "admin".to_string(),
            Credentials::new("admin".to_string(), "admin"),
        );

        manager
    }

    pub fn authenticate(&self, username: &str, password: &str) -> Result<()> {
        let entry = self.users.get(username);

        match entry {
            Some(creds) => {
                if creds.verify(password) {
                    Ok(())
                } else {
                    anyhow::bail!("Invalid password")
                }
            }
            None => anyhow::bail!("User not found"),
        }
    }

    pub fn add_user(&self, username: String, password: &str) -> Result<()> {
        if self.users.contains_key(&username) {
            anyhow::bail!("User already exists");
        }

        self.users
            .insert(username.clone(), Credentials::new(username, password));
        Ok(())
    }
}
