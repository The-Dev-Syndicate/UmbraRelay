use super::traits::{IngestSource, IngestedItem};
use anyhow::{Result, Context};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
struct GitHubIssue {
    id: u64,
    number: u64,
    title: String,
    body: Option<String>,
    html_url: String,
    state: String,
    #[serde(rename = "pull_request")]
    pull_request: Option<serde_json::Value>,
    created_at: String,
    updated_at: String,
}

pub struct GitHubIngester {
    owner: String,
    repo: String,
    token: String,
    assigned_only: bool,
    client: Client,
}

impl GitHubIngester {
    pub fn new(owner: String, repo: String, token: String, assigned_only: bool) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;
        
        Ok(GitHubIngester {
            owner,
            repo,
            token,
            assigned_only,
            client,
        })
    }

    fn fetch_issues(&self) -> Result<Vec<GitHubIssue>> {
        let mut url = format!(
            "https://api.github.com/repos/{}/{}/issues",
            self.owner, self.repo
        );
        
        // Add query parameters
        let mut query_params = vec!["state=open".to_string()];
        if self.assigned_only {
            query_params.push("filter=assigned".to_string());
        }
        query_params.push("per_page=100".to_string());
        
        url.push_str(&format!("?{}", query_params.join("&")));
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("token {}", self.token))
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "UmbraRelay")
            .send()
            .context("Failed to fetch GitHub issues")?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "GitHub API error: {}",
                response.status()
            ));
        }
        
        let issues: Vec<GitHubIssue> = response.json()
            .context("Failed to parse GitHub API response")?;
        
        Ok(issues)
    }
}

impl IngestSource for GitHubIngester {
    fn poll(&self) -> Result<Vec<IngestedItem>> {
        let issues = self.fetch_issues()?;
        
        let items: Vec<IngestedItem> = issues
            .into_iter()
            .map(|issue| {
                let is_pr = issue.pull_request.is_some();
                let item_type = if is_pr { "pr" } else { "issue" };
                
                let external_id = format!("{}/{}#{}", self.owner, self.repo, issue.number);
                
                let summary = issue.body
                    .as_ref()
                    .and_then(|b| {
                        // Truncate long bodies
                        if b.len() > 500 {
                            Some(format!("{}...", &b[..500]))
                        } else {
                            Some(b.clone())
                        }
                    });
                
                // Parse GitHub timestamp
                let occurred_at = chrono::DateTime::parse_from_rfc3339(&issue.updated_at)
                    .ok()
                    .map(|dt| dt.timestamp());
                
                IngestedItem {
                    external_id,
                    title: issue.title,
                    summary,
                    url: issue.html_url,
                    item_type: item_type.to_string(),
                    occurred_at,
                }
            })
            .collect();
        
        Ok(items)
    }
}

