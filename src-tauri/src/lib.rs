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
            
            // Start background polling service
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
                rt.block_on(async {
                    background_polling_service(app_handle).await;
                });
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
