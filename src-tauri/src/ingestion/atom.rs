use super::traits::{IngestSource, IngestedItem};
use anyhow::{Result, Context};
use reqwest::blocking::Client;
use atom_syndication::{Feed, Entry};
use std::time::Duration;
use regex::Regex;

// Strip HTML tags and decode entities from text
fn strip_html(html: &str) -> String {
    // Remove HTML tags using regex
    let re = Regex::new(r"<[^>]*>").unwrap_or_else(|_| Regex::new("").unwrap());
    let mut text = re.replace_all(html, "").to_string();
    
    // Decode common HTML entities
    text = text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
        .replace("&apos;", "'");
    
    // Remove common unwanted patterns like "Comments" links
    let lines: Vec<&str> = text.lines().collect();
    let filtered_lines: Vec<&str> = lines.iter()
        .filter(|line| {
            let trimmed = line.trim().to_lowercase();
            !trimmed.eq("comments") 
                && !trimmed.starts_with("comments")
                && !trimmed.eq("read more")
                && !trimmed.is_empty()
        })
        .copied()
        .collect();
    
    let cleaned = filtered_lines.join(" ").trim().to_string();
    
    if cleaned.to_lowercase().trim() == "comments" || cleaned.trim().is_empty() {
        return String::new();
    }
    
    cleaned
}

pub struct AtomIngester {
    url: String,
    client: Client,
}

impl AtomIngester {
    pub fn new(url: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;
        
        Ok(AtomIngester { url, client })
    }
}

// Extract content from ATOM entry content element
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

// Extract image URL from entry links (enclosure or media)
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
            .context("Failed to fetch ATOM feed")?;
        
        let content = response.text()
            .context("Failed to read ATOM feed content")?;
        
        let feed = Feed::read_from(content.as_bytes())
            .context("Failed to parse ATOM feed")?;
        
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
                            let cleaned = strip_html(&s.value);
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

