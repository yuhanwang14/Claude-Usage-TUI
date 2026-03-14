use serde::{Deserialize, Deserializer};

// ---------------------------------------------------------------------------
// Flexible numeric deserializer
// The claude.ai API returns utilization values as int, float, or quoted string.
// ---------------------------------------------------------------------------

fn deserialize_f64_flexible<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum FlexNum {
        Float(f64),
        Int(i64),
        Str(String),
    }
    match FlexNum::deserialize(deserializer)? {
        FlexNum::Float(v) => Ok(v),
        FlexNum::Int(v) => Ok(v as f64),
        FlexNum::Str(s) => s.parse::<f64>().map_err(serde::de::Error::custom),
    }
}

fn deserialize_opt_f64_flexible<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum FlexNum {
        Float(f64),
        Int(i64),
        Str(String),
        Null,
    }
    match Option::<FlexNum>::deserialize(deserializer)? {
        None => Ok(None),
        Some(FlexNum::Null) => Ok(None),
        Some(FlexNum::Float(v)) => Ok(Some(v)),
        Some(FlexNum::Int(v)) => Ok(Some(v as f64)),
        Some(FlexNum::Str(s)) => s
            .parse::<f64>()
            .map(Some)
            .map_err(serde::de::Error::custom),
    }
}

// ---------------------------------------------------------------------------
// Cookie-path response structs — claude.ai JSON schemas
// ---------------------------------------------------------------------------

/// One element of GET /api/organizations
#[derive(Debug, Deserialize, Clone)]
pub struct Organization {
    pub uuid: String,
    pub name: String,
}

/// A single utilization bucket inside the usage endpoint.
/// All fields are optional because not every bucket has every field.
#[derive(Debug, Deserialize, Clone, Default)]
pub struct UsageBucket {
    #[serde(
        default,
        deserialize_with = "deserialize_opt_f64_flexible",
        rename = "utilization"
    )]
    pub utilization: Option<f64>,
    pub resets_at: Option<String>,
}

/// GET /api/organizations/{orgId}/usage
/// Example: {"five_hour": {...}, "seven_day": {...}, "seven_day_opus": {...}, "seven_day_sonnet": {...}}
#[derive(Debug, Deserialize, Clone, Default)]
pub struct OrgUsageResponse {
    pub five_hour: Option<UsageBucket>,
    pub seven_day: Option<UsageBucket>,
    pub seven_day_opus: Option<UsageBucket>,
    pub seven_day_sonnet: Option<UsageBucket>,
}

/// GET /api/organizations/{orgId}/overage_spend_limit
/// Example: {"monthly_credit_limit": 50.0, "currency": "GBP", "used_credits": 38.32, "is_enabled": true}
#[derive(Debug, Deserialize, Clone, Default)]
pub struct OrgSpendLimit {
    pub monthly_credit_limit: Option<f64>,
    pub currency: Option<String>,
    pub used_credits: Option<f64>,
    pub is_enabled: Option<bool>,
}

/// GET /api/organizations/{orgId}/overage_credit_grant
/// Example: {"remaining_balance": -0.03, "currency": "GBP"}
#[derive(Debug, Deserialize, Clone, Default)]
pub struct OrgCreditGrant {
    pub remaining_balance: Option<f64>,
    pub currency: Option<String>,
}

// ---------------------------------------------------------------------------
// Legacy OAuth-path structs (kept for any future use)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Unified app data model
// ---------------------------------------------------------------------------

/// Combined data model used by the app — populated by either auth path.
#[derive(Debug, Clone, Default)]
pub struct UsageData {
    // Session (5-hour window)
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

    // Currency
    pub spend_currency: Option<String>,
}
