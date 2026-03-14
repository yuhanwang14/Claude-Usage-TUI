use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::time::{SystemTime, UNIX_EPOCH};

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
    let redirect_uri = format!("http://localhost:{port}/callback");

    // 2. Generate state parameter for CSRF protection
    let state = format!(
        "claude-usage-tui-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    );

    // 3. Generate PKCE code_verifier and code_challenge (S256)
    let code_verifier = generate_code_verifier();
    let code_challenge = generate_code_challenge(&code_verifier);

    // 4. Build authorize URL using proper URL encoding
    let auth_url = format!(
        "{AUTHORIZE_URL}?response_type=code\
        &client_id={OAUTH_CLIENT_ID}\
        &redirect_uri={}\
        &scope={}\
        &state={state}\
        &code_challenge={code_challenge}\
        &code_challenge_method=S256",
        urlencoded(&redirect_uri),
        urlencoded("user:inference user:profile"),
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

    // Parse query params from "GET /callback?code=XXX&state=YYY HTTP/1.1"
    let query_string = request_line
        .split_whitespace()
        .nth(1)
        .and_then(|path| path.split('?').nth(1))
        .unwrap_or("");

    let params: std::collections::HashMap<&str, &str> = query_string
        .split('&')
        .filter_map(|param| {
            let mut kv = param.splitn(2, '=');
            Some((kv.next()?, kv.next()?))
        })
        .collect();

    // Verify state to prevent CSRF
    let returned_state = params.get("state").ok_or_else(|| anyhow!("Missing state in callback"))?;
    if *returned_state != state {
        return Err(anyhow!("State mismatch — possible CSRF attack"));
    }

    let code = params
        .get("code")
        .ok_or_else(|| anyhow!("No authorization code in callback"))?
        .to_string();

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
            ("code_verifier", &code_verifier),
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

/// Generate a random 128-byte code_verifier (base64url-encoded, no padding)
fn generate_code_verifier() -> String {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};

    // Generate 32 random bytes using multiple hashers seeded by RandomState
    let mut bytes = Vec::with_capacity(32);
    for _ in 0..4 {
        let s = RandomState::new();
        let mut h = s.build_hasher();
        h.write_u64(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
        );
        bytes.extend_from_slice(&h.finish().to_le_bytes());
    }
    base64url_encode(&bytes)
}

/// SHA-256 hash the verifier, then base64url-encode it
fn generate_code_challenge(verifier: &str) -> String {
    // Simple SHA-256 implementation (no external crate needed)
    // We use the system's openssl/sha256 via a command, or implement inline
    let digest = sha256(verifier.as_bytes());
    base64url_encode(&digest)
}

fn base64url_encode(data: &[u8]) -> String {
    // Standard base64 then convert to URL-safe variant without padding
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        }
        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        }
    }
    // Convert to URL-safe: + -> -, / -> _
    result.replace('+', "-").replace('/', "_")
}

/// Minimal SHA-256 implementation
fn sha256(data: &[u8]) -> [u8; 32] {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];

    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];

    // Padding
    let bit_len = (data.len() as u64) * 8;
    let mut msg = data.to_vec();
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    // Process blocks
    for block in msg.chunks(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([block[i*4], block[i*4+1], block[i*4+2], block[i*4+3]]);
        }
        for i in 16..64 {
            let s0 = w[i-15].rotate_right(7) ^ w[i-15].rotate_right(18) ^ (w[i-15] >> 3);
            let s1 = w[i-2].rotate_right(17) ^ w[i-2].rotate_right(19) ^ (w[i-2] >> 10);
            w[i] = w[i-16].wrapping_add(s0).wrapping_add(w[i-7]).wrapping_add(s1);
        }

        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = hh.wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);
            hh = g; g = f; f = e; e = d.wrapping_add(temp1);
            d = c; c = b; b = a; a = temp1.wrapping_add(temp2);
        }
        h[0] = h[0].wrapping_add(a); h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c); h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e); h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g); h[7] = h[7].wrapping_add(hh);
    }

    let mut result = [0u8; 32];
    for (i, val) in h.iter().enumerate() {
        result[i*4..i*4+4].copy_from_slice(&val.to_be_bytes());
    }
    result
}

fn urlencoded(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                String::from(b as char)
            }
            _ => format!("%{:02X}", b),
        })
        .collect()
}
