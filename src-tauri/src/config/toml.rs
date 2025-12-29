use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use anyhow::{Result, Context};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub github: GitHubConfig,
    #[serde(default)]
    pub rss: Vec<RssSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    #[serde(default = "default_poll_interval")]
    pub poll_interval: String,
    #[serde(default)]
    pub repos: Vec<GitHubRepo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRepo {
    pub owner: String,
    pub repo: String,
    #[serde(default)]
    pub assigned_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RssSource {
    pub name: String,
    pub url: String,
    #[serde(default = "default_poll_interval")]
    pub poll_interval: String,
}

fn default_poll_interval() -> String {
    "10m".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Config {
            github: GitHubConfig {
                poll_interval: "5m".to_string(),
                repos: vec![],
            },
            rss: vec![],
        }
    }
}

impl Default for GitHubConfig {
    fn default() -> Self {
        GitHubConfig {
            poll_interval: "5m".to_string(),
            repos: vec![],
        }
    }
}

pub fn get_config_path(app_data_dir: &PathBuf) -> PathBuf {
    app_data_dir.join("umbrarelay.toml")
}

pub fn load_config(app_data_dir: &PathBuf) -> Result<Config> {
    let config_path = get_config_path(app_data_dir);
    
    if !config_path.exists() {
        // Return default config if file doesn't exist
        return Ok(Config::default());
    }

    let content = fs::read_to_string(&config_path)
        .context("Failed to read config file")?;
    
    let config: Config = toml::from_str(&content)
        .context("Failed to parse config file")?;
    
    Ok(config)
}

pub fn save_config(app_data_dir: &PathBuf, config: &Config) -> Result<()> {
    let config_path = get_config_path(app_data_dir);
    
    // Ensure directory exists
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .context("Failed to create config directory")?;
    }
    
    let content = toml::to_string_pretty(config)
        .context("Failed to serialize config")?;
    
    fs::write(&config_path, content)
        .context("Failed to write config file")?;
    
    Ok(())
}

pub fn parse_duration(duration_str: &str) -> Result<u64> {
    // Parse duration strings like "5m", "10m", "1h"
    let duration_str = duration_str.trim();
    
    if duration_str.is_empty() {
        return Ok(600); // Default 10 minutes
    }
    
    let (num_str, unit) = if duration_str.ends_with('m') {
        (&duration_str[..duration_str.len() - 1], "m")
    } else if duration_str.ends_with('h') {
        (&duration_str[..duration_str.len() - 1], "h")
    } else if duration_str.ends_with('s') {
        (&duration_str[..duration_str.len() - 1], "s")
    } else {
        return Err(anyhow::anyhow!("Invalid duration format: {}", duration_str));
    };
    
    let num: u64 = num_str.parse()
        .context("Failed to parse duration number")?;
    
    let seconds = match unit {
        "s" => num,
        "m" => num * 60,
        "h" => num * 3600,
        _ => return Err(anyhow::anyhow!("Invalid duration unit: {}", unit)),
    };
    
    Ok(seconds)
}

