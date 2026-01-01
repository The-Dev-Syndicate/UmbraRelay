use crate::storage::{Database, models::{Item, CustomView, Group}};
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
    pub group_ids: Option<Vec<i64>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateSourceInput {
    pub name: Option<String>,
    pub config_json: Option<serde_json::Value>,
    pub enabled: Option<bool>,
    pub token: Option<String>,
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
    
    // Convert Option<Vec<i64>> to Option<Option<&[i64]>>
    // None = don't update groups, Some(vec) = set groups (empty vec clears)
    let group_ids_ref: Option<Option<&[i64]>> = update.group_ids.as_ref().map(|v| Some(v.as_slice()));
    
    db_guard.update_source(
        id,
        update.name.as_deref(),
        config_json_str.as_deref(),
        update.enabled,
        group_ids_ref,
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


