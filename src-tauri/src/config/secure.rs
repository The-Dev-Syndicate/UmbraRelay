use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

// Legacy TokenStore for migration purposes
pub type TokenStore = HashMap<i64, String>;

#[derive(Debug, Serialize, Deserialize, Default)]
struct TokenPair {
    access_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    refresh_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct SecretStoreData {
    secrets: HashMap<String, String>, // key: secret_id as string, value: JSON-encoded TokenPair or plain secret
}

// SecretStore for storing encrypted secret values
// Uses file-based storage in app data directory
pub struct SecretStore {
    file_path: PathBuf,
    data: Mutex<SecretStoreData>,
}

impl SecretStore {
    pub fn new(app_data_dir: &PathBuf) -> Result<Self, String> {
        let file_path = app_data_dir.join("secrets_store.json");
        
        // Load existing data or create new
        let data = if file_path.exists() {
            let content = fs::read_to_string(&file_path)
                .map_err(|e| format!("Failed to read secrets store: {}", e))?;
            serde_json::from_str::<SecretStoreData>(&content)
                .unwrap_or_default()
        } else {
            SecretStoreData::default()
        };
        
        Ok(Self {
            file_path,
            data: Mutex::new(data),
        })
    }

    // Get access token by secret_id
    pub fn get(&self, secret_id: i64) -> Result<Option<String>, String> {
        let data = self.data.lock()
            .map_err(|e| format!("Failed to lock secret store: {}", e))?;
        let key = secret_id.to_string();
        
        if let Some(value) = data.secrets.get(&key) {
            // Try to parse as TokenPair (new format)
            if let Ok(token_pair) = serde_json::from_str::<TokenPair>(value) {
                return Ok(Some(token_pair.access_token));
            }
            // Fall back to plain string (legacy format)
            return Ok(Some(value.clone()));
        }
        Ok(None)
    }

    // Set access token by secret_id (preserves refresh token if it exists)
    pub fn set(&self, secret_id: i64, value: &str) -> Result<(), String> {
        let mut data = self.data.lock()
            .map_err(|e| format!("Failed to lock secret store: {}", e))?;
        let key = secret_id.to_string();
        
        // Get existing refresh token if present
        let refresh_token = data.secrets.get(&key)
            .and_then(|v| serde_json::from_str::<TokenPair>(v).ok())
            .and_then(|tp| tp.refresh_token);
        
        // Store as TokenPair
        let token_pair = TokenPair {
            access_token: value.to_string(),
            refresh_token,
        };
        let json = serde_json::to_string(&token_pair)
            .map_err(|e| format!("Failed to serialize token pair: {}", e))?;
        data.secrets.insert(key, json);
        self.save(&data)?;
        Ok(())
    }

    // Delete secret value by secret_id
    pub fn delete(&self, secret_id: i64) -> Result<(), String> {
        let mut data = self.data.lock()
            .map_err(|e| format!("Failed to lock secret store: {}", e))?;
        let key = secret_id.to_string();
        data.secrets.remove(&key);
        self.save(&data)?;
        Ok(())
    }

    // Get refresh token by secret_id (stored together with access token)
    pub fn get_refresh_token(&self, secret_id: i64) -> Result<Option<String>, String> {
        let data = self.data.lock()
            .map_err(|e| format!("Failed to lock secret store: {}", e))?;
        let key = secret_id.to_string();
        
        if let Some(value) = data.secrets.get(&key) {
            // Try to parse as TokenPair (new format)
            if let Ok(token_pair) = serde_json::from_str::<TokenPair>(value) {
                return Ok(token_pair.refresh_token);
            }
        }
        Ok(None)
    }

    // Set both tokens together (more efficient)
    pub fn set_tokens(&self, secret_id: i64, access_token: &str, refresh_token: Option<&str>) -> Result<(), String> {
        let mut data = self.data.lock()
            .map_err(|e| format!("Failed to lock secret store: {}", e))?;
        let key = secret_id.to_string();
        
        let token_pair = TokenPair {
            access_token: access_token.to_string(),
            refresh_token: refresh_token.map(|s| s.to_string()),
        };
        let json = serde_json::to_string(&token_pair)
            .map_err(|e| format!("Failed to serialize token pair: {}", e))?;
        data.secrets.insert(key, json);
        self.save(&data)?;
        Ok(())
    }

    // Save data to file
    fn save(&self, data: &SecretStoreData) -> Result<(), String> {
        let json = serde_json::to_string_pretty(data)
            .map_err(|e| format!("Failed to serialize secrets store: {}", e))?;
        fs::write(&self.file_path, json)
            .map_err(|e| format!("Failed to write secrets store: {}", e))?;
        Ok(())
    }
}

