use super::traits::{IngestSource, IngestedItem};
use anyhow::{Result, Context};
use reqwest::blocking::Client;
use rss::Channel;
use std::time::Duration;

pub struct RssIngester {
    url: String,
    client: Client,
}

impl RssIngester {
    pub fn new(url: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;
        
        Ok(RssIngester { url, client })
    }
}

impl IngestSource for RssIngester {
    fn poll(&self) -> Result<Vec<IngestedItem>> {
        let response = self.client
            .get(&self.url)
            .send()
            .context("Failed to fetch RSS feed")?;
        
        let content = response.text()
            .context("Failed to read RSS feed content")?;
        
        let channel = Channel::read_from(content.as_bytes())
            .context("Failed to parse RSS feed")?;
        
        let items: Vec<IngestedItem> = channel.items()
            .iter()
            .map(|item| {
                let external_id = item.guid()
                    .map(|g| g.value().to_string())
                    .unwrap_or_else(|| {
                        // Fallback to link if no GUID
                        item.link().unwrap_or("").to_string()
                    });
                
                let title = item.title().unwrap_or("Untitled").to_string();
                let summary = item.description().map(|s| s.to_string());
                let url = item.link().unwrap_or("").to_string();
                
                // Parse pub_date if available
                let occurred_at = item.pub_date()
                    .and_then(|date_str| {
                        chrono::DateTime::parse_from_rfc2822(date_str)
                            .ok()
                            .map(|dt| dt.timestamp())
                    });
                
                IngestedItem {
                    external_id,
                    title,
                    summary,
                    url,
                    item_type: "post".to_string(),
                    occurred_at,
                }
            })
            .collect();
        
        Ok(items)
    }
}

