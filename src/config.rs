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

    /// Save a session key to the config file.
    pub fn save_session_key(key: &str) -> Result<()> {
        let dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("No config directory"))?
            .join("claude-usage-tui");
        fs::create_dir_all(&dir)?;
        let path = dir.join("config.toml");

        // Read existing or create new
        let mut content = if path.exists() {
            fs::read_to_string(&path)?
        } else {
            String::new()
        };

        // Replace or append session_key
        if content.contains("session_key") {
            // Replace existing line
            let lines: Vec<&str> = content.lines().collect();
            let new_lines: Vec<String> = lines
                .iter()
                .map(|l| {
                    if l.trim_start().starts_with("session_key") || l.trim_start().starts_with("# session_key") {
                        format!("session_key = \"{}\"", key)
                    } else {
                        l.to_string()
                    }
                })
                .collect();
            content = new_lines.join("\n") + "\n";
        } else {
            content.push_str(&format!("session_key = \"{}\"\n", key));
        }

        fs::write(&path, &content)?;
        eprintln!("Cookie saved to {}", path.display());
        Ok(())
    }
}
