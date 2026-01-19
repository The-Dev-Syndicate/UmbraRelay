use anyhow::{Result, Context};
use reqwest::blocking::Client;
use std::time::Duration;
use readabilityrs::{Readability, ReadabilityOptions};
use ammonia::clean;

#[derive(Debug, Clone)]
pub struct ExtractionResult {
    pub content: String,
    #[allow(dead_code)] // Reserved for future use (e.g., comparing with feed title)
    pub title: Option<String>,
}

/// Extracts full article content from a URL using readability algorithm.
/// Sanitizes HTML for safe rendering.
pub fn extract_full_text(url: &str) -> Result<ExtractionResult> {
    // Create HTTP client with timeout
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("UmbraRelay/1.0")
        .build()
        .context("Failed to create HTTP client")?;
    
    // Fetch the HTML
    let response = client
        .get(url)
        .send()
        .context("Failed to fetch article URL")?;
    
    if !response.status().is_success() {
        anyhow::bail!("HTTP error {} when fetching article", response.status());
    }
    
    let html = response.text()
        .context("Failed to read response body")?;
    
    // Use readability to extract main content
    let options = ReadabilityOptions::default();
    let readability = Readability::new(&html, Some(url), Some(options))
        .context("Failed to initialize readability")?;
    
    let article = readability.parse()
        .ok_or_else(|| anyhow::anyhow!("Failed to extract article content - no readable content found"))?;
    
    // Get extracted content - handle Option types
    let extracted_html = article.content
        .ok_or_else(|| anyhow::anyhow!("No content extracted from article"))?;
    let extracted_title = article.title;
    
    // Sanitize HTML to remove dangerous elements (scripts, iframes, etc.)
    let sanitized = clean(&extracted_html);
    
    Ok(ExtractionResult {
        content: sanitized,
        title: extracted_title,
    })
}

#[cfg(test)]
mod tests {
    
    // Note: These tests require network access and may be flaky
    // In a real scenario, you'd mock the HTTP client
    
    #[test]
    #[ignore] // Ignore by default since it requires network
    fn test_extract_from_real_url() {
        // This would test with a real URL, but we'll skip it for now
        // In production, you'd use a mock HTTP client
    }
}
