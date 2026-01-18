use super::traits::{IngestSource, IngestedItem};
use super::utils;
use anyhow::{Result, Context};
use reqwest::blocking::Client;
use atom_syndication::{Feed, Entry};
use std::time::Duration;

pub struct AtomIngester {
    url: String,
    client: Client,
}

impl AtomIngester {
    pub fn new(url: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(60)) // Increased to 60 seconds for slow feeds
            .build()
            .context("Failed to create HTTP client")?;
        
        Ok(AtomIngester { url, client })
    }
}

/// Extracts content from ATOM entry, trying content element first, then summary fallback.
fn extract_content(entry: &Entry) -> Option<String> {
    // Try content first
    if let Some(content) = entry.content() {
        // Check if content has a value
        if let Some(value) = content.value.as_ref() {
            if !value.is_empty() {
                return Some(value.clone());
            }
        }
        // Check if there's a src attribute (external content)
        if let Some(src) = content.src.as_ref() {
            return Some(src.clone());
        }
    }
    
    // Fallback to summary if no content
    entry.summary()
        .map(|s| s.value.clone())
}

/// Extracts image URL from ATOM entry links with rel="enclosure" and image MIME type.
fn extract_image_url(entry: &Entry) -> Option<String> {
    // Look for link with rel="enclosure" that has an image type
    for link in entry.links() {
        if link.rel == "enclosure" {
            // Check if it's an image based on type attribute
            if let Some(mime_type) = link.mime_type.as_ref() {
                if mime_type.starts_with("image/") {
                    return Some(link.href.clone());
                }
            }
        }
    }
    
    // Also check for media:content (Media RSS extension)
    // This would require parsing the raw XML, but for now we'll skip it
    // as it's not standard ATOM
    
    None
}

impl IngestSource for AtomIngester {
    fn poll(&self) -> Result<Vec<IngestedItem>> {
        let response = self.client
            .get(&self.url)
            .send()
            .with_context(|| format!("Failed to fetch ATOM feed from: {}", self.url))?;
        
        let status = response.status();
        if !status.is_success() {
            let error_msg = match status.as_u16() {
                504 => format!(
                    "Gateway timeout (504) when fetching ATOM feed from: {}. The server took too long to respond. This may indicate the feed URL is incorrect or the server is overloaded.",
                    self.url
                ),
                404 => format!(
                    "Feed not found (404) at: {}. Please verify the URL is correct. For GitHub user feeds, use: https://github.com/USERNAME.private.atom (with authentication) or https://github.com/USERNAME.atom (public)",
                    self.url
                ),
                403 => format!(
                    "Access forbidden (403) when fetching ATOM feed from: {}. The feed may require authentication or the server is blocking requests.",
                    self.url
                ),
                _ => format!(
                    "HTTP error {} when fetching ATOM feed from: {}",
                    status, self.url
                ),
            };
            anyhow::bail!("{}", error_msg);
        }
        
        let content = response.text()
            .with_context(|| format!("Failed to read ATOM feed content from: {}", self.url))?;
        
        // Check if content is empty
        if content.trim().is_empty() {
            anyhow::bail!("ATOM feed from {} is empty", self.url);
        }
        
        // Check if content looks like XML/ATOM
        if !content.trim_start().starts_with("<?xml") && !content.trim_start().starts_with("<feed") {
            anyhow::bail!(
                "ATOM feed from {} does not appear to be valid XML (starts with: {})",
                self.url,
                content.chars().take(50).collect::<String>()
            );
        }
        
        let feed = Feed::read_from(content.as_bytes())
            .with_context(|| {
                // Try to provide more context about the parse error
                let preview = if content.len() > 200 {
                    format!("{}...", &content[..200])
                } else {
                    content.clone()
                };
                format!(
                    "Failed to parse ATOM feed from {}: Invalid XML structure. Content preview: {}",
                    self.url,
                    preview.replace('\n', " ").replace('\r', " ")
                )
            })?;
        
        let items: Vec<IngestedItem> = feed.entries()
            .iter()
            .map(|entry| {
                // Required fields
                let external_id = entry.id.clone();
                let title = if entry.title.value.is_empty() {
                    "Untitled".to_string()
                } else {
                    entry.title.value.clone()
                };
                
                // Updated timestamp (required) - use for occurred_at
                let occurred_at = entry.updated.timestamp();
                
                // Published timestamp (optional) - could be used instead
                // For now, we'll use updated as occurred_at
                // If published exists and is different, we could store it separately later
                
                // Summary (recommended)
                let summary = entry.summary()
                    .and_then(|s| {
                        if s.value.is_empty() {
                            None
                        } else {
                            let cleaned = utils::strip_html(&s.value);
                            if cleaned.is_empty() { None } else { Some(cleaned) }
                        }
                    });
                
                // Content (recommended)
                let content_html = extract_content(entry);
                
                // Link with rel="alternate" (recommended for url)
                let url = entry.links()
                    .iter()
                    .find(|link| link.rel == "alternate" || link.rel == "self")
                    .map(|link| link.href.clone())
                    .unwrap_or_else(|| {
                        // Fallback to id if it's a URL
                        if external_id.starts_with("http://") || external_id.starts_with("https://") {
                            external_id.clone()
                        } else {
                            String::new()
                        }
                    });
                
                // Image URL from enclosure links
                let image_url = extract_image_url(entry);
                
                // Author (recommended)
                let author = entry.authors()
                    .first()
                    .map(|a| a.name.clone());
                
                // Categories (optional)
                let category: Option<Vec<String>> = {
                    let categories: Vec<String> = entry.categories()
                        .iter()
                        .map(|c| c.term.clone())
                        .collect();
                    if categories.is_empty() {
                        None
                    } else {
                        Some(categories)
                    }
                };
                
                // ATOM doesn't have a comments field like RSS
                // Comments would typically be in a link with rel="replies"
                // For now, we'll leave it as None
                let comments = entry.links()
                    .iter()
                    .find(|link| link.rel == "replies" || link.rel == "comments")
                    .map(|link| link.href.clone());
                
                IngestedItem {
                    external_id,
                    title,
                    summary,
                    url,
                    item_type: "atom".to_string(),
                    occurred_at: Some(occurred_at),
                    image_url,
                    content_html,
                    author,
                    category,
                    comments,
                    thread_id: None,
                }
            })
            .collect();
        
        Ok(items)
    }
}

