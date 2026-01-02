use super::traits::{IngestSource, IngestedItem};
use anyhow::{Result, Context};
use reqwest::blocking::Client;
use std::time::Duration;

pub struct GitHubNotificationsIngester {
    token: String,
    client: Client,
}

impl GitHubNotificationsIngester {
    pub fn new(token: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;
        
        Ok(GitHubNotificationsIngester {
            token,
            client,
        })
    }

    fn notification_to_item(&self, notification: serde_json::Value) -> Result<Option<IngestedItem>> {
        let id = notification.get("id").and_then(|v| v.as_str()).unwrap_or("");
        if id.is_empty() {
            return Ok(None);
        }
        
        let thread_id = id.to_string();
        let external_id = format!("notification_{}", id);
        
        let subject = match notification.get("subject") {
            Some(s) => s,
            None => return Ok(None),
        };
        
        let title = subject.get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Untitled")
            .to_string();
        
        let subject_type = subject.get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        
        let reason = notification.get("reason")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        
        // Convert API URL to web URL - subject.url can be null
        let url = subject.get("url")
            .and_then(|u| {
                if u.is_null() {
                    None
                } else {
                    u.as_str()
                }
            })
            .map(|s| {
                // Handle both API URLs and web URLs
                if s.starts_with("https://api.github.com/repos/") {
                    s.replace("https://api.github.com/repos/", "https://github.com/")
                } else if s.starts_with("https://github.com/") {
                    s.to_string()
                } else {
                    // If it's a relative URL or something else, construct the full URL
                    format!("https://github.com/notifications/threads/{}", id)
                }
            })
            .unwrap_or_else(|| {
                format!("https://github.com/notifications/threads/{}", id)
            });
        
        let repo = notification.get("repository")
            .and_then(|r| r.get("full_name"))
            .and_then(|n| n.as_str())
            .unwrap_or("unknown");
        
        let occurred_at = notification.get("updated_at")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.timestamp());
        
        let summary = Some(format!("{} - {}", subject_type, reason));
        
        Ok(Some(IngestedItem {
            external_id,
            title,
            summary,
            url,
            item_type: "notification".to_string(),
            occurred_at,
            image_url: None,
            content_html: None,
            author: None,
            category: Some(vec![repo.to_string()]),
            comments: None,
            thread_id: Some(thread_id),
        }))
    }
}

impl IngestSource for GitHubNotificationsIngester {
    fn poll(&self) -> Result<Vec<IngestedItem>> {
        let mut all_items = Vec::new();
        let mut thread_ids = Vec::new();
        let mut page = 1;
        let per_page = 100;
        
        loop {
            // Fetch ALL notifications (both read and unread)
            // The API defaults to unread only, so we use all=true to get everything
            let url = format!(
                "https://api.github.com/notifications?per_page={}&page={}&all=true",
                per_page, page
            );
            
            let response = self.client
                .get(&url)
                .header("Authorization", format!("Bearer {}", self.token))
                .header("Accept", "application/vnd.github.v3+json")
                .header("User-Agent", "UmbraRelay")
                .send()
                .context("Failed to send GitHub API request")?;
            
            let status = response.status();
            
            // Check for 304 Not Modified (no new notifications)
            if status == 304 {
                break;
            }
            
            if status == 401 {
                let error_text = response.text().unwrap_or_else(|_| "Unknown error".to_string());
                return Err(anyhow::anyhow!(
                    "GitHub API returned 401 Unauthorized - token may be expired or invalid. Response: {}",
                    error_text
                ));
            }
            
            if status == 403 {
                let error_text = response.text().unwrap_or_else(|_| "Unknown error".to_string());
                return Err(anyhow::anyhow!(
                    "GitHub API returned 403 Forbidden. \
                    Your Personal Access Token must have the 'notifications' scope. \
                    Please create a new PAT with notifications read permission. \
                    Response: {}",
                    error_text
                ));
            }
            
            if !status.is_success() {
                let error_text = response.text().unwrap_or_else(|_| format!("Status: {}", status));
                return Err(anyhow::anyhow!(
                    "GitHub API error: {} - {}",
                    status,
                    error_text
                ));
            }
            
            let notifications: Vec<serde_json::Value> = response.json()
                .context("Failed to parse notifications response")?;
            
            let notifications_len = notifications.len();
            
            if notifications_len == 0 {
                break;
            }
            
            for notification in &notifications {
                if let Some(thread_id) = notification.get("id").and_then(|v| v.as_str()) {
                    thread_ids.push(thread_id.to_string());
                }
                
                if let Ok(Some(item)) = self.notification_to_item(notification.clone()) {
                    all_items.push(item);
                }
            }
            
            if notifications_len < per_page {
                break;
            }
            
            page += 1;
        }
        
        
        // NOTE: We're NOT marking notifications as read on GitHub automatically
        // This allows users to manage their GitHub notifications separately
        // If you want to mark them as read, uncomment the code below
        
        // Mark all fetched notifications as read on GitHub (DISABLED)
        // if !thread_ids.is_empty() {
        //     eprintln!("GitHub notifications: Marking {} notifications as read on GitHub", thread_ids.len());
        //     let mut marked_count = 0;
        //     for thread_id in &thread_ids {
        //         let mark_url = format!("https://api.github.com/notifications/threads/{}", thread_id);
        //         match self.client
        //             .patch(&mark_url)
        //             .header("Authorization", format!("Bearer {}", self.token))
        //             .header("Accept", "application/vnd.github.v3+json")
        //             .header("User-Agent", "UmbraRelay")
        //             .send()
        //         {
        //             Ok(resp) => {
        //                 if resp.status().is_success() {
        //                     marked_count += 1;
        //                 } else {
        //                     eprintln!("GitHub notifications: Failed to mark thread {} as read: HTTP {}", thread_id, resp.status());
        //                 }
        //             }
        //             Err(e) => {
        //                 eprintln!("GitHub notifications: Error marking thread {} as read: {}", thread_id, e);
        //             }
        //         }
        //     }
        //     eprintln!("GitHub notifications: Successfully marked {} of {} notifications as read", marked_count, thread_ids.len());
        // }
        
        Ok(all_items)
    }
}

