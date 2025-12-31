use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestedItem {
    pub external_id: String,
    pub title: String,
    pub summary: Option<String>,
    pub url: String,
    pub item_type: String,
    pub occurred_at: Option<i64>,
    pub image_url: Option<String>,
    pub content_html: Option<String>,
}

pub trait IngestSource: Send + Sync {
    fn poll(&self) -> Result<Vec<IngestedItem>>;
}

