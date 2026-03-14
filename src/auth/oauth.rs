use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const OAUTH_TOKEN_URL: &str = "https://platform.claude.com/v1/oauth/token";
const OAUTH_CLIENT_ID: &str = "9d1c250a-e61b-44d9-88ed-5944d1962f5e";

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct CredentialsFile {
    claude_ai_oauth: Option<OAuthToken>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct OAuthToken {
    access_token: Option<String>,
    refresh_token: Option<String>,
    expires_at: Option<i64>,
    subscription_type: Option<String>,
    rate_limit_tier: Option<String>,
    #[serde(default)]
    scopes: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RefreshResponse {
    access_token: String,
    expires_in: Option<i64>,
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

fn is_expired(token: &OAuthToken) -> bool {
    match token.expires_at {
        Some(expires_at) => {
            let now_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64;
            // Expired or within 5 minutes of expiry
            now_ms + 300_000 >= expires_at
        }
        None => false,
    }
}

/// Try to refresh the token using the refresh_token. Updates the credentials file on success.
fn try_refresh(token: &OAuthToken, creds_path: Option<&Path>) -> Result<String> {
    let refresh_token = token
        .refresh_token
        .as_deref()
        .ok_or_else(|| anyhow!("No refresh_token available"))?;

    eprintln!("Token expired, refreshing...");

    // Synchronous HTTP request for refresh (runs before tokio runtime in some paths)
    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(OAUTH_TOKEN_URL)
        .form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", OAUTH_CLIENT_ID),
        ])
        .send()?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        return Err(anyhow!("Token refresh failed: {} {}", status, body));
    }

    let refresh_resp: RefreshResponse = resp.json()?;

    // Update the credentials file with the new access_token and expiry
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;
    let new_expires_at = refresh_resp
        .expires_in
        .map(|secs| now_ms + secs * 1000)
        .or(token.expires_at);

    let mut updated_token = token.clone();
    updated_token.access_token = Some(refresh_resp.access_token.clone());
    updated_token.expires_at = new_expires_at;

    let updated_file = CredentialsFile {
        claude_ai_oauth: Some(updated_token),
    };
    if let Some(path) = creds_path {
        if let Ok(json) = serde_json::to_string_pretty(&updated_file) {
            let _ = fs::write(path, json);
        }
    }

    eprintln!("Token refreshed successfully.");
    Ok(refresh_resp.access_token)
}

fn load_credentials_from(path: &Path) -> Result<(String, String)> {
    let contents = fs::read_to_string(path)?;
    let file: CredentialsFile = serde_json::from_str(&contents)?;

    let token = file
        .claude_ai_oauth
        .ok_or_else(|| anyhow!("No claudeAiOauth in credentials"))?;

    let plan_name = plan_name_from_credentials(&token);

    // If token is expired, try to refresh
    let access_token = if is_expired(&token) {
        match try_refresh(&token, Some(path)) {
            Ok(new_token) => new_token,
            Err(e) => {
                // Refresh failed — still try the old token, API might accept it
                eprintln!("Warning: Token refresh failed ({}), trying expired token...", e);
                token
                    .access_token
                    .ok_or_else(|| anyhow!("No accessToken in credentials"))?
            }
        }
    } else {
        token
            .access_token
            .ok_or_else(|| anyhow!("No accessToken in credentials"))?
    };

    Ok((access_token, plan_name))
}

pub fn load_oauth_credentials() -> Result<(String, String)> {
    // 1. Try credential files on disk
    let candidates: Vec<PathBuf> = [
        dirs::config_dir().map(|d| d.join("claude-usage-tui").join("credentials.json")),
        dirs::home_dir().map(|d| d.join(".claude").join(".credentials.json")),
    ]
    .into_iter()
    .flatten()
    .collect();

    for path in &candidates {
        if path.exists() {
            match load_credentials_from(path) {
                Ok(result) => return Ok(result),
                Err(e) => {
                    eprintln!("Skipping {:?}: {}", path, e);
                }
            }
        }
    }

    // 2. Try macOS Keychain (Claude Code v2.1.52+)
    match load_from_keychain() {
        Ok(result) => return Ok(result),
        Err(e) => {
            eprintln!("Keychain: {}", e);
        }
    }

    Err(anyhow!(
        "No valid OAuth credentials found.\n\
         Tried: credential files, macOS Keychain.\n\
         Run `claude-usage-tui login` or pass `--cookie <sessionKey>`."
    ))
}

/// Read Claude Code credentials from macOS Keychain
fn load_from_keychain() -> Result<(String, String)> {
    let output = std::process::Command::new("security")
        .args(["find-generic-password", "-s", "Claude Code-credentials", "-w"])
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("No Claude Code credentials in Keychain"));
    }

    let json_str = String::from_utf8(output.stdout)?;
    let file: CredentialsFile = serde_json::from_str(json_str.trim())?;

    let token = file
        .claude_ai_oauth
        .ok_or_else(|| anyhow!("No claudeAiOauth in Keychain credentials"))?;

    let plan_name = plan_name_from_credentials(&token);

    let access_token = if is_expired(&token) {
        match try_refresh(&token, None) {
            Ok(new_token) => new_token,
            Err(e) => {
                eprintln!("Warning: Token refresh failed ({}), trying expired token...", e);
                token.access_token.ok_or_else(|| anyhow!("No accessToken"))?
            }
        }
    } else {
        token.access_token.ok_or_else(|| anyhow!("No accessToken"))?
    };

    Ok((access_token, plan_name))
}
