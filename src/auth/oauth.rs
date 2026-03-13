use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
struct Credentials {
    #[serde(rename = "accessToken")]
    access_token: Option<String>,
    #[serde(rename = "expiresAt")]
    expires_at: Option<i64>,
    #[serde(rename = "subscriptionType")]
    subscription_type: Option<String>,
    #[serde(rename = "rateLimitTier")]
    rate_limit_tier: Option<String>,
}

fn plan_name_from_credentials(creds: &Credentials) -> String {
    match (
        creds.subscription_type.as_deref(),
        creds.rate_limit_tier.as_deref(),
    ) {
        (Some("claude_pro"), _) => "Pro".to_string(),
        (Some("claude_team"), _) => "Team".to_string(),
        (Some("claude_max"), Some(tier)) => format!("Max ({})", tier),
        (Some("claude_max"), None) => "Max".to_string(),
        (_, Some(tier)) => tier.to_string(),
        _ => "Unknown".to_string(),
    }
}

fn load_credentials_from(path: &std::path::Path) -> Result<(String, String)> {
    let contents = fs::read_to_string(path)?;
    let creds: Credentials = serde_json::from_str(&contents)?;

    let access_token = creds
        .access_token
        .as_deref()
        .ok_or_else(|| anyhow!("No accessToken in credentials"))?
        .to_string();

    // Check expiry
    if let Some(expires_at) = creds.expires_at {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        if expires_at < now {
            return Err(anyhow!("OAuth token has expired"));
        }
    }

    let plan_name = plan_name_from_credentials(&creds);
    Ok((access_token, plan_name))
}

pub fn load_oauth_credentials() -> Result<(String, String)> {
    let candidates = [
        dirs::config_dir().map(|d| d.join("claude-usage-tui").join("credentials.json")),
        dirs::home_dir().map(|d| d.join(".claude").join(".credentials.json")),
    ];

    for candidate in &candidates {
        if let Some(path) = candidate {
            if path.exists() {
                match load_credentials_from(path) {
                    Ok(result) => return Ok(result),
                    Err(e) => {
                        eprintln!("Skipping {:?}: {}", path, e);
                    }
                }
            }
        }
    }

    Err(anyhow!(
        "No valid OAuth credentials found. Tried ~/.config/claude-usage-tui/credentials.json and ~/.claude/.credentials.json"
    ))
}
