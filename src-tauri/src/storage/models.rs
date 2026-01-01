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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_html: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>, // JSON array string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_group: Option<String>,
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
            image_url: row.get(10).ok(),
            content_html: row.get(11).ok(),
            author: row.get(12).ok(),
            category: row.get(13).ok(),
            comments: row.get(14).ok(),
            source_name: None,
            source_group: None,
        })
    }
    
    pub fn from_row_with_source(row: &Row) -> rusqlite::Result<Item> {
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
            image_url: row.get(10).ok(),
            content_html: row.get(11).ok(),
            author: row.get(12).ok(),
            category: row.get(13).ok(),
            comments: row.get(14).ok(),
            source_name: row.get(15).ok(),
            source_group: row.get(16).ok(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: i64,
    pub name: String,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Group {
    pub fn from_row(row: &Row) -> rusqlite::Result<Group> {
        Ok(Group {
            id: row.get(0)?,
            name: row.get(1)?,
            created_at: row.get(2)?,
            updated_at: row.get(3)?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomView {
    pub id: i64,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_ids: Option<String>, // JSON array of source IDs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_names: Option<String>, // JSON array of group names
    pub created_at: i64,
    pub updated_at: i64,
}

impl CustomView {
    pub fn from_row(row: &Row) -> rusqlite::Result<CustomView> {
        Ok(CustomView {
            id: row.get(0)?,
            name: row.get(1)?,
            source_ids: row.get(2)?,
            group_names: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    }
}


