pub mod types;

use anyhow::{anyhow, Context, Result};
use reqwest::{header::HeaderMap, Client};
use types::{OrgCreditGrant, OrgSpendLimit, OrgUsageResponse, Organization, UsageData};

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const CLAUDE_AI_BASE: &str = "https://claude.ai/api";

pub struct ClaudeClient {
    client: Client,
    /// Kept to decide which fetch path to use at runtime.
    auth: crate::auth::Auth,
    /// Optional org ID override (cookie path only).
    org_id_override: Option<String>,
}

impl ClaudeClient {
    pub async fn new(auth: &crate::auth::Auth, org_id_override: Option<&str>) -> Result<Self> {
        let headers = auth.headers();
        let client = Client::builder()
            .default_headers(headers)
            .user_agent("claude-usage-tui/0.1.0")
            .build()?;

        Ok(Self {
            client,
            auth: auth.clone(),
            org_id_override: org_id_override.map(|s| s.to_string()),
        })
    }

    /// Top-level fetch: branches on auth type.
    pub async fn fetch_all(&self) -> Result<UsageData> {
        match &self.auth {
            crate::auth::Auth::Cookie { .. } => self.fetch_cookie_path().await,
            crate::auth::Auth::OAuth { .. } => self.fetch_oauth_path().await,
        }
    }

    // -----------------------------------------------------------------------
    // Cookie path — calls claude.ai REST endpoints
    // -----------------------------------------------------------------------

    async fn fetch_cookie_path(&self) -> Result<UsageData> {
        // 1. Resolve org ID
        let org_id = match &self.org_id_override {
            Some(id) => id.clone(),
            None => self.fetch_org_id().await?,
        };

        // 2. Fire all three data requests concurrently.
        // URLs must be bound to locals so temporaries outlive the join! macro.
        let usage_url = format!("{CLAUDE_AI_BASE}/organizations/{org_id}/usage");
        let spend_url = format!("{CLAUDE_AI_BASE}/organizations/{org_id}/overage_spend_limit");
        let credit_url = format!("{CLAUDE_AI_BASE}/organizations/{org_id}/overage_credit_grant");
        let (usage_res, spend_res, credit_res) = tokio::join!(
            self.get_json::<OrgUsageResponse>(&usage_url),
            self.get_json::<OrgSpendLimit>(&spend_url),
            self.get_json::<OrgCreditGrant>(&credit_url),
        );

        // Tolerate partial failures — missing spend/credit data is not fatal
        let usage = usage_res.context("fetching usage")?;
        let spend = spend_res.unwrap_or_default();
        let credit = credit_res.unwrap_or_default();

        let mut data = UsageData::default();

        // Session (5-hour window) — utilization is 0.0–100.0 in claude.ai schema
        if let Some(bucket) = &usage.five_hour {
            data.session_percent_used = bucket.utilization;
            data.session_reset_at = bucket.resets_at.clone();
        }

        // Weekly (all models)
        if let Some(bucket) = &usage.seven_day {
            data.weekly_percent_used = bucket.utilization;
            data.weekly_reset_at = bucket.resets_at.clone();
        }

        // Weekly Opus
        if let Some(bucket) = &usage.seven_day_opus {
            data.weekly_opus_percent = bucket.utilization;
        }

        // Weekly Sonnet
        if let Some(bucket) = &usage.seven_day_sonnet {
            data.weekly_sonnet_percent = bucket.utilization;
        }

        // Spend limit — API returns cents, convert to currency units
        data.spend_limit_dollars = spend.monthly_credit_limit.map(|v| v / 100.0);
        data.current_spend_dollars = spend.used_credits.map(|v| v / 100.0);
        data.spend_limit_enabled = spend.is_enabled;
        data.spend_currency = spend.currency;

        // Credit grant — also in cents
        data.credit_remaining_dollars = credit.remaining_balance.map(|v| v / 100.0);

        Ok(data)
    }

    /// GET /api/organizations and return the first org's UUID.
    async fn fetch_org_id(&self) -> Result<String> {
        let orgs: Vec<Organization> = self
            .get_json(&format!("{CLAUDE_AI_BASE}/organizations"))
            .await
            .context("fetching organizations list")?;

        orgs.into_iter()
            .next()
            .map(|o| o.uuid)
            .ok_or_else(|| anyhow!("no organizations returned from claude.ai"))
    }

    // -----------------------------------------------------------------------
    // OAuth path — POST to api.anthropic.com and read rate-limit headers
    // -----------------------------------------------------------------------

    async fn fetch_oauth_path(&self) -> Result<UsageData> {
        let body = serde_json::json!({
            "model": "claude-haiku-4-5-20251001",
            "max_tokens": 1,
            "messages": [{"role": "user", "content": "hi"}]
        });

        let resp = self
            .client
            .post(ANTHROPIC_API_URL)
            .json(&body)
            .send()
            .await?;

        // Only the response headers carry the rate-limit data
        let headers = resp.headers().clone();

        let mut data = UsageData::default();

        // Utilization values: 0.0–1.0 in OAuth headers → convert to percent
        let raw_5h =
            parse_header_f64(&headers, "anthropic-ratelimit-unified-5h-utilization");
        let raw_7d =
            parse_header_f64(&headers, "anthropic-ratelimit-unified-7d-utilization");

        if raw_5h > 0.0 {
            data.session_percent_used = Some(raw_5h * 100.0);
        }
        if raw_7d > 0.0 {
            data.weekly_percent_used = Some(raw_7d * 100.0);
        }

        // Reset timestamps (unix seconds)
        let session_reset_ts =
            parse_header_f64(&headers, "anthropic-ratelimit-unified-5h-reset");
        if session_reset_ts > 0.0 {
            data.session_reset_at = Some(format_unix_timestamp(session_reset_ts as i64));
        }

        let weekly_reset_ts =
            parse_header_f64(&headers, "anthropic-ratelimit-unified-7d-reset");
        if weekly_reset_ts > 0.0 {
            data.weekly_reset_at = Some(format_unix_timestamp(weekly_reset_ts as i64));
        }

        // Overage hint from headers
        let overage_status = headers
            .get("anthropic-ratelimit-unified-overage-status")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if overage_status == "allowed" {
            data.spend_limit_enabled = Some(true);
        }

        Ok(data)
    }

    // -----------------------------------------------------------------------
    // Shared helpers
    // -----------------------------------------------------------------------

    async fn get_json<T: serde::de::DeserializeOwned>(&self, url: &str) -> Result<T> {
        let resp = self.client.get(url).send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("GET {url} returned {status}: {body}"));
        }
        let value = resp.json::<T>().await?;
        Ok(value)
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
