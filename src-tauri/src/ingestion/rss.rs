use super::traits::{IngestSource, IngestedItem};
use anyhow::{Result, Context};
use reqwest::blocking::Client;
use rss::Channel;
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
    // Remove lines that are just "Comments" or similar link text
    let lines: Vec<&str> = text.lines().collect();
    let filtered_lines: Vec<&str> = lines.iter()
        .filter(|line| {
            let trimmed = line.trim().to_lowercase();
            // Filter out lines that are just "comments", "read more", etc.
            !trimmed.eq("comments") 
                && !trimmed.starts_with("comments")
                && !trimmed.eq("read more")
                && !trimmed.is_empty()
        })
        .copied()
        .collect();
    
    let cleaned = filtered_lines.join(" ").trim().to_string();
    
    // If the cleaned text is just "Comments" or similar, return empty
    if cleaned.to_lowercase().trim() == "comments" || cleaned.trim().is_empty() {
        return String::new();
    }
    
    cleaned
}

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

// Extract image URL and content from a single item's XML
fn extract_item_extras(item_xml: &str) -> (Option<String>, Option<String>) {
    let mut image_url = None;
    let mut content_html = None;
    
    // Try media:content first (Media RSS) - look for url attribute
    let media_content_re = Regex::new(r#"<media:content[^>]*url=["']([^"']+)["']"#).ok();
    if let Some(re) = media_content_re {
        if let Some(captures) = re.captures(item_xml) {
            if let Some(url) = captures.get(1) {
                let url_str = url.as_str();
                if url_str.starts_with("http://") || url_str.starts_with("https://") {
                    image_url = Some(url_str.to_string());
                }
            }
        }
    }
    
    // Try enclosure with image type (standard RSS)
    if image_url.is_none() {
        let enclosure_re = Regex::new(r#"<enclosure[^>]*url=["']([^"']+)["'][^>]*type=["']([^"']+)["']"#).ok();
        if let Some(re) = enclosure_re {
            if let Some(captures) = re.captures(item_xml) {
                if let Some(mime_type) = captures.get(2) {
                    if mime_type.as_str().to_lowercase().starts_with("image/") {
                        if let Some(url) = captures.get(1) {
                            let url_str = url.as_str();
                            if url_str.starts_with("http://") || url_str.starts_with("https://") {
                                image_url = Some(url_str.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Try content:encoded (Content module) - handle both CDATA and regular content, multiline
    // First try CDATA format: <content:encoded><![CDATA[...]]></content:encoded>
    if let Some(cdata_start) = item_xml.find("<content:encoded><![") {
        if let Some(cdata_content_start) = item_xml[cdata_start..].find("CDATA[") {
            let actual_content_start = cdata_start + cdata_content_start + 6; // 6 = len("CDATA[")
            if let Some(cdata_end) = item_xml[actual_content_start..].find("]]></content:encoded>") {
                let content = item_xml[actual_content_start..actual_content_start + cdata_end].trim();
                if !content.is_empty() {
                    content_html = Some(content.to_string());
                }
            }
        }
    }
    
    // If no CDATA, try regular format: <content:encoded>...</content:encoded>
    if content_html.is_none() {
        if let Some(start) = item_xml.find("<content:encoded>") {
            let content_start = start + 17; // length of "<content:encoded>"
            if let Some(end) = item_xml[content_start..].find("</content:encoded>") {
                let content = item_xml[content_start..content_start + end].trim();
                if !content.is_empty() {
                    content_html = Some(content.to_string());
                }
            }
        }
    }
    
    // Fallback: check if description contains HTML
    if content_html.is_none() {
        let desc_re = Regex::new(r#"<description><!\[CDATA\[(.*?)\]\]></description>"#).ok()
            .or_else(|| Regex::new(r#"<description>(.*?)</description>"#).ok());
        if let Some(re) = desc_re {
            if let Some(captures) = re.captures(item_xml) {
                if let Some(desc) = captures.get(1) {
                    let desc_str = desc.as_str().trim();
                    // Check if it contains HTML tags
                    if desc_str.contains('<') && desc_str.contains('>') && desc_str.len() > 50 {
                        content_html = Some(desc_str.to_string());
                    }
                }
            }
        }
    }
    
    (image_url, content_html)
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
        
        // Parse items with enhanced extraction
        // Extract all item blocks from XML first
        let item_blocks: Vec<String> = {
            let mut blocks = Vec::new();
            let mut start = 0;
            while let Some(item_start) = content[start..].find("<item") {
                let actual_start = start + item_start;
                // Find the end of this item
                if let Some(item_end) = content[actual_start..].find("</item>") {
                    let block = content[actual_start..actual_start + item_end + 7].to_string();
                    blocks.push(block);
                    start = actual_start + item_end + 7;
                } else {
                    break;
                }
            }
            blocks
        };
        
        // Match parsed items with their XML blocks
        let items: Vec<IngestedItem> = channel.items()
            .iter()
            .enumerate()
            .map(|(idx, item)| {
                let external_id = item.guid()
                    .map(|g| g.value().to_string())
                    .unwrap_or_else(|| {
                        // Fallback to link if no GUID
                        item.link().unwrap_or("").to_string()
                    });
                
                let title = item.title().unwrap_or("Untitled").to_string();
                let summary = item.description()
                    .and_then(|s| {
                        let cleaned = strip_html(s);
                        if cleaned.is_empty() { None } else { Some(cleaned) }
                    });
                let url = item.link().unwrap_or("").to_string();
                
                // Parse pub_date if available
                let occurred_at = item.pub_date()
                    .and_then(|date_str| {
                        chrono::DateTime::parse_from_rfc2822(date_str)
                            .ok()
                            .map(|dt| dt.timestamp())
                    });
                
                // Get the corresponding XML block by matching link or title
                // Use index as primary method since items should be in same order
                let item_xml = if idx < item_blocks.len() {
                    item_blocks[idx].clone()
                } else if let Some(link) = item.link() {
                    // Fallback: try to find by link
                    item_blocks.iter()
                        .find(|block| {
                            // Check for link tag containing this URL (try various formats)
                            block.contains(link) || 
                            block.contains(&format!("<link>{}</link>", link)) ||
                            block.contains(&format!(">{}</link>", link)) ||
                            block.contains(&format!("<link>{}</link>", link.replace("https://", "").replace("http://", "")))
                        })
                        .cloned()
                        .unwrap_or_default()
                } else if let Some(title) = item.title() {
                    // Fallback: try to match by title
                    item_blocks.iter()
                        .find(|block| block.contains(title))
                        .cloned()
                        .unwrap_or_default()
                } else {
                    String::new()
                };
                
                // Extract image and content from this item's XML
                let (image_url, content_html) = if !item_xml.is_empty() {
                    extract_item_extras(&item_xml)
                } else {
                    (None, None)
                };
                
                // Extract RSS 2.0 optional fields
                let author = item.author().map(|s| s.to_string());
                let category: Option<Vec<String>> = {
                    let categories: Vec<String> = item.categories()
                        .iter()
                        .map(|c| c.name().to_string())
                        .collect();
                    if categories.is_empty() {
                        None
                    } else {
                        Some(categories)
                    }
                };
                let comments = item.comments().map(|s| s.to_string());
                
                IngestedItem {
                    external_id,
                    title,
                    summary,
                    url,
                    item_type: "rss".to_string(),
                    occurred_at,
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

