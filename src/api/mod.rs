pub mod types;

use anyhow::{anyhow, Result};
use reqwest::{header::HeaderMap, Client};

use crate::auth::Auth;
use types::{
    OrgUsage, OverageCreditGrant, OverageSpendLimit, UsageData, UsageResponse,
};

const BASE_URL: &str = "https://claude.ai";

pub struct ClaudeClient {
    client: Client,
    org_id: String,
}

impl ClaudeClient {
    /// Create a new client, fetching org_id from the API if not provided.
    pub async fn new(auth: &Auth, org_id_override: Option<&str>) -> Result<Self> {
        let headers = auth.headers();
        let client = build_client(headers)?;

        let org_id = if let Some(id) = org_id_override {
            id.to_string()
        } else {
            fetch_org_id(&client).await?
        };

        Ok(Self { client, org_id })
    }

    pub async fn fetch_usage(&self) -> Result<UsageResponse> {
        let url = format!(
            "{}/api/organizations/{}/claude_ai_limit_status",
            BASE_URL, self.org_id
        );
        let resp = self.client.get(&url).send().await?;
        if !resp.status().is_success() {
            return Err(anyhow!("Usage API returned {}", resp.status()));
        }
        Ok(resp.json::<UsageResponse>().await?)
    }

    pub async fn fetch_spend_limit(&self) -> Result<OverageSpendLimit> {
        let url = format!(
            "{}/api/organizations/{}/overage_spend_limit",
            BASE_URL, self.org_id
        );
        let resp = self.client.get(&url).send().await?;
        if !resp.status().is_success() {
            return Err(anyhow!("Spend limit API returned {}", resp.status()));
        }
        Ok(resp.json::<OverageSpendLimit>().await?)
    }

    pub async fn fetch_credit_grant(&self) -> Result<OverageCreditGrant> {
        let url = format!(
            "{}/api/organizations/{}/overage_credit_grant",
            BASE_URL, self.org_id
        );
        let resp = self.client.get(&url).send().await?;
        if !resp.status().is_success() {
            return Err(anyhow!("Credit grant API returned {}", resp.status()));
        }
        Ok(resp.json::<OverageCreditGrant>().await?)
    }

    pub async fn fetch_all(&self) -> Result<UsageData> {
        let (usage_resp, spend, credit) = tokio::try_join!(
            self.fetch_usage(),
            self.fetch_spend_limit(),
            self.fetch_credit_grant(),
        )?;

        let mut data = UsageData::default();

        // Populate spend data
        data.spend_limit_dollars = spend.spend_limit_dollars;
        data.current_spend_dollars = spend.current_spend_dollars;
        data.spend_limit_enabled = spend.spend_limit_enabled;
        data.spend_reset_at = spend.reset_at;

        // Populate credit data
        data.credit_remaining_dollars = credit.remaining_dollars;
        data.credit_grant_dollars = credit.grant_amount_dollars;
        data.credit_used_dollars = credit.used_dollars;

        // Populate utilization from the first org that matches our org_id (or first org)
        let org = find_org(&usage_resp, &self.org_id);
        if let Some(org) = org {
            if let Some(ref session) = org.session_utilization {
                data.session_messages_sent = session.messages_sent;
                data.session_messages_limit = session.messages_limit;
                data.session_reset_at = session.reset_at.clone();
                data.session_percent_used = session.percent_used;
            }
            if let Some(ref weekly) = org.weekly_utilization {
                data.weekly_messages_sent = weekly.messages_sent;
                data.weekly_messages_limit = weekly.messages_limit;
                data.weekly_reset_at = weekly.reset_at.clone();
                data.weekly_percent_used = weekly.percent_used;
            }
            if let Some(ref sonnet) = org.weekly_sonnet_utilization {
                data.weekly_sonnet_sent = sonnet.messages_sent;
                data.weekly_sonnet_limit = sonnet.messages_limit;
                data.weekly_sonnet_percent = sonnet.percent_used;
            }
            if let Some(ref opus) = org.weekly_opus_utilization {
                data.weekly_opus_sent = opus.messages_sent;
                data.weekly_opus_limit = opus.messages_limit;
                data.weekly_opus_percent = opus.percent_used;
            }
        }

        Ok(data)
    }
}

fn find_org<'a>(resp: &'a UsageResponse, org_id: &str) -> Option<&'a OrgUsage> {
    let orgs = resp.orgs.as_deref()?;
    // Try exact match first, then fall back to first org
    orgs.iter()
        .find(|o| o.uuid.as_deref() == Some(org_id))
        .or_else(|| orgs.first())
}

async fn fetch_org_id(client: &Client) -> Result<String> {
    let url = format!("{}/api/organizations", BASE_URL);
    let resp = client.get(&url).send().await?;
    if !resp.status().is_success() {
        return Err(anyhow!("Organizations API returned {}", resp.status()));
    }
    let orgs: Vec<types::Organization> = resp.json().await?;
    orgs.into_iter()
        .next()
        .map(|o| o.uuid)
        .ok_or_else(|| anyhow!("No organizations found"))
}

fn build_client(default_headers: HeaderMap) -> Result<Client> {
    let client = Client::builder()
        .default_headers(default_headers)
        .user_agent("claude-usage-tui/0.1.0")
        .build()?;
    Ok(client)
}
