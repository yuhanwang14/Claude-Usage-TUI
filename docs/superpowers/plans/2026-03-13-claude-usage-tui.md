# claude-usage-tui Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a btop-style TUI that displays Claude.ai usage limits (session, weekly, spend) in real-time.

**Architecture:** Rust binary with tokio async runtime. Auth module reads OAuth credentials or cookies, API module polls claude.ai endpoints, App module merges terminal events with API poll via `tokio::select!`, UI module renders ratatui widgets. All rendering is a pure function of app state.

**Tech Stack:** Rust 2021, ratatui, crossterm (with `event-stream` feature), tokio, reqwest, serde, clap, chrono

---

## File Structure

| File | Responsibility |
|------|---------------|
| `Cargo.toml` | Dependencies and metadata |
| `src/main.rs` | CLI parsing (clap), dispatch to `run` or `login` |
| `src/app.rs` | `App` struct (state), event loop (`tokio::select!`), tick/key/api event handling |
| `src/config.rs` | Load/parse `~/.config/claude-usage-tui/config.toml` |
| `src/auth/mod.rs` | `AuthProvider` trait, `resolve_auth()` priority chain |
| `src/auth/oauth.rs` | Read `credentials.json` files, token expiry check |
| `src/auth/cookie.rs` | Session cookie from config or CLI flag |
| `src/api/mod.rs` | `ClaudeClient` struct, HTTP client setup, `fetch_all()` |
| `src/api/types.rs` | Serde structs for all API responses |
| `src/ui/mod.rs` | Top-level `draw()` function, layout splits |
| `src/ui/session.rs` | `render_session()` — gauge + sparkline |
| `src/ui/weekly.rs` | `render_weekly()` — 3 gauges + reset timer |
| `src/ui/spend.rs` | `render_spend()` — gauge + amount text + balance |
| `src/ui/status_bar.rs` | `render_status_bar()` — plan, connection, interval, keys |
| `src/ui/theme.rs` | Color constants, `gauge_color(pct)` helper |
| `LICENSE` | MIT license text |
| `README.md` | Usage instructions, screenshots placeholder |
| `.gitignore` | Rust defaults + .superpowers/ |

---

## Chunk 1: Project Scaffold + Types + Config

### Task 1: Initialize Cargo project and dependencies

**Files:**
- Create: `Cargo.toml`
- Create: `.gitignore`
- Create: `LICENSE`
- Create: `src/main.rs` (placeholder)

- [ ] **Step 1: Create Cargo.toml**

```toml
[package]
name = "claude-usage-tui"
version = "0.1.0"
edition = "2021"
description = "btop-style TUI for monitoring Claude.ai usage limits"
license = "MIT"
repository = "https://github.com/yuhanwang/claude-usage-tui"

[dependencies]
ratatui = "0.29"
crossterm = { version = "0.28", features = ["event-stream"] }
futures = "0.3"
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json", "cookies"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
clap = { version = "4", features = ["derive"] }
dirs = "6"
toml = "0.8"
chrono = { version = "0.4", features = ["serde"] }
open = "5"
anyhow = "1"
```

- [ ] **Step 2: Create .gitignore**

```
/target
.superpowers/
```

- [ ] **Step 3: Create LICENSE (MIT)**

Standard MIT license with `2026 yuhanwang`.

- [ ] **Step 4: Create placeholder src/main.rs**

```rust
fn main() {
    println!("claude-usage-tui");
}
```

- [ ] **Step 5: Verify it compiles**

Run: `cd ~/Programming/Tmp/claude-usage-tui && cargo build`
Expected: Compiles with no errors.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock .gitignore LICENSE src/main.rs
git commit -m "feat: initialize cargo project with dependencies"
```

---

### Task 2: API response types

**Files:**
- Create: `src/api/types.rs`
- Create: `src/api/mod.rs` (re-export only)

- [ ] **Step 1: Write src/api/types.rs**

```rust
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Organization {
    pub uuid: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Utilization {
    pub utilization: f64,
    pub resets_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UsageResponse {
    pub five_hour: Utilization,
    pub seven_day: Utilization,
    pub seven_day_opus: Option<Utilization>,
    pub seven_day_sonnet: Option<Utilization>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverageSpendLimit {
    pub monthly_credit_limit: f64,
    pub used_credits: f64,
    pub currency: String,
    pub is_enabled: bool,
    pub resets_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverageCreditGrant {
    pub remaining_balance: f64,
    pub currency: String,
}

/// Combined data from all endpoints, used by the UI.
#[derive(Debug, Clone, Default)]
pub struct UsageData {
    pub session_pct: f64,
    pub session_resets_at: Option<String>,
    pub weekly_pct: f64,
    pub weekly_resets_at: Option<String>,
    pub opus_pct: f64,
    pub sonnet_pct: f64,
    pub spend_used: f64,
    pub spend_limit: f64,
    pub spend_currency: String,
    pub spend_enabled: bool,
    pub spend_resets_at: Option<String>,
    pub balance: f64,
}
```

- [ ] **Step 2: Write src/api/mod.rs**

```rust
pub mod types;
```

- [ ] **Step 3: Update src/main.rs to include module**

```rust
mod api;

fn main() {
    println!("claude-usage-tui");
}
```

- [ ] **Step 4: Verify it compiles**

Run: `cargo build`
Expected: Compiles with no errors (unused warnings OK).

- [ ] **Step 5: Commit**

```bash
git add src/api/
git commit -m "feat: add API response types and UsageData model"
```

---

### Task 3: Config module

**Files:**
- Create: `src/config.rs`

- [ ] **Step 1: Write src/config.rs**

```rust
use anyhow::Result;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    #[serde(default = "default_interval")]
    pub refresh_interval: u64,
    pub session_key: Option<String>,
    pub org_id: Option<String>,
}

fn default_interval() -> u64 {
    5
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            Ok(toml::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("claude-usage-tui")
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    pub fn credentials_path() -> PathBuf {
        Self::config_dir().join("credentials.json")
    }
}
```

- [ ] **Step 2: Add to main.rs**

```rust
mod api;
mod config;

fn main() {
    println!("claude-usage-tui");
}
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo build`

- [ ] **Step 4: Commit**

```bash
git add src/config.rs src/main.rs
git commit -m "feat: add config module with TOML loading"
```

---

## Chunk 2: Authentication

### Task 4: Auth trait and OAuth credential reading

**Files:**
- Create: `src/auth/mod.rs`
- Create: `src/auth/oauth.rs`
- Create: `src/auth/cookie.rs`

- [ ] **Step 1: Write src/auth/mod.rs**

```rust
pub mod cookie;
pub mod oauth;

use anyhow::Result;
use reqwest::header::{HeaderMap, HeaderValue};

/// Resolved authentication — either a Bearer token or a session cookie.
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
    /// Build request headers for claude.ai API calls.
    pub fn headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert("Accept", HeaderValue::from_static("application/json"));
        match self {
            Auth::OAuth { access_token, .. } => {
                headers.insert(
                    "Authorization",
                    HeaderValue::from_str(&format!("Bearer {access_token}"))?,
                );
                headers.insert(
                    "anthropic-version",
                    HeaderValue::from_static("2023-06-01"),
                );
            }
            Auth::Cookie { session_key } => {
                headers.insert(
                    "Cookie",
                    HeaderValue::from_str(&format!("sessionKey={session_key}"))?,
                );
            }
        }
        Ok(headers)
    }

    pub fn plan_name(&self) -> &str {
        match self {
            Auth::OAuth { plan_name, .. } => plan_name,
            Auth::Cookie { .. } => "Pro",
        }
    }
}

/// Try all auth methods in priority order.
pub fn resolve_auth(
    cookie_override: Option<&str>,
    config_cookie: Option<&str>,
) -> Result<Auth> {
    // 1. App's own credentials
    if let Some(auth) = oauth::try_app_credentials()? {
        return Ok(auth);
    }
    // 2. Claude Code credentials
    if let Some(auth) = oauth::try_claude_code_credentials()? {
        return Ok(auth);
    }
    // 3. Cookie (CLI flag or config)
    if let Some(key) = cookie_override.or(config_cookie) {
        return Ok(Auth::Cookie {
            session_key: key.to_string(),
        });
    }
    anyhow::bail!(
        "No authentication found.\n\
         Run `claude-usage-tui login` or pass `--cookie <sessionKey>`."
    )
}
```

- [ ] **Step 2: Write src/auth/oauth.rs**

```rust
use super::Auth;
use anyhow::Result;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Credentials {
    claude_ai_oauth: Option<OAuthToken>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OAuthToken {
    access_token: String,
    #[allow(dead_code)]
    refresh_token: Option<String>,
    expires_at: Option<u64>,
    subscription_type: Option<String>,
    rate_limit_tier: Option<String>,
}

impl OAuthToken {
    fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(exp) => {
                let now_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;
                // Expired or within 5 minutes of expiry
                now_ms + 300_000 >= exp
            }
            None => false,
        }
    }

    fn plan_name(&self) -> String {
        let sub = self
            .subscription_type
            .as_deref()
            .unwrap_or("pro");
        let tier = self.rate_limit_tier.as_deref().unwrap_or("");

        let base = match sub {
            "max" => "Max",
            "pro" => "Pro",
            "team" => "Team",
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
}

fn try_load(path: PathBuf) -> Result<Option<Auth>> {
    if !path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&path)?;
    let creds: Credentials = serde_json::from_str(&content)?;
    match creds.claude_ai_oauth {
        Some(token) if !token.is_expired() => Ok(Some(Auth::OAuth {
            plan_name: token.plan_name(),
            access_token: token.access_token,
        })),
        _ => Ok(None),
    }
}

pub fn try_app_credentials() -> Result<Option<Auth>> {
    try_load(crate::config::Config::credentials_path())
}

pub fn try_claude_code_credentials() -> Result<Option<Auth>> {
    let home = dirs::home_dir().unwrap_or_default();
    try_load(home.join(".claude").join(".credentials.json"))
}
```

- [ ] **Step 3: Write src/auth/cookie.rs**

```rust
// Cookie auth is fully handled in auth/mod.rs resolve_auth().
// This file exists for future expansion (cookie validation, etc).
```

- [ ] **Step 4: Add to main.rs**

```rust
mod api;
mod auth;
mod config;

fn main() {
    println!("claude-usage-tui");
}
```

- [ ] **Step 5: Verify it compiles**

Run: `cargo build`

- [ ] **Step 6: Commit**

```bash
git add src/auth/ src/main.rs
git commit -m "feat: add auth module with OAuth and cookie support"
```

---

## Chunk 3: API Client

### Task 5: HTTP client and API fetching

**Files:**
- Modify: `src/api/mod.rs`

- [ ] **Step 1: Write the API client**

Replace `src/api/mod.rs`:

```rust
pub mod types;

use anyhow::{Context, Result};
use reqwest::Client;
use types::*;

pub struct ClaudeClient {
    client: Client,
    base_url: String,
    org_id: String,
}

impl ClaudeClient {
    pub async fn new(
        headers: reqwest::header::HeaderMap,
        org_id_override: Option<&str>,
    ) -> Result<Self> {
        let client = Client::builder()
            .default_headers(headers)
            .build()?;

        let base_url = "https://claude.ai".to_string();

        let org_id = match org_id_override {
            Some(id) => id.to_string(),
            None => {
                let orgs: Vec<Organization> = client
                    .get(format!("{base_url}/api/organizations"))
                    .send()
                    .await?
                    .error_for_status()
                    .context("Failed to fetch organizations — check your authentication")?
                    .json()
                    .await?;
                orgs.first()
                    .context("No organizations found")?
                    .uuid
                    .clone()
            }
        };

        Ok(Self {
            client,
            base_url,
            org_id,
        })
    }

    pub async fn fetch_usage(&self) -> Result<UsageResponse> {
        Ok(self
            .client
            .get(format!(
                "{}/api/organizations/{}/usage",
                self.base_url, self.org_id
            ))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    pub async fn fetch_spend_limit(&self) -> Result<OverageSpendLimit> {
        Ok(self
            .client
            .get(format!(
                "{}/api/organizations/{}/overage_spend_limit",
                self.base_url, self.org_id
            ))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    pub async fn fetch_credit_grant(&self) -> Result<OverageCreditGrant> {
        Ok(self
            .client
            .get(format!(
                "{}/api/organizations/{}/overage_credit_grant",
                self.base_url, self.org_id
            ))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    /// Fetch all endpoints and combine into UsageData.
    pub async fn fetch_all(&self) -> Result<UsageData> {
        let (usage, spend, credit) = tokio::try_join!(
            self.fetch_usage(),
            self.fetch_spend_limit(),
            self.fetch_credit_grant(),
        )?;

        Ok(UsageData {
            session_pct: usage.five_hour.utilization,
            session_resets_at: usage.five_hour.resets_at,
            weekly_pct: usage.seven_day.utilization,
            weekly_resets_at: usage.seven_day.resets_at,
            opus_pct: usage
                .seven_day_opus
                .map(|u| u.utilization)
                .unwrap_or(0.0),
            sonnet_pct: usage
                .seven_day_sonnet
                .map(|u| u.utilization)
                .unwrap_or(0.0),
            spend_used: spend.used_credits,
            spend_limit: spend.monthly_credit_limit,
            spend_currency: spend.currency,
            spend_enabled: spend.is_enabled,
            spend_resets_at: spend.resets_at,
            balance: credit.remaining_balance,
        })
    }
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo build`

- [ ] **Step 3: Commit**

```bash
git add src/api/mod.rs
git commit -m "feat: add Claude API client with fetch_all"
```

---

## Chunk 4: App State (needed before UI)

### Task 6: App state

**Files:**
- Create: `src/app.rs`

- [ ] **Step 1: Write src/app.rs**

```rust
use crate::api::types::UsageData;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Online,
    Offline,
    Disconnected,
}

pub struct App {
    pub data: UsageData,
    pub sparkline_data: Vec<f64>,
    pub connection: ConnectionStatus,
    pub refresh_interval: u64,
    pub plan_name: String,
    pub running: bool,
}

impl App {
    pub fn new(plan_name: String, refresh_interval: u64) -> Self {
        Self {
            data: UsageData::default(),
            sparkline_data: Vec::with_capacity(60),
            connection: ConnectionStatus::Disconnected,
            refresh_interval,
            plan_name,
            running: true,
        }
    }

    pub fn update_data(&mut self, data: UsageData) {
        self.sparkline_data.push(data.session_pct);
        if self.sparkline_data.len() > 60 {
            self.sparkline_data.remove(0);
        }
        self.data = data;
        self.connection = ConnectionStatus::Online;
    }

    pub fn set_error(&mut self, is_network: bool) {
        self.connection = if is_network {
            ConnectionStatus::Disconnected
        } else {
            ConnectionStatus::Offline
        };
    }

    pub fn increase_interval(&mut self) {
        self.refresh_interval = (self.refresh_interval + 1).min(60);
    }

    pub fn decrease_interval(&mut self) {
        self.refresh_interval = (self.refresh_interval.saturating_sub(1)).max(1);
    }

    pub fn session_reset_text(&self) -> String {
        format_reset_time(self.data.session_resets_at.as_deref())
    }

    pub fn weekly_reset_text(&self) -> String {
        format_reset_time(self.data.weekly_resets_at.as_deref())
    }

    pub fn spend_reset_text(&self) -> String {
        format_reset_time(self.data.spend_resets_at.as_deref())
    }
}

fn format_reset_time(resets_at: Option<&str>) -> String {
    let Some(s) = resets_at else {
        return String::new();
    };
    let Ok(dt) = s.parse::<DateTime<Utc>>() else {
        return String::new();
    };
    let now = Utc::now();
    let diff = dt.signed_duration_since(now);

    if diff.num_seconds() <= 0 {
        return "Resetting...".to_string();
    }

    let hours = diff.num_hours();
    let minutes = diff.num_minutes() % 60;

    if hours >= 24 {
        let days = hours / 24;
        format!("Resets in {days}d {h}h", h = hours % 24)
    } else if hours > 0 {
        format!("Resets in {hours}h {minutes}m")
    } else {
        format!("Resets in {minutes}m")
    }
}
```

- [ ] **Step 2: Add `mod app;` to main.rs**

- [ ] **Step 3: Verify it compiles**

Run: `cargo build`

- [ ] **Step 4: Commit**

```bash
git add src/app.rs src/main.rs
git commit -m "feat: add App state with reset time formatting"
```

---

## Chunk 5: Theme + UI Panels

### Task 7: Theme constants and all UI panel stubs

**Files:**
- Create: `src/ui/theme.rs`
- Create: `src/ui/mod.rs`
- Create: `src/ui/session.rs` (stub)
- Create: `src/ui/weekly.rs` (stub)
- Create: `src/ui/spend.rs` (stub)
- Create: `src/ui/status_bar.rs` (stub)

All four panel files start as empty stubs with just `use ratatui::{layout::Rect, Frame}; use crate::app::App; pub fn render(_f: &mut Frame, _area: Rect, _app: &App) {}` so the project compiles after this task. They get filled in by Tasks 8-11.

- [ ] **Step 1: Write src/ui/theme.rs**

```rust
use ratatui::style::{Color, Style};

// btop-inspired palette
pub const GREEN: Color = Color::Rgb(78, 197, 108);
pub const YELLOW: Color = Color::Rgb(232, 197, 71);
pub const RED: Color = Color::Rgb(224, 108, 117);
pub const BLUE: Color = Color::Rgb(126, 200, 227);
pub const DIM: Color = Color::Rgb(102, 102, 102);
pub const TEXT: Color = Color::Rgb(224, 224, 224);
pub const SUBTEXT: Color = Color::Rgb(136, 136, 136);

pub fn gauge_color(pct: f64) -> Color {
    if pct <= 50.0 {
        GREEN
    } else if pct <= 80.0 {
        YELLOW
    } else {
        RED
    }
}

pub fn gauge_style(pct: f64) -> Style {
    Style::default().fg(gauge_color(pct))
}
```

- [ ] **Step 2: Write src/ui/mod.rs (scaffold)**

```rust
pub mod session;
pub mod spend;
pub mod status_bar;
pub mod theme;
pub mod weekly;

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};

use crate::app::App;

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Ratio(6, 10),
            Constraint::Ratio(3, 10),
            Constraint::Min(1),
        ])
        .split(f.area());

    // Top row: session (40%) | weekly (60%)
    let top_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(60),
        ])
        .split(chunks[0]);

    session::render(f, top_row[0], app);
    weekly::render(f, top_row[1], app);
    spend::render(f, chunks[1], app);
    status_bar::render(f, chunks[2], app);
}
```

- [ ] **Step 3: Commit**

```bash
git add src/ui/theme.rs src/ui/mod.rs
git commit -m "feat: add theme constants and UI layout scaffold"
```

---

### Task 7: Session panel

**Files:**
- Create: `src/ui/session.rs`

- [ ] **Step 1: Write src/ui/session.rs**

```rust
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, Sparkline};
use ratatui::Frame;

use super::theme;
use crate::app::App;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(theme::DIM))
        .title(Line::from(vec![
            Span::styled("¹", Style::default().fg(theme::BLUE)),
            Span::styled(
                "session",
                Style::default().fg(theme::TEXT).add_modifier(ratatui::style::Modifier::BOLD),
            ),
        ]));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // gauge + pct
            Constraint::Length(1), // reset timer
            Constraint::Min(1),   // sparkline
        ])
        .split(inner);

    // Gauge
    let pct = app.data.session_pct;
    let gauge = Gauge::default()
        .gauge_style(theme::gauge_style(pct))
        .ratio(pct.clamp(0.0, 100.0) / 100.0)
        .label(format!("{pct:.0}%"));
    f.render_widget(gauge, chunks[0]);

    // Reset timer
    let reset_text = app.session_reset_text();
    let reset = Line::from(Span::styled(reset_text, Style::default().fg(theme::SUBTEXT)));
    f.render_widget(reset, chunks[1]);

    // Sparkline
    if !app.sparkline_data.is_empty() {
        let data: Vec<u64> = app
            .sparkline_data
            .iter()
            .map(|v| (*v * 100.0).clamp(0.0, 10000.0) as u64)
            .collect();
        let spark = Sparkline::default()
            .data(&data)
            .style(Style::default().fg(theme::BLUE));
        f.render_widget(spark, chunks[2]);
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add src/ui/session.rs
git commit -m "feat: add session panel with gauge and sparkline"
```

---

### Task 8: Weekly panel

**Files:**
- Create: `src/ui/weekly.rs`

- [ ] **Step 1: Write src/ui/weekly.rs**

```rust
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, BorderType, Gauge};
use ratatui::Frame;

use super::theme;
use crate::app::App;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::DIM))
        .title(Line::from(vec![
            Span::styled("²", Style::default().fg(theme::BLUE)),
            Span::styled(
                "weekly",
                Style::default().fg(theme::TEXT).add_modifier(ratatui::style::Modifier::BOLD),
            ),
        ]));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // All models
            Constraint::Length(2), // Sonnet
            Constraint::Length(2), // Opus
            Constraint::Min(1),   // reset timer
        ])
        .split(inner);

    render_gauge(f, chunks[0], "All   ", app.data.weekly_pct);
    render_gauge(f, chunks[1], "Sonnet", app.data.sonnet_pct);
    render_gauge(f, chunks[2], "Opus  ", app.data.opus_pct);

    let reset_text = app.weekly_reset_text();
    let reset = Line::from(Span::styled(reset_text, Style::default().fg(theme::SUBTEXT)));
    f.render_widget(reset, chunks[3]);
}

fn render_gauge(f: &mut Frame, area: Rect, label: &str, pct: f64) {
    let row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(8), Constraint::Min(1)])
        .split(area);

    let label_widget = Line::from(Span::styled(
        label,
        Style::default().fg(theme::SUBTEXT),
    ));
    f.render_widget(label_widget, row[0]);

    let gauge = Gauge::default()
        .gauge_style(theme::gauge_style(pct))
        .ratio(pct.clamp(0.0, 100.0) / 100.0)
        .label(format!("{pct:.0}%"));
    f.render_widget(gauge, row[1]);
}
```

- [ ] **Step 2: Commit**

```bash
git add src/ui/weekly.rs
git commit -m "feat: add weekly panel with three gauges"
```

---

### Task 9: Spend panel

**Files:**
- Create: `src/ui/spend.rs`

- [ ] **Step 1: Write src/ui/spend.rs**

```rust
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, BorderType, Gauge};
use ratatui::Frame;

use super::theme;
use crate::app::App;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::DIM))
        .title(Line::from(vec![
            Span::styled("³", Style::default().fg(theme::BLUE)),
            Span::styled(
                "spend",
                Style::default().fg(theme::TEXT).add_modifier(ratatui::style::Modifier::BOLD),
            ),
        ]));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if !app.data.spend_enabled {
        let msg = Line::from(Span::styled(
            "Extra usage not enabled",
            Style::default().fg(theme::SUBTEXT),
        ));
        f.render_widget(msg, inner);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // gauge
            Constraint::Length(1), // amount + reset
            Constraint::Min(1),   // balance
        ])
        .split(inner);

    let pct = if app.data.spend_limit > 0.0 {
        (app.data.spend_used / app.data.spend_limit) * 100.0
    } else {
        0.0
    };

    let gauge = Gauge::default()
        .gauge_style(theme::gauge_style(pct))
        .ratio(pct.clamp(0.0, 100.0) / 100.0)
        .label(format!("{pct:.0}%"));
    f.render_widget(gauge, chunks[0]);

    let currency = &app.data.spend_currency;
    let symbol = currency_symbol(currency);
    let amount_text = format!(
        "{symbol}{:.2} / {symbol}{:.2}   {}",
        app.data.spend_used,
        app.data.spend_limit,
        app.spend_reset_text(),
    );
    let amount = Line::from(Span::styled(amount_text, Style::default().fg(theme::SUBTEXT)));
    f.render_widget(amount, chunks[1]);

    let bal_color = if app.data.balance < 0.0 {
        theme::RED
    } else {
        theme::TEXT
    };
    let balance = Line::from(Span::styled(
        format!("Balance: {symbol}{:.2}", app.data.balance),
        Style::default().fg(bal_color),
    ));
    f.render_widget(balance, chunks[2]);
}

fn currency_symbol(code: &str) -> &str {
    match code {
        "GBP" => "£",
        "EUR" => "€",
        "JPY" | "CNY" => "¥",
        _ => "$",
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add src/ui/spend.rs
git commit -m "feat: add spend panel with gauge and balance"
```

---

### Task 10: Status bar

**Files:**
- Create: `src/ui/status_bar.rs`

- [ ] **Step 1: Write src/ui/status_bar.rs**

```rust
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::Frame;

use super::theme;
use crate::app::{App, ConnectionStatus};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let (dot, dot_color) = match app.connection {
        ConnectionStatus::Online => ("●", theme::GREEN),
        ConnectionStatus::Offline => ("●", theme::RED),
        ConnectionStatus::Disconnected => ("●", theme::YELLOW),
    };

    let status_text = match app.connection {
        ConnectionStatus::Online => "online",
        ConnectionStatus::Offline => "offline",
        ConnectionStatus::Disconnected => "disconnected",
    };

    let line = Line::from(vec![
        Span::styled(
            format!(" {} ", app.plan_name),
            Style::default().fg(theme::TEXT),
        ),
        Span::styled("│ ", Style::default().fg(theme::DIM)),
        Span::styled(format!("{dot} "), Style::default().fg(dot_color)),
        Span::styled(
            format!("{status_text} "),
            Style::default().fg(theme::SUBTEXT),
        ),
        Span::styled("│ ", Style::default().fg(theme::DIM)),
        Span::styled(
            format!("{}s ", app.refresh_interval),
            Style::default().fg(theme::TEXT),
        ),
        Span::styled("│ ", Style::default().fg(theme::DIM)),
        Span::styled(
            "q quit  +/- interval  r refresh",
            Style::default().fg(theme::SUBTEXT),
        ),
    ]);

    f.render_widget(line, area);
}
```

- [ ] **Step 2: Commit**

```bash
git add src/ui/status_bar.rs
git commit -m "feat: add status bar with plan, connection, and keybinds"
```

---

## Chunk 6: Event Loop + main.rs

### Task 12: Wire everything in main.rs

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Write the full main.rs**

```rust
mod api;
mod app;
mod auth;
mod config;
mod ui;

use std::io;
use std::time::Duration;

use anyhow::Result;
use app::App;
use clap::Parser;
use crossterm::event::{Event, EventStream, KeyCode, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use futures::StreamExt;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use tokio::time;

#[derive(Parser)]
#[command(name = "claude-usage-tui", about = "btop-style Claude.ai usage monitor")]
struct Cli {
    /// Session cookie (sk-ant-...)
    #[arg(long)]
    cookie: Option<String>,

    /// Organization ID override
    #[arg(long)]
    org: Option<String>,

    /// Refresh interval in seconds
    #[arg(short, long, default_value = "5")]
    interval: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = config::Config::load().unwrap_or_default();

    let interval = if cli.interval != 5 {
        cli.interval
    } else {
        config.refresh_interval
    };

    let auth = auth::resolve_auth(
        cli.cookie.as_deref(),
        config.session_key.as_deref(),
    )?;

    let headers = auth.headers()?;
    let plan_name = auth.plan_name().to_string();
    let org_override = cli.org.as_deref().or(config.org_id.as_deref());

    let client = api::ClaudeClient::new(headers, org_override).await?;

    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Panic hook to restore terminal
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(info);
    }));

    let mut app = App::new(plan_name, interval);

    // Initial fetch
    match client.fetch_all().await {
        Ok(data) => app.update_data(data),
        Err(e) => app.set_error(is_network_error(&e)),
    }

    let mut poll_interval = time::interval(Duration::from_secs(app.refresh_interval));
    poll_interval.tick().await; // consume first immediate tick

    let mut event_stream = EventStream::new();

    loop {
        // Check terminal size
        let size = terminal.size()?;
        if size.width < 40 || size.height < 12 {
            terminal.draw(|f| {
                let msg = ratatui::widgets::Paragraph::new("Terminal too small (min 40x12)")
                    .style(ratatui::style::Style::default().fg(ui::theme::YELLOW));
                f.render_widget(msg, f.area());
            })?;
        } else {
            terminal.draw(|f| ui::draw(f, &app))?;
        }

        tokio::select! {
            _ = poll_interval.tick() => {
                match client.fetch_all().await {
                    Ok(data) => app.update_data(data),
                    Err(e) => app.set_error(is_network_error(&e)),
                }
                // Reset interval in case user changed it
                poll_interval = time::interval(Duration::from_secs(app.refresh_interval));
                poll_interval.tick().await;
            }
            Some(Ok(event)) = event_stream.next() => {
                if let Event::Key(key) = event {
                    match (key.code, key.modifiers) {
                        (KeyCode::Char('q'), _) | (KeyCode::Esc, _) => {
                            app.running = false;
                        }
                        (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                            app.running = false;
                        }
                        (KeyCode::Char('r'), _) => {
                            match client.fetch_all().await {
                                Ok(data) => app.update_data(data),
                                Err(e) => app.set_error(is_network_error(&e)),
                            }
                        }
                        (KeyCode::Char('+') | KeyCode::Char('='), _) => {
                            app.increase_interval();
                        }
                        (KeyCode::Char('-'), _) => {
                            app.decrease_interval();
                        }
                        _ => {}
                    }
                }
            }
        }

        if !app.running {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn is_network_error(e: &anyhow::Error) -> bool {
    if let Some(re) = e.downcast_ref::<reqwest::Error>() {
        re.is_connect() || re.is_timeout()
    } else {
        false
    }
}
```

**Note:** The `login` subcommand is deferred to a follow-up task. MVP ships with OAuth auto-read + cookie auth. The error message in `resolve_auth` references `login` but the command won't exist yet — acceptable for MVP.

- [ ] **Step 2: Verify it compiles**

Run: `cargo build`
Expected: Compiles successfully.

- [ ] **Step 3: Commit**

```bash
git add src/main.rs
git commit -m "feat: wire up event loop, terminal, and full TUI"
```

---

## Chunk 7: README + GitHub Repo + Open Source

### Task 13: README

**Files:**
- Create: `README.md`

- [ ] **Step 1: Write README.md**

```markdown
# claude-usage-tui

A btop-style terminal UI for monitoring your Claude.ai usage limits in real-time.

![screenshot placeholder](docs/screenshot.png)

## Features

- **Session usage** — 5-hour rolling window with sparkline history
- **Weekly limits** — All models, Sonnet, and Opus breakdowns
- **Extra spend** — Monthly spend tracking with balance
- **btop aesthetics** — Rounded borders, color-coded gauges, compact layout
- **Auto-auth** — Reads Claude Code OAuth credentials automatically

## Install

```bash
cargo install claude-usage-tui
```

Or build from source:

```bash
git clone https://github.com/yuhanwang/claude-usage-tui
cd claude-usage-tui
cargo build --release
```

## Usage

If you have Claude Code installed, it just works:

```bash
claude-usage-tui
```

Otherwise, pass a session cookie:

```bash
claude-usage-tui --cookie "sk-ant-sid01-..."
```

### Keybindings

| Key | Action |
|-----|--------|
| `q` / `Esc` | Quit |
| `r` | Manual refresh |
| `+` / `=` | Increase refresh interval |
| `-` | Decrease refresh interval |

### Config

`~/.config/claude-usage-tui/config.toml`:

```toml
refresh_interval = 5
# session_key = "sk-ant-..."
# org_id = "org-xxx"
```

## License

MIT
```

- [ ] **Step 2: Commit**

```bash
git add README.md
git commit -m "docs: add README with usage instructions"
```

---

### Task 14: Create GitHub repo and push

- [ ] **Step 1: Create public repo**

```bash
cd ~/Programming/Tmp/claude-usage-tui
gh repo create claude-usage-tui --public --source=. --description "btop-style TUI for monitoring Claude.ai usage limits" --push
```

- [ ] **Step 2: Verify repo exists**

Run: `gh repo view yuhanwang/claude-usage-tui --json url`
Expected: Returns the repo URL.

---

## Chunk 8: Smoke Test + Polish

### Task 15: Run the app and verify it works

- [ ] **Step 1: Build release**

Run: `cargo build --release`

- [ ] **Step 2: Run it**

Run: `./target/release/claude-usage-tui`
Expected: TUI renders with session/weekly/spend panels. Data from claude.ai API populates. Press `q` to quit. Terminal restores correctly.

- [ ] **Step 3: Test error states**

Run: `./target/release/claude-usage-tui --cookie "invalid"`
Expected: Status bar shows `● offline`, app doesn't crash.

- [ ] **Step 4: Test keybinds**

- Press `+` / `-` — interval changes in status bar
- Press `r` — data refreshes
- Press `q` — app exits cleanly

- [ ] **Step 5: Fix any issues found during testing**

- [ ] **Step 6: Final commit if fixes needed**

```bash
git add -A && git commit -m "fix: polish after smoke testing"
git push
```
