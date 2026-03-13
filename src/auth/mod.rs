pub mod cookie;
pub mod login;
pub mod oauth;

use anyhow::Result;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};

use crate::config::Config;

#[derive(Debug, Clone)]
pub enum Auth {
    OAuth {
        access_token: String,
        plan_name: String,
    },
    Cookie {
        session_key: String,
    },
}

impl Auth {
    pub fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        match self {
            Auth::OAuth { access_token, .. } => {
                let bearer = format!("Bearer {}", access_token);
                if let Ok(val) = HeaderValue::from_str(&bearer) {
                    headers.insert(AUTHORIZATION, val);
                }
            }
            Auth::Cookie { session_key } => {
                let cookie_val = format!("sessionKey={}", session_key);
                if let Ok(val) = HeaderValue::from_str(&cookie_val) {
                    headers.insert("Cookie", val);
                }
            }
        }
        headers
    }

    pub fn plan_name(&self) -> String {
        match self {
            Auth::OAuth { plan_name, .. } => plan_name.clone(),
            Auth::Cookie { .. } => "Cookie Auth".to_string(),
        }
    }
}

/// Resolve auth using priority: CLI session key > config session key > OAuth credentials
pub fn resolve_auth(config: &Config, cli_session_key: Option<&str>) -> Result<Auth> {
    // 1. CLI-provided session key takes highest priority
    if let Some(key) = cli_session_key {
        return Ok(Auth::Cookie {
            session_key: key.to_string(),
        });
    }

    // 2. Config file session key
    if let Some(ref key) = config.session_key {
        return Ok(Auth::Cookie {
            session_key: key.clone(),
        });
    }

    // 3. OAuth credentials from disk
    let (access_token, plan_name) = oauth::load_oauth_credentials()?;
    Ok(Auth::OAuth {
        access_token,
        plan_name,
    })
}
