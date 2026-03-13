use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Organization {
    pub uuid: String,
    pub name: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Utilization {
    #[serde(rename = "messagesSent")]
    pub messages_sent: Option<u64>,
    #[serde(rename = "messagesLimit")]
    pub messages_limit: Option<u64>,
    #[serde(rename = "resetAt")]
    pub reset_at: Option<String>,
    #[serde(rename = "percentUsed")]
    pub percent_used: Option<f64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UsageResponse {
    pub orgs: Option<Vec<OrgUsage>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OrgUsage {
    pub uuid: Option<String>,
    #[serde(rename = "sessionUtilization")]
    pub session_utilization: Option<Utilization>,
    #[serde(rename = "weeklyUtilization")]
    pub weekly_utilization: Option<Utilization>,
    #[serde(rename = "weeklyOpusUtilization")]
    pub weekly_opus_utilization: Option<Utilization>,
    #[serde(rename = "weeklySonnetUtilization")]
    pub weekly_sonnet_utilization: Option<Utilization>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct OverageSpendLimit {
    #[serde(rename = "spendLimitDollars")]
    pub spend_limit_dollars: Option<f64>,
    #[serde(rename = "currentSpendDollars")]
    pub current_spend_dollars: Option<f64>,
    #[serde(rename = "spendLimitEnabled")]
    pub spend_limit_enabled: Option<bool>,
    #[serde(rename = "resetAt")]
    pub reset_at: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct OverageCreditGrant {
    #[serde(rename = "grantAmountDollars")]
    pub grant_amount_dollars: Option<f64>,
    #[serde(rename = "remainingDollars")]
    pub remaining_dollars: Option<f64>,
    #[serde(rename = "usedDollars")]
    pub used_dollars: Option<f64>,
}

/// Combined data model used by the app
#[derive(Debug, Clone, Default)]
pub struct UsageData {
    // Session
    pub session_messages_sent: Option<u64>,
    pub session_messages_limit: Option<u64>,
    pub session_reset_at: Option<String>,
    pub session_percent_used: Option<f64>,

    // Weekly (all models)
    pub weekly_messages_sent: Option<u64>,
    pub weekly_messages_limit: Option<u64>,
    pub weekly_reset_at: Option<String>,
    pub weekly_percent_used: Option<f64>,

    // Weekly Sonnet
    pub weekly_sonnet_sent: Option<u64>,
    pub weekly_sonnet_limit: Option<u64>,
    pub weekly_sonnet_percent: Option<f64>,

    // Weekly Opus
    pub weekly_opus_sent: Option<u64>,
    pub weekly_opus_limit: Option<u64>,
    pub weekly_opus_percent: Option<f64>,

    // Spend
    pub spend_limit_dollars: Option<f64>,
    pub current_spend_dollars: Option<f64>,
    pub spend_limit_enabled: Option<bool>,
    pub spend_reset_at: Option<String>,

    // Credit grant
    pub credit_remaining_dollars: Option<f64>,
    pub credit_grant_dollars: Option<f64>,
    pub credit_used_dollars: Option<f64>,
}
