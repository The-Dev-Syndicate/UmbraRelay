use rusqlite::{Connection, Result, params};
use rusqlite_migration::{Migrations, M};
use std::sync::{Arc, Mutex};
use chrono::Utc;
use super::models::{Source, Item};

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
        ]);

        migrations.to_latest(&mut conn)
            .map_err(|e| {
                // Convert migration error to rusqlite error
                rusqlite::Error::SqliteFailure(
                    rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_MISUSE),
                    Some(format!("Migration error: {}", e))
                )
            })?;

        Ok(Database {
            conn: Arc::new(Mutex::new(conn)),
        })
    }


    // Source CRUD operations
    pub fn create_source(&self, source_type: &str, name: &str, config_json: &str, group: Option<&str>) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().timestamp();
        conn.execute(
            r#"INSERT INTO sources (type, name, config_json, enabled, "group", created_at, updated_at) VALUES (?1, ?2, ?3, 1, ?4, ?5, ?5)"#,
            params![source_type, name, config_json, group, now],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_all_sources(&self) -> Result<Vec<Source>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"SELECT id, type, name, config_json, enabled, last_synced_at, "group", created_at, updated_at FROM sources ORDER BY created_at DESC"#
        )?;
        let sources = stmt.query_map([], |row| Source::from_row(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(sources)
    }

    pub fn get_source(&self, id: i64) -> Result<Source> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"SELECT id, type, name, config_json, enabled, last_synced_at, "group", created_at, updated_at FROM sources WHERE id = ?1"#
        )?;
        stmt.query_row(params![id], |row| Source::from_row(row))
    }

    pub fn update_source(&self, id: i64, name: Option<&str>, config_json: Option<&str>, enabled: Option<bool>, group: Option<Option<&str>>) -> Result<()> {
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
        
        if let Some(group) = group {
            // Treat empty strings as NULL
            let group_value: Option<&str> = if let Some(g) = group {
                if g.is_empty() {
                    None
                } else {
                    Some(g)
                }
            } else {
                None
            };
            conn.execute(
                r#"UPDATE sources SET "group" = ?1, updated_at = ?2 WHERE id = ?3"#,
                params![group_value, now, id],
            )?;
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
                    "UPDATE items SET title = ?1, summary = ?2, url = ?3, item_type = ?4, image_url = ?5, content_html = ?6, updated_at = ?7 WHERE id = ?8",
                    params![title, summary, url, item_type, image_url, content_html, now, id],
                )?;
                Ok(id)
            }
            Err(_) => {
                // Insert new item
                conn.execute(
                    "INSERT INTO items (source_id, external_id, title, summary, url, item_type, image_url, content_html, state, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'unread', ?9, ?9)",
                    params![source_id, external_id, title, summary, url, item_type, image_url, content_html, now],
                )?;
                Ok(conn.last_insert_rowid())
            }
        }
    }

    pub fn get_items(&self, state_filter: Option<&str>, group_filter: Option<&str>) -> Result<Vec<Item>> {
        let conn = self.conn.lock().unwrap();
        
        // Always JOIN with sources to get source info
        let mut query = "SELECT i.id, i.source_id, i.external_id, i.title, i.summary, i.url, i.item_type, i.state, i.created_at, i.updated_at, i.image_url, i.content_html, s.name as source_name, s.\"group\" as source_group FROM items i INNER JOIN sources s ON i.source_id = s.id".to_string();
        let mut conditions = Vec::new();
        
        if let Some(_) = state_filter {
            conditions.push("i.state = ?1");
        }
        
        if let Some(_) = group_filter {
            // Support comma-separated groups - check if group appears in the comma-separated string
            // Match: exact match, at start (group,), in middle (%, group, %), or at end (%, group)
            if state_filter.is_some() {
                conditions.push(r#"(s."group" = ?2 OR s."group" LIKE ?3 OR s."group" LIKE ?4 OR s."group" LIKE ?5)"#);
            } else {
                conditions.push(r#"(s."group" = ?1 OR s."group" LIKE ?2 OR s."group" LIKE ?3 OR s."group" LIKE ?4)"#);
            }
        }
        
        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }
        
        query.push_str(" ORDER BY i.created_at DESC");
        
        let mut stmt = conn.prepare(&query)?;
        let items: Vec<Item> = match (state_filter, group_filter) {
            (Some(state), Some(group)) => {
                // For comma-separated groups, check if group appears at start, middle, or end
                let like_start = format!("{},%", group);
                let like_middle = format!("%, {}, %", group);
                let like_end = format!("%, {}", group);
                stmt.query_map(params![state, group, like_start, like_middle, like_end], |row| Item::from_row_with_source(row))?
                    .collect::<Result<Vec<_>, _>>()?
            }
            (Some(state), None) => {
                stmt.query_map(params![state], |row| Item::from_row_with_source(row))?
                    .collect::<Result<Vec<_>, _>>()?
            }
            (None, Some(group)) => {
                // For comma-separated groups, check if group appears at start, middle, or end
                let like_start = format!("{},%", group);
                let like_middle = format!("%, {}, %", group);
                let like_end = format!("%, {}", group);
                stmt.query_map(params![group, like_start, like_middle, like_end], |row| Item::from_row_with_source(row))?
                    .collect::<Result<Vec<_>, _>>()?
            }
            (None, None) => {
                stmt.query_map([], |row| Item::from_row_with_source(row))?
                    .collect::<Result<Vec<_>, _>>()?
            }
        };
        Ok(items)
    }

    pub fn get_item(&self, id: i64) -> Result<Item> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, source_id, external_id, title, summary, url, item_type, state, created_at, updated_at, image_url, content_html FROM items WHERE id = ?1"
        )?;
        stmt.query_row(params![id], |row| Item::from_row(row))
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

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_creation() {
        let db = Database::new(":memory:").unwrap();
        let conn = db.get_conn();
        let conn = conn.lock().unwrap();
        
        // Test that tables exist
        let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table'").unwrap();
        let tables: Vec<String> = stmt.query_map([], |row| row.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        
        assert!(tables.contains(&"sources".to_string()));
        assert!(tables.contains(&"items".to_string()));
        assert!(tables.contains(&"events".to_string()));
    }
}

