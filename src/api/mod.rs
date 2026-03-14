pub mod types;

use anyhow::{anyhow, Result};
use reqwest::{header::HeaderMap, Client};
use types::UsageData;

const API_URL: &str = "https://api.anthropic.com/v1/messages";

pub struct ClaudeClient {
    client: Client,
}

impl ClaudeClient {
    pub async fn new(auth: &crate::auth::Auth, _org_id_override: Option<&str>) -> Result<Self> {
        let headers = auth.headers();
        let client = Client::builder()
            .default_headers(headers)
            .user_agent("claude-usage-tui/0.1.0")
            .build()?;

        Ok(Self { client })
    }

    /// Send a minimal API request and parse usage from rate-limit response headers.
    /// This bypasses Cloudflare since api.anthropic.com doesn't have browser challenges.
    pub async fn fetch_all(&self) -> Result<UsageData> {
        // Send smallest possible valid request — haiku with 1 max token
        let body = serde_json::json!({
            "model": "claude-haiku-4-5-20251001",
            "max_tokens": 1,
            "messages": [{"role": "user", "content": "hi"}]
        });

        let resp = self.client
            .post(API_URL)
            .json(&body)
            .send()
            .await?;

        // We don't care about the response body — only the headers
        let headers = resp.headers().clone();

        let mut data = UsageData::default();

        // Parse rate-limit headers
        // anthropic-ratelimit-unified-5h-utilization: 0.0 to 1.0
        // anthropic-ratelimit-unified-5h-reset: unix timestamp
        // anthropic-ratelimit-unified-7d-utilization: 0.0 to 1.0
        // anthropic-ratelimit-unified-7d-reset: unix timestamp
        data.session_percent_used = Some(parse_header_f64(&headers, "anthropic-ratelimit-unified-5h-utilization") * 100.0);
        data.weekly_percent_used = Some(parse_header_f64(&headers, "anthropic-ratelimit-unified-7d-utilization") * 100.0);

        // Reset times (unix timestamps)
        let session_reset_ts = parse_header_f64(&headers, "anthropic-ratelimit-unified-5h-reset");
        if session_reset_ts > 0.0 {
            data.session_reset_at = Some(format_unix_timestamp(session_reset_ts as i64));
        }

        let weekly_reset_ts = parse_header_f64(&headers, "anthropic-ratelimit-unified-7d-reset");
        if weekly_reset_ts > 0.0 {
            data.weekly_reset_at = Some(format_unix_timestamp(weekly_reset_ts as i64));
        }

        // Also parse overage info from headers
        let overage_status = headers.get("anthropic-ratelimit-unified-overage-status")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if overage_status == "allowed" {
            data.spend_limit_enabled = Some(true);
        }

        Ok(data)
    }
}

fn parse_header_f64(headers: &HeaderMap, name: &str) -> f64 {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0)
}

fn format_unix_timestamp(ts: i64) -> String {
    chrono::DateTime::from_timestamp(ts, 0)
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_default()
}
