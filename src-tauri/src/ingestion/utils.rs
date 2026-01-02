use regex::Regex;

/// Strip HTML tags and decode entities from text.
/// Removes HTML tags, decodes common entities, and filters out unwanted patterns like "Comments" links.
pub fn strip_html(html: &str) -> String {
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

