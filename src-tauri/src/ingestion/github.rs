use super::traits::{IngestSource, IngestedItem};
use anyhow::{Result, Context};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
struct GitHubCommit {
    sha: String,
    commit: GitHubCommitDetails,
    html_url: String,
    author: Option<GitHubUser>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GitHubCommitDetails {
    message: String,
    author: GitHubCommitAuthor,
}

#[derive(Debug, Serialize, Deserialize)]
struct GitHubCommitAuthor {
    name: String,
    date: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GitHubUser {
    login: String,
    html_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GitHubPullRequest {
    id: u64,
    number: u64,
    title: String,
    body: Option<String>,
    html_url: String,
    state: String,
    created_at: String,
    updated_at: String,
    user: GitHubUser,
}

pub struct GitHubIngester {
    token: String,
    repositories: Vec<String>, // Format: "owner/repo"
    endpoints: Vec<String>, // "user", "repository", "commits", "prs"
    client: Client,
}

impl GitHubIngester {
    pub fn new(_secret_id: i64, token: String, repositories: Vec<String>, endpoints: Vec<String>) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;
        
        Ok(GitHubIngester {
            token,
            repositories,
            endpoints,
            client,
        })
    }

    fn make_request(&self, url: &str) -> Result<reqwest::blocking::Response> {
        // GitHub API accepts both "token" and "Bearer" format, but Bearer is more standard
        let response = self.client
            .get(url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "UmbraRelay")
            .send()
            .context("Failed to send GitHub API request")?;
        
        // Check for 401 Unauthorized - token expired
        if response.status() == 401 {
            return Err(anyhow::anyhow!("GitHub API returned 401 Unauthorized - token may be expired"));
        }
        
        // Check for 403 Forbidden - might be missing scope or insufficient permissions
        if response.status() == 403 {
            let error_text = response.text().unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow::anyhow!(
                "GitHub API returned 403 Forbidden - token may be missing required scope or have insufficient permissions. Error: {}",
                error_text
            ));
        }
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "GitHub API error: {}",
                response.status()
            ));
        }
        
        Ok(response)
    }
    
    // Helper to make a request that may return 404/410 (feature not available)
    fn make_request_optional(&self, url: &str) -> Result<Option<reqwest::blocking::Response>> {
        let response = self.client
            .get(url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "UmbraRelay")
            .send()
            .context("Failed to send GitHub API request")?;
        
        // Check for 401 Unauthorized - token expired
        if response.status() == 401 {
            return Err(anyhow::anyhow!("GitHub API returned 401 Unauthorized - token may be expired"));
        }
        
        // 404/410 means feature not available - return None
        if response.status() == 404 || response.status() == 410 {
            return Ok(None);
        }
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "GitHub API error: {}",
                response.status()
            ));
        }
        
        Ok(Some(response))
    }

    fn fetch_commits(&self, repo: &str) -> Result<Vec<IngestedItem>> {
        let mut all_items = Vec::new();
        let mut page = 1;
        let per_page = 100;
        
        // Fetch commits from the last 7 days
        let since = chrono::Utc::now() - chrono::Duration::days(7);
        let since_str = since.format("%Y-%m-%dT%H:%M:%SZ").to_string();
        
        loop {
            let url = format!(
                "https://api.github.com/repos/{}/commits?since={}&per_page={}&page={}",
                repo, since_str, per_page, page
            );
            
            let response = self.make_request(&url)?;
            let commits: Vec<GitHubCommit> = response.json()
                .context("Failed to parse commits response")?;
            
            let commits_len = commits.len();
            if commits_len == 0 {
                break;
            }
            
            for commit in commits {
                let item = self.commit_to_item(commit, repo)?;
                all_items.push(item);
            }
            
            if commits_len < per_page {
                break;
            }
            
            page += 1;
        }
        
        Ok(all_items)
    }

    fn fetch_pull_requests(&self, repo: &str) -> Result<Vec<IngestedItem>> {
        let mut all_items = Vec::new();
        let mut page = 1;
        let per_page = 100;
        
        loop {
            let url = format!(
                "https://api.github.com/repos/{}/pulls?state=open&per_page={}&page={}",
                repo, per_page, page
            );
            
            let response = self.make_request(&url)?;
            let prs: Vec<GitHubPullRequest> = response.json()
                .context("Failed to parse pull requests response")?;
            
            let prs_len = prs.len();
            if prs_len == 0 {
                break;
            }
            
            for pr in prs {
                let item = self.pr_to_item(pr, repo)?;
                all_items.push(item);
            }
            
            if prs_len < per_page {
                break;
            }
            
            page += 1;
        }
        
        Ok(all_items)
    }

    fn commit_to_item(&self, commit: GitHubCommit, repo: &str) -> Result<IngestedItem> {
        let external_id = format!("{}/commit/{}", repo, commit.sha);
        
        let message_lines: Vec<&str> = commit.commit.message.lines().collect();
        let title = message_lines.first().map(|s| s.to_string())
            .unwrap_or_else(|| commit.sha.clone());
        let summary = if message_lines.len() > 1 {
            Some(message_lines[1..].join("\n"))
        } else {
            None
        };
        
        let occurred_at = chrono::DateTime::parse_from_rfc3339(&commit.commit.author.date)
            .ok()
            .map(|dt| dt.timestamp());
        
        let author = commit.author.as_ref().map(|a| a.login.clone());
        
        Ok(IngestedItem {
            external_id,
            title,
            summary,
            url: commit.html_url,
            item_type: "commit".to_string(),
            occurred_at,
            image_url: None,
            content_html: None,
            author,
            category: Some(vec![repo.to_string()]),
            comments: None,
            thread_id: None,
        })
    }

    fn pr_to_item(&self, pr: GitHubPullRequest, repo: &str) -> Result<IngestedItem> {
        let external_id = format!("{}/pull/{}", repo, pr.number);
        
        let summary = pr.body.as_ref().and_then(|b| {
            if b.len() > 500 {
                Some(format!("{}...", &b[..500]))
            } else {
                Some(b.clone())
            }
        });
        
        let occurred_at = chrono::DateTime::parse_from_rfc3339(&pr.updated_at)
            .ok()
            .map(|dt| dt.timestamp());
        
        Ok(IngestedItem {
            external_id,
            title: pr.title,
            summary,
            url: pr.html_url,
            item_type: "pr".to_string(),
            occurred_at,
            image_url: None,
            content_html: None,
            author: Some(pr.user.login),
            category: Some(vec![repo.to_string()]),
            comments: None,
            thread_id: None,
        })
    }
    
    // Fetch account-level events (filtered to subscribed repos only)
    fn fetch_events(&self) -> Result<Vec<IngestedItem>> {
        let mut all_items = Vec::new();
        
        // Instead of fetching public events, fetch events for each subscribed repository
        // This ensures we only get events from repos the user is subscribed to
        for repo in &self.repositories {
            let mut page = 1;
            let per_page = 100;
            
            loop {
                let url = format!(
                    "https://api.github.com/repos/{}/events?per_page={}&page={}",
                    repo, per_page, page
                );
                
                let response = match self.make_request_optional(&url)? {
                    Some(r) => r,
                    None => break, // Repo not found or no access
                };
                
                let events: Vec<serde_json::Value> = response.json()
                    .context("Failed to parse events response")?;
                
                let events_len = events.len();
                if events_len == 0 {
                    break;
                }
                
                for event in &events {
                    if let Some(item) = self.event_to_item(event.clone(), repo)? {
                        all_items.push(item);
                    }
                }
                
                if events_len < per_page {
                    break;
                }
                
                page += 1;
            }
        }
        
        Ok(all_items)
    }
    
    fn event_to_item(&self, event: serde_json::Value, repo: &str) -> Result<Option<IngestedItem>> {
        let event_type = event.get("type").and_then(|v| v.as_str()).unwrap_or("unknown");
        
        let external_id = format!("{}/event/{}", repo, event.get("id").and_then(|v| v.as_str()).unwrap_or(""));
        let title = format!("{}: {}", event_type, repo);
        let url = format!("https://github.com/{}", repo);
        
        let occurred_at = event.get("created_at")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.timestamp());
        
        Ok(Some(IngestedItem {
            external_id,
            title,
            summary: Some(format!("Event type: {}", event_type)),
            url,
            item_type: "event".to_string(),
            occurred_at,
            image_url: None,
            content_html: None,
            author: event.get("actor")
                .and_then(|a| a.get("login"))
                .and_then(|l| l.as_str())
                .map(|s| s.to_string()),
            category: Some(vec![repo.to_string()]),
            comments: None,
            thread_id: None,
        }))
    }
    
    // Fetch issues
    fn fetch_issues(&self, repo: &str) -> Result<Vec<IngestedItem>> {
        let mut all_items = Vec::new();
        let mut page = 1;
        let per_page = 100;
        
        loop {
            let url = format!(
                "https://api.github.com/repos/{}/issues?state=all&per_page={}&page={}",
                repo, per_page, page
            );
            
            let response = self.make_request(&url)?;
            let issues: Vec<serde_json::Value> = response.json()
                .context("Failed to parse issues response")?;
            
            let issues_len = issues.len();
            if issues_len == 0 {
                break;
            }
            
            for issue in &issues {
                // Skip pull requests (they're included in issues but we handle them separately)
                if issue.get("pull_request").is_some() {
                    continue;
                }
                
                if let Some(item) = self.issue_to_item(issue.clone(), repo)? {
                    all_items.push(item);
                }
            }
            
            if issues_len < per_page {
                break;
            }
            
            page += 1;
        }
        
        Ok(all_items)
    }
    
    fn issue_to_item(&self, issue: serde_json::Value, repo: &str) -> Result<Option<IngestedItem>> {
        let number = issue.get("number").and_then(|v| v.as_u64()).unwrap_or(0);
        let external_id = format!("{}/issues/{}", repo, number);
        let title = issue.get("title").and_then(|v| v.as_str()).unwrap_or("Untitled").to_string();
        let url = issue.get("html_url").and_then(|v| v.as_str()).unwrap_or("").to_string();
        
        let summary = issue.get("body")
            .and_then(|v| v.as_str())
            .map(|b| {
                if b.len() > 500 {
                    format!("{}...", &b[..500])
                } else {
                    b.to_string()
                }
            });
        
        let occurred_at = issue.get("updated_at")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.timestamp());
        
        Ok(Some(IngestedItem {
            external_id,
            title,
            summary,
            url,
            item_type: "issue".to_string(),
            occurred_at,
            image_url: None,
            content_html: None,
            author: issue.get("user")
                .and_then(|u| u.get("login"))
                .and_then(|l| l.as_str())
                .map(|s| s.to_string()),
            category: Some(vec![repo.to_string()]),
            comments: None,
            thread_id: None,
        }))
    }
    
    // Fetch actions (workflow runs)
    fn fetch_actions(&self, repo: &str) -> Result<Vec<IngestedItem>> {
        let mut all_items = Vec::new();
        let mut page = 1;
        let per_page = 100;
        
        loop {
            let url = format!(
                "https://api.github.com/repos/{}/actions/runs?per_page={}&page={}",
                repo, per_page, page
            );
            
            let response = self.make_request(&url)?;
            let data: serde_json::Value = response.json()
                .context("Failed to parse actions response")?;
            
            let runs = data.get("workflow_runs")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_else(|| vec![]);
            
            let runs_len = runs.len();
            if runs_len == 0 {
                break;
            }
            
            for run in runs {
                if let Some(item) = self.action_to_item(run, repo)? {
                    all_items.push(item);
                }
            }
            
            if runs_len < per_page {
                break;
            }
            
            page += 1;
        }
        
        Ok(all_items)
    }
    
    fn action_to_item(&self, run: serde_json::Value, repo: &str) -> Result<Option<IngestedItem>> {
        let id = run.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
        let external_id = format!("{}/actions/runs/{}", repo, id);
        let name = run.get("name").and_then(|v| v.as_str()).unwrap_or("Workflow Run").to_string();
        let url = run.get("html_url").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let status = run.get("status").and_then(|v| v.as_str()).unwrap_or("unknown");
        let conclusion = run.get("conclusion").and_then(|v| v.as_str()).unwrap_or("unknown");
        
        let title = format!("{} - {} ({})", name, status, conclusion);
        
        let occurred_at = run.get("updated_at")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.timestamp());
        
        Ok(Some(IngestedItem {
            external_id,
            title,
            summary: Some(format!("Status: {}, Conclusion: {}", status, conclusion)),
            url,
            item_type: "action".to_string(),
            occurred_at,
            image_url: None,
            content_html: None,
            author: run.get("actor")
                .and_then(|a| a.get("login"))
                .and_then(|l| l.as_str())
                .map(|s| s.to_string()),
            category: Some(vec![repo.to_string()]),
            comments: None,
            thread_id: None,
        }))
    }
    
    // Fetch contents (recent commits serve as content changes)
    fn fetch_contents(&self, repo: &str) -> Result<Vec<IngestedItem>> {
        // Use commits as a proxy for content changes
        self.fetch_commits(repo)
    }
    
    // Fetch discussions
    fn fetch_discussions(&self, repo: &str) -> Result<Vec<IngestedItem>> {
        let mut all_items = Vec::new();
        let mut page = 1;
        let per_page = 100;
        
        loop {
            let url = format!(
                "https://api.github.com/repos/{}/discussions?per_page={}&page={}",
                repo, per_page, page
            );
            
            let response = match self.make_request_optional(&url)? {
                Some(r) => r,
                None => return Ok(all_items), // Feature not available
            };
            let discussions: Vec<serde_json::Value> = response.json()
                .context("Failed to parse discussions response")?;
            
            if discussions.is_empty() {
                break;
            }
            
            for discussion in &discussions {
                if let Some(item) = self.discussion_to_item(discussion.clone(), repo)? {
                    all_items.push(item);
                }
            }
            
            if discussions.len() < per_page {
                break;
            }
            
            page += 1;
        }
        
        Ok(all_items)
    }
    
    fn discussion_to_item(&self, discussion: serde_json::Value, repo: &str) -> Result<Option<IngestedItem>> {
        let number = discussion.get("number").and_then(|v| v.as_u64()).unwrap_or(0);
        let external_id = format!("{}/discussions/{}", repo, number);
        let title = discussion.get("title").and_then(|v| v.as_str()).unwrap_or("Untitled").to_string();
        let url = discussion.get("html_url").and_then(|v| v.as_str()).unwrap_or("").to_string();
        
        let summary = discussion.get("body")
            .and_then(|v| v.as_str())
            .map(|b| {
                if b.len() > 500 {
                    format!("{}...", &b[..500])
                } else {
                    b.to_string()
                }
            });
        
        let occurred_at = discussion.get("updated_at")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.timestamp());
        
        Ok(Some(IngestedItem {
            external_id,
            title,
            summary,
            url,
            item_type: "discussion".to_string(),
            occurred_at,
            image_url: None,
            content_html: None,
            author: discussion.get("user")
                .and_then(|u| u.get("login"))
                .and_then(|l| l.as_str())
                .map(|s| s.to_string()),
            category: Some(vec![repo.to_string()]),
            comments: None,
            thread_id: None,
        }))
    }
    
    // Fetch code scanning alerts
    fn fetch_code_scanning_alerts(&self, repo: &str) -> Result<Vec<IngestedItem>> {
        let mut all_items = Vec::new();
        let mut page = 1;
        let per_page = 100;
        
        loop {
            let url = format!(
                "https://api.github.com/repos/{}/code-scanning/alerts?per_page={}&page={}",
                repo, per_page, page
            );
            
            let response = match self.make_request_optional(&url)? {
                Some(r) => r,
                None => return Ok(all_items), // Feature not available
            };
            let alerts: Vec<serde_json::Value> = response.json()
                .context("Failed to parse code scanning alerts response")?;
            
            if alerts.is_empty() {
                break;
            }
            
            for alert in &alerts {
                if let Some(item) = self.code_scanning_alert_to_item(alert.clone(), repo)? {
                    all_items.push(item);
                }
            }
            
            if alerts.len() < per_page {
                break;
            }
            
            page += 1;
        }
        
        Ok(all_items)
    }
    
    fn code_scanning_alert_to_item(&self, alert: serde_json::Value, repo: &str) -> Result<Option<IngestedItem>> {
        let number = alert.get("number").and_then(|v| v.as_u64()).unwrap_or(0);
        let external_id = format!("{}/security/code-scanning/{}", repo, number);
        let rule = alert.get("rule")
            .and_then(|r| r.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("Unknown Rule");
        let title = format!("Code Scanning Alert: {}", rule);
        
        let url = format!("https://github.com/{}/security/code-scanning/{}", repo, number);
        
        let summary = alert.get("message")
            .and_then(|m| m.get("text"))
            .and_then(|t| t.as_str())
            .map(|s| s.to_string());
        
        let occurred_at = alert.get("created_at")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.timestamp());
        
        let state = alert.get("state").and_then(|v| v.as_str()).unwrap_or("unknown");
        
        Ok(Some(IngestedItem {
            external_id,
            title,
            summary: summary.or(Some(format!("State: {}", state))),
            url,
            item_type: "code_scanning_alert".to_string(),
            occurred_at,
            image_url: None,
            content_html: None,
            author: None,
            category: Some(vec![repo.to_string()]),
            comments: None,
            thread_id: None,
        }))
    }
    
    // Fetch checks (check runs) - simplified version
    fn fetch_checks(&self, repo: &str) -> Result<Vec<IngestedItem>> {
        // Get recent commits first, then fetch check runs for the latest commit
        let commits = self.fetch_commits(repo)?;
        let mut all_items = Vec::new();
        
        // Get check runs for the most recent commit
        if let Some(first_commit) = commits.first() {
            if let Some(sha) = first_commit.external_id.split('/').last() {
                let url = format!(
                    "https://api.github.com/repos/{}/commits/{}/check-runs",
                    repo, sha
                );
                
                if let Ok(response) = self.make_request(&url) {
                    if let Ok(data) = response.json::<serde_json::Value>() {
                        if let Some(check_runs) = data.get("check_runs").and_then(|v| v.as_array()) {
                            for run in check_runs {
                                if let Some(item) = self.check_to_item(run.clone(), repo, sha)? {
                                    all_items.push(item);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(all_items)
    }
    
    fn check_to_item(&self, run: serde_json::Value, repo: &str, sha: &str) -> Result<Option<IngestedItem>> {
        let id = run.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
        let external_id = format!("{}/check/{}", repo, id);
        let name = run.get("name").and_then(|v| v.as_str()).unwrap_or("Check Run").to_string();
        let url = run.get("html_url").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let status = run.get("status").and_then(|v| v.as_str()).unwrap_or("unknown");
        let conclusion = run.get("conclusion").and_then(|v| v.as_str()).unwrap_or("unknown");
        
        let title = format!("{} - {} ({})", name, status, conclusion);
        
        let occurred_at = run.get("completed_at")
            .or_else(|| run.get("started_at"))
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.timestamp());
        
        Ok(Some(IngestedItem {
            external_id,
            title,
            summary: Some(format!("Commit: {}, Status: {}, Conclusion: {}", &sha[..7.min(sha.len())], status, conclusion)),
            url,
            item_type: "check".to_string(),
            occurred_at,
            image_url: None,
            content_html: None,
            author: None,
            category: Some(vec![repo.to_string()]),
            comments: None,
            thread_id: None,
        }))
    }
    
    // Fetch packages
    fn fetch_packages(&self, repo: &str) -> Result<Vec<IngestedItem>> {
        let mut all_items = Vec::new();
        let mut page = 1;
        let per_page = 100;
        
        loop {
            let url = format!(
                "https://api.github.com/repos/{}/packages?per_page={}&page={}",
                repo, per_page, page
            );
            
            let response = self.make_request(&url)?;
            let packages: Vec<serde_json::Value> = response.json()
                .context("Failed to parse packages response")?;
            
            if packages.is_empty() {
                break;
            }
            
            for package in &packages {
                if let Some(item) = self.package_to_item(package.clone(), repo)? {
                    all_items.push(item);
                }
            }
            
            if packages.len() < per_page {
                break;
            }
            
            page += 1;
        }
        
        Ok(all_items)
    }
    
    fn package_to_item(&self, package: serde_json::Value, repo: &str) -> Result<Option<IngestedItem>> {
        let id = package.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
        let external_id = format!("{}/package/{}", repo, id);
        let name = package.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown Package").to_string();
        let package_type = package.get("package_type").and_then(|v| v.as_str()).unwrap_or("unknown");
        
        let title = format!("{} ({})", name, package_type);
        let url = format!("https://github.com/{}/packages", repo);
        
        let occurred_at = package.get("updated_at")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.timestamp());
        
        Ok(Some(IngestedItem {
            external_id,
            title,
            summary: Some(format!("Package type: {}", package_type)),
            url,
            item_type: "package".to_string(),
            occurred_at,
            image_url: None,
            content_html: None,
            author: None,
            category: Some(vec![repo.to_string()]),
            comments: None,
            thread_id: None,
        }))
    }
    
    // Fetch projects
    fn fetch_projects(&self, repo: &str) -> Result<Vec<IngestedItem>> {
        let mut all_items = Vec::new();
        let mut page = 1;
        let per_page = 100;
        
        loop {
            let url = format!(
                "https://api.github.com/repos/{}/projects?per_page={}&page={}",
                repo, per_page, page
            );
            
            let response = match self.make_request_optional(&url)? {
                Some(r) => r,
                None => return Ok(all_items), // Feature not available
            };
            let projects: Vec<serde_json::Value> = response.json()
                .context("Failed to parse projects response")?;
            
            if projects.is_empty() {
                break;
            }
            
            for project in &projects {
                if let Some(item) = self.project_to_item(project.clone(), repo)? {
                    all_items.push(item);
                }
            }
            
            if projects.len() < per_page {
                break;
            }
            
            page += 1;
        }
        
        Ok(all_items)
    }
    
    fn project_to_item(&self, project: serde_json::Value, repo: &str) -> Result<Option<IngestedItem>> {
        let id = project.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
        let external_id = format!("{}/project/{}", repo, id);
        let name = project.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown Project").to_string();
        let url = project.get("html_url").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let body = project.get("body").and_then(|v| v.as_str());
        
        let summary = body.map(|b| {
            if b.len() > 500 {
                format!("{}...", &b[..500])
            } else {
                b.to_string()
            }
        });
        
        let occurred_at = project.get("updated_at")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.timestamp());
        
        Ok(Some(IngestedItem {
            external_id,
            title: name,
            summary,
            url,
            item_type: "project".to_string(),
            occurred_at,
            image_url: None,
            content_html: None,
            author: None,
            category: Some(vec![repo.to_string()]),
            comments: None,
            thread_id: None,
        }))
    }
    
    // Fetch administration (repository events)
    fn fetch_administration(&self, repo: &str) -> Result<Vec<IngestedItem>> {
        let mut all_items = Vec::new();
        let mut page = 1;
        let per_page = 100;
        
        loop {
            let url = format!(
                "https://api.github.com/repos/{}/events?per_page={}&page={}",
                repo, per_page, page
            );
            
            let response = self.make_request(&url)?;
            let events: Vec<serde_json::Value> = response.json()
                .context("Failed to parse repository events response")?;
            
            if events.is_empty() {
                break;
            }
            
            for event in &events {
                if let Some(item) = self.repo_event_to_item(event.clone(), repo)? {
                    all_items.push(item);
                }
            }
            
            if events.len() < per_page {
                break;
            }
            
            page += 1;
        }
        
        Ok(all_items)
    }
    
    fn repo_event_to_item(&self, event: serde_json::Value, repo: &str) -> Result<Option<IngestedItem>> {
        let id = event.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let external_id = format!("{}/event/{}", repo, id);
        let event_type = event.get("type").and_then(|v| v.as_str()).unwrap_or("unknown");
        let title = format!("{}: {}", repo, event_type);
        
        let url = format!("https://github.com/{}", repo);
        
        let occurred_at = event.get("created_at")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.timestamp());
        
        Ok(Some(IngestedItem {
            external_id,
            title,
            summary: Some(format!("Event type: {}", event_type)),
            url,
            item_type: "administration".to_string(),
            occurred_at,
            image_url: None,
            content_html: None,
            author: event.get("actor")
                .and_then(|a| a.get("login"))
                .and_then(|l| l.as_str())
                .map(|s| s.to_string()),
            category: Some(vec![repo.to_string()]),
            comments: None,
            thread_id: None,
        }))
    }
}

impl IngestSource for GitHubIngester {
    fn poll(&self) -> Result<Vec<IngestedItem>> {
        let mut all_items = Vec::new();
        
        // Fetch account-level events (filtered to subscribed repos)
        if self.endpoints.contains(&"events".to_string()) {
            match self.fetch_events() {
                Ok(items) => all_items.extend(items),
                Err(e) => {
                    eprintln!("Failed to fetch events: {}", e);
                }
            }
        }
        
        // Fetch repository-specific data for each repository
        for repo in &self.repositories {
            // Commits
            if self.endpoints.contains(&"commits".to_string()) {
                match self.fetch_commits(repo) {
                    Ok(items) => all_items.extend(items),
                    Err(e) => {
                        eprintln!("Failed to fetch commits for {}: {}", repo, e);
                    }
                }
            }
            
            // Pull requests
            if self.endpoints.contains(&"prs".to_string()) {
                match self.fetch_pull_requests(repo) {
                    Ok(items) => all_items.extend(items),
                    Err(e) => {
                        eprintln!("Failed to fetch pull requests for {}: {}", repo, e);
                    }
                }
            }
            
            // Issues
            if self.endpoints.contains(&"issues".to_string()) {
                match self.fetch_issues(repo) {
                    Ok(items) => all_items.extend(items),
                    Err(e) => {
                        eprintln!("Failed to fetch issues for {}: {}", repo, e);
                    }
                }
            }
            
            // Actions (workflow runs)
            if self.endpoints.contains(&"actions".to_string()) {
                match self.fetch_actions(repo) {
                    Ok(items) => all_items.extend(items),
                    Err(e) => {
                        eprintln!("Failed to fetch actions for {}: {}", repo, e);
                    }
                }
            }
            
            // Contents (recent file changes)
            if self.endpoints.contains(&"contents".to_string()) {
                match self.fetch_contents(repo) {
                    Ok(items) => all_items.extend(items),
                    Err(e) => {
                        eprintln!("Failed to fetch contents for {}: {}", repo, e);
                    }
                }
            }
            
            // Discussions
            if self.endpoints.contains(&"discussions".to_string()) {
                match self.fetch_discussions(repo) {
                    Ok(items) => all_items.extend(items),
                    Err(e) => {
                        // Only log as error if it's not a "feature not available" case
                        let error_msg = e.to_string();
                        if !error_msg.contains("404") && !error_msg.contains("410") {
                            eprintln!("Failed to fetch discussions for {}: {}", repo, e);
                        }
                    }
                }
            }
            
            // Code Scanning Alerts
            if self.endpoints.contains(&"code_scanning_alerts".to_string()) {
                match self.fetch_code_scanning_alerts(repo) {
                    Ok(items) => all_items.extend(items),
                    Err(e) => {
                        // Only log as error if it's not a "feature not available" case
                        let error_msg = e.to_string();
                        if !error_msg.contains("404") && !error_msg.contains("410") {
                            eprintln!("Failed to fetch code scanning alerts for {}: {}", repo, e);
                        }
                    }
                }
            }
            
            // Checks (check runs)
            if self.endpoints.contains(&"checks".to_string()) {
                match self.fetch_checks(repo) {
                    Ok(items) => all_items.extend(items),
                    Err(e) => {
                        eprintln!("Failed to fetch checks for {}: {}", repo, e);
                    }
                }
            }
            
            // Packages
            if self.endpoints.contains(&"packages".to_string()) {
                match self.fetch_packages(repo) {
                    Ok(items) => all_items.extend(items),
                    Err(e) => {
                        // Only log as error if it's not a "feature not available" case
                        let error_msg = e.to_string();
                        if !error_msg.contains("404") && !error_msg.contains("410") {
                            eprintln!("Failed to fetch packages for {}: {}", repo, e);
                        }
                    }
                }
            }
            
            // Projects
            if self.endpoints.contains(&"projects".to_string()) {
                match self.fetch_projects(repo) {
                    Ok(items) => all_items.extend(items),
                    Err(e) => {
                        // Only log as error if it's not a "feature not available" case
                        let error_msg = e.to_string();
                        if !error_msg.contains("404") && !error_msg.contains("410") {
                            eprintln!("Failed to fetch projects for {}: {}", repo, e);
                        }
                    }
                }
            }
            
            // Administration (repository events)
            if self.endpoints.contains(&"administration".to_string()) {
                match self.fetch_administration(repo) {
                    Ok(items) => all_items.extend(items),
                    Err(e) => {
                        eprintln!("Failed to fetch administration events for {}: {}", repo, e);
                    }
                }
            }
            
            // Metadata is always included, no separate fetch needed
        }
        
        Ok(all_items)
    }
}
