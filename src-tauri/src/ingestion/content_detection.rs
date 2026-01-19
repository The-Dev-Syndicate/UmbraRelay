use super::traits::IngestedItem;
use regex::Regex;
use once_cell::sync::Lazy;

// Compile regexes once instead of on every call
static HTML_TAG_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"<[^>]+>").unwrap());
static LINK_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"<a[^>]*>.*?</a>").unwrap());

#[derive(Debug, Clone, PartialEq)]
pub enum ContentCompleteness {
    Full,
    Partial,
    Unknown,
}

impl ContentCompleteness {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentCompleteness::Full => "full",
            ContentCompleteness::Partial => "partial",
            ContentCompleteness::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone)]
pub struct DetectionResult {
    pub completeness: ContentCompleteness,
    #[allow(dead_code)] // Reserved for future debugging/logging UI
    pub confidence: String, // "high", "medium", "low"
    #[allow(dead_code)] // Reserved for future debugging/logging UI
    pub reason: String,     // Human-readable reason
}

/// Detects whether an RSS/Atom feed entry contains full content or just a summary.
/// Uses multiple signals to determine completeness with confidence levels.
/// 
/// Note: Extraction will use `item.url` (from RSS `<link>` or Atom `<link rel="alternate">`),
/// not URLs parsed from CDATA or content_html. This ensures we fetch the canonical article URL.
pub fn detect_content_completeness(item: &IngestedItem) -> DetectionResult {
    // Check if we have content_html
    // Note: We use item.url (from link tag) for extraction, not URLs from content_html
    let content_html = item.content_html.as_deref().unwrap_or("");
    let summary = item.summary.as_deref().unwrap_or("");
    
    // Signal 1: Content length
    let content_length = content_html.len();
    let summary_length = summary.len();
    
    // Signal 2: HTML structure detection
    let has_html_tags = content_html.contains('<') && content_html.contains('>');
    let html_tag_count = if has_html_tags {
        HTML_TAG_REGEX.find_iter(content_html).count()
    } else {
        0
    };
    
    // Signal 4: Check if content_html is just CDATA or minimal content (like Hacker News)
    // Hacker News often has CDATA in description that's just a link
    // Note: We use item.url (from RSS <link> or Atom <link rel="alternate">) for extraction,
    // not URLs parsed from CDATA content
    let is_cdata_only = content_html.starts_with("<![CDATA[") && content_html.len() < 200;
    
    // Also check if summary contains CDATA (Hacker News pattern)
    let summary_is_cdata = summary.starts_with("<![CDATA[") && summary.len() < 200;
    
    let is_mostly_link = {
        if content_length == 0 {
            false
        } else {
            let link_matches: Vec<_> = LINK_REGEX.find_iter(content_html).collect();
            let link_text_length: usize = link_matches.iter()
                .map(|m| m.as_str().len())
                .sum();
            // If links make up more than 70% of content, it's likely just a link
            link_text_length > 0 && (link_text_length as f64 / content_length as f64) > 0.7
        }
    };
    
    // Signal 5: Check if we have a URL but minimal/no content (should fetch)
    // This is the canonical URL from RSS <link> or Atom <link rel="alternate"> tag
    let has_url = !item.url.is_empty();
    
    // Signal 3: Content vs summary ratio
    let content_summary_ratio = if summary_length > 0 {
        content_length as f64 / summary_length as f64
    } else {
        if content_length > 0 {
            10.0 // Content exists but no summary
        } else {
            0.0
        }
    };
    
    // Decision logic
    // High confidence: Full content
    if content_length > 500 && has_html_tags && html_tag_count > 5 {
        return DetectionResult {
            completeness: ContentCompleteness::Full,
            confidence: "high".to_string(),
            reason: format!(
                "Content has {} characters with substantial HTML structure ({} tags)",
                content_length, html_tag_count
            ),
        };
    }
    
    // High confidence: Partial content - CDATA only or mostly links (like Hacker News)
    // Hacker News feeds often have CDATA in description/content_html that's just a link
    // We'll use item.url (from <link> tag) to fetch the full article
    if is_cdata_only || summary_is_cdata || is_mostly_link {
        return DetectionResult {
            completeness: ContentCompleteness::Partial,
            confidence: "high".to_string(),
            reason: format!(
                "Content is CDATA-only or mostly links ({} chars), will fetch from link tag URL",
                content_length
            ),
        };
    }
    
    // High confidence: Partial content
    if content_length < 100 && summary_length > 0 {
        return DetectionResult {
            completeness: ContentCompleteness::Partial,
            confidence: "high".to_string(),
            reason: format!(
                "Content is very short ({} chars) and summary exists ({} chars)",
                content_length, summary_length
            ),
        };
    }
    
    // High confidence: Partial if we have URL but no/minimal content
    let is_minimal_with_url = has_url && content_length < 150 && (content_length == 0 || is_cdata_only);
    if is_minimal_with_url {
        return DetectionResult {
            completeness: ContentCompleteness::Partial,
            confidence: "high".to_string(),
            reason: format!(
                "URL available but content is minimal/empty ({} chars), should fetch",
                content_length
            ),
        };
    }
    
    // Medium confidence: Likely partial if only summary exists
    if content_length == 0 && summary_length > 0 {
        return DetectionResult {
            completeness: ContentCompleteness::Partial,
            confidence: "medium".to_string(),
            reason: format!(
                "No content_html found, only summary ({} chars)",
                summary_length
            ),
        };
    }
    
    // Medium confidence: Likely full if content is substantial but no summary
    if content_length > 300 && summary_length == 0 && has_html_tags {
        return DetectionResult {
            completeness: ContentCompleteness::Full,
            confidence: "medium".to_string(),
            reason: format!(
                "Content has {} characters with HTML structure, no summary",
                content_length
            ),
        };
    }
    
    // Medium confidence: Content much longer than summary suggests full
    if content_summary_ratio > 3.0 && content_length > 200 {
        return DetectionResult {
            completeness: ContentCompleteness::Full,
            confidence: "medium".to_string(),
            reason: format!(
                "Content is {}x longer than summary ({} vs {} chars)",
                content_summary_ratio as i32, content_length, summary_length
            ),
        };
    }
    
    // Low confidence: Content exists but short
    if content_length > 0 && content_length < 300 {
        return DetectionResult {
            completeness: ContentCompleteness::Partial,
            confidence: "low".to_string(),
            reason: format!(
                "Content exists but is short ({} chars), may be partial",
                content_length
            ),
        };
    }
    
    // Default: Unknown
    DetectionResult {
        completeness: ContentCompleteness::Unknown,
        confidence: "low".to_string(),
        reason: format!(
            "Unable to determine completeness (content: {} chars, summary: {} chars)",
            content_length, summary_length
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_detect_full_content() {
        let item = IngestedItem {
            external_id: "test".to_string(),
            title: "Test".to_string(),
            summary: Some("Short summary".to_string()),
            url: "http://example.com".to_string(),
            item_type: "rss".to_string(),
            occurred_at: None,
            image_url: None,
            content_html: Some("<p>This is a long article with substantial content. ".repeat(20)),
            author: None,
            category: None,
            comments: None,
            thread_id: None,
        };
        
        let result = detect_content_completeness(&item);
        assert_eq!(result.completeness, ContentCompleteness::Full);
        assert_eq!(result.confidence, "high");
    }
    
    #[test]
    fn test_detect_partial_content() {
        let item = IngestedItem {
            external_id: "test".to_string(),
            title: "Test".to_string(),
            summary: Some("This is a longer summary that provides some context about the article.".to_string()),
            url: "http://example.com".to_string(),
            item_type: "rss".to_string(),
            occurred_at: None,
            image_url: None,
            content_html: Some("Read more...".to_string()),
            author: None,
            category: None,
            comments: None,
            thread_id: None,
        };
        
        let result = detect_content_completeness(&item);
        assert_eq!(result.completeness, ContentCompleteness::Partial);
    }
    
    #[test]
    fn test_detect_no_content() {
        let item = IngestedItem {
            external_id: "test".to_string(),
            title: "Test".to_string(),
            summary: Some("Summary only".to_string()),
            url: "http://example.com".to_string(),
            item_type: "rss".to_string(),
            occurred_at: None,
            image_url: None,
            content_html: None,
            author: None,
            category: None,
            comments: None,
            thread_id: None,
        };
        
        let result = detect_content_completeness(&item);
        assert_eq!(result.completeness, ContentCompleteness::Partial);
    }
}
