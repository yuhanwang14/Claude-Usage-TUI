# claude-usage-tui — Design Spec

A btop-style terminal UI for monitoring Claude.ai usage limits in real-time.

## Overview

Rust + ratatui TUI that polls Claude.ai's internal API to display session usage, weekly limits, and extra spend data. Designed to sit alongside btop in a terminal grid with matching visual aesthetics.

## Data Sources

### Endpoints (all on `https://claude.ai`)

| Endpoint | Purpose |
|----------|---------|
| `GET /api/organizations` | Fetch org ID |
| `GET /api/organizations/{id}/usage` | Session (5h) and weekly utilization percentages |
| `GET /api/organizations/{id}/overage_spend_limit` | Extra spend limit and current usage |
| `GET /api/organizations/{id}/overage_credit_grant` | Remaining balance |

### Response Structures

**Usage:**
```json
{
  "five_hour": { "utilization": 18.0, "resets_at": "2026-03-14T02:00:00Z" },
  "seven_day": { "utilization": 10.0, "resets_at": "2026-03-19T19:00:00Z" },
  "seven_day_opus": { "utilization": 0.0 },
  "seven_day_sonnet": { "utilization": 0.0, "resets_at": "..." }
}
```

All `resets_at` fields are `Option<String>` — some variants (e.g. `seven_day_opus`) may omit it.

**Organizations:**
```json
[{ "uuid": "org-xxx", "name": "Personal", "capabilities": ["..."] }]
```

If multiple orgs are returned, use the first one. A `--org` CLI flag or `org_id` config field can override this.
```

**Overage Spend Limit:**
```json
{
  "monthlyCreditLimit": 50.0,
  "usedCredits": 32.31,
  "currency": "GBP",
  "isEnabled": true,
  "resetsAt": "2026-04-01T00:00:00Z"
}
```

**Overage Credit Grant:**
```json
{
  "remainingBalance": -0.03,
  "currency": "GBP"
}
```

### Future Extension

Architecture uses a trait-based API client to allow adding Console API data (token consumption, cost by model) without restructuring.

## Authentication

Priority order at startup:

1. **App's own OAuth credentials** — `~/.config/claude-usage-tui/credentials.json` (from `login` subcommand).
2. **Claude Code OAuth** — `~/.claude/.credentials.json`. Check `expiresAt`, auto-refresh with `refreshToken` if expired.
3. **Stored cookie** — `~/.config/claude-usage-tui/config.toml` `session_key` field, or `--cookie` CLI flag.
4. **None found** — Print error, prompt user to run `claude-usage-tui login` or pass `--cookie`.

Token refresh strategy: reactive — refresh when `expiresAt` is in the past or within 5 minutes of expiry. On refresh failure, fall through to next auth method.

### `login` subcommand

Authorization-code flow with local redirect (same pattern as Claude Code):
1. Start local HTTP server on a random port for OAuth callback
2. Open system browser (`open` on macOS, `xdg-open` on Linux) to Anthropic OAuth authorize URL
3. Receive redirect with auth code, exchange for access_token + refresh_token
4. Store to `~/.config/claude-usage-tui/credentials.json`

**OAuth details (to be extracted from Claude Code source at implementation time):**
- `client_id`: Claude Code's registered OAuth client ID
- Authorize URL: `https://claude.ai/oauth/authorize` (to be confirmed)
- Token URL: `https://claude.ai/oauth/token` (to be confirmed)
- Scopes: `user:inference`, `user:profile`
- Refresh: `POST` to token URL with `grant_type=refresh_token`

If Claude Code's client_id cannot be reused (redirect URI validation), register a new OAuth app or fall back to cookie-only auth as MVP.

### Credentials schema

`~/.config/claude-usage-tui/credentials.json`:
```json
{
  "claudeAiOauth": {
    "accessToken": "string",
    "refreshToken": "string",
    "expiresAt": 1773225011408,
    "subscriptionType": "max",
    "rateLimitTier": "default_claude_max_5x"
  }
}
```

Same schema as `~/.claude/.credentials.json` for interoperability.

### Plan name

Derived from the `subscriptionType` field in the OAuth credentials (e.g. `"max"` → "Max") combined with `rateLimitTier` (e.g. `"default_claude_max_5x"` → "5x"). Fallback: `"Pro"` if unknown. For cookie auth, plan name is fetched from the organizations endpoint if available, otherwise hidden.

### Auth headers

- OAuth: `Authorization: Bearer {accessToken}` + `anthropic-version: 2023-06-01`
- Cookie: `Cookie: sessionKey={key}`

Both include `Accept: application/json`.

## Architecture

```
claude-usage-tui/
├── src/
│   ├── main.rs           # Entry point, clap CLI parsing
│   ├── app.rs            # App state, event loop (tokio::select!)
│   ├── auth/
│   │   ├── mod.rs        # AuthProvider trait
│   │   ├── oauth.rs      # Read ~/.claude/.credentials.json + refresh
│   │   ├── cookie.rs     # Session cookie from config
│   │   └── login.rs      # `login` subcommand, OAuth authorization-code flow
│   ├── api/
│   │   ├── mod.rs        # ClaudeApi trait
│   │   ├── usage.rs      # /organizations/{id}/usage
│   │   ├── spend.rs      # overage_spend_limit + credit_grant
│   │   └── types.rs      # Serde response structs
│   └── ui/
│       ├── mod.rs         # Main render function
│       ├── session.rs     # ¹session panel (gauge + sparkline)
│       ├── weekly.rs      # ²weekly panel (3 gauges)
│       ├── spend.rs       # ³spend panel (gauge + text)
│       └── theme.rs       # btop color constants
├── Cargo.toml
├── LICENSE                # MIT
└── README.md
```

### Data Flow

```
Auth → API Client → App State → UI Render
         ↑                         |
         └── tokio interval ───────┘
```

- `App` holds `UsageData` + `Vec<f64>` (sparkline history, last 60 samples)
- `tokio::time::interval` triggers API poll
- `crossterm` event stream merged with API poll via `tokio::select!`
- UI render is a pure function of `App` state

### Dependencies

| Crate | Purpose |
|-------|---------|
| `ratatui` | TUI framework |
| `crossterm` | Terminal backend |
| `tokio` | Async runtime |
| `reqwest` | HTTP client |
| `serde` / `serde_json` | JSON parsing |
| `clap` | CLI argument parsing |
| `dirs` | XDG config/home paths |
| `toml` | Config file parsing |
| `chrono` | Time formatting, reset countdown |
| `open` | Open system browser for login |
| `scopeguard` | Terminal cleanup on panic |

## UI Layout

Layout uses `Layout::vertical` with `Constraint::Ratio`:

```
┌──────────────────────────────────────────┐
│  Top Row (ratio 6/10)                    │
│  ╭─ ¹session ────╮ ╭─ ²weekly ────────╮ │  (horizontal split: 40% / 60%)
│  (session left)    (weekly right)        │
│  │ Gauge 18%     │ │ All      10%     │ │
│  │ Resets 4h20m  │ │ Sonnet    0%     │ │
│  │               │ │ Opus      0%     │ │
│  │ ⣀⣠⣤⣶⣿⣶⣤⣠⣀     │ │ Resets Thu 7 PM │ │
│  ╰───────────────╯ ╰─────────────────╯ │
├──────────────────────────────────────────┤
│  Bottom Row (ratio 3/10)                 │
│  ╭─ ³spend ───────────────────────────╮ │
│  │ Gauge 65%   £32.31 / £50.00       │ │
│  │ Balance: -£0.03    Resets 1st     │ │
│  ╰────────────────────────────────────╯ │
├──────────────────────────────────────────┤
│  Status Bar (ratio 1/10, min 1 line)     │
│  Max 5x │ ● online │ 5s │ q quit +/-   │
└──────────────────────────────────────────┘
```

### btop Style Elements

- `Border::Rounded` (╭╮╰╯)
- Numbered titles: `¹session`, `²weekly`, `³spend` with `Style::bold()`
- Gauge color thresholds: ≤50% green, 50-80% yellow, >80% red
- `Sparkline` widget in session panel, in-memory rolling buffer (lost on restart). Buffer stores one sample per poll. At default 5s interval, 60 samples = 5 min of history.
- Status bar: plan name (derived from OAuth credentials, see Auth section) + connection dot + refresh interval + keybinds
- Currency symbol rendered dynamically from API `currency` field (not hardcoded)

### Interaction

| Key | Action |
|-----|--------|
| `q` / `Esc` | Quit |
| `r` | Manual refresh |
| `+` / `=` | Increase refresh interval |
| `-` | Decrease refresh interval |

Refresh interval range: 1s – 60s, default 5s. Step size: 1s.

### Terminal Lifecycle

App enters alternate screen and raw mode on start. Cleanup (restore terminal) is guaranteed via `scopeguard` or `Drop` impl, including on panic (via `panic::set_hook`).

## Error Handling

| Scenario | Behavior |
|----------|----------|
| OAuth token expired | Auto-refresh with refresh_token; on failure, prompt re-login |
| Session cookie invalid | Status bar shows `● offline` in red, keep retrying |
| Network disconnected | Retain last data, show `● disconnected`, auto-resume on recovery |
| API 429 rate limit | Exponential backoff, temporarily increase poll interval |
| Org ID fetch fails | Exit with error message about authentication |
| Terminal too small | Show "resize terminal" message (minimum 40x12) |

## Config

`~/.config/claude-usage-tui/config.toml`:

```toml
refresh_interval = 5
# session_key = "sk-ant-..."
```

## Open Source

- License: MIT
- GitHub repo: `yuhanwang/claude-usage-tui` (created via `gh repo create`)
