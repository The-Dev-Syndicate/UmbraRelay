use crate::storage::Database;
use crate::ingestion::traits::IngestedItem;
use anyhow::Result;

pub fn normalize_and_dedupe(
    db: &Database,
    source_id: i64,
    items: Vec<IngestedItem>,
) -> Result<Vec<i64>> {
    let mut item_ids = Vec::new();
    
    for item in items {
        // Convert category Vec<String> to JSON string
        let category_json = item.category.as_ref().map(|cats| {
            serde_json::to_string(cats).unwrap_or_default()
        });
        
        let item_id = db.upsert_item(
            source_id,
            &item.external_id,
            &item.title,
            item.summary.as_deref(),
            &item.url,
            &item.item_type,
            item.image_url.as_deref(),
            item.content_html.as_deref(),
            item.author.as_deref(),
            category_json.as_deref(),
            item.comments.as_deref(),
        )?;
        
        item_ids.push(item_id);
        
        // Optionally create an event record
        if let Some(occurred_at) = item.occurred_at {
            let _ = db.create_event(
                item_id,
                "update",
                Some(&serde_json::json!({
                    "occurred_at": occurred_at
                }).to_string()),
            );
        }
    }
    
    Ok(item_ids)
}

