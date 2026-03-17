# Contributing

Thanks for your interest in contributing to claude-usage-tui!

## Development Setup

### Prerequisites

- Rust 1.84+ (`rustup update`)
- macOS (primary target; Linux support welcome via PRs)

### Build & Run

```bash
git clone https://github.com/yuhanwang14/Claude-Usage-TUI
cd claude-usage-tui
cargo build
cargo run -- --cookie "your-session-key"
```

### Run Tests

```bash
cargo test
cargo clippy -- -W warnings
```

## How to Contribute

1. **Open an issue first** for features or non-trivial changes
2. Fork and create a branch (`git checkout -b fix/thing`)
3. Make your changes
4. Ensure `cargo build` and `cargo clippy` pass
5. Submit a PR with a clear description

## What We Care About

- Clean, readable Rust
- No unnecessary dependencies
- btop-style visual consistency in UI changes
- Backwards-compatible config changes

## Commit Style

```
feat: add new widget
fix: correct spend currency display
docs: update README
```

## Code of Conduct

See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
