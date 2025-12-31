use crate::storage::{Database, models::{Source, Item}};
use crate::config::TokenStore;
use crate::ingestion::{RssIngester, GitHubIngester, traits::IngestSource};
use crate::normalization::normalize_and_dedupe;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};

#[derive(Debug, Serialize, Deserialize)]
pub struct SourceInput {
    pub source_type: String,
    pub name: String,
    pub config_json: serde_json::Value,
    pub token: Option<String>, // For GitHub sources
    pub group: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateSourceInput {
    pub name: Option<String>,
    pub config_json: Option<serde_json::Value>,
    pub enabled: Option<bool>,
    pub token: Option<String>,
    pub group: Option<Option<String>>, // Option<Option> to allow setting to None
}

#[tauri::command]
pub async fn get_items(
    db: State<'_, Mutex<Database>>,
    state_filter: Option<String>,
    group_filter: Option<String>,
) -> Result<Vec<Item>, String> {
    let db = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    db.get_items(state_filter.as_deref(), group_filter.as_deref())
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
pub async fn get_sources(
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<Source>, String> {
    let db = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    db.get_all_sources()
        .map_err(|e| format!("Failed to get sources: {}", e))
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
        source.group.as_deref(),
    ).map_err(|e| format!("Failed to create source: {}", e))?;
    
    // Store token if provided (for GitHub sources)
    if let Some(token) = source.token {
        let token_store: State<'_, Mutex<TokenStore>> = app.state();
        let mut store = token_store.lock().map_err(|e| format!("Token store lock error: {}", e))?;
        store.insert(source_id, token);
    }
    
    drop(db_guard);
    
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
    
    db_guard.update_source(
        id,
        update.name.as_deref(),
        config_json_str.as_deref(),
        update.enabled,
        update.group.as_ref().map(|g| g.as_deref()),
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
pub async fn sync_source(
    app: AppHandle,
    db: State<'_, Mutex<Database>>,
    id: i64,
) -> Result<(), String> {
    // Get source info and drop guard before await
    let (source_type, config_json_str) = {
        let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
        let source = db_guard.get_source(id)
            .map_err(|e| format!("Failed to get source: {}", e))?;
        (source.source_type, source.config_json)
    };
    
    let config: serde_json::Value = serde_json::from_str(&config_json_str)
        .map_err(|e| format!("Failed to parse source config: {}", e))?;
    
    // Create appropriate ingester and poll (using spawn_blocking for blocking HTTP calls)
    let items = match source_type.as_str() {
        "rss" => {
            let url = config.get("url")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing RSS URL in config".to_string())?;
            
            let url = url.to_string();
            tokio::task::spawn_blocking(move || {
                let ingester = RssIngester::new(url)
                    .map_err(|e| format!("Failed to create RSS ingester: {}", e))?;
                ingester.poll()
                    .map_err(|e| format!("Failed to poll RSS feed: {}", e))
            })
            .await
            .map_err(|e| format!("Task join error: {}", e))?
        }
        "github" => {
            // Get token before spawn_blocking to avoid holding guard across await
            let token = {
                let token_store: State<'_, Mutex<TokenStore>> = app.state();
                let store = token_store.lock().map_err(|e| format!("Token store lock error: {}", e))?;
                store.get(&id)
                    .ok_or_else(|| "GitHub token not found".to_string())?
                    .clone()
            };
            
            let owner = config.get("owner")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing owner in GitHub config".to_string())?
                .to_string();
            
            let repo = config.get("repo")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing repo in GitHub config".to_string())?
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
                ).map_err(|e| format!("Failed to create GitHub ingester: {}", e))?;
                
                ingester.poll()
                    .map_err(|e| format!("Failed to poll GitHub: {}", e))
            })
            .await
            .map_err(|e| format!("Task join error: {}", e))?
        }
        _ => return Err(format!("Unknown source type: {}", source_type)),
    };
    
    // Normalize and store items
    let items = items?; // Unwrap the Result from spawn_blocking
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    normalize_and_dedupe(&db_guard, id, items)
        .map_err(|e| format!("Failed to normalize items: {}", e))?;
    
    // Update sync time
    db_guard.update_source_sync_time(id)
        .map_err(|e| format!("Failed to update sync time: {}", e))?;
    
    Ok(())
}


