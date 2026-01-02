use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use reqwest::blocking::Client;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    #[serde(default)]
    pub verification_uri_complete: Option<String>,
    pub expires_in: u64,
    pub interval: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    scope: String,
    #[serde(default)]
    refresh_token: Option<String>,
    #[serde(default)]
    expires_in: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenErrorResponse {
    error: String,
    error_description: Option<String>,
    error_uri: Option<String>,
    #[serde(default)]
    interval: Option<u64>,
}

#[derive(Debug)]
pub enum PollResult {
    Pending,
    SlowDown { new_interval: u64 },
    Success(TokenPair),
    Error(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubRepository {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub private: bool,
    pub html_url: String,
}

#[derive(Debug)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: Option<String>,
}

pub struct GitHubOAuth {
    client_id: String,
    client: Client,
}

impl GitHubOAuth {
    pub fn new(client_id: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            client_id,
            client,
        }
    }

    /// Start device flow - get device code and user code
    pub fn start_device_flow(&self) -> Result<DeviceCodeResponse> {
        let mut params = HashMap::new();
        params.insert("client_id", self.client_id.as_str());
        // Request all scopes needed for the endpoints we support:
        // - repo: Full repository access (commits, PRs, issues, contents, actions, checks, etc.)
        // - read:org: Organization membership and events
        // - read:user: User profile and account-level events
        // - read:packages: Package access
        // - read:project: Project access
        // - read:discussion: Discussion access
        params.insert("scope", "repo read:org read:user read:packages read:project read:discussion");
        
        let response = self.client
            .post("https://github.com/login/device/code")
            .header("Accept", "application/json")
            .form(&params)
            .send()
            .context("Failed to send device code request")?;
        
        let status = response.status();
        
        if status == 404 {
            // Return a generic error that will be converted to user-friendly message
            return Err(anyhow::anyhow!("Device Flow not available (404)"));
        }
        
        if !status.is_success() {
            let error_text = response.text().unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow::anyhow!(
                "Device code request failed: {} - {}",
                status,
                error_text
            ));
        }
        
        // GitHub returns JSON when Accept: application/json is set
        let text = response.text()
            .context("Failed to read device code response")?;
        
        // Try JSON parsing first
        match serde_json::from_str::<DeviceCodeResponse>(&text) {
            Ok(device_response) => return Ok(device_response),
            Err(_) => {
                // Fall back to form-encoded parsing
            }
        }
        
        // Fall back to form-encoded parsing
        let mut device_code = None;
        let mut user_code = None;
        let mut verification_uri = None;
        let mut verification_uri_complete = None;
        let mut expires_in = None;
        let mut interval = None;
        
        for pair in text.split('&') {
            let parts: Vec<&str> = pair.splitn(2, '=').collect();
            if parts.len() == 2 {
                let key = parts[0];
                let value = urlencoding::decode(parts[1])
                    .map(|v| v.to_string())
                    .unwrap_or_else(|_| parts[1].to_string());
                
                match key {
                    "device_code" => device_code = Some(value),
                    "user_code" => user_code = Some(value),
                    "verification_uri" => verification_uri = Some(value),
                    "verification_uri_complete" => verification_uri_complete = Some(value),
                    "expires_in" => expires_in = Some(value.parse::<u64>().unwrap_or(900)),
                    "interval" => interval = Some(value.parse::<u64>().unwrap_or(5)),
                    _ => {}
                }
            }
        }
        
        // Better error message with full response
        let device_code_val = device_code.ok_or_else(|| {
            anyhow::anyhow!(
                "Missing device_code in response. Response (first 500 chars): {}",
                text.chars().take(500).collect::<String>()
            )
        })?;
        let user_code_val = user_code.ok_or_else(|| {
            anyhow::anyhow!(
                "Missing user_code in response. Response (first 500 chars): {}",
                text.chars().take(500).collect::<String>()
            )
        })?;
        let verification_uri_val = verification_uri.unwrap_or_else(|| "https://github.com/login/device".to_string());
        
        let device_response = DeviceCodeResponse {
            device_code: device_code_val,
            user_code: user_code_val.clone(),
            verification_uri: verification_uri_val.clone(),
            verification_uri_complete: verification_uri_complete.or_else(|| {
                // Construct verification_uri_complete from verification_uri and user_code if not provided
                Some(format!("{}?user_code={}", verification_uri_val, user_code_val))
            }),
            expires_in: expires_in.unwrap_or(900),
            interval: interval.unwrap_or(5),
        };
        
        Ok(device_response)
    }

    /// Poll for access token using device code
    /// Returns PollResult indicating the status of the authorization
    pub fn poll_for_token(&self, device_code: &str) -> Result<PollResult> {
        let mut params = HashMap::new();
        params.insert("client_id", self.client_id.as_str());
        params.insert("device_code", device_code);
        params.insert("grant_type", "urn:ietf:params:oauth:grant-type:device_code");
        
        let response = self.client
            .post("https://github.com/login/oauth/access_token")
            .header("Accept", "application/json")
            .form(&params)
            .send()
            .context("Failed to send token poll request")?;
        
        let status = response.status();
        let text = response.text().context("Failed to read response")?;
        
        // Handle 400 status codes (error responses)
        if status == 400 {
            // Try to parse as JSON error response first
            match serde_json::from_str::<TokenErrorResponse>(&text) {
                Ok(error_response) => {
                    match error_response.error.as_str() {
                        "authorization_pending" => {
                            return Ok(PollResult::Pending);
                        }
                        "slow_down" => {
                            // GitHub adds 5 seconds to the interval when slow_down occurs
                            let new_interval = error_response.interval.unwrap_or(5) + 5;
                            return Ok(PollResult::SlowDown { new_interval });
                        }
                        "expired_token" => {
                            return Ok(PollResult::Error(
                                "Device code has expired. Please start a new authorization.".to_string()
                            ));
                        }
                        "incorrect_device_code" => {
                            return Ok(PollResult::Error(
                                "Invalid device code. Please start a new authorization.".to_string()
                            ));
                        }
                        "access_denied" => {
                            return Ok(PollResult::Error(
                                "Authorization was cancelled. Please try again.".to_string()
                            ));
                        }
                        "device_flow_disabled" => {
                            return Ok(PollResult::Error(
                                "Device flow is not enabled for this application.".to_string()
                            ));
                        }
                        "unsupported_grant_type" => {
                            return Ok(PollResult::Error(
                                "Unsupported grant type. This is an application error.".to_string()
                            ));
                        }
                        "incorrect_client_credentials" => {
                            return Ok(PollResult::Error(
                                "Invalid client credentials. This is an application error.".to_string()
                            ));
                        }
                        _ => {
                            let error_msg = error_response.error_description
                                .unwrap_or_else(|| error_response.error.clone());
                            return Ok(PollResult::Error(format!("Token request failed: {}", error_msg)));
                        }
                    }
                }
                Err(_) => {
                    // Fall back to string matching only if JSON parsing fails
                    if text.contains("authorization_pending") {
                        return Ok(PollResult::Pending);
                    }
                    if text.contains("slow_down") {
                        // Try to extract interval from form-encoded response
                        let mut interval = 5;
                        for pair in text.split('&') {
                            if let Some((key, value)) = pair.split_once('=') {
                                if key == "interval" {
                                    if let Ok(parsed) = value.parse::<u64>() {
                                        interval = parsed + 5; // Add 5 seconds for slow_down
                                    }
                                }
                            }
                        }
                        return Ok(PollResult::SlowDown { new_interval: interval });
                    }
                    return Ok(PollResult::Error(format!("Token request failed: {}", text)));
                }
            }
        }
        
        // If we get here, status should be 200 (success)
        // But GitHub might return error JSON even with 200 status, so check for errors first
        if let Ok(error_response) = serde_json::from_str::<TokenErrorResponse>(&text) {
            // This is actually an error response, handle it
            match error_response.error.as_str() {
                "authorization_pending" => {
                    return Ok(PollResult::Pending);
                }
                "slow_down" => {
                    let new_interval = error_response.interval.unwrap_or(5) + 5;
                    return Ok(PollResult::SlowDown { new_interval });
                }
                "expired_token" => {
                    return Ok(PollResult::Error(
                        "Device code has expired. Please start a new authorization.".to_string()
                    ));
                }
                "incorrect_device_code" => {
                    return Ok(PollResult::Error(
                        "Invalid device code. Please start a new authorization.".to_string()
                    ));
                }
                "access_denied" => {
                    return Ok(PollResult::Error(
                        "Authorization was cancelled. Please try again.".to_string()
                    ));
                }
                _ => {
                    let error_msg = error_response.error_description
                        .unwrap_or_else(|| error_response.error.clone());
                    return Ok(PollResult::Error(format!("Token request failed: {}", error_msg)));
                }
            }
        }
        
        if !status.is_success() {
            return Ok(PollResult::Error(format!(
                "Token request failed with status: {}",
                status
            )));
        }
        
        // GitHub may return JSON or form-encoded response
        // Text is already read above
        
        // Try JSON parsing first
        let token_response = match serde_json::from_str::<TokenResponse>(&text) {
            Ok(tr) => tr,
            Err(_) => {
                // Fall back to form-encoded parsing
                let mut access_token = None;
                let mut refresh_token = None;
                
                for pair in text.split('&') {
                    let parts: Vec<&str> = pair.splitn(2, '=').collect();
                    if parts.len() == 2 {
                        let key = parts[0];
                        let value = urlencoding::decode(parts[1])
                            .map(|v| v.to_string())
                            .unwrap_or_else(|_| parts[1].to_string());
                        
                        match key {
                            "access_token" => access_token = Some(value),
                            "refresh_token" => refresh_token = Some(value),
                            _ => {}
                        }
                    }
                }
                
                TokenResponse {
                    access_token: access_token.ok_or_else(|| {
                        anyhow::anyhow!(
                            "Missing access_token in response. Response (first 500 chars): {}",
                            text.chars().take(500).collect::<String>()
                        )
                    })?,
                    token_type: "bearer".to_string(),
                    scope: String::new(),
                    refresh_token,
                    expires_in: None,
                }
            }
        };
        
        Ok(PollResult::Success(TokenPair {
            access_token: token_response.access_token,
            refresh_token: token_response.refresh_token,
        }))
    }

    /// Fetch user's repositories
    pub fn get_repositories(&self, access_token: &str) -> Result<Vec<GitHubRepository>> {
        let mut all_repos = Vec::new();
        let mut page = 1;
        let per_page = 100;
        
        loop {
            let url = format!(
                "https://api.github.com/user/repos?type=all&per_page={}&page={}",
                per_page, page
            );
            
            let response = self.client
                .get(&url)
                .header("Authorization", format!("token {}", access_token))
                .header("Accept", "application/vnd.github.v3+json")
                .header("User-Agent", "UmbraRelay")
                .send()
                .context("Failed to fetch repositories")?;
            
            if !response.status().is_success() {
                return Err(anyhow::anyhow!(
                    "Failed to fetch repositories: {}",
                    response.status()
                ));
            }
            
            let repos: Vec<GitHubRepository> = response.json()
                .context("Failed to parse repositories response")?;
            
            let repos_len = repos.len();
            if repos_len == 0 {
                break;
            }
            
            all_repos.extend(repos);
            
            if repos_len < per_page {
                break;
            }
            
            page += 1;
        }
        
        Ok(all_repos)
    }

    /// Refresh access token using refresh token
    pub fn refresh_token(&self, refresh_token: &str) -> Result<TokenPair> {
        let mut params = HashMap::new();
        params.insert("client_id", self.client_id.as_str());
        params.insert("grant_type", "refresh_token");
        params.insert("refresh_token", refresh_token);
        
        let response = self.client
            .post("https://github.com/login/oauth/access_token")
            .header("Accept", "application/json")
            .form(&params)
            .send()
            .context("Failed to send refresh token request")?;
        
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow::anyhow!(
                "Refresh token request failed: {} - {}",
                status,
                error_text
            ));
        }
        
        let token_response: TokenResponse = response.json()
            .context("Failed to parse refresh token response")?;
        
        Ok(TokenPair {
            access_token: token_response.access_token,
            refresh_token: token_response.refresh_token,
        })
    }
}
