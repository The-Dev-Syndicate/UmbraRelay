mod storage;
mod config;
mod ingestion;
mod normalization;
mod commands;

use storage::Database;
use config::TokenStore;
use std::sync::Mutex;
use std::collections::HashMap;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
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
            
            // Start background polling service using Tauri's async runtime
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                background_polling_service(app_handle).await;
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
        ])
        .run(tauri::generate_context!())
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

// Helper function to parse duration strings like "5m", "10m", "1h"
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

async fn sync_source_internal(app: &tauri::AppHandle, source: storage::models::Source) -> anyhow::Result<()> {
    use crate::ingestion::{RssIngester, GitHubIngester, traits::IngestSource};
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
        "github" => {
            // Extract token before async operations to avoid holding MutexGuard across await
            let token = {
                let token_store: tauri::State<'_, Mutex<TokenStore>> = app.state();
                let store = token_store.lock()
                    .map_err(|_| anyhow::anyhow!("Failed to lock token store"))?;
                store.get(&source.id)
                    .ok_or_else(|| anyhow::anyhow!("GitHub token not found"))?
                    .clone()
            }; // Guard is dropped here
            
            let owner = config.get("owner")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing owner in GitHub config"))?
                .to_string();
            
            let repo = config.get("repo")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing repo in GitHub config"))?
                .to_string();
            
            let assigned_only = config.get("assigned_only")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            
            tokio::task::spawn_blocking(move || {
                let ingester = GitHubIngester::new(
                    owner,
                    repo,
                    token,
                    assigned_only,
                )?;
                ingester.poll()
            })
            .await
            .map_err(|e| anyhow::anyhow!("Task join error: {}", e))?
        }
        _ => return Err(anyhow::anyhow!("Unknown source type: {}", source.source_type)),
    }?;
    
    // Normalize and store items
    let db_state: tauri::State<'_, Mutex<Database>> = app.state();
    let db_guard = db_state.lock()
        .map_err(|_| anyhow::anyhow!("Failed to lock database"))?;
    
    normalize_and_dedupe(&db_guard, source.id, items)?;
    
    // Update sync time
    db_guard.update_source_sync_time(source.id)?;
    
    Ok(())
}
