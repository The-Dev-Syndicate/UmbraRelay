use crate::storage::{Database, models::{Item, CustomView, Group, Secret}};
use crate::config::{TokenStore, SecretStore};
use crate::oauth::github::{GitHubOAuth, GitHubRepository, PollResult};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};

#[derive(Debug, Serialize, Deserialize)]
pub struct SourceInput {
    pub source_type: String,
    pub name: String,
    pub config_json: serde_json::Value,
    pub token: Option<String>, // Deprecated, use secret_id
    pub secret_id: Option<i64>, // For GitHub sources and other API-based sources
    pub group_ids: Option<Vec<i64>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateSourceInput {
    pub name: Option<String>,
    pub config_json: Option<serde_json::Value>,
    pub enabled: Option<bool>,
    pub token: Option<String>, // Deprecated, use secret_id
    pub secret_id: Option<Option<i64>>, // None = don't update, Some(None) = clear, Some(Some(id)) = set
    pub group_ids: Option<Vec<i64>>, // None = don't update, Some(vec) = set groups (empty vec clears)
}

#[tauri::command]
pub async fn get_items(
    db: State<'_, Mutex<Database>>,
    state_filter: Option<String>,
    group_filter: Option<String>,
    source_ids: Option<Vec<i64>>,
    group_names: Option<Vec<String>>,
) -> Result<Vec<Item>, String> {
    let db = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    db.get_items(
        state_filter.as_deref(),
        group_filter.as_deref(),
        source_ids.as_deref(),
        group_names.as_deref(),
    )
        .map_err(|e| format!("Failed to get items: {}", e))
}

#[tauri::command]
pub async fn get_item(
    db: State<'_, Mutex<Database>>,
    id: i64,
) -> Result<Item, String> {
    let db = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    db.get_item(id)
        .map_err(|e| format!("Failed to get item: {}", e))
}

#[tauri::command]
pub async fn update_item_state(
    db: State<'_, Mutex<Database>>,
    id: i64,
    state: String,
) -> Result<(), String> {
    let db = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    db.update_item_state(id, &state)
        .map_err(|e| format!("Failed to update item state: {}", e))
}

#[tauri::command]
pub async fn clear_source_items(
    db: State<'_, Mutex<Database>>,
    source_name: String,
) -> Result<usize, String> {
    let db = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    db.delete_items_by_source_name(&source_name)
        .map_err(|e| format!("Failed to clear items for source: {}", e))
}

#[tauri::command]
pub async fn get_sources(
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<serde_json::Value>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let sources = db_guard.get_all_sources()
        .map_err(|e| format!("Failed to get sources: {}", e))?;
    
    // Get group_ids for each source
    let mut result = Vec::new();
    for source in sources {
        let group_ids = db_guard.get_source_groups(source.id)
            .unwrap_or_default();
        
        // Convert Source to JSON and add group_ids
        let mut source_json = serde_json::to_value(&source)
            .map_err(|e| format!("Failed to serialize source: {}", e))?;
        
        if let Some(obj) = source_json.as_object_mut() {
            if !group_ids.is_empty() {
                obj.insert("group_ids".to_string(), serde_json::to_value(group_ids).unwrap());
            } else {
                obj.insert("group_ids".to_string(), serde_json::json!([]));
            }
        }
        
        result.push(source_json);
    }
    
    Ok(result)
}

#[tauri::command]
pub async fn get_source_secret_id(
    db: State<'_, Mutex<Database>>,
    id: i64,
) -> Result<Option<i64>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    db_guard.get_source_secret_id(id)
        .map_err(|e| format!("Failed to get source secret_id: {}", e))
}

#[tauri::command]
pub async fn add_source(
    app: AppHandle,
    db: State<'_, Mutex<Database>>,
    source: SourceInput,
) -> Result<i64, String> {
    let config_json_str = serde_json::to_string(&source.config_json)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let source_id = db_guard.create_source(
        &source.source_type,
        &source.name,
        &config_json_str,
        source.group_ids.as_deref(),
        source.secret_id, // Use secret_id from input
    ).map_err(|e| format!("Failed to create source: {}", e))?;
    
    
    // Store token if provided (for GitHub sources)
    if let Some(token) = source.token {
        let token_store: State<'_, Mutex<TokenStore>> = app.state();
        let mut store = token_store.lock().map_err(|e| format!("Token store lock error: {}", e))?;
        store.insert(source_id, token);
    }
    
    drop(db_guard);
    
    // Immediately sync the newly added source
    let app_handle = app.clone();
    let source_id_for_sync = source_id;
    tauri::async_runtime::spawn(async move {
        // Wait a moment to ensure the source is fully created
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        // Get the source we just created
        let db_state: State<'_, Mutex<Database>> = app_handle.state();
        let source = {
            let db_guard = match db_state.lock() {
                Ok(db) => db,
                Err(_) => return,
            };
            match db_guard.get_source(source_id_for_sync) {
                Ok(s) => s,
                Err(_) => return,
            }
        };
        
        // Sync the source
        use crate::sync_source_internal;
        if let Err(e) = sync_source_internal(&app_handle, source).await {
            eprintln!("Failed to sync newly added source {}: {}", source_id_for_sync, e);
        }
    });
    
    Ok(source_id)
}

#[tauri::command]
pub async fn update_source(
    app: AppHandle,
    db: State<'_, Mutex<Database>>,
    id: i64,
    update: UpdateSourceInput,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    
    let config_json_str = update.config_json
        .as_ref()
        .map(|c| serde_json::to_string(c))
        .transpose()
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    
    // Convert Option<Vec<i64>> to Option<Option<&[i64]>>
    // None = don't update groups, Some(vec) = set groups (empty vec clears)
    let group_ids_ref: Option<Option<&[i64]>> = update.group_ids.as_ref().map(|v| Some(v.as_slice()));
    
    // Convert Option<Option<i64>> to Option<Option<&i64>>
    // None = don't update, Some(None) = clear, Some(Some(id)) = set
    let secret_id_ref: Option<Option<&i64>> = update.secret_id.as_ref().map(|opt| opt.as_ref());
    
    db_guard.update_source(
        id,
        update.name.as_deref(),
        config_json_str.as_deref(),
        update.enabled,
        group_ids_ref,
        secret_id_ref,
    ).map_err(|e| format!("Failed to update source: {}", e))?;
    
    // Update token if provided
    if let Some(token) = update.token {
        let token_store: State<'_, Mutex<TokenStore>> = app.state();
        let mut store = token_store.lock().map_err(|e| format!("Token store lock error: {}", e))?;
        store.insert(id, token);
    }
    
    Ok(())
}

#[tauri::command]
pub async fn remove_source(
    app: AppHandle,
    db: State<'_, Mutex<Database>>,
    id: i64,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    db_guard.delete_source(id)
        .map_err(|e| format!("Failed to delete source: {}", e))?;
    
    // Remove token if exists
    let token_store: State<'_, Mutex<TokenStore>> = app.state();
    let mut store = token_store.lock().map_err(|e| format!("Token store lock error: {}", e))?;
    store.remove(&id);
    
    Ok(())
}

#[tauri::command]
pub async fn test_github_notifications(secret_id: i64, app: tauri::AppHandle) -> Result<String, String> {
    use crate::ingestion::GitHubNotificationsIngester;
    use std::sync::Mutex;
    use tauri::State;
    
    // Get token from SecretStore
    let token = {
        let secret_store: State<'_, Mutex<SecretStore>> = app.state();
        let store = secret_store.lock()
            .map_err(|e| format!("Failed to lock secret store: {}", e))?;
        store.get(secret_id)
            .map_err(|e| format!("Failed to get secret: {}", e))?
            .ok_or_else(|| format!("Secret {} not found", secret_id))?
    };
    
    eprintln!("Testing GitHub notifications with secret_id: {} (token length: {})", secret_id, token.len());
    
    // Test the API call
    let result = tokio::task::spawn_blocking(move || {
        use crate::ingestion::traits::IngestSource;
        let ingester = GitHubNotificationsIngester::new(token)
            .map_err(|e| anyhow::anyhow!("Failed to create ingester: {}", e))?;
        ingester.poll()
            .map_err(|e| anyhow::anyhow!("Failed to poll: {}", e))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?;
    
    match result {
        Ok(items) => {
            Ok(format!("Success! Fetched {} notifications", items.len()))
        }
        Err(e) => {
            Err(format!("Error: {}", e))
        }
    }
}

#[tauri::command]
pub async fn sync_source(
    app: AppHandle,
    db: State<'_, Mutex<Database>>,
    id: i64,
) -> Result<(), String> {
    // Get source and use sync_source_internal (which handles all source types including GitHub with secrets)
    let source = {
        let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
        db_guard.get_source(id)
            .map_err(|e| format!("Failed to get source: {}", e))?
    };
    
    // Use the internal sync function which handles all source types properly
    use crate::sync_source_internal;
    sync_source_internal(&app, source)
        .await
        .map_err(|e| format!("Failed to sync source: {}", e))?;
    
    Ok(())
}

#[tauri::command]
pub async fn cleanup_old_items(
    db: State<'_, Mutex<Database>>,
    days: Option<i64>,
) -> Result<usize, String> {
    let db = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let days_to_keep = days.unwrap_or(30); // Default to 30 days
    db.cleanup_old_items(days_to_keep)
        .map_err(|e| format!("Failed to cleanup old items: {}", e))
}

#[tauri::command]
pub async fn make_items_leaving_soon(
    db: State<'_, Mutex<Database>>,
    count: Option<i64>,
) -> Result<usize, String> {
    let db = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let item_count = count.unwrap_or(7); // Default to 7 items
    db.make_items_leaving_soon(item_count)
        .map_err(|e| format!("Failed to make items leaving soon: {}", e))
}

#[tauri::command]
pub async fn get_custom_views(
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<CustomView>, String> {
    let db = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    db.get_all_custom_views()
        .map_err(|e| format!("Failed to get custom views: {}", e))
}

#[tauri::command]
pub async fn get_custom_view(
    db: State<'_, Mutex<Database>>,
    id: i64,
) -> Result<CustomView, String> {
    let db = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    db.get_custom_view(id)
        .map_err(|e| format!("Failed to get custom view: {}", e))
}

#[tauri::command]
pub async fn add_custom_view(
    db: State<'_, Mutex<Database>>,
    name: String,
    source_ids: Option<String>, // JSON array string
    group_names: Option<String>, // JSON array string
) -> Result<i64, String> {
    let db = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    db.create_custom_view(
        &name,
        source_ids.as_deref(),
        group_names.as_deref(),
    )
        .map_err(|e| format!("Failed to create custom view: {}", e))
}

#[tauri::command]
pub async fn update_custom_view(
    db: State<'_, Mutex<Database>>,
    id: i64,
    name: String,
    source_ids: Option<String>, // JSON array string
    group_names: Option<String>, // JSON array string
) -> Result<(), String> {
    let db = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    db.update_custom_view(
        id,
        &name,
        source_ids.as_deref(),
        group_names.as_deref(),
    )
        .map_err(|e| format!("Failed to update custom view: {}", e))
}

#[tauri::command]
pub async fn remove_custom_view(
    db: State<'_, Mutex<Database>>,
    id: i64,
) -> Result<(), String> {
    let db = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    db.delete_custom_view(id)
        .map_err(|e| format!("Failed to delete custom view: {}", e))
}

// Group commands
#[tauri::command]
pub async fn get_groups(
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<Group>, String> {
    let db = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    db.get_all_groups()
        .map_err(|e| format!("Failed to get groups: {}", e))
}

#[tauri::command]
pub async fn add_group(
    db: State<'_, Mutex<Database>>,
    name: String,
) -> Result<i64, String> {
    let db = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    db.create_group(&name)
        .map_err(|e| format!("Failed to create group: {}", e))
}

#[tauri::command]
pub async fn update_group(
    db: State<'_, Mutex<Database>>,
    id: i64,
    name: String,
) -> Result<(), String> {
    let db = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    db.update_group(id, &name)
        .map_err(|e| format!("Failed to update group: {}", e))
}

#[tauri::command]
pub async fn remove_group(
    db: State<'_, Mutex<Database>>,
    id: i64,
) -> Result<(), String> {
    let db = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    db.delete_group(id)
        .map_err(|e| format!("Failed to delete group: {}", e))
}

#[tauri::command]
pub async fn sync_all_sources(
    app: AppHandle,
    db: State<'_, Mutex<Database>>,
) -> Result<(), String> {
    // Get all enabled sources
    let sources = {
        let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
        let all_sources = db_guard.get_all_sources()
            .map_err(|e| format!("Failed to get sources: {}", e))?;
        drop(db_guard);
        all_sources.into_iter().filter(|s| s.enabled).collect::<Vec<_>>()
    };

    // Sync each source
    for source in sources {
        let _ = sync_source(app.clone(), db.clone(), source.id).await;
    }

    Ok(())
}

#[tauri::command]
pub async fn get_user_preference(
    db: State<'_, Mutex<Database>>,
    key: String,
) -> Result<Option<String>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    db_guard.get_user_preference(&key)
        .map_err(|e| format!("Failed to get user preference: {}", e))
}

#[tauri::command]
pub async fn set_user_preference(
    db: State<'_, Mutex<Database>>,
    key: String,
    value: String,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    db_guard.set_user_preference(&key, &value)
        .map_err(|e| format!("Failed to set user preference: {}", e))
}

// Secret management commands
#[tauri::command]
pub async fn get_secrets(
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<Secret>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    db_guard.get_all_secrets()
        .map_err(|e| format!("Failed to get secrets: {}", e))
}

#[tauri::command]
pub async fn get_secret(
    db: State<'_, Mutex<Database>>,
    id: i64,
) -> Result<Secret, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    db_guard.get_secret(id)
        .map_err(|e| format!("Failed to get secret: {}", e))
}

#[tauri::command]
pub async fn create_secret(
    app: AppHandle,
    db: State<'_, Mutex<Database>>,
    name: String,
    value: String,
    ttl_type: Option<String>,
    ttl_value: Option<String>,
    refresh_token: Option<String>,
) -> Result<i64, String> {
    let ttl_type = ttl_type.unwrap_or_else(|| "forever".to_string());
    
    // Create secret in database (refresh_token_id will be set after we create the secret)
    let secret_id = {
        let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
        db_guard.create_secret(
            &name,
            &ttl_type,
            ttl_value.as_deref(),
            None, // refresh_token_id will be set below if we have a refresh token
        ).map_err(|e| format!("Failed to create secret: {}", e))?
    };
    
    // Store both tokens together using set_tokens (more efficient)
    let secret_store: State<'_, Mutex<SecretStore>> = app.state();
    let store = secret_store.lock().map_err(|e| format!("Secret store lock error: {}", e))?;
    store.set_tokens(secret_id, &value, refresh_token.as_deref())
        .map_err(|e| format!("Failed to store tokens: {}", e))?;
    drop(store);
    
    // Update the secret to indicate it has a refresh token (we use secret_id as refresh_token_id for simplicity)
    if refresh_token.is_some() {
        let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
        db_guard.set_refresh_token_id(secret_id, Some(secret_id))
            .map_err(|e| format!("Failed to set refresh_token_id: {}", e))?;
    }
    
    Ok(secret_id)
}

#[tauri::command]
pub async fn update_secret(
    app: AppHandle,
    db: State<'_, Mutex<Database>>,
    id: i64,
    name: Option<String>,
    value: Option<String>,
    ttl_type: Option<String>,
    ttl_value: Option<String>,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    
    if let Some(name) = name {
        db_guard.update_secret(id, Some(name.as_str()), None, None)
            .map_err(|e| format!("Failed to update secret name: {}", e))?;
    }
    
    if let Some(ttl_type) = ttl_type {
        db_guard.update_secret(id, None, Some(ttl_type.as_str()), Some(ttl_value.as_deref()))
            .map_err(|e| format!("Failed to update secret TTL: {}", e))?;
    }
    
    if let Some(value) = value {
        let secret_store: State<'_, Mutex<SecretStore>> = app.state();
        let store = secret_store.lock().map_err(|e| format!("Secret store lock error: {}", e))?;
        // set() preserves refresh token automatically
        store.set(id, &value)
            .map_err(|e| format!("Failed to update secret value: {}", e))?;
    }
    
    Ok(())
}

#[tauri::command]
pub async fn delete_secret(
    app: AppHandle,
    db: State<'_, Mutex<Database>>,
    id: i64,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    db_guard.delete_secret(id)
        .map_err(|e| format!("Failed to delete secret: {}", e))?;
    
    let secret_store: State<'_, Mutex<SecretStore>> = app.state();
    let store = secret_store.lock().map_err(|e| format!("Secret store lock error: {}", e))?;
    store.delete(id)
        .map_err(|e| format!("Failed to delete secret value: {}", e))?;
    
    Ok(())
}

#[tauri::command]
pub async fn get_secret_value(
    app: AppHandle,
    id: i64,
) -> Result<String, String> {
    let secret_store: State<'_, Mutex<SecretStore>> = app.state();
    let store = secret_store.lock().map_err(|e| format!("Secret store lock error: {}", e))?;
    store.get(id)
        .map_err(|e| format!("Failed to get secret: {}", e))?
        .ok_or_else(|| "Secret not found".to_string())
}

#[tauri::command]
pub async fn detect_github_token_expiration(
    token: String,
) -> Result<Option<serde_json::Value>, String> {
    use reqwest::blocking::Client;
    use std::time::Duration;
    use chrono::Utc;
    
    // Check if it looks like a GitHub token (starts with ghp_ for PAT or gho_ for OAuth)
    if !token.starts_with("ghp_") && !token.starts_with("gho_") {
        // Not a GitHub token, return None
        return Ok(None);
    }
    
    // For OAuth tokens, we can't detect expiration from the token itself
    // For PATs, we also can't detect expiration from the token itself
    // However, we can try to make an API call to check if the token is valid
    // and see if we can get any expiration info
    
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    
    // Try to get user info to verify token and check rate limit headers
    let response = client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "application/vnd.github.v3+json")
        .header("User-Agent", "UmbraRelay")
        .send();
    
    match response {
        Ok(resp) => {
            if resp.status() == 401 {
                // Token is invalid/expired
                return Ok(Some(serde_json::json!({
                    "ttl_type": "absolute",
                    "ttl_value": Utc::now().to_rfc3339()
                })));
            }
            
            // Check for rate limit headers - these might give us hints
            // But GitHub doesn't expose token expiration in API responses
            // For PATs, expiration is set when the token is created, not embedded in the token
            
            // For now, we can't reliably detect expiration from the token
            // Return None to use "forever" as default
            Ok(None)
        }
        Err(_) => {
            // Network error or other issue - can't detect
            Ok(None)
        }
    }
}

// GitHub OAuth commands using Device Flow
fn get_github_oauth_config() -> String {
    std::env::var("GITHUB_CLIENT_ID")
        .unwrap_or_else(|_| {
            // Embedded client ID - UmbraRelay OAuth App
            "Iv23liLrOhnkpjmdUx4D".to_string()
        })
}

#[tauri::command]
pub async fn start_github_oauth() -> Result<serde_json::Value, String> {
    let client_id = get_github_oauth_config();
    
    let device_response = tokio::task::spawn_blocking(move || {
        let oauth = GitHubOAuth::new(client_id);
        oauth.start_device_flow()
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
    .map_err(|e| {
        let error_msg = e.to_string();
        if error_msg.contains("404") || error_msg.contains("not enabled") {
            "GitHub authorization is temporarily unavailable. Please try again later.".to_string()
        } else if error_msg.contains("network") || error_msg.contains("timeout") {
            "Unable to connect to GitHub. Please check your internet connection and try again.".to_string()
        } else {
            format!("Unable to start GitHub authorization: {}", error_msg)
        }
    })?;
    
    Ok(serde_json::json!({
        "user_code": device_response.user_code,
        "verification_uri": device_response.verification_uri,
        "verification_uri_complete": device_response.verification_uri_complete.unwrap_or_else(|| {
            format!("{}?user_code={}", device_response.verification_uri, device_response.user_code)
        }),
        "device_code": device_response.device_code,
        "interval": device_response.interval,
        "expires_in": device_response.expires_in,
    }))
}

#[tauri::command]
pub async fn poll_github_oauth_token(
    app: AppHandle,
    db: State<'_, Mutex<Database>>,
    #[allow(non_snake_case)]
    deviceCode: String,
) -> Result<serde_json::Value, String> {
    let client_id = get_github_oauth_config();
    let device_code_clone = deviceCode.clone();
    
    let poll_result = tokio::task::spawn_blocking(move || {
        let oauth = GitHubOAuth::new(client_id);
        oauth.poll_for_token(&device_code_clone)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
    .map_err(|e| format!("Failed to poll for token: {}", e))?;
    
    match poll_result {
        PollResult::Pending => {
            Ok(serde_json::json!({
                "status": "pending"
            }))
        }
        PollResult::SlowDown { new_interval } => {
            Ok(serde_json::json!({
                "status": "slow_down",
                "interval": new_interval
            }))
        }
        PollResult::Success(token_pair) => {
            // Check if a GitHub secret already exists
            let secret_name = "GitHub Device Flow Token";
            let existing_secret_id = {
                let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
                match db_guard.get_secret_by_name(secret_name)
                    .map_err(|e| format!("Failed to check for existing secret: {}", e))? {
                    Some(existing_secret) => Some(existing_secret.id),
                    None => None,
                }
            };
            
            let secret_id = if let Some(existing_id) = existing_secret_id {
                // Update existing secret
                let secret_store: State<'_, Mutex<SecretStore>> = app.state();
                let store = secret_store.lock().map_err(|e| format!("Secret store lock error: {}", e))?;
                store.set_tokens(existing_id, &token_pair.access_token, token_pair.refresh_token.as_deref())
                    .map_err(|e| format!("Failed to update secret tokens: {}", e))?;
                drop(store);
                existing_id
            } else {
                // Create new secret
                create_secret(
                    app.clone(),
                    db,
                    secret_name.to_string(),
                    token_pair.access_token,
                    Some("forever".to_string()),
                    None,
                    token_pair.refresh_token,
                ).await?
            };
            
            Ok(serde_json::json!({
                "status": "success",
                "secret_id": secret_id
            }))
        }
        PollResult::Error(error_msg) => {
            Err(error_msg)
        }
    }
}

#[tauri::command]
pub async fn get_github_repositories(
    app: AppHandle,
    secret_id: i64,
) -> Result<Vec<GitHubRepository>, String> {
    let access_token = get_secret_value(app.clone(), secret_id).await?;
    
    let client_id = get_github_oauth_config();
    
    let repos = tokio::task::spawn_blocking(move || {
        let oauth = GitHubOAuth::new(client_id);
        oauth.get_repositories(&access_token)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
    .map_err(|e| format!("Failed to fetch repositories: {}", e))?;
    
    Ok(repos)
}

// Internal helper to attempt token refresh
pub(crate) async fn refresh_github_token_internal(
    app: &AppHandle,
    secret_id: i64,
) -> Result<String, String> {
    use std::sync::Mutex;
    use tauri::State;
    
    // Get secret to check for refresh token
    let secret = {
        let db_state: State<'_, Mutex<Database>> = app.state();
        let db_guard = db_state.lock().map_err(|e| format!("Database lock error: {}", e))?;
        db_guard.get_secret(secret_id)
            .map_err(|e| format!("Failed to get secret: {}", e))?
    };
    
    // Check if we have a refresh token
    if secret.refresh_token_id.is_none() {
        return Err("No refresh token available".to_string());
    }
    
    // Get refresh token value using helper method
    let refresh_token = {
        let secret_store: State<'_, Mutex<SecretStore>> = app.state();
        let store = secret_store.lock().map_err(|e| format!("Secret store lock error: {}", e))?;
        store.get_refresh_token(secret_id)
            .map_err(|e| format!("Failed to get refresh token: {}", e))?
            .ok_or_else(|| "Refresh token not found".to_string())?
    };
    
    // Attempt to refresh
    let client_id = get_github_oauth_config();
    let token_pair = tokio::task::spawn_blocking(move || {
        let oauth = GitHubOAuth::new(client_id);
        oauth.refresh_token(&refresh_token)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
    .map_err(|e| format!("Failed to refresh token: {}", e))?;
    
    // Update both tokens together using set_tokens (more efficient)
    {
        let secret_store: State<'_, Mutex<SecretStore>> = app.state();
        let store = secret_store.lock().map_err(|e| format!("Secret store lock error: {}", e))?;
        store.set_tokens(secret_id, &token_pair.access_token, token_pair.refresh_token.as_deref())
            .map_err(|e| format!("Failed to update tokens: {}", e))?;
    }
    
    // Reset failure count on success
    {
        let db_state: State<'_, Mutex<Database>> = app.state();
        let db_guard = db_state.lock().map_err(|e| format!("Database lock error: {}", e))?;
        db_guard.reset_refresh_failure_count(secret_id)
            .map_err(|e| format!("Failed to reset failure count: {}", e))?;
    }
    
    Ok(token_pair.access_token)
}

// Internal helper to expire a secret (set expires_at to now and disable sources)
pub(crate) fn expire_secret_internal(app: &AppHandle, secret_id: i64) -> Result<(), String> {
    use std::sync::Mutex;
    use tauri::State;
    
    // Set expires_at to now
    {
        let db_state: State<'_, Mutex<Database>> = app.state();
        let db_guard = db_state.lock().map_err(|e| format!("Database lock error: {}", e))?;
        db_guard.expire_secret(secret_id)
            .map_err(|e| format!("Failed to expire secret: {}", e))?;
    }
    
    // Disable all sources using this secret
    {
        let db_state: State<'_, Mutex<Database>> = app.state();
        let db_guard = db_state.lock().map_err(|e| format!("Database lock error: {}", e))?;
        let source_ids = db_guard.get_sources_using_secret(secret_id)
            .map_err(|e| format!("Failed to get sources using secret: {}", e))?;
        
        for source_id in source_ids {
            db_guard.update_source(
                source_id,
                None,
                None,
                Some(false), // Disable
                None,
                None,
            ).map_err(|e| format!("Failed to disable source: {}", e))?;
        }
    }
    
    Ok(())
}

