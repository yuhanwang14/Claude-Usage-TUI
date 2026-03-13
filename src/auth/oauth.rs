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
    let sub = creds.subscription_type.as_deref().unwrap_or("pro");
    let tier = creds.rate_limit_tier.as_deref().unwrap_or("");

    let base = match sub {
        "max" | "claude_max" => "Max",
        "pro" | "claude_pro" => "Pro",
        "team" | "claude_team" => "Team",
        other => other,
    };

    // Extract multiplier from tier like "default_claude_max_5x"
    if let Some(pos) = tier.rfind('_') {
        let suffix = &tier[pos + 1..];
        if suffix.ends_with('x') {
            return format!("{base} {suffix}");
        }
    }
    base.to_string()
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

    // Don't reject expired tokens locally — let the API decide.
    // The server may still accept them, and local clock skew causes false rejections.
    // If truly expired, the API returns 401 and we show "offline" in the status bar.
    if let Some(expires_at) = token.expires_at {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;
        if expires_at < now_ms {
            eprintln!("Warning: OAuth token may be expired, trying anyway...");
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
