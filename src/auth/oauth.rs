use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CredentialsFile {
    claude_ai_oauth: Option<OAuthToken>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OAuthToken {
    access_token: Option<String>,
    expires_at: Option<i64>,
    subscription_type: Option<String>,
    rate_limit_tier: Option<String>,
}

fn plan_name_from_credentials(creds: &OAuthToken) -> String {
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
    let file: CredentialsFile = serde_json::from_str(&contents)?;

    let token = file
        .claude_ai_oauth
        .ok_or_else(|| anyhow!("No claudeAiOauth in credentials"))?;

    let access_token = token
        .access_token
        .as_deref()
        .ok_or_else(|| anyhow!("No accessToken in credentials"))?
        .to_string();

    // Check expiry (expiresAt is in milliseconds)
    if let Some(expires_at) = token.expires_at {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;
        if expires_at < now_ms {
            return Err(anyhow!("OAuth token has expired"));
        }
    }

    let plan_name = plan_name_from_credentials(&token);
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
