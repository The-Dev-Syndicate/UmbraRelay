use serde::{Deserialize, Serialize};
use rusqlite::Row;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub id: i64,
    pub source_type: String,
    pub name: String,
    pub config_json: String,
    pub enabled: bool,
    pub last_synced_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: i64,
    pub source_id: i64,
    pub external_id: String,
    pub title: String,
    pub summary: Option<String>,
    pub url: String,
    pub item_type: String,
    pub state: String,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Source {
    pub fn from_row(row: &Row) -> rusqlite::Result<Source> {
        Ok(Source {
            id: row.get(0)?,
            source_type: row.get(1)?,
            name: row.get(2)?,
            config_json: row.get(3)?,
            enabled: row.get::<_, i64>(4)? != 0,
            last_synced_at: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    }
}

impl Item {
    pub fn from_row(row: &Row) -> rusqlite::Result<Item> {
        Ok(Item {
            id: row.get(0)?,
            source_id: row.get(1)?,
            external_id: row.get(2)?,
            title: row.get(3)?,
            summary: row.get(4)?,
            url: row.get(5)?,
            item_type: row.get(6)?,
            state: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
        })
    }
}


