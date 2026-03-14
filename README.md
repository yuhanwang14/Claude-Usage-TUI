# claude-usage-tui

A btop-style terminal UI for monitoring your Claude.ai usage limits in real-time.

> **Status:** Alpha. Works on macOS and Linux.

## Features

- **Session usage** — 5-hour rolling window with gauge and reset countdown
- **Weekly limits** — All models, Sonnet, and Opus breakdowns
- **Extra spend** — Monthly spend tracking with balance (cookie auth)
- **btop aesthetics** — Rounded borders, color-coded gauges, compact layout
- **Dual auth** — Claude Code OAuth (auto) or session cookie (manual)
- **Mouse support** — Click `- 30s +` in status bar to adjust refresh interval

## Install

### Homebrew (macOS)

```bash
brew tap yuhanwang14/tap
brew install claude-usage-tui
```

### Shell script (macOS / Linux)

```bash
curl -fsSL https://raw.githubusercontent.com/yuhanwang14/claude-usage-tui/main/install.sh | sh
```

### From source

```bash
cargo install --git https://github.com/yuhanwang14/claude-usage-tui
```

## Usage

### With Claude Code (automatic)

If you have [Claude Code](https://claude.ai/download) installed and logged in, credentials are read from macOS Keychain automatically:

```bash
claude-usage-tui
```

If your token is expired, re-login:

```bash
claude-usage-tui login
```

> OAuth auth shows session % and weekly % only. For full data (spend, balance), use cookie auth.

### With Chrome (easiest, full data)

Auto-extracts session cookie from Chrome — no copy-paste needed:

```bash
claude-usage-tui --browser chrome
```

Save it so you never need to do it again:

```bash
claude-usage-tui --browser chrome --save
```

### With session cookie (manual)

1. Open https://claude.ai, press F12 (DevTools)
2. Go to **Application** > **Cookies** > `https://claude.ai`
3. Copy the `sessionKey` value

```bash
claude-usage-tui --cookie "sk-ant-sid02-..." --save
```

> `--save` persists the cookie to config. After that, just run `claude-usage-tui`.

## Keybindings

| Key | Action |
|-----|--------|
| `q` / `Esc` | Quit |
| `r` | Manual refresh |
| `+` / `-` | Adjust refresh interval |
| Mouse `- +` | Adjust refresh interval (click) |

## Config

`~/.config/claude-usage-tui/config.toml`:

```toml
refresh_interval = 30
# session_key = "sk-ant-..."
# org_id = "org-xxx"
```

## How It Works

| Auth method | Data source | What you get |
|-------------|------------|--------------|
| **Cookie** | `claude.ai/api/organizations/{id}/usage` | Session %, weekly %, per-model, spend, balance |
| **OAuth** | `api.anthropic.com/v1/messages` headers | Session % and weekly % only |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Security

See [SECURITY.md](SECURITY.md).

## License

MIT — see [LICENSE](LICENSE).
