# claude-usage-tui

A btop-style terminal UI for monitoring your Claude.ai usage limits in real-time.

> **Status:** Alpha. Works on macOS. Linux support welcome via PRs.

## Features

- **Session usage** — 5-hour rolling window with gauge and reset countdown
- **Weekly limits** — All models, Sonnet, and Opus breakdowns
- **Extra spend** — Monthly spend tracking with balance (cookie auth)
- **btop aesthetics** — Rounded borders, color-coded gauges, compact layout
- **Dual auth** — Claude Code OAuth (auto) or session cookie (manual)
- **Mouse support** — Click `- 30s +` in status bar to adjust refresh interval

## Install

Build from source:

```bash
git clone https://github.com/yuhanwang14/claude-usage-tui
cd claude-usage-tui
cargo build --release
```

## Authentication

**Option 1: Claude Code (automatic)**

If you have [Claude Code](https://claude.ai/download) installed and logged in, credentials are read from macOS Keychain automatically:

```bash
./target/release/claude-usage-tui
```

If your token is expired, re-login:

```bash
./target/release/claude-usage-tui login
```

> Note: OAuth auth shows session % and weekly % only. For full data (spend, balance), use cookie auth.

**Option 2: Session cookie (full data)**

1. Open https://claude.ai, press F12 (DevTools)
2. Go to **Application** > **Cookies** > `https://claude.ai`
3. Copy the `sessionKey` value

```bash
./target/release/claude-usage-tui --cookie "sk-ant-sid02-..."
```

## Keybindings

| Key | Action |
|-----|--------|
| `q` / `Esc` | Quit |
| `r` | Manual refresh |
| `+` / `-` | Adjust refresh interval (keyboard) |
| Mouse click `- +` | Adjust refresh interval (mouse) |

## Config

`~/.config/claude-usage-tui/config.toml`:

```toml
refresh_interval = 30
# session_key = "sk-ant-..."
# org_id = "org-xxx"
```

## How It Works

- **Cookie auth path:** Calls `claude.ai/api/organizations/{id}/usage` + spend endpoints directly. Returns session %, weekly %, per-model breakdown, spend, and balance.
- **OAuth auth path:** Sends a minimal request to `api.anthropic.com/v1/messages` and reads `anthropic-ratelimit-unified-*` response headers. Returns session % and weekly % only.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Security

See [SECURITY.md](SECURITY.md).

## License

MIT — see [LICENSE](LICENSE).
