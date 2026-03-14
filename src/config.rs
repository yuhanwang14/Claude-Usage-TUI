use anyhow::Result;
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    #[serde(default = "default_refresh_interval")]
    pub refresh_interval: u64,
    pub session_key: Option<String>,
    pub org_id: Option<String>,
}

fn default_refresh_interval() -> u64 {
    30
}

impl Default for Config {
    fn default() -> Self {
        Self {
            refresh_interval: default_refresh_interval(),
            session_key: None,
            org_id: None,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = dirs::config_dir()
            .map(|d| d.join("claude-usage-tui").join("config.toml"));

        match config_path {
            Some(path) if path.exists() => {
                let contents = fs::read_to_string(&path)?;
                let config: Config = toml::from_str(&contents)?;
                Ok(config)
            }
            _ => Ok(Config::default()),
        }
    }
}
