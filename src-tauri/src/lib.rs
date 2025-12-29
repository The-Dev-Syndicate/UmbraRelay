mod storage;
mod config;
mod ingestion;
mod normalization;
mod commands;

use storage::Database;
use config::{TokenStore, Config};
use std::sync::Mutex;
use std::collections::HashMap;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            // Initialize database
            let app_data_dir = app.path().app_data_dir()
                .expect("Failed to get app data directory");
            
            std::fs::create_dir_all(&app_data_dir)
                .expect("Failed to create app data directory");
            
            let db_path = app_data_dir.join("umbrarelay.db");
            let db = Database::new(
                db_path.to_str().expect("Invalid database path")
            ).expect("Failed to initialize database");
            
            app.manage(Mutex::new(db));
            
            // Initialize token store
            let token_store: TokenStore = HashMap::new();
            app.manage(Mutex::new(token_store));
            
            // Load config and sync sources to database
            if let Ok(config) = config::load_config() {
                eprintln!("[UmbraRelay] Loaded config, syncing sources to database...");
                sync_config_to_database(app.handle(), &config);
            } else {
                eprintln!("[UmbraRelay] Failed to load config or config file doesn't exist");
            }
            
            // Start background polling service
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                // Create a multi-threaded runtime and keep it alive
                let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
                // Spawn the background service on the runtime
                rt.spawn(async move {
                    background_polling_service(app_handle).await;
                });
                // Keep the runtime alive for the thread's lifetime
                rt.block_on(std::future::pending::<()>());
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_items,
            commands::get_item,
            commands::update_item_state,
            commands::get_sources,
            commands::add_source,
            commands::update_source,
            commands::remove_source,
            commands::sync_source,
            commands::get_config,
            commands::update_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Sync config file sources to database on startup
fn sync_config_to_database(app: &tauri::AppHandle, config: &Config) {
    use serde_json::json;
    
    let db_state: tauri::State<'_, Mutex<Database>> = app.state();
    let db = match db_state.lock() {
        Ok(db) => db,
        Err(e) => {
            eprintln!("[UmbraRelay] Failed to lock database for config sync: {}", e);
            return;
        }
    };
    
    // Get existing sources to check for duplicates and track what's in config
    let existing_sources = match db.get_all_sources() {
        Ok(sources) => sources,
        Err(e) => {
            eprintln!("[UmbraRelay] Failed to get existing sources: {}", e);
            return;
        }
    };
    
    // Build set of sources that should exist in config
    let mut config_source_keys = std::collections::HashSet::new();
    
    // Helper to check if a source already exists (by name and type)
    let source_exists = |name: &str, source_type: &str| -> bool {
        existing_sources.iter().any(|s| s.name == name && s.source_type == source_type)
    };
    
    // Helper to get source key for matching
    let get_source_key = |name: &str, source_type: &str| -> String {
        format!("{}:{}", source_type, name)
    };
    
    // Sync RSS sources
    for rss_source in &config.rss {
        let key = get_source_key(&rss_source.name, "rss");
        config_source_keys.insert(key.clone());
        
        if source_exists(&rss_source.name, "rss") {
            // Source exists, ensure it's enabled (in case it was previously disabled)
            if let Some(existing) = existing_sources.iter().find(|s| s.name == rss_source.name && s.source_type == "rss") {
                if !existing.enabled {
                    if let Err(e) = db.update_source(existing.id, None, None, Some(true)) {
                        eprintln!("[UmbraRelay] Failed to re-enable RSS source '{}': {}", rss_source.name, e);
                    } else {
                        eprintln!("[UmbraRelay] Re-enabled RSS source '{}'", rss_source.name);
                    }
                }
            }
            continue;
        }
        
        let config_json = json!({
            "url": rss_source.url,
            "poll_interval": rss_source.poll_interval,
            "_from_config": true  // Marker to track config-sourced items
        });
        
        match db.create_source("rss", &rss_source.name, &config_json.to_string()) {
            Ok(id) => {
                eprintln!("[UmbraRelay] Created RSS source '{}' (id: {})", rss_source.name, id);
            }
            Err(e) => {
                eprintln!("[UmbraRelay] Failed to create RSS source '{}': {}", rss_source.name, e);
            }
        }
    }
    
    // Sync GitHub repos
    for repo in &config.github.repos {
        let name = format!("{}/{}", repo.owner, repo.repo);
        let key = get_source_key(&name, "github");
        config_source_keys.insert(key.clone());
        
        if source_exists(&name, "github") {
            // Source exists, ensure it's enabled (in case it was previously disabled)
            if let Some(existing) = existing_sources.iter().find(|s| s.name == name && s.source_type == "github") {
                if !existing.enabled {
                    if let Err(e) = db.update_source(existing.id, None, None, Some(true)) {
                        eprintln!("[UmbraRelay] Failed to re-enable GitHub source '{}': {}", name, e);
                    } else {
                        eprintln!("[UmbraRelay] Re-enabled GitHub source '{}'", name);
                    }
                }
            }
            continue;
        }
        
        let config_json = json!({
            "owner": repo.owner,
            "repo": repo.repo,
            "assigned_only": repo.assigned_only,
            "_from_config": true  // Marker to track config-sourced items
        });
        
        match db.create_source("github", &name, &config_json.to_string()) {
            Ok(id) => {
                eprintln!("[UmbraRelay] Created GitHub source '{}' (id: {})", name, id);
                eprintln!("[UmbraRelay] Note: GitHub token must be added via UI for source {}", id);
            }
            Err(e) => {
                eprintln!("[UmbraRelay] Failed to create GitHub source '{}': {}", name, e);
            }
        }
    }
    
    // Disable sources that were in config but are no longer present
    // Only disable sources that have the _from_config marker
    for existing_source in &existing_sources {
        let key = get_source_key(&existing_source.name, &existing_source.source_type);
        
        // Check if this source was from config (has _from_config marker)
        let from_config = if let Ok(config_json) = serde_json::from_str::<serde_json::Value>(&existing_source.config_json) {
            config_json.get("_from_config").and_then(|v| v.as_bool()).unwrap_or(false)
        } else {
            false
        };
        
        // If source was from config but is no longer in config, disable it
        if from_config && !config_source_keys.contains(&key) {
            if existing_source.enabled {
                if let Err(e) = db.update_source(existing_source.id, None, None, Some(false)) {
                    eprintln!("[UmbraRelay] Failed to disable removed config source '{}': {}", existing_source.name, e);
                } else {
                    eprintln!("[UmbraRelay] Disabled source '{}' (removed from config)", existing_source.name);
                }
            }
        }
    }
    
    eprintln!("[UmbraRelay] Config sync complete");
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

async fn background_polling_service(app: tauri::AppHandle) {
    use config::parse_duration;
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
                "rss" => {
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
            
            let interval_seconds = parse_duration(&poll_interval)
                .unwrap_or(600); // Default 10 minutes
            
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

async fn sync_source_internal(app: &tauri::AppHandle, source: storage::models::Source) -> anyhow::Result<()> {
    use crate::ingestion::{RssIngester, GitHubIngester, traits::IngestSource};
    use crate::normalization::normalize_and_dedupe;
    use anyhow::Context;
    
    let config: serde_json::Value = serde_json::from_str(&source.config_json)
        .context("Failed to parse source config")?;
    
    // Create appropriate ingester and poll
    let items = match source.source_type.as_str() {
        "rss" => {
            let url = config.get("url")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing RSS URL in config"))?;
            
            let ingester = RssIngester::new(url.to_string())?;
            ingester.poll()?
        }
        "github" => {
            let token_store: tauri::State<'_, Mutex<TokenStore>> = app.state();
            let store = token_store.lock()
                .map_err(|_| anyhow::anyhow!("Failed to lock token store"))?;
            let token = store.get(&source.id)
                .ok_or_else(|| anyhow::anyhow!("GitHub token not found"))?
                .clone();
            drop(store);
            
            let owner = config.get("owner")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing owner in GitHub config"))?;
            
            let repo = config.get("repo")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing repo in GitHub config"))?;
            
            let assigned_only = config.get("assigned_only")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            
            let ingester = GitHubIngester::new(
                owner.to_string(),
                repo.to_string(),
                token,
                assigned_only,
            )?;
            
            ingester.poll()?
        }
        _ => return Err(anyhow::anyhow!("Unknown source type: {}", source.source_type)),
    };
    
    // Normalize and store items
    let db_state: tauri::State<'_, Mutex<Database>> = app.state();
    let db_guard = db_state.lock()
        .map_err(|_| anyhow::anyhow!("Failed to lock database"))?;
    
    normalize_and_dedupe(&db_guard, source.id, items)?;
    
    // Update sync time
    db_guard.update_source_sync_time(source.id)?;
    
    Ok(())
}
