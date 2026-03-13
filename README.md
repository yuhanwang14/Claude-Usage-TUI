# claude-usage-tui

A btop-style terminal UI for monitoring your Claude.ai usage limits in real-time.

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
