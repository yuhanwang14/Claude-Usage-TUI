use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

const OAUTH_CLIENT_ID: &str = "9d1c250a-e61b-44d9-88ed-5944d1962f5e";
const AUTHORIZE_URL: &str = "https://platform.claude.com/oauth/authorize";
const TOKEN_URL: &str = "https://platform.claude.com/v1/oauth/token";

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: Option<i64>,
}

/// Run the OAuth login flow: open browser, receive callback, exchange code, save credentials.
pub fn run_login() -> Result<()> {
    // 1. Bind to a random port
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    let redirect_uri = format!("http://127.0.0.1:{port}/callback");

    // 2. Build authorize URL
    let auth_url = format!(
        "{AUTHORIZE_URL}?response_type=code&client_id={OAUTH_CLIENT_ID}&redirect_uri={redirect_uri}&scope=user:inference+user:profile"
    );

    println!("Opening browser for authentication...");
    println!("If the browser doesn't open, visit:\n{auth_url}\n");

    // 3. Open browser
    if let Err(e) = open::that(&auth_url) {
        eprintln!("Failed to open browser: {e}");
        println!("Please open the URL above manually.");
    }

    println!("Waiting for authentication callback...");

    // 4. Wait for the callback
    let (mut stream, _) = listener.accept().context("Failed to accept callback connection")?;
    let mut reader = BufReader::new(&stream);
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;

    // Parse the auth code from "GET /callback?code=XXX HTTP/1.1"
    let code = request_line
        .split_whitespace()
        .nth(1) // the path
        .and_then(|path| {
            path.split('?')
                .nth(1)
                .and_then(|query| {
                    query.split('&').find_map(|param| {
                        let mut kv = param.splitn(2, '=');
                        match (kv.next(), kv.next()) {
                            (Some("code"), Some(val)) => Some(val.to_string()),
                            _ => None,
                        }
                    })
                })
        })
        .ok_or_else(|| anyhow!("No authorization code in callback"))?;

    // 5. Send success response to browser
    let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n\
        <html><body><h1>Authentication successful!</h1>\
        <p>You can close this tab and return to the terminal.</p>\
        </body></html>";
    stream.write_all(response.as_bytes())?;
    drop(stream);

    println!("Received authorization code, exchanging for token...");

    // 6. Exchange code for token
    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(TOKEN_URL)
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", &code),
            ("client_id", OAUTH_CLIENT_ID),
            ("redirect_uri", &redirect_uri),
        ])
        .send()?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        return Err(anyhow!("Token exchange failed: {status} {body}"));
    }

    let token_resp: TokenResponse = resp.json()?;

    // 7. Save credentials
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;
    let expires_at = token_resp
        .expires_in
        .map(|secs| now_ms + secs * 1000)
        .unwrap_or(now_ms + 86400 * 1000); // default 24h

    let creds = serde_json::json!({
        "claudeAiOauth": {
            "accessToken": token_resp.access_token,
            "refreshToken": token_resp.refresh_token,
            "expiresAt": expires_at,
        }
    });

    let creds_dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("~/.config"))
        .join("claude-usage-tui");
    std::fs::create_dir_all(&creds_dir)?;
    let creds_path = creds_dir.join("credentials.json");
    std::fs::write(&creds_path, serde_json::to_string_pretty(&creds)?)?;

    println!("Credentials saved to {}", creds_path.display());
    println!("You can now run `claude-usage-tui` to start monitoring.");

    Ok(())
}
