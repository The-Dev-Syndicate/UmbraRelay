use rusqlite::{Connection, Result, params};
use rusqlite_migration::{Migrations, M};
use std::sync::{Arc, Mutex};
use chrono::Utc;
use super::models::{Source, Item, Secret};

pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn new(db_path: &str) -> Result<Self> {
        let mut conn = Connection::open(db_path)?;
        
        // Run migrations
        let migrations = Migrations::new(vec![
            M::up(
                r#"
                CREATE TABLE IF NOT EXISTS sources (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    type TEXT NOT NULL,
                    name TEXT NOT NULL,
                    config_json TEXT NOT NULL,
                    enabled INTEGER NOT NULL DEFAULT 1,
                    last_synced_at INTEGER,
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL
                );
                "#
            ),
            M::up(
                r#"
                CREATE TABLE IF NOT EXISTS items (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    source_id INTEGER NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
                    external_id TEXT NOT NULL,
                    title TEXT NOT NULL,
                    summary TEXT,
                    url TEXT NOT NULL,
                    item_type TEXT NOT NULL,
                    state TEXT NOT NULL DEFAULT 'unread',
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL,
                    UNIQUE(source_id, external_id)
                );
                "#
            ),
            M::up(
                r#"
                CREATE TABLE IF NOT EXISTS events (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    item_id INTEGER NOT NULL REFERENCES items(id) ON DELETE CASCADE,
                    event_type TEXT NOT NULL,
                    payload_json TEXT,
                    occurred_at INTEGER NOT NULL
                );
                "#
            ),
            M::up(
                r#"
                CREATE INDEX IF NOT EXISTS idx_items_source_id ON items(source_id);
                CREATE INDEX IF NOT EXISTS idx_items_state ON items(state);
                CREATE INDEX IF NOT EXISTS idx_items_created_at ON items(created_at);
                CREATE INDEX IF NOT EXISTS idx_events_item_id ON events(item_id);
                "#
            ),
            M::up(
                r#"
                ALTER TABLE sources ADD COLUMN "group" TEXT;
                CREATE INDEX IF NOT EXISTS idx_sources_group ON sources("group");
                "#
            ),
            M::up(
                r#"
                ALTER TABLE items ADD COLUMN image_url TEXT;
                ALTER TABLE items ADD COLUMN content_html TEXT;
                "#
            ),
            M::up(
                r#"
                CREATE TABLE IF NOT EXISTS custom_views (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL,
                    source_ids TEXT,
                    group_names TEXT,
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_custom_views_name ON custom_views(name);
                "#
            ),
            M::up(
                r#"
                ALTER TABLE items ADD COLUMN author TEXT;
                ALTER TABLE items ADD COLUMN category TEXT;
                ALTER TABLE items ADD COLUMN comments TEXT;
                "#
            ),
            M::up(
                r#"
                CREATE TABLE IF NOT EXISTS groups (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL UNIQUE,
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_groups_name ON groups(name);
                "#
            ),
            M::up(
                r#"
                CREATE TABLE IF NOT EXISTS source_groups (
                    source_id INTEGER NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
                    group_id INTEGER NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
                    PRIMARY KEY (source_id, group_id)
                );
                CREATE INDEX IF NOT EXISTS idx_source_groups_source_id ON source_groups(source_id);
                CREATE INDEX IF NOT EXISTS idx_source_groups_group_id ON source_groups(group_id);
                "#
            ),
            M::up(
                r#"
                -- Drop the legacy group column from sources table
                -- SQLite doesn't support DROP COLUMN directly, so we use a workaround
                CREATE TABLE sources_new (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    type TEXT NOT NULL,
                    name TEXT NOT NULL,
                    config_json TEXT NOT NULL,
                    enabled INTEGER NOT NULL DEFAULT 1,
                    last_synced_at INTEGER,
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL
                );
                INSERT INTO sources_new (id, type, name, config_json, enabled, last_synced_at, created_at, updated_at)
                SELECT id, type, name, config_json, enabled, last_synced_at, created_at, updated_at FROM sources;
                DROP TABLE sources;
                ALTER TABLE sources_new RENAME TO sources;
                DROP INDEX IF EXISTS idx_sources_group;
                "#
            ),
            M::up(
                r#"
                CREATE TABLE IF NOT EXISTS user_preferences (
                    key TEXT PRIMARY KEY,
                    value TEXT NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_user_preferences_key ON user_preferences(key);
                "#
            ),
            M::up(
                r#"
                CREATE TABLE IF NOT EXISTS secrets (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL UNIQUE,
                    ttl_type TEXT NOT NULL DEFAULT 'forever',
                    ttl_value TEXT,
                    expires_at INTEGER,
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_secrets_name ON secrets(name);
                CREATE INDEX IF NOT EXISTS idx_secrets_expires_at ON secrets(expires_at);
                "#
            ),
            M::up(
                r#"
                ALTER TABLE sources ADD COLUMN secret_id INTEGER REFERENCES secrets(id);
                CREATE INDEX IF NOT EXISTS idx_sources_secret_id ON sources(secret_id);
                "#
            ),
            M::up(
                r#"
                ALTER TABLE items ADD COLUMN thread_id TEXT;
                CREATE INDEX IF NOT EXISTS idx_items_thread_id ON items(thread_id);
                "#
            ),
            M::up(
                r#"
                ALTER TABLE secrets ADD COLUMN refresh_token_id INTEGER;
                ALTER TABLE secrets ADD COLUMN refresh_failure_count INTEGER NOT NULL DEFAULT 0;
                "#
            ),
        ]);

        migrations.to_latest(&mut conn)
            .map_err(|e| {
                // Convert migration error to rusqlite error
                rusqlite::Error::SqliteFailure(
                    rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_MISUSE),
                    Some(format!("Migration error: {}", e))
                )
            })?;

        // Migrate existing groups from comma-separated strings to groups table
        let _ = Self::migrate_existing_groups(&conn);

        Ok(Database {
            conn: Arc::new(Mutex::new(conn)),
        })
    }


    // Source CRUD operations
    pub fn create_source(&self, source_type: &str, name: &str, config_json: &str, group_ids: Option<&[i64]>, secret_id: Option<i64>) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().timestamp();
        conn.execute(
            r#"INSERT INTO sources (type, name, config_json, enabled, secret_id, created_at, updated_at) VALUES (?1, ?2, ?3, 1, ?4, ?5, ?5)"#,
            params![source_type, name, config_json, secret_id, now],
        )?;
        let source_id = conn.last_insert_rowid();
        
        // Set source-group relationships
        if let Some(group_ids) = group_ids {
            for group_id in group_ids {
                let _ = conn.execute(
                    "INSERT INTO source_groups (source_id, group_id) VALUES (?1, ?2)",
                    params![source_id, group_id],
                );
            }
        }
        
        Ok(source_id)
    }

    pub fn get_all_sources(&self) -> Result<Vec<Source>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"SELECT id, type, name, config_json, enabled, last_synced_at, created_at, updated_at FROM sources ORDER BY created_at DESC"#
        )?;
        let sources = stmt.query_map([], |row| Source::from_row(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(sources)
    }

    pub fn get_source(&self, id: i64) -> Result<Source> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"SELECT id, type, name, config_json, enabled, last_synced_at, created_at, updated_at FROM sources WHERE id = ?1"#
        )?;
        stmt.query_row(params![id], |row| Source::from_row(row))
    }

    pub fn get_source_secret_id(&self, id: i64) -> Result<Option<i64>> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT secret_id FROM sources WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
    }

    pub fn update_source(&self, id: i64, name: Option<&str>, config_json: Option<&str>, enabled: Option<bool>, group_ids: Option<Option<&[i64]>>, secret_id: Option<Option<&i64>>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().timestamp();
        
        if let Some(name) = name {
            conn.execute(
                "UPDATE sources SET name = ?1, updated_at = ?2 WHERE id = ?3",
                params![name, now, id],
            )?;
        }
        
        if let Some(config_json) = config_json {
            conn.execute(
                "UPDATE sources SET config_json = ?1, updated_at = ?2 WHERE id = ?3",
                params![config_json, now, id],
            )?;
        }
        
        if let Some(enabled) = enabled {
            conn.execute(
                "UPDATE sources SET enabled = ?1, updated_at = ?2 WHERE id = ?3",
                params![if enabled { 1 } else { 0 }, now, id],
            )?;
        }
        
        if let Some(secret_id) = secret_id {
            conn.execute(
                "UPDATE sources SET secret_id = ?1, updated_at = ?2 WHERE id = ?3",
                params![secret_id, now, id],
            )?;
        }
        
        if let Some(group_ids) = group_ids {
            // Update source-group relationships - pass the already-locked connection
            Self::set_source_groups_internal(&conn, id, group_ids.unwrap_or(&[]))?;
        }
        
        Ok(())
    }

    pub fn delete_source(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM sources WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn update_source_sync_time(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().timestamp();
        conn.execute(
            "UPDATE sources SET last_synced_at = ?1 WHERE id = ?2",
            params![now, id],
        )?;
        Ok(())
    }

    // Item CRUD operations
    pub fn upsert_item(
        &self,
        source_id: i64,
        external_id: &str,
        title: &str,
        summary: Option<&str>,
        url: &str,
        item_type: &str,
        image_url: Option<&str>,
        content_html: Option<&str>,
        author: Option<&str>,
        category: Option<&str>,
        comments: Option<&str>,
        thread_id: Option<&str>,
    ) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().timestamp();
        
        // Try to find existing item
        let existing: Result<i64, _> = conn.query_row(
            "SELECT id FROM items WHERE source_id = ?1 AND external_id = ?2",
            params![source_id, external_id],
            |row| row.get(0),
        );

        match existing {
            Ok(id) => {
                // Update existing item
                conn.execute(
                    "UPDATE items SET title = ?1, summary = ?2, url = ?3, item_type = ?4, image_url = ?5, content_html = ?6, author = ?7, category = ?8, comments = ?9, thread_id = ?10, updated_at = ?11 WHERE id = ?12",
                    params![title, summary, url, item_type, image_url, content_html, author, category, comments, thread_id, now, id],
                )?;
                Ok(id)
            }
            Err(_) => {
                // Insert new item
                conn.execute(
                    "INSERT INTO items (source_id, external_id, title, summary, url, item_type, image_url, content_html, author, category, comments, thread_id, state, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, 'unread', ?13, ?13)",
                    params![source_id, external_id, title, summary, url, item_type, image_url, content_html, author, category, comments, thread_id, now],
                )?;
                Ok(conn.last_insert_rowid())
            }
        }
    }

    pub fn get_items(&self, state_filter: Option<&str>, group_filter: Option<&str>, source_ids: Option<&[i64]>, group_names: Option<&[String]>) -> Result<Vec<Item>> {
        let conn = self.conn.lock().unwrap();
        
        // JOIN with sources and groups tables to get group names
        // Use LEFT JOIN for groups since a source might not have any groups
        let mut query = "SELECT i.id, i.source_id, i.external_id, i.title, i.summary, i.url, i.item_type, i.state, i.created_at, i.updated_at, i.image_url, i.content_html, i.author, i.category, i.comments, s.name as source_name, GROUP_CONCAT(g.name, ', ') as source_group FROM items i INNER JOIN sources s ON i.source_id = s.id LEFT JOIN source_groups sg ON s.id = sg.source_id LEFT JOIN groups g ON sg.group_id = g.id".to_string();
        let mut conditions: Vec<String> = Vec::new();
        
        // Build params in order - need to store owned values for string parameters
        let mut string_params: Vec<String> = Vec::new();
        let mut int_params: Vec<i64> = Vec::new();
        
        // State filter
        if let Some(state) = state_filter {
            conditions.push("i.state = ?".to_string());
            string_params.push(state.to_string());
        }
        
        // Source IDs filter (from custom views)
        if let Some(ids) = source_ids {
            if !ids.is_empty() {
                let placeholders: Vec<String> = (1..=ids.len()).map(|_| "?".to_string()).collect();
                conditions.push(format!("i.source_id IN ({})", placeholders.join(", ")));
                int_params.extend_from_slice(ids);
            }
        }
        
        // Group names filter (from custom views) - takes precedence over legacy group_filter
        if let Some(groups) = group_names {
            if !groups.is_empty() {
                // Filter by group names using EXISTS subquery
                let placeholders: Vec<String> = (1..=groups.len()).map(|_| "?".to_string()).collect();
                conditions.push(format!(
                    "EXISTS (SELECT 1 FROM source_groups sg2 INNER JOIN groups g2 ON sg2.group_id = g2.id WHERE sg2.source_id = s.id AND g2.name IN ({}))",
                    placeholders.join(", ")
                ));
                for group in groups {
                    string_params.push(group.clone());
                }
            }
        } else if let Some(group) = group_filter {
            // Legacy single group filter support - now uses groups table
            conditions.push("EXISTS (SELECT 1 FROM source_groups sg2 INNER JOIN groups g2 ON sg2.group_id = g2.id WHERE sg2.source_id = s.id AND g2.name = ?)".to_string());
            string_params.push(group.to_string());
        }
        
        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }
        
        // Group by item fields to handle GROUP_CONCAT properly
        query.push_str(" GROUP BY i.id, i.source_id, i.external_id, i.title, i.summary, i.url, i.item_type, i.state, i.created_at, i.updated_at, i.image_url, i.content_html, i.author, i.category, i.comments, s.name");
        query.push_str(" ORDER BY i.created_at DESC");
        
        // Build params vector with proper references
        let mut stmt = conn.prepare(&query)?;
        let items: Vec<Item> = if !string_params.is_empty() || !int_params.is_empty() {
            let mut params: Vec<&dyn rusqlite::ToSql> = Vec::new();
            let mut str_idx = 0;
            let mut int_idx = 0;
            
            // Add state if present
            if state_filter.is_some() {
                params.push(&string_params[str_idx]);
                str_idx += 1;
            }
            
            // Add source IDs if present
            if source_ids.is_some() && !int_params.is_empty() {
                for _ in 0..int_params.len() {
                    params.push(&int_params[int_idx]);
                    int_idx += 1;
                }
            }
            
            // Add group parameters if present
            if let Some(groups) = group_names {
                if !groups.is_empty() {
                    for _group in groups {
                        params.push(&string_params[str_idx]);
                        str_idx += 1;
                    }
                }
            } else if group_filter.is_some() {
                params.push(&string_params[str_idx]);
            }
            
            stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| Item::from_row_with_source(row))?
                .collect::<Result<Vec<_>, _>>()?
        } else {
            stmt.query_map([], |row| Item::from_row_with_source(row))?
                .collect::<Result<Vec<_>, _>>()?
        };
        Ok(items)
    }

    pub fn get_item(&self, id: i64) -> Result<Item> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, source_id, external_id, title, summary, url, item_type, state, created_at, updated_at, image_url, content_html, author, category, comments FROM items WHERE id = ?1"
        )?;
        stmt.query_row(params![id], |row| Item::from_row(row))
    }

    // Cleanup old items (older than specified days, but preserve archived items)
    pub fn cleanup_old_items(&self, days: i64) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let cutoff_timestamp = Utc::now().timestamp() - (days * 24 * 60 * 60);
        
        // Delete items older than cutoff, but preserve archived items
        let deleted = conn.execute(
            "DELETE FROM items WHERE state != 'archived' AND created_at < ?1",
            params![cutoff_timestamp],
        )?;
        
        Ok(deleted)
    }

    pub fn delete_items_by_source_name(&self, source_name: &str) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        
        // First find the source by name
        let source_id: Result<i64, _> = conn.query_row(
            "SELECT id FROM sources WHERE name = ?1",
            params![source_name],
            |row| row.get(0),
        );
        
        match source_id {
            Ok(id) => {
                // Delete all items for this source
                let deleted = conn.execute(
                    "DELETE FROM items WHERE source_id = ?1",
                    params![id],
                )?;
                Ok(deleted)
            }
            Err(_) => {
                // Source not found
                Ok(0)
            }
        }
    }

    pub fn update_item_state(&self, id: i64, state: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().timestamp();
        conn.execute(
            "UPDATE items SET state = ?1, updated_at = ?2 WHERE id = ?3",
            params![state, now, id],
        )?;
        Ok(())
    }


    // Update created_at timestamp to make items appear in "leaving soon" (for testing)
    pub fn make_items_leaving_soon(&self, count: i64) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().timestamp();
        
        // Calculate timestamps for items that will be leaving soon (23-29 days old)
        // We'll create items with different ages: 23, 24, 25, 26, 27, 28, 29 days old
        let days_old = vec![23, 24, 25, 26, 27, 28, 29];
        
        // Get IDs of first N non-archived items
        let mut stmt = conn.prepare(
            "SELECT id FROM items WHERE state != 'archived' ORDER BY id LIMIT ?1"
        )?;
        let item_ids: Vec<i64> = stmt.query_map(params![count], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;
        
        // Update each item with a different age
        let mut updated = 0;
        for (idx, item_id) in item_ids.iter().enumerate() {
            if let Some(&days) = days_old.get(idx) {
                let old_timestamp = now - (days * 24 * 60 * 60);
                let result = conn.execute(
                    "UPDATE items SET created_at = ?1, updated_at = ?2 WHERE id = ?3",
                    params![old_timestamp, now, item_id],
                )?;
                updated += result;
            }
        }
        
        Ok(updated)
    }

    // Custom View operations
    pub fn create_custom_view(&self, name: &str, source_ids: Option<&str>, group_names: Option<&str>) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().timestamp();
        conn.execute(
            "INSERT INTO custom_views (name, source_ids, group_names, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?4)",
            params![name, source_ids, group_names, now],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_all_custom_views(&self) -> Result<Vec<super::models::CustomView>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, source_ids, group_names, created_at, updated_at FROM custom_views ORDER BY name"
        )?;
        let views = stmt.query_map([], |row| super::models::CustomView::from_row(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(views)
    }

    pub fn get_custom_view(&self, id: i64) -> Result<super::models::CustomView> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, source_ids, group_names, created_at, updated_at FROM custom_views WHERE id = ?1"
        )?;
        stmt.query_row(params![id], |row| super::models::CustomView::from_row(row))
    }

    pub fn update_custom_view(&self, id: i64, name: &str, source_ids: Option<&str>, group_names: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().timestamp();
        conn.execute(
            "UPDATE custom_views SET name = ?1, source_ids = ?2, group_names = ?3, updated_at = ?4 WHERE id = ?5",
            params![name, source_ids, group_names, now, id],
        )?;
        Ok(())
    }

    pub fn delete_custom_view(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM custom_views WHERE id = ?1", params![id])?;
        Ok(())
    }

    // Event operations
    pub fn create_event(&self, item_id: i64, event_type: &str, payload_json: Option<&str>) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().timestamp();
        conn.execute(
            "INSERT INTO events (item_id, event_type, payload_json, occurred_at) VALUES (?1, ?2, ?3, ?4)",
            params![item_id, event_type, payload_json, now],
        )?;
        Ok(conn.last_insert_rowid())
    }

    // Group operations
    pub fn get_all_groups(&self) -> Result<Vec<super::models::Group>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, created_at, updated_at FROM groups ORDER BY name"
        )?;
        let groups = stmt.query_map([], |row| super::models::Group::from_row(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(groups)
    }

    pub fn create_group(&self, name: &str) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().timestamp();
        conn.execute(
            "INSERT INTO groups (name, created_at, updated_at) VALUES (?1, ?2, ?2)",
            params![name, now],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_group(&self, id: i64, name: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().timestamp();
        conn.execute(
            "UPDATE groups SET name = ?1, updated_at = ?2 WHERE id = ?3",
            params![name, now, id],
        )?;
        Ok(())
    }

    pub fn delete_group(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        // Delete from source_groups first (CASCADE should handle this, but explicit is better)
        conn.execute(
            "DELETE FROM source_groups WHERE group_id = ?1",
            params![id],
        )?;
        // Delete the group
        conn.execute(
            "DELETE FROM groups WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    pub fn get_source_groups(&self, source_id: i64) -> Result<Vec<i64>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT group_id FROM source_groups WHERE source_id = ?1"
        )?;
        let group_ids = stmt.query_map(params![source_id], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(group_ids)
    }

    // Internal helper that takes a connection reference (for use when connection is already locked)
    fn set_source_groups_internal(conn: &Connection, source_id: i64, group_ids: &[i64]) -> Result<()> {
        // Delete existing relationships
        conn.execute(
            "DELETE FROM source_groups WHERE source_id = ?1",
            params![source_id],
        )?;
        
        // Insert new relationships
        for group_id in group_ids {
            conn.execute(
                "INSERT INTO source_groups (source_id, group_id) VALUES (?1, ?2)",
                params![source_id, group_id],
            )?;
        }
        
        Ok(())
    }
    

    // Migrate existing groups from comma-separated strings to groups table
    fn migrate_existing_groups(conn: &Connection) -> Result<()> {
        // Check if groups table exists and has data
        let has_groups: bool = conn.query_row(
            "SELECT COUNT(*) FROM groups",
            [],
            |row| Ok(row.get::<_, i64>(0)? > 0),
        ).unwrap_or(false);

        // Only migrate if groups table is empty
        if has_groups {
            return Ok(());
        }

        // Get all sources with groups
        let mut stmt = conn.prepare(
            r#"SELECT id, "group" FROM sources WHERE "group" IS NOT NULL AND "group" != ''"#
        )?;
        let source_groups: Vec<(i64, String)> = stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;

        let now = Utc::now().timestamp();
        let mut group_map: std::collections::HashMap<String, i64> = std::collections::HashMap::new();

        for (source_id, group_string) in source_groups {
            // Parse comma-separated groups
            let groups: Vec<String> = group_string
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            for group_name in groups {
                // Get or create group
                let group_id = match group_map.get(&group_name) {
                    Some(&id) => id,
                    None => {
                        // Try to find existing group
                        let existing_id: Option<i64> = conn.query_row(
                            "SELECT id FROM groups WHERE name = ?1",
                            params![&group_name],
                            |row| row.get(0),
                        ).ok();

                        match existing_id {
                            Some(id) => {
                                group_map.insert(group_name.clone(), id);
                                id
                            }
                            None => {
                                // Create new group
                                conn.execute(
                                    "INSERT INTO groups (name, created_at, updated_at) VALUES (?1, ?2, ?2)",
                                    params![&group_name, now],
                                )?;
                                let id = conn.last_insert_rowid();
                                group_map.insert(group_name.clone(), id);
                                id
                            }
                        }
                    }
                };

                // Create source-group relationship (ignore if already exists)
                let _ = conn.execute(
                    "INSERT OR IGNORE INTO source_groups (source_id, group_id) VALUES (?1, ?2)",
                    params![source_id, group_id],
                );
            }
        }

        Ok(())
    }

    // Secret operations
    pub fn create_secret(&self, name: &str, ttl_type: &str, ttl_value: Option<&str>, refresh_token_id: Option<i64>) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().timestamp();
        
        // Calculate expires_at from TTL
        let expires_at = Self::calculate_expires_at(ttl_type, ttl_value)?;
        
        conn.execute(
            "INSERT INTO secrets (name, ttl_type, ttl_value, expires_at, refresh_token_id, refresh_failure_count, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, ?6)",
            params![name, ttl_type, ttl_value, expires_at, refresh_token_id, now],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_all_secrets(&self) -> Result<Vec<Secret>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, ttl_type, ttl_value, expires_at, refresh_token_id, refresh_failure_count, created_at, updated_at FROM secrets ORDER BY created_at DESC"
        )?;
        let secrets = stmt.query_map([], |row| Secret::from_row(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(secrets)
    }

    pub fn get_secret(&self, id: i64) -> Result<Secret> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, ttl_type, ttl_value, expires_at, refresh_token_id, refresh_failure_count, created_at, updated_at FROM secrets WHERE id = ?1"
        )?;
        stmt.query_row(params![id], |row| Secret::from_row(row))
    }

    pub fn get_secret_by_name(&self, name: &str) -> Result<Option<Secret>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, ttl_type, ttl_value, expires_at, refresh_token_id, refresh_failure_count, created_at, updated_at FROM secrets WHERE name = ?1"
        )?;
        match stmt.query_row(params![name], |row| Secret::from_row(row)) {
            Ok(secret) => Ok(Some(secret)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn update_secret(&self, id: i64, name: Option<&str>, ttl_type: Option<&str>, ttl_value: Option<Option<&str>>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().timestamp();
        
        if let Some(name) = name {
            conn.execute(
                "UPDATE secrets SET name = ?1, updated_at = ?2 WHERE id = ?3",
                params![name, now, id],
            )?;
        }
        
        if let Some(ttl_type) = ttl_type {
            let ttl_val = ttl_value.flatten();
            let expires_at = Self::calculate_expires_at(ttl_type, ttl_val.as_deref())?;
            conn.execute(
                "UPDATE secrets SET ttl_type = ?1, ttl_value = ?2, expires_at = ?3, updated_at = ?4 WHERE id = ?5",
                params![ttl_type, ttl_val, expires_at, now, id],
            )?;
        }
        
        Ok(())
    }

    pub fn delete_secret(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM secrets WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn get_expired_secrets(&self) -> Result<Vec<Secret>> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().timestamp();
        let mut stmt = conn.prepare(
            "SELECT id, name, ttl_type, ttl_value, expires_at, refresh_token_id, refresh_failure_count, created_at, updated_at FROM secrets WHERE expires_at IS NOT NULL AND expires_at < ?1"
        )?;
        let secrets = stmt.query_map(params![now], |row| Secret::from_row(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(secrets)
    }

    pub fn get_sources_using_secret(&self, secret_id: i64) -> Result<Vec<i64>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id FROM sources WHERE secret_id = ?1"
        )?;
        let source_ids = stmt.query_map(params![secret_id], |row| row.get::<_, i64>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(source_ids)
    }

    pub fn expire_secret(&self, secret_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().timestamp();
        conn.execute(
            "UPDATE secrets SET expires_at = ?1, updated_at = ?1 WHERE id = ?2",
            params![now, secret_id],
        )?;
        Ok(())
    }

    pub fn increment_refresh_failure_count(&self, secret_id: i64) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().timestamp();
        conn.execute(
            "UPDATE secrets SET refresh_failure_count = refresh_failure_count + 1, updated_at = ?1 WHERE id = ?2",
            params![now, secret_id],
        )?;
        // Get the new count
        let mut stmt = conn.prepare("SELECT refresh_failure_count FROM secrets WHERE id = ?1")?;
        let count: i64 = stmt.query_row(params![secret_id], |row| row.get(0))?;
        Ok(count)
    }

    pub fn reset_refresh_failure_count(&self, secret_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().timestamp();
        conn.execute(
            "UPDATE secrets SET refresh_failure_count = 0, updated_at = ?1 WHERE id = ?2",
            params![now, secret_id],
        )?;
        Ok(())
    }

    pub fn set_refresh_token_id(&self, secret_id: i64, refresh_token_id: Option<i64>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().timestamp();
        conn.execute(
            "UPDATE secrets SET refresh_token_id = ?1, updated_at = ?2 WHERE id = ?3",
            params![refresh_token_id, now, secret_id],
        )?;
        Ok(())
    }

    // Helper function to calculate expires_at from TTL
    fn calculate_expires_at(ttl_type: &str, ttl_value: Option<&str>) -> Result<Option<i64>> {
        match ttl_type {
            "forever" => Ok(None),
            "relative" => {
                if let Some(value) = ttl_value {
                    // Parse relative duration (e.g., "30d", "1h", "2w")
                    let duration = Self::parse_relative_duration(value)?;
                    let expires_at = Utc::now().timestamp() + duration;
                    Ok(Some(expires_at))
                } else {
                    Ok(None) // No value means forever
                }
            }
            "absolute" => {
                if let Some(value) = ttl_value {
                    // Parse ISO 8601 date
                    let dt = chrono::DateTime::parse_from_rfc3339(value)
                        .map_err(|e| rusqlite::Error::InvalidParameterName(format!("Invalid date format: {}", e)))?;
                    Ok(Some(dt.timestamp()))
                } else {
                    Ok(None) // No value means forever
                }
            }
            _ => Ok(None), // Unknown type defaults to forever
        }
    }

    // Helper function to parse relative duration (e.g., "30d", "1h", "2w")
    fn parse_relative_duration(value: &str) -> Result<i64> {
        if value.is_empty() {
            return Err(rusqlite::Error::InvalidParameterName("Empty duration value".to_string()));
        }
        
        let (num_str, unit) = value.split_at(value.len() - 1);
        let num: i64 = num_str.parse()
            .map_err(|e| rusqlite::Error::InvalidParameterName(format!("Invalid number in duration: {}", e)))?;
        
        let seconds = match unit {
            "s" => num,
            "m" => num * 60,
            "h" => num * 3600,
            "d" => num * 86400,
            "w" => num * 604800,
            "M" => num * 2592000, // Approximate month (30 days)
            "y" => num * 31536000, // Approximate year (365 days)
            _ => return Err(rusqlite::Error::InvalidParameterName(format!("Unknown duration unit: {}", unit))),
        };
        
        Ok(seconds)
    }

    // User preferences operations
    pub fn get_user_preference(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT value FROM user_preferences WHERE key = ?1")?;
        let mut rows = stmt.query_map(params![key], |row| {
            Ok(row.get::<_, String>(0)?)
        })?;
        
        if let Some(row) = rows.next() {
            Ok(Some(row?))
        } else {
            Ok(None)
        }
    }

    pub fn set_user_preference(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO user_preferences (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_creation() {
        let db = Database::new(":memory:").unwrap();
        let conn = db.conn.lock().unwrap();
        
        // Test that tables exist
        let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table'").unwrap();
        let tables: Vec<String> = stmt.query_map([], |row| row.get::<_, String>(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        
        assert!(tables.contains(&"sources".to_string()));
        assert!(tables.contains(&"items".to_string()));
        assert!(tables.contains(&"events".to_string()));
    }
}

