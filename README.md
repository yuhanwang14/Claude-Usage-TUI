# claude-usage-tui

A btop-style terminal UI for monitoring your Claude.ai usage limits in real-time.

> **Status:** Beta. Works on macOS and Linux.

## Features

- **Session usage** — 5-hour rolling window with gauge and reset countdown
- **Weekly limits** — All models, Sonnet, and Opus breakdowns
- **Extra spend** — Monthly spend tracking with balance
- **btop aesthetics** — Rounded borders, color-coded gauges, compact layout
- **Auto cookie extraction** — `--browser chrome` reads your session cookie automatically
- **Mouse support** — Click `- 30s +` in status bar to adjust refresh interval

## Install

### Homebrew (macOS)

```bash
brew tap yuhanwang14/tap
brew install claude-usage-tui
```

### Shell script (macOS / Linux)

```bash
curl -fsSL https://raw.githubusercontent.com/yuhanwang14/Claude-Usage-TUI/main/install.sh | sh
```

### From source

```bash
cargo install --git https://github.com/yuhanwang14/Claude-Usage-TUI
```

## Quick Start

### Recommended: Auto-extract from Chrome

```bash
# Extract cookie from Chrome and save it — one-time setup
claude-usage-tui --browser chrome --save

# After that, just run:
claude-usage-tui
```

### With Claude Code (OAuth, limited data)

If you have [Claude Code](https://claude.ai/download) installed:

```bash
claude-usage-tui
```

> OAuth only shows session % and weekly %. Use `--browser chrome` for full data (spend, balance).

### Manual cookie

1. Open https://claude.ai → F12 → **Application** → **Cookies** → copy `sessionKey`

```bash
claude-usage-tui --cookie "sk-ant-sid02-..." --save
```

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
| **Cookie** (`--browser chrome` or `--cookie`) | `claude.ai/api/organizations/{id}/usage` | Session %, weekly %, per-model, spend, balance |
| **OAuth** (Claude Code) | `api.anthropic.com/v1/messages` headers | Session % and weekly % only |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Security

See [SECURITY.md](SECURITY.md).

## License

MIT — see [LICENSE](LICENSE).
