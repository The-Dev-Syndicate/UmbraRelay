mod storage;
mod config;
mod ingestion;
mod normalization;
mod commands;
mod oauth;

use storage::Database;
use config::{TokenStore, SecretStore};
use std::sync::Mutex;
use std::collections::HashMap;
use tauri::Manager;
use std::fs::OpenOptions;
use std::io::Write;

/// Log error to file for debugging on Windows
/// Tries to write to app data directory, falls back to current directory
fn log_error_to_file(message: &str) {
    // Try to get app data directory first
    let log_path = if let Ok(app_data) = std::env::var("APPDATA") {
        std::path::PathBuf::from(app_data)
            .join("UmbraRelay")
            .join("umbrarelay_error.log")
    } else if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        std::path::PathBuf::from(local_app_data)
            .join("UmbraRelay")
            .join("umbrarelay_error.log")
    } else {
        std::path::PathBuf::from("umbrarelay_error.log")
    };
    
    // Create parent directory if needed
    if let Some(parent) = log_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        let _ = writeln!(file, "[{}] {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"), message);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            // Initialize database with proper error handling
            let app_data_dir = match app.path().app_data_dir() {
                Ok(dir) => dir,
                Err(e) => {
                    let error_msg = format!("Failed to get app data directory: {}", e);
                    eprintln!("{}", error_msg);
                    log_error_to_file(&error_msg);
                    return Err(e.into());
                }
            };
            
            if let Err(e) = std::fs::create_dir_all(&app_data_dir) {
                let error_msg = format!("Failed to create app data directory: {} (path: {:?})", e, app_data_dir);
                eprintln!("{}", error_msg);
                log_error_to_file(&error_msg);
                return Err(e.into());
            }
            
            let db_path = app_data_dir.join("umbrarelay.db");
            // Use to_string_lossy() for Windows compatibility (handles non-UTF8 paths)
            let db_path_str = db_path.to_string_lossy().to_string();
            
            let db = match Database::new(&db_path_str) {
                Ok(db) => db,
                Err(e) => {
                    let error_msg = format!("Failed to initialize database at {}: {}", db_path_str, e);
                    eprintln!("{}", error_msg);
                    log_error_to_file(&error_msg);
                    return Err(format!("Failed to initialize database: {}", e).into());
                }
            };
            
            app.manage(Mutex::new(db));
            
            // Initialize legacy token store (for migration)
            let token_store: TokenStore = HashMap::new();
            app.manage(Mutex::new(token_store));
            
            // Initialize secret store
            let secret_store = match SecretStore::new(&app_data_dir) {
                Ok(store) => store,
                Err(e) => {
                    let error_msg = format!("Failed to initialize secret store: {} (path: {:?})", e, app_data_dir);
                    eprintln!("{}", error_msg);
                    log_error_to_file(&error_msg);
                    return Err(format!("Failed to initialize secret store: {}", e).into());
                }
            };
            app.manage(Mutex::new(secret_store));
            
            // Migrate existing tokens to secrets (after all state is initialized)
            let app_handle = app.handle().clone();
            migrate_tokens_to_secrets(&app_handle);
            
            // Start background polling service using Tauri's async runtime
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                background_polling_service(app_handle).await;
            });
            
            // Start secret cleanup task
            let app_handle_cleanup = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                cleanup_expired_secrets_task(app_handle_cleanup).await;
            });
            
            // Sync all enabled sources on app startup
            let app_handle_sync = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                // Wait a moment for the app to fully initialize
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                
                // Proactively refresh GitHub tokens before syncing
                refresh_github_tokens_on_startup(&app_handle_sync).await;
                
                // Get all enabled sources and sync them
                let sources = match get_sources_sync(&app_handle_sync) {
                    Ok(sources) => sources.into_iter().filter(|s| s.enabled).collect::<Vec<_>>(),
                    Err(e) => {
                        eprintln!("Failed to get sources for initial sync: {}", e);
                        return;
                    }
                };
                
                // Sync each enabled source
                for source in sources {
                    let result = sync_source_internal(&app_handle_sync, source).await;
                    if let Err(e) = result {
                        eprintln!("Failed to sync source during startup: {}", e);
                    }
                }
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_items,
            commands::get_item,
            commands::update_item_state,
            commands::clear_source_items,
            commands::get_sources,
            commands::get_source_secret_id,
            commands::add_source,
            commands::update_source,
            commands::remove_source,
            commands::sync_source,
            commands::sync_all_sources,
            commands::cleanup_old_items,
            commands::make_items_leaving_soon,
            commands::get_custom_views,
            commands::get_custom_view,
            commands::add_custom_view,
            commands::update_custom_view,
            commands::remove_custom_view,
            commands::get_groups,
            commands::add_group,
            commands::update_group,
            commands::remove_group,
            commands::get_user_preference,
            commands::set_user_preference,
            commands::trigger_extraction,
            commands::get_item_with_content,
            commands::get_secrets,
            commands::get_secret,
            commands::create_secret,
            commands::update_secret,
            commands::delete_secret,
            commands::get_secret_value,
            commands::detect_github_token_expiration,
            commands::start_github_oauth,
            commands::poll_github_oauth_token,
            commands::get_github_repositories,
            commands::test_github_notifications,
            commands::test_github_token,
        ])
        .run(tauri::generate_context!())
        .map_err(|e| {
            let error_msg = format!("Fatal error while running tauri application: {}", e);
            eprintln!("{}", error_msg);
            log_error_to_file(&error_msg);
            e
        })
        .expect("error while running tauri application");
}

// Helper function to get sources without holding guard across await
fn get_sources_sync(app: &tauri::AppHandle) -> Result<Vec<storage::models::Source>, rusqlite::Error> {
    let db_state: tauri::State<'_, Mutex<Database>> = app.state();
    let db = db_state.lock().map_err(|_| rusqlite::Error::SqliteFailure(
        rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_MISUSE),
        Some("Failed to lock database".to_string())
    ))?;
    db.get_all_sources()
}

/// Parses duration strings (e.g., "5m", "10m", "1h") into seconds.
fn parse_duration(duration_str: &str) -> u64 {
    let duration_str = duration_str.trim();
    
    if duration_str.is_empty() {
        return 600; // Default 10 minutes
    }
    
    let (num_str, unit) = if duration_str.ends_with('m') {
        (&duration_str[..duration_str.len() - 1], "m")
    } else if duration_str.ends_with('h') {
        (&duration_str[..duration_str.len() - 1], "h")
    } else if duration_str.ends_with('s') {
        (&duration_str[..duration_str.len() - 1], "s")
    } else {
        return 600; // Default on parse error
    };
    
    let num: u64 = num_str.parse().unwrap_or(10);
    
    match unit {
        "s" => num,
        "m" => num * 60,
        "h" => num * 3600,
        _ => 600, // Default on invalid unit
    }
}

/// Background service that periodically polls enabled sources based on their poll intervals.
async fn background_polling_service(app: tauri::AppHandle) {
    use std::time::Duration;
    use tokio::time::sleep;
    
    loop {
        // Wait a bit before starting
        sleep(Duration::from_secs(10)).await;
        
        // Get all enabled sources (helper function ensures guard is dropped)
        let sources = match get_sources_sync(&app) {
            Ok(sources) => sources,
            Err(e) => {
                eprintln!("Failed to get sources: {}", e);
                sleep(Duration::from_secs(60)).await;
                continue;
            }
        };
        
        // Poll each enabled source
        for source in sources {
            if !source.enabled {
                continue;
            }
            
            // Determine poll interval
            let poll_interval = match source.source_type.as_str() {
                "rss" | "atom" => {
                    let config_json: serde_json::Value = serde_json::from_str(&source.config_json)
                        .unwrap_or_default();
                    config_json.get("poll_interval")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "10m".to_string())
                }
                "github" => "5m".to_string(), // Default from config
                _ => "10m".to_string(),
            };
            
            let interval_seconds = parse_duration(&poll_interval);
            
            // Check if it's time to sync
            let should_sync = match source.last_synced_at {
                None => true,
                Some(last_sync) => {
                    let elapsed = chrono::Utc::now().timestamp() - last_sync;
                    elapsed >= interval_seconds as i64
                }
            };
            
            if should_sync {
                // Sync this source directly
                let db_state: tauri::State<'_, Mutex<Database>> = app.state();
                let db_guard = match db_state.lock() {
                    Ok(db) => db,
                    Err(_) => continue,
                };
                
                let source_clone = source.clone();
                drop(db_guard);
                
                let result = sync_source_internal(&app, source_clone).await;
                
                if let Err(e) = result {
                    eprintln!("Failed to sync source {}: {}", source.id, e);
                }
            }
        }
        
        // Sleep for a minute before next check
        sleep(Duration::from_secs(60)).await;
    }
}

/// Background task that periodically cleans up expired secrets and disables associated sources.
async fn cleanup_expired_secrets_task(app: tauri::AppHandle) {
    use std::time::Duration;
    use tokio::time::sleep;
    
    // Run cleanup on startup
    cleanup_expired_secrets_internal(&app);
    
    // Then run periodically (every hour)
    loop {
        sleep(Duration::from_secs(3600)).await; // 1 hour
        cleanup_expired_secrets_internal(&app);
    }
}

/// Internal function to clean up expired secrets and disable sources using them.
fn cleanup_expired_secrets_internal(app: &tauri::AppHandle) {
    use std::sync::Mutex;
    use tauri::State;
    
    let db_state: State<'_, Mutex<Database>> = app.state();
    let secret_store_state: State<'_, Mutex<SecretStore>> = app.state();
    
    let expired_secrets = {
        let db_guard = match db_state.lock() {
            Ok(db) => db,
            Err(_) => {
                eprintln!("Failed to lock database for cleanup");
                return;
            }
        };
        match db_guard.get_expired_secrets() {
            Ok(secrets) => secrets,
            Err(e) => {
                eprintln!("Failed to get expired secrets: {}", e);
                return;
            }
        }
    };
    
    for secret in expired_secrets {
        eprintln!("Cleaning up expired secret: {} (id: {})", secret.name, secret.id);
        
        // Get sources using this secret
        let source_ids = {
            let db_guard = match db_state.lock() {
                Ok(db) => db,
                Err(_) => {
                    eprintln!("Failed to lock database for cleanup");
                    continue;
                }
            };
            match db_guard.get_sources_using_secret(secret.id) {
                Ok(ids) => ids,
                Err(e) => {
                    eprintln!("Failed to get sources using secret: {}", e);
                    continue;
                }
            }
        };
        
        // Disable all sources using this secret
        for source_id in &source_ids {
            let db_guard = match db_state.lock() {
                Ok(db) => db,
                Err(_) => continue,
            };
            let _ = db_guard.update_source(
                *source_id,
                None,
                None,
                Some(false), // Disable
                None,
                None,
            );
        }
        
        // Delete secret value from SecretStore
        {
            let store = match secret_store_state.lock() {
                Ok(store) => store,
                Err(_) => continue,
            };
            let _ = store.delete(secret.id);
        }
        
        // Delete secret from database
        {
            let db_guard = match db_state.lock() {
                Ok(db) => db,
                Err(_) => continue,
            };
            let _ = db_guard.delete_secret(secret.id);
        }
        
        eprintln!("Cleaned up secret {} and disabled {} source(s)", secret.name, source_ids.len());
    }
}

/// Migrates legacy tokens from TokenStore to the new secrets system on app startup.
fn migrate_tokens_to_secrets(app: &tauri::AppHandle) {
    use std::sync::Mutex;
    use tauri::State;
    
    let token_store_state: State<'_, Mutex<TokenStore>> = app.state();
    let token_store = match token_store_state.lock() {
        Ok(store) => store,
        Err(_) => {
            eprintln!("Failed to lock token store for migration");
            return;
        }
    };
    
    if token_store.is_empty() {
        return; // No tokens to migrate
    }
    
    // Clone the tokens to avoid holding the lock
    let tokens_to_migrate: Vec<(i64, String)> = token_store.iter()
        .map(|(k, v)| (*k, v.clone()))
        .collect();
    drop(token_store);
    
    let secret_store_state: State<'_, Mutex<SecretStore>> = app.state();
    let db_state: State<'_, Mutex<Database>> = app.state();
    
    // Migrate each token
    for (source_id, token) in tokens_to_migrate {
        let secret_store = match secret_store_state.lock() {
            Ok(store) => store,
            Err(_) => {
                eprintln!("Failed to lock secret store for migration");
                continue;
            }
        };
        
        let db_guard = match db_state.lock() {
            Ok(db) => db,
            Err(_) => {
                eprintln!("Failed to lock database for migration");
                continue;
            }
        };
        
        // Get source name for secret name
        let source_name = match db_guard.get_source(source_id) {
            Ok(source) => source.name,
            Err(_) => format!("Source {}", source_id),
        };
        
        // Create secret
        let secret_id = match db_guard.create_secret(
            &format!("Migrated from {}", source_name),
            "forever",
            None,
            None,
        ) {
            Ok(id) => id,
            Err(e) => {
                eprintln!("Failed to create secret for source {}: {}", source_id, e);
                continue;
            }
        };
        
        // Store token value in SecretStore
        if let Err(e) = secret_store.set(secret_id, &token) {
            eprintln!("Failed to store secret value for source {}: {}", source_id, e);
            // Delete the secret we just created
            let _ = db_guard.delete_secret(secret_id);
            continue;
        }
        
        // Update source with secret_id
        if let Err(e) = db_guard.update_source(
            source_id,
            None,
            None,
            None,
            None,
            Some(Some(&secret_id)),
        ) {
            eprintln!("Failed to update source {} with secret_id: {}", source_id, e);
            // Clean up: delete secret and remove from SecretStore
            let _ = db_guard.delete_secret(secret_id);
            let _ = secret_store.delete(secret_id);
            continue;
        }
        
        eprintln!("Migrated token for source {} to secret {}", source_id, secret_id);
    }
    
    // Clear token store after migration
    let mut token_store = token_store_state.lock().unwrap();
    token_store.clear();
}

/// Migrates a single source's token to a secret (on-the-fly migration).
/// Runs in a blocking task to avoid Send trait issues.
fn migrate_source_token_to_secret_blocking(
    app: tauri::AppHandle,
    source_id: i64,
    token: String,
) -> anyhow::Result<i64> {
    use std::sync::Mutex;
    use tauri::State;
    
    let db_state: State<'_, Mutex<Database>> = app.state();
    let secret_store_state: State<'_, Mutex<SecretStore>> = app.state();
    
    // Get source name for secret name
    let source_name = {
        let db_guard = db_state.lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock database"))?;
        match db_guard.get_source(source_id) {
            Ok(source) => source.name,
            Err(_) => format!("Source {}", source_id),
        }
    };
    
    // Create secret
    let secret_id = {
        let db_guard = db_state.lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock database"))?;
        db_guard.create_secret(
            &format!("Migrated from {}", source_name),
            "forever",
            None,
            None,
        )
        .map_err(|e| anyhow::anyhow!("Failed to create secret: {}", e))?
    };
    
    // Store token value in SecretStore
    {
        let secret_store = secret_store_state.lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock secret store"))?;
        secret_store.set(secret_id, &token)
            .map_err(|e| anyhow::anyhow!("Failed to store secret value: {}", e))?;
    }
    
    // Update source with secret_id
    {
        let db_guard = db_state.lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock database"))?;
        db_guard.update_source(
            source_id,
            None,
            None,
            None,
            None,
            Some(Some(&secret_id)),
        )
        .map_err(|e| anyhow::anyhow!("Failed to update source with secret_id: {}", e))?;
    }
    
    // Remove token from TokenStore
    {
        let token_store_state: State<'_, Mutex<TokenStore>> = app.state();
        let mut token_store = token_store_state.lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock token store"))?;
        token_store.remove(&source_id);
    }
    
    eprintln!("Migrated token for source {} to secret {} (on-the-fly)", source_id, secret_id);
    Ok(secret_id)
}

/// Syncs a source by creating the appropriate ingester, fetching items, and storing them.
/// Handles token refresh for GitHub sources on 401 errors.
pub async fn sync_source_internal(app: &tauri::AppHandle, source: storage::models::Source) -> anyhow::Result<()> {
    use crate::ingestion::{RssIngester, AtomIngester, GitHubIngester, GitHubNotificationsIngester, traits::IngestSource};
    use crate::normalization::normalize_and_dedupe;
    use anyhow::Context;
    
    let config: serde_json::Value = serde_json::from_str(&source.config_json)
        .context("Failed to parse source config")?;
    
    // Create appropriate ingester and poll (using spawn_blocking for blocking operations)
    let items = match source.source_type.as_str() {
        "rss" => {
            let url = config.get("url")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing RSS URL in config"))?
                .to_string();
            
            tokio::task::spawn_blocking(move || {
                let ingester = RssIngester::new(url)?;
                ingester.poll()
            })
            .await
            .map_err(|e| anyhow::anyhow!("Task join error: {}", e))?
        }
        "atom" => {
            let url = config.get("url")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing ATOM URL in config"))?
                .to_string();
            
            tokio::task::spawn_blocking(move || {
                let ingester = AtomIngester::new(url)?;
                ingester.poll()
            })
            .await
            .map_err(|e| anyhow::anyhow!("Task join error: {}", e))?
        }
        "github" => {
            // Get secret_id from source
            let secret_id = {
                let db_state: tauri::State<'_, Mutex<Database>> = app.state();
                let db_guard = db_state.lock()
                    .map_err(|_| anyhow::anyhow!("Failed to lock database"))?;
                db_guard.get_source_secret_id(source.id)
                    .context("Failed to get source secret_id")?
            };
            
            // If secret_id is missing, try to migrate from TokenStore
            let secret_id = if let Some(id) = secret_id {
                id
            } else {
                // Try to migrate from TokenStore
                let token_store_state: tauri::State<'_, Mutex<TokenStore>> = app.state();
                let token_opt = {
                    let token_store = token_store_state.lock()
                        .map_err(|_| anyhow::anyhow!("Failed to lock token store"))?;
                    token_store.get(&source.id).cloned()
                };
                
                if let Some(token) = token_opt {
                    // Found token in TokenStore, migrate on-the-fly
                    let app_clone = app.clone();
                    
                    tokio::task::spawn_blocking(move || {
                        migrate_source_token_to_secret_blocking(app_clone, source.id, token)
                    })
                    .await
                    .map_err(|e| anyhow::anyhow!("Task join error: {}", e))?
                    .context("Failed to migrate token to secret")?
                } else {
                    return Err(anyhow::anyhow!(
                        "GitHub source missing secret_id and no token found. Please re-authorize this source."
                    ));
                }
            };
            
            // Get token from SecretStore
            let token = {
                let secret_store: tauri::State<'_, Mutex<SecretStore>> = app.state();
                let store = secret_store.lock()
                    .map_err(|_| anyhow::anyhow!("Failed to lock secret store"))?;
                store.get(secret_id)
                    .map_err(|e| anyhow::anyhow!("Failed to get secret: {}", e))?
                    .ok_or_else(|| anyhow::anyhow!("Secret not found"))?
            };
            
            // Parse repositories and endpoints from config
            let repositories: Vec<String> = config.get("repositories")
                .and_then(|v| v.as_array())
                .ok_or_else(|| anyhow::anyhow!("Missing repositories in GitHub config"))?
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            
            let endpoints: Vec<String> = config.get("endpoints")
                .and_then(|v| v.as_array())
                .ok_or_else(|| anyhow::anyhow!("Missing endpoints in GitHub config"))?
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            
            let secret_id_clone = secret_id;
            let app_clone = app.clone();
            
            // First attempt with current token
            let result = tokio::task::spawn_blocking({
                let token_clone = token.clone();
                let repositories_clone = repositories.clone();
                let endpoints_clone = endpoints.clone();
                move || {
                    let ingester = GitHubIngester::new(
                        secret_id_clone,
                        token_clone,
                        repositories_clone,
                        endpoints_clone,
                    )?;
                    ingester.poll()
                }
            })
            .await
            .map_err(|e| anyhow::anyhow!("Task join error: {}", e))?;
            
            // Check if we got a 401 error
            match result {
                Ok(items) => Ok(items),
                Err(e) => {
                    let error_msg = e.to_string();
                    if error_msg.contains("401") {
                        // Attempt to refresh token
                        match crate::commands::refresh_github_token_internal(&app_clone, secret_id_clone).await {
                            Ok(new_token) => {
                                // Retry with new token
                                let repositories_retry = repositories.clone();
                                let endpoints_retry = endpoints.clone();
                                tokio::task::spawn_blocking(move || {
                                    let ingester = GitHubIngester::new(
                                        secret_id_clone,
                                        new_token,
                                        repositories_retry,
                                        endpoints_retry,
                                    )?;
                                    ingester.poll()
                                })
                                .await
                                .map_err(|e| anyhow::anyhow!("Task join error: {}", e))?
                            }
                            Err(_) => {
                                // Increment failure count
                                let db_state: tauri::State<'_, std::sync::Mutex<Database>> = app_clone.state();
                                let db_guard = db_state.lock()
                                    .map_err(|_| anyhow::anyhow!("Failed to lock database"))?;
                                let failure_count = db_guard.increment_refresh_failure_count(secret_id_clone)
                                    .map_err(|e| anyhow::anyhow!("Failed to increment failure count: {}", e))?;
                                drop(db_guard);
                                
                                // If 3 or more failures, expire and disable
                                if failure_count >= 3 {
                                    let _ = crate::commands::expire_secret_internal(&app_clone, secret_id_clone);
                                    return Err(anyhow::anyhow!("Token refresh failed 3 times. Please re-authorize in source settings."));
                                }
                                
                                // Return original error
                                Err(e)
                            }
                        }
                    } else {
                        // Not a 401 error, return original error
                        Err(e)
                    }
                }
            }
        }
        "github_notifications" => {
            // Get secret_id from source
            let secret_id = {
                let db_state: tauri::State<'_, Mutex<Database>> = app.state();
                let db_guard = db_state.lock()
                    .map_err(|_| anyhow::anyhow!("Failed to lock database"))?;
                let secret_id_result = db_guard.get_source_secret_id(source.id)
                    .context("Failed to get source secret_id")?;
                
                secret_id_result.ok_or_else(|| anyhow::anyhow!("GitHub notifications source requires a Personal Access Token (PAT). Please configure a secret."))?
            };
            
            // Get token from SecretStore
            let token = {
                let secret_store: tauri::State<'_, Mutex<SecretStore>> = app.state();
                let store = secret_store.lock()
                    .map_err(|_| anyhow::anyhow!("Failed to lock secret store"))?;
                store.get(secret_id)
                    .map_err(|e| anyhow::anyhow!("Failed to get secret: {}", e))?
                    .ok_or_else(|| anyhow::anyhow!("Secret not found in secure storage"))?
            };
            
            let secret_id_clone = secret_id;
            let app_clone = app.clone();
            
            // First attempt with current token
            let result = tokio::task::spawn_blocking({
                let token_clone = token.clone();
                move || {
                    let ingester = GitHubNotificationsIngester::new(token_clone)?;
                    ingester.poll()
                }
            })
            .await
            .map_err(|e| anyhow::anyhow!("Task join error: {}", e))?;
            
            // Check if we got a 401 error and try to refresh
            match result {
                Ok(items) => Ok(items),
                Err(e) => {
                    let error_msg = e.to_string();
                    if error_msg.contains("401") {
                        // Attempt to refresh token
                        match crate::commands::refresh_github_token_internal(&app_clone, secret_id_clone).await {
                            Ok(new_token) => {
                                // Retry with new token
                                tokio::task::spawn_blocking(move || {
                                    let ingester = GitHubNotificationsIngester::new(new_token)?;
                                    ingester.poll()
                                })
                                .await
                                .map_err(|e| anyhow::anyhow!("Task join error: {}", e))?
                            }
                            Err(_) => {
                                // Increment failure count
                                let db_state: tauri::State<'_, std::sync::Mutex<Database>> = app_clone.state();
                                let db_guard = db_state.lock()
                                    .map_err(|_| anyhow::anyhow!("Failed to lock database"))?;
                                let failure_count = db_guard.increment_refresh_failure_count(secret_id_clone)
                                    .map_err(|e| anyhow::anyhow!("Failed to increment failure count: {}", e))?;
                                drop(db_guard);
                                
                                // If 3 or more failures, expire and disable
                                if failure_count >= 3 {
                                    let _ = crate::commands::expire_secret_internal(&app_clone, secret_id_clone);
                                    return Err(anyhow::anyhow!("Token refresh failed 3 times. Please re-authorize in source settings."));
                                }
                                
                                // Return original error
                                Err(e)
                            }
                        }
                    } else {
                        // Not a 401 error, return original error
                        Err(e)
                    }
                }
            }
        }
        _ => return Err(anyhow::anyhow!("Unknown source type: {}", source.source_type)),
    }?;
    
    // Normalize and store items
    let db_state: tauri::State<'_, Mutex<Database>> = app.state();
    let db_guard = db_state.lock()
        .map_err(|_| anyhow::anyhow!("Failed to lock database"))?;
    
    let item_ids = normalize_and_dedupe(&db_guard, source.id, items)?;
    
    // Update sync time
    db_guard.update_source_sync_time(source.id)?;
    drop(db_guard);
    
    // Spawn background extraction task for items that need it
    let app_clone = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = process_background_extraction(&app_clone, item_ids).await {
            eprintln!("Background extraction error: {}", e);
        }
    });
    
    Ok(())
}

/// Proactively refreshes GitHub tokens on startup to prevent 401 errors
async fn refresh_github_tokens_on_startup(app: &tauri::AppHandle) {
    use std::sync::Mutex;
    use tauri::State;
    
    // Get all sources
    let sources = match get_sources_sync(app) {
        Ok(sources) => sources,
        Err(e) => {
            eprintln!("Failed to get sources for token refresh: {}", e);
            return;
        }
    };
    
    // Find all GitHub sources with secrets
    for source in sources {
        if source.source_type != "github" && source.source_type != "github_notifications" {
            continue;
        }
        
        let secret_id = {
            let db_state: State<'_, Mutex<Database>> = app.state();
            let db_guard = match db_state.lock() {
                Ok(db) => db,
                Err(_) => continue,
            };
            match db_guard.get_source_secret_id(source.id) {
                Ok(Some(id)) => id,
                Ok(None) => continue,
                Err(_) => continue,
            }
        };
        
        // Check if secret has refresh token capability
        let has_refresh_token = {
            let db_state: State<'_, Mutex<Database>> = app.state();
            let db_guard = match db_state.lock() {
                Ok(db) => db,
                Err(_) => continue,
            };
            match db_guard.get_secret(secret_id) {
                Ok(secret) => secret.refresh_token_id.is_some(),
                Err(_) => false,
            }
        };
        
        if has_refresh_token {
            // Try to refresh token proactively
            match crate::commands::refresh_github_token_internal(app, secret_id).await {
                Ok(_) => {
                    eprintln!("Successfully refreshed GitHub token for source: {}", source.name);
                }
                Err(e) => {
                    eprintln!("Failed to refresh GitHub token for source {}: {} (will retry on sync)", source.name, e);
                }
            }
        }
    }
}

/// Background task that extracts full content for items marked as partial
async fn process_background_extraction(
    app: &tauri::AppHandle,
    item_ids: Vec<i64>,
) -> anyhow::Result<()> {
    use std::sync::Mutex;
    use tauri::State;
    use crate::ingestion::extraction::extract_full_text;
    
    // Check user preference for extraction
    let extraction_enabled = {
        let db_state: State<'_, Mutex<Database>> = app.state();
        let db_guard = db_state.lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock database"))?;
        let enabled = db_guard.get_user_preference("extraction_enabled")
            .unwrap_or(None)
            .unwrap_or_else(|| "true".to_string());
        drop(db_guard);
        enabled == "true"
    };
    
    if !extraction_enabled {
        return Ok(()); // Extraction disabled by user
    }
    
    // Get article view mode preference
    let view_mode = {
        let db_state: State<'_, Mutex<Database>> = app.state();
        let db_guard = db_state.lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock database"))?;
        let mode = db_guard.get_user_preference("article_view_mode")
            .unwrap_or(None)
            .unwrap_or_else(|| "auto".to_string());
        drop(db_guard);
        mode
    };
    
    // Only extract if mode is "auto" or "always_fetch"
    if view_mode != "auto" && view_mode != "always_fetch" {
        return Ok(());
    }
    
    // Process each item (with small delay to avoid overwhelming servers)
    for item_id in item_ids {
        // Small delay between extractions
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        let db_state: State<'_, Mutex<Database>> = app.state();
        
        // Check if item needs extraction
        let (needs_extraction, url, completeness) = {
            let db_guard = db_state.lock()
                .map_err(|_| anyhow::anyhow!("Failed to lock database"))?;
            
            // Get item details
            let item = match db_guard.get_item(item_id) {
                Ok(item) => item,
                Err(_) => {
                    continue; // Item not found, skip
                }
            };
            
            let status = item.content_status.as_deref().unwrap_or("");
            let completeness = item.content_completeness.as_deref().unwrap_or("");
            let url = item.url;
            
            // Only extract if:
            // - completeness is "partial"
            // - status is NULL or "feed_only" (not already extracted/fetching/failed)
            // - URL exists
            let needs = completeness == "partial" 
                && (status.is_empty() || status == "feed_only")
                && !url.is_empty();
            
            drop(db_guard);
            (needs, url, completeness.to_string())
        };
        
        if !needs_extraction {
            continue;
        }
        
        // Set status to "fetching"
        {
            let db_guard = db_state.lock()
                .map_err(|_| anyhow::anyhow!("Failed to lock database"))?;
            db_guard.update_item_content_status(
                item_id,
                "fetching",
                None,
                Some(&completeness),
                None,
            )?;
            drop(db_guard);
        }
        
        // Extract content in blocking task
        // Note: url comes from item.url which is the RSS <link> or Atom <link rel="alternate"> tag
        // This is the canonical article URL, not parsed from CDATA or content_html
        let extraction_result = tokio::task::spawn_blocking({
            let url_clone = url.clone();
            move || extract_full_text(&url_clone)
        })
        .await
        .map_err(|e| anyhow::anyhow!("Task join error: {}", e))?;
        
        // Update database with result
        let db_guard = db_state.lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock database"))?;
        
        match extraction_result {
            Ok(result) => {
                // Success - update with extracted content
                db_guard.update_item_content_status(
                    item_id,
                    "extracted",
                    Some(&result.content),
                    Some(&completeness),
                    None,
                )?;
            }
            Err(e) => {
                // Failure - update with error reason
                db_guard.update_item_content_status(
                    item_id,
                    "failed",
                    None,
                    Some(&completeness),
                    Some(&format!("{}", e)),
                )?;
            }
        }
    }
    
    Ok(())
}
