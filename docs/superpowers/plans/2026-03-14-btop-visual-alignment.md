# btop Visual Alignment — Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Align claude-usage-tui's visual language with btop — 3-column layout, textured `█░` gauge bars, braille sparkline, and consistent title/color polish.

**Architecture:** Replace the 2-row vertical layout (session+weekly / spend) with a single-row 3-column layout. Extract gauge bar rendering into shared theme helpers (`gauge_bar`, `pct_span`, `render_gauge_row`) used by all three panels. Add the sparkline widget to session using the already-collected `sparkline_data`.

**Tech Stack:** Rust, ratatui 0.29 (Sparkline, Paragraph, Block, Layout), crossterm 0.28

**Spec document:** `Design_refinement.md` (root of repo)

---

## File Structure

| File | Action | Responsibility |
|------|--------|----------------|
| `src/ui/theme.rs` | Modify | Add `gauge_bar()`, `pct_span()`, `render_gauge_row()` helpers; tweak `DIM` constant |
| `src/ui/mod.rs` | Modify | Replace 2-row layout with single-row 3-column + status bar |
| `src/ui/session.rs` | Modify | Add sparkline; replace `Gauge` with `render_gauge_row`; title padding |
| `src/ui/weekly.rs` | Modify | Delete local `render_gauge_row`; use `theme::render_gauge_row`; anchor reset bottom; title padding |
| `src/ui/spend.rs` | Modify | Adapt to narrow column; replace `Gauge` with `render_gauge_row`; title padding |
| `src/ui/status_bar.rs` | Modify | Dim auth label; add right-aligned refresh countdown |
| `src/main.rs` | Modify | Adjust minimum terminal size check from `40x12` to `60x8` |

---

## Chunk 1: Theme Helpers + Layout Overhaul

### Task 1: Add gauge bar helpers to theme.rs

**Files:**
- Modify: `src/ui/theme.rs`

- [ ] **Step 1: Add imports to theme.rs**

Open `src/ui/theme.rs`. Add these imports at the top, above the existing `use ratatui::style::Color;`:

```rust
use ratatui::text::{Span, Line};
use ratatui::style::{Style, Modifier};
use ratatui::layout::{Layout, Direction, Constraint, Rect, Alignment};
use ratatui::widgets::Paragraph;
use ratatui::Frame;
```

- [ ] **Step 2: Change DIM constant**

Replace `DIM` from `(102, 102, 102)` to `(80, 80, 80)` for dimmer borders matching btop:

```rust
pub const DIM: Color = Color::Rgb(80, 80, 80);
```

- [ ] **Step 3: Add `gauge_bar()` function**

After `gauge_color()`, add:

```rust
/// Render a btop-style gauge bar as a Line of Spans.
/// `width` is the number of characters available for the bar (not including label or pct).
pub fn gauge_bar(pct: f64, width: usize) -> Line<'static> {
    let filled = ((pct / 100.0) * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    let color = gauge_color(pct);

    Line::from(vec![
        Span::styled("█".repeat(filled), Style::default().fg(color)),
        Span::styled("░".repeat(empty), Style::default().fg(BAR_BG)),
    ])
}
```

- [ ] **Step 4: Add `pct_span()` function**

Immediately after `gauge_bar`:

```rust
/// Colored percentage span, right-aligned friendly.
pub fn pct_span(pct: f64) -> Span<'static> {
    Span::styled(
        format!("{:>3.0}%", pct),
        Style::default()
            .fg(gauge_color(pct))
            .add_modifier(Modifier::BOLD),
    )
}
```

- [ ] **Step 5: Add `render_gauge_row()` function**

Immediately after `pct_span`:

```rust
/// Render a full gauge row: [label | bar | pct%]
/// `label_width` is the fixed width for the label column. Pass 0 for no label.
pub fn render_gauge_row(f: &mut Frame, area: Rect, label: &str, pct: f64, label_width: u16) {
    if label_width > 0 {
        let row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(label_width),
                Constraint::Min(1),
                Constraint::Length(5),
            ])
            .split(area);

        let label_widget = Paragraph::new(Span::styled(
            label.to_string(),
            Style::default().fg(SUBTEXT),
        ));
        f.render_widget(label_widget, row[0]);

        let bar = gauge_bar(pct, row[1].width as usize);
        f.render_widget(Paragraph::new(bar), row[1]);

        let pct_widget = Paragraph::new(pct_span(pct))
            .alignment(Alignment::Right);
        f.render_widget(pct_widget, row[2]);
    } else {
        let row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(1),
                Constraint::Length(5),
            ])
            .split(area);

        let bar = gauge_bar(pct, row[0].width as usize);
        f.render_widget(Paragraph::new(bar), row[0]);

        let pct_widget = Paragraph::new(pct_span(pct))
            .alignment(Alignment::Right);
        f.render_widget(pct_widget, row[1]);
    }
}
```

- [ ] **Step 6: Verify compilation**

Run: `cargo check 2>&1 | head -20`
Expected: compiles with no errors (warnings about unused imports are fine at this stage — they'll be used in later tasks)

- [ ] **Step 7: Commit**

```bash
git add src/ui/theme.rs
git commit -m "feat(ui): add btop-style gauge_bar, pct_span, render_gauge_row helpers"
```

---

### Task 2: Replace layout in mod.rs — 3-column single row

**Files:**
- Modify: `src/ui/mod.rs`

- [ ] **Step 1: Replace the `draw` function body**

Replace the entire body of `pub fn draw(f: &mut Frame, app: &App)` in `src/ui/mod.rs` with:

```rust
pub fn draw(f: &mut Frame, app: &App) {
    let size = f.area();

    // Two rows: main content (fills all available space) + status bar (1 line)
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(6),    // three-column panel row
            Constraint::Length(1), // status bar
        ])
        .split(size);

    // Three columns: session 33% | weekly 34% | spend 33%
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ])
        .split(rows[0]);

    session::render(f, cols[0], app);
    weekly::render(f, cols[1], app);
    spend::render(f, cols[2], app);
    status_bar::render(f, rows[1], app);
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check 2>&1 | head -20`
Expected: compiles (panels still use old Gauge internally — that's fine, they'll render in the new layout)

- [ ] **Step 3: Commit**

```bash
git add src/ui/mod.rs
git commit -m "feat(ui): replace 2-row layout with 3-column single row"
```

---

## Chunk 2: Panel Rewrites — Session, Weekly, Spend

### Task 3: Rewrite session.rs — sparkline + new gauge + title padding

**Files:**
- Modify: `src/ui/session.rs`

- [ ] **Step 1: Update imports**

Replace the entire import block at the top of `src/ui/session.rs` with:

```rust
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Sparkline},
    Frame,
};

use crate::app::App;
use super::theme::{self, BLUE, DIM, GREEN, SUBTEXT, TEXT};
```

Key changes: removed `Gauge`, added `Sparkline`, swapped `BAR_BG` for `GREEN` (sparkline color).

- [ ] **Step 2: Replace the entire `render` function body**

Replace the full `pub fn render(...)` function in `src/ui/session.rs`:

```rust
pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(DIM))
        .title(Line::from(vec![
            Span::raw(" "),
            Span::styled("¹", Style::default().fg(BLUE)),
            Span::styled("session ", Style::default().fg(TEXT).add_modifier(Modifier::BOLD)),
        ]));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let pct = app.data.session_percent_used.unwrap_or(0.0);
    let reset_str = App::format_reset_time(app.data.session_reset_at.as_deref());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // "5-hour rolling window"
            Constraint::Length(1), // gauge bar
            Constraint::Min(1),   // sparkline — fills ALL remaining space
            Constraint::Length(1), // reset text (anchored to bottom)
        ])
        .split(inner);

    // Subtitle
    let subtitle = Paragraph::new(Span::styled(
        "5-hour rolling window",
        Style::default().fg(SUBTEXT),
    ));
    f.render_widget(subtitle, chunks[0]);

    // Gauge bar (btop-style)
    theme::render_gauge_row(f, chunks[1], "", pct, 0);

    // Sparkline (braille graph filling remaining vertical space)
    if !app.sparkline_data.is_empty() {
        let data: Vec<u64> = app.sparkline_data
            .iter()
            .map(|v| (*v as u64).clamp(0, 100))
            .collect();
        let sparkline = Sparkline::default()
            .data(&data)
            .max(100)
            .style(Style::default().fg(GREEN));
        f.render_widget(sparkline, chunks[2]);
    }

    // Reset timer (anchored to bottom)
    let reset_line = Paragraph::new(Span::styled(reset_str, Style::default().fg(SUBTEXT)));
    f.render_widget(reset_line, chunks[3]);
}
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check 2>&1 | head -20`
Expected: compiles with no errors

- [ ] **Step 4: Commit**

```bash
git add src/ui/session.rs
git commit -m "feat(ui): session panel — sparkline, btop gauge, title padding"
```

---

### Task 4: Rewrite weekly.rs — shared gauge + bottom-anchor reset

**Files:**
- Modify: `src/ui/weekly.rs`

- [ ] **Step 1: Update imports**

Replace the entire import block at the top of `src/ui/weekly.rs` with:

```rust
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use super::theme::{self, BLUE, DIM, SUBTEXT, TEXT};
```

Key change: removed `Gauge` and `BAR_BG`.

- [ ] **Step 2: Replace the entire `render` function and delete `render_gauge_row`**

Replace the full contents after the imports with just one function (delete the local `render_gauge_row`):

```rust
pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(DIM))
        .title(Line::from(vec![
            Span::raw(" "),
            Span::styled("²", Style::default().fg(BLUE)),
            Span::styled("weekly ", Style::default().fg(TEXT).add_modifier(Modifier::BOLD)),
        ]));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let reset_str = App::format_reset_time(app.data.weekly_reset_at.as_deref());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // All gauge row
            Constraint::Length(1), // Sonnet gauge row
            Constraint::Length(1), // Opus gauge row
            Constraint::Min(0),   // spacer
            Constraint::Length(1), // reset text (anchored bottom)
        ])
        .split(inner);

    theme::render_gauge_row(f, chunks[0], "All    ", app.data.weekly_percent_used.unwrap_or(0.0), 8);
    theme::render_gauge_row(f, chunks[1], "Sonnet ", app.data.weekly_sonnet_percent.unwrap_or(0.0), 8);
    theme::render_gauge_row(f, chunks[2], "Opus   ", app.data.weekly_opus_percent.unwrap_or(0.0), 8);

    let reset_line = Paragraph::new(Span::styled(reset_str, Style::default().fg(SUBTEXT)));
    f.render_widget(reset_line, chunks[4]);
}
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check 2>&1 | head -20`
Expected: compiles with no errors

- [ ] **Step 4: Commit**

```bash
git add src/ui/weekly.rs
git commit -m "feat(ui): weekly panel — shared gauge helpers, bottom-anchor reset"
```

---

### Task 5: Rewrite spend.rs — narrow column + new gauge

**Files:**
- Modify: `src/ui/spend.rs`

- [ ] **Step 1: Update imports**

Replace the entire import block at the top of `src/ui/spend.rs` with:

```rust
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use super::theme::{self, BLUE, DIM, SUBTEXT, TEXT, YELLOW};
```

Key change: removed `Gauge`, `BAR_BG`, `gauge_color`.

- [ ] **Step 2: Replace the entire `render` function**

```rust
pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(DIM))
        .title(Line::from(vec![
            Span::raw(" "),
            Span::styled("³", Style::default().fg(BLUE)),
            Span::styled("spend ", Style::default().fg(TEXT).add_modifier(Modifier::BOLD)),
        ]));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let spend_enabled = app.data.spend_limit_enabled == Some(true);
    let has_spend = spend_enabled && app.data.current_spend_dollars.is_some();

    if has_spend {
        let current = app.data.current_spend_dollars.unwrap_or(0.0);
        let limit = app.data.spend_limit_dollars.unwrap_or(0.0);
        let pct = if limit > 0.0 { (current / limit * 100.0).clamp(0.0, 100.0) } else { 0.0 };

        let sym = match app.data.spend_currency.as_deref() {
            Some("GBP") => "£",
            Some("EUR") => "€",
            Some("JPY") | Some("CNY") => "¥",
            _ => "$",
        };

        let balance = limit - current;

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // gauge bar
                Constraint::Length(1), // spend text
                Constraint::Length(1), // balance text
                Constraint::Min(0),   // absorb remaining space
            ])
            .split(inner);

        // Gauge bar (btop-style)
        theme::render_gauge_row(f, chunks[0], "", pct, 0);

        let spend_line = Paragraph::new(Span::styled(
            format!("{sym}{:.2} / {sym}{:.2}", current, limit),
            Style::default().fg(TEXT),
        ));
        f.render_widget(spend_line, chunks[1]);

        let balance_line = Paragraph::new(Span::styled(
            format!("Balance: {sym}{:.2}", balance),
            Style::default().fg(SUBTEXT),
        ));
        f.render_widget(balance_line, chunks[2]);
    } else {
        let msg = Paragraph::new(Span::styled(
            "Spend data requires --cookie auth",
            Style::default().fg(YELLOW),
        ));
        f.render_widget(msg, inner);
    }
}
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check 2>&1 | head -20`
Expected: compiles with no errors

- [ ] **Step 4: Commit**

```bash
git add src/ui/spend.rs
git commit -m "feat(ui): spend panel — narrow column layout, btop gauge"
```

---

## Chunk 3: Status Bar Polish + Terminal Size

### Task 6: Polish status_bar.rs — dim auth label + right-aligned countdown

**Files:**
- Modify: `src/ui/status_bar.rs`

- [ ] **Step 1: Update imports**

Replace the import block at the top of `src/ui/status_bar.rs` with:

```rust
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::{App, ConnectionStatus};
use super::theme::{DIM, GREEN, RED, SUBTEXT, TEXT, YELLOW};
```

Added `Alignment`, `Constraint`, `Direction`, `Layout` for the two-column split.

- [ ] **Step 2: Replace the `render` function (keep `check_interval_click` untouched)**

Replace only the `pub fn render(...)` function:

```rust
pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let (dot, dot_color) = match app.connection {
        ConnectionStatus::Online => ("●", GREEN),
        ConnectionStatus::Offline => ("●", RED),
        ConnectionStatus::Disconnected => ("●", YELLOW),
    };

    let connection_label = match app.connection {
        ConnectionStatus::Online => "online",
        ConnectionStatus::Offline => "offline",
        ConnectionStatus::Disconnected => "connecting…",
    };

    let sep = Span::styled(" │ ", Style::default().fg(DIM));

    // Left side: auth + connection + interval controls + key hints
    let left = Line::from(vec![
        Span::styled(" ", Style::default()),
        Span::styled(&app.plan_name, Style::default().fg(SUBTEXT)), // dimmed (was TEXT)
        sep.clone(),
        Span::styled(dot, Style::default().fg(dot_color)),
        Span::styled(format!(" {}", connection_label), Style::default().fg(SUBTEXT)),
        sep.clone(),
        Span::styled("- ", Style::default().fg(RED)),
        Span::styled(format!("{}s", app.refresh_interval), Style::default().fg(TEXT)),
        Span::styled(" +", Style::default().fg(GREEN)),
        sep.clone(),
        Span::styled("q", Style::default().fg(TEXT)),
        Span::styled(" quit", Style::default().fg(DIM)),
        Span::styled("  ", Style::default()),
        Span::styled("r", Style::default().fg(TEXT)),
        Span::styled(" refresh", Style::default().fg(DIM)),
    ]);

    // Right side: next refresh countdown
    let right = Line::from(vec![
        Span::styled(format!("Next: {}s ", app.refresh_interval), Style::default().fg(DIM)),
    ]);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(1),    // left content
            Constraint::Length(right.width() as u16), // right content
        ])
        .split(area);

    f.render_widget(Paragraph::new(left), cols[0]);
    f.render_widget(Paragraph::new(right).alignment(Alignment::Right), cols[1]);
}
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check 2>&1 | head -20`
Expected: compiles with no errors

- [ ] **Step 4: Commit**

```bash
git add src/ui/status_bar.rs
git commit -m "feat(ui): status bar — dimmed auth label, right-aligned countdown"
```

---

### Task 7: Adjust minimum terminal size in main.rs

**Files:**
- Modify: `src/main.rs:166-167`

- [ ] **Step 1: Update minimum size check**

In `src/main.rs`, find the terminal size check (line ~167):

```rust
        if size.width < 40 || size.height < 12 {
```

Replace with:

```rust
        if size.width < 60 || size.height < 8 {
```

The 3-column layout needs wider terminals (3 panels × ~20 chars each) but less height (single row instead of stacked).

- [ ] **Step 2: Update the error message**

On the next line, change the message:

```rust
                let msg = ratatui::widgets::Paragraph::new("Terminal too small (min 60x8)")
```

- [ ] **Step 3: Verify full build**

Run: `cargo build 2>&1 | tail -5`
Expected: `Finished` with no errors

- [ ] **Step 4: Commit**

```bash
git add src/main.rs
git commit -m "fix: adjust minimum terminal size to 60x8 for 3-column layout"
```

---

## Chunk 4: Final Verification

### Task 8: Full build + manual smoke test

- [ ] **Step 1: Clean build**

Run: `cargo build --release 2>&1 | tail -5`
Expected: `Finished` with no errors and no warnings in our files

- [ ] **Step 2: Run clippy**

Run: `cargo clippy 2>&1 | tail -20`
Expected: no warnings in `src/ui/` files

- [ ] **Step 3: Manual smoke test checklist**

Launch the TUI and verify each item:

```bash
cargo run -- --cookie "$CLAUDE_COOKIE"
```

Verify:
- [ ] Three panels render side-by-side at 80+ column width
- [ ] Sparkline fills remaining vertical space in session panel (braille characters)
- [ ] All gauge bars use `█` (filled) + `░` (empty), no solid background fills
- [ ] Percentage values are colored green/yellow/red matching the bar
- [ ] Panel titles have space padding: ` ¹session ` not `¹session`
- [ ] All borders are rounded (`╭╮╰╯`)
- [ ] Reset timestamps in session and weekly are bottom-anchored
- [ ] Status bar fits on a single line with no wrapping at 80 columns
- [ ] Auth label is dimmed (SUBTEXT color, not TEXT)
- [ ] Right-aligned "Next: Ns" shows in status bar
- [ ] `--cookie` and OAuth paths both render correctly
- [ ] 60-column terminal shows all three panels (cramped but readable)
- [ ] Below 60 columns shows "Terminal too small" message

- [ ] **Step 4: Final commit (if any fixups needed)**

```bash
git add -A
git commit -m "fix: post-review visual polish"
```
