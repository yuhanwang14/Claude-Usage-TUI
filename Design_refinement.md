# claude-usage-tui — btop Visual Alignment Design Report

## Context

Design spec for aligning `claude-usage-tui` (Rust + ratatui 0.29) with btop's visual language. The screenshot shows btop running above claude-usage-tui in the same terminal — the goal is to make them look like they belong to the same application family.

Reference: btop on macOS M2 Pro, alongside claude-usage-tui v0.2.1.

---

## 1. Layout Overhaul — Three-Column Single Row

### Current Problem

The current layout stacks panels vertically:
- Top 60%: session (40%) + weekly (60%) side-by-side
- Middle 30%: spend (full width)
- Bottom: status bar

This creates massive vertical waste. Session is 60% of screen height but only needs ~4 lines of content. Spend gets 30% but also only needs ~3 lines.

### Target Layout

Three panels side-by-side in a single row, mirroring btop's `²mem | disks | ⁴proc` horizontal density:

```
╭─ ¹session ────────────╮╭─ ²weekly ────────────╮╭─ ³spend ─────────────╮
│ 5-hour rolling window ││ All    ██████░░  23% ││ ████████████░░░  77% │
│ ██░░░░░░░░░░░░░░  1%  ││ Sonnet ░░░░░░░░   1% ││ £38.32 / £50.00     │
│ ▁▁▂▁▁▁▁▂▃▂▁▁▁▁▂▁▁▁▁  ││ Opus   ░░░░░░░░   0% ││ Balance: £11.68     │
│ Resets Today 17:00    ││ Resets Mar 19, 19:00  ││                      │
╰───────────────────────╯╰───────────────────────╯╰──────────────────────╯
 Cookie Auth │ ● online │ - 30s + │ q quit  r refresh
```

### Implementation (`src/ui/mod.rs`)

Replace the entire `draw` function:

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

**Key change**: From 2 vertical regions (top=session+weekly, middle=spend) to 1 horizontal row of 3 equal columns. All vertical space goes to the panels, so the sparkline in session has maximum room to breathe.

---

## 2. Sparkline in Session Panel

### What btop does

btop's CPU panel fills its main area with a braille-dot graph of CPU history. This is the single biggest visual element that makes btop look like btop.

### What to add

`App` already collects `sparkline_data: Vec<f64>` (up to 60 points of session usage history). This data is **never rendered**. Add a sparkline that fills the remaining vertical space in the session panel — in a 3-column layout, this could be 10+ rows of braille graph depending on terminal height.

### Implementation (`src/ui/session.rs`)

```rust
use ratatui::widgets::Sparkline;

let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Length(1), // "5-hour rolling window"
        Constraint::Length(1), // gauge bar
        Constraint::Min(1),   // sparkline — fills ALL remaining space
        Constraint::Length(1), // reset text (anchored to bottom)
    ])
    .split(inner);

// Render sparkline in chunks[2]
if !app.sparkline_data.is_empty() {
    let data: Vec<u64> = app.sparkline_data
        .iter()
        .map(|v| (*v as u64).clamp(0, 100))
        .collect();
    let sparkline = Sparkline::default()
        .data(&data)
        .max(100)
        .style(Style::default().fg(theme::GREEN));
    f.render_widget(sparkline, chunks[2]);
}
```

The sparkline uses braille characters by default in ratatui, matching btop's graph rendering. With the 3-column layout giving session its own full-height column, the sparkline area scales naturally with terminal height.

---

## 3. Gauge Bar Rendering

### Current Problem

Using `ratatui::widgets::Gauge` with default rendering. This produces a solid block fill with the percentage label **centered inside the bar**. btop's gauges are different: filled portion + percentage label **right-aligned after the bar**.

### Target Style

```
All    ██████░░░░░░░░░░░░░  23%
```

Label left, `█░` textured bar middle, colored percentage right — NOT overlaid on the bar.

### Implementation — Shared Helper (`src/ui/theme.rs`)

Add these reusable functions since all three panels need the same bar style:

```rust
use ratatui::text::{Span, Line};
use ratatui::style::{Style, Modifier};
use ratatui::layout::{Layout, Direction, Constraint, Rect, Alignment};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

/// Render a btop-style gauge bar as a Line of Spans.
pub fn gauge_bar(pct: f64, width: usize) -> Line<'static> {
    let filled = ((pct / 100.0) * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    let color = gauge_color(pct);

    Line::from(vec![
        Span::styled("█".repeat(filled), Style::default().fg(color)),
        Span::styled("░".repeat(empty), Style::default().fg(BAR_BG)),
    ])
}

/// Colored percentage span, right-aligned friendly.
pub fn pct_span(pct: f64) -> Span<'static> {
    Span::styled(
        format!("{:>3.0}%", pct),
        Style::default()
            .fg(gauge_color(pct))
            .add_modifier(Modifier::BOLD),
    )
}

/// Render a full gauge row: [label | bar | pct%]
/// `label_width` is the fixed width for the label column. Pass 0 for no label.
pub fn render_gauge_row(f: &mut Frame, area: Rect, label: &str, pct: f64, label_width: u16) {
    if label_width > 0 {
        let row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(label_width), // label
                Constraint::Min(1),             // bar
                Constraint::Length(5),           // " 23%"
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
                Constraint::Min(1),   // bar
                Constraint::Length(5), // " 23%"
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

### Panel-Specific Changes

**`src/ui/session.rs`**: Replace `Gauge::default()...` with:
```rust
theme::render_gauge_row(f, chunks[1], "", pct, 0);
```

**`src/ui/weekly.rs`**: Replace the entire local `render_gauge_row` with calls to the shared helper:
```rust
theme::render_gauge_row(f, chunks[0], "All    ", all_pct, 8);
theme::render_gauge_row(f, chunks[1], "Sonnet ", sonnet_pct, 8);
theme::render_gauge_row(f, chunks[2], "Opus   ", opus_pct, 8);
```

**`src/ui/spend.rs`**: No label:
```rust
theme::render_gauge_row(f, chunks[0], "", pct, 0);
```

---

## 4. Color & Theming

### Current Palette (mostly correct, minor tweaks)

```rust
// Keep these — already close to btop:
pub const GREEN: Color = Color::Rgb(78, 197, 108);   // ✓
pub const YELLOW: Color = Color::Rgb(232, 197, 71);   // ✓
pub const RED: Color = Color::Rgb(224, 108, 117);      // ✓
pub const BLUE: Color = Color::Rgb(126, 200, 227);     // ✓ (superscript numbers)
pub const SUBTEXT: Color = Color::Rgb(136, 136, 136);  // ✓

// Change these:
pub const DIM: Color = Color::Rgb(80, 80, 80);         // was (102,102,102) — dimmer borders
pub const BAR_BG: Color = Color::Rgb(51, 51, 51);      // ✓ but now used as fg on '░', not bg
```

### Key Change: Bar Background Technique

btop uses `░` (light shade U+2591) character with a dim foreground color — NOT a background color fill. The current code sets `.bg(BAR_BG)` on `Gauge`. The new manual bar rendering uses `Span::styled("░".repeat(empty), Style::default().fg(BAR_BG))` which gives the correct textured look.

### Percentage Values Must Be Colored

Currently the `Gauge` label is plain. In the new rendering, `pct_span()` colors the percentage text with `gauge_color(pct)` — green at low usage, yellow mid, red high. Matches btop where numeric values reflect their severity.

---

## 5. Spend Panel for 3-Column

### Layout Inside Panel

Spend now lives in a narrow column (~33% width) instead of full-width. Stack content vertically:

```rust
let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Length(1), // gauge bar
        Constraint::Length(1), // £38.32 / £50.00
        Constraint::Length(1), // Balance: £11.68
        Constraint::Min(0),   // absorb remaining space
    ])
    .split(inner);
```

Each text line is its own row — no horizontal splitting needed since the column is narrow. Keep spend amount in `TEXT` color, balance in `SUBTEXT`.

---

## 6. Weekly Panel for 3-Column

### Layout Inside Panel

```rust
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
```

Anchor the reset text to the bottom of the panel (same as session's reset text) so both panels have their reset timestamps at the same vertical position — visual alignment across columns.

---

## 7. Border & Title Details

### Title Padding

btop's panel titles have space padding inside the border line. Current code omits this. Add leading/trailing spaces to ALL three panels:

```rust
.title(Line::from(vec![
    Span::raw(" "),
    Span::styled("¹", Style::default().fg(BLUE)),
    Span::styled("session ", Style::default().fg(TEXT).add_modifier(Modifier::BOLD)),
]))
```

The space before `¹` and after the label name creates breathing room that matches btop's title rendering.

### Borders Already Correct

`BorderType::Rounded` + `DIM` fg color is right. No changes needed.

---

## 8. Status Bar Polish

### Minor Fixes

**1. Auth label styling**: Dim the auth method label slightly:
```rust
Span::styled(&app.plan_name, Style::default().fg(SUBTEXT)),  // was TEXT
```

**2. Consider right-aligned refresh countdown**: btop utilizes the full status bar width. Add something like `Next: 25s` right-aligned to fill the horizontal space and add utility.

---

## 9. Summary of File Changes

| File | Changes |
|------|---------|
| `src/ui/mod.rs` | Replace 2-row layout with single-row 3-column `Ratio(1,3)` + status bar |
| `src/ui/theme.rs` | Add `gauge_bar()`, `pct_span()`, `render_gauge_row()` helpers; tweak `DIM` color |
| `src/ui/session.rs` | Add sparkline (fills `Min(1)` area); replace `Gauge` with `render_gauge_row`; title padding |
| `src/ui/weekly.rs` | Delete local `render_gauge_row`; use `theme::render_gauge_row`; anchor reset text bottom; title padding |
| `src/ui/spend.rs` | Adapt to narrow column; replace `Gauge` with `render_gauge_row`; title padding |
| `src/ui/status_bar.rs` | Dim auth label; optionally add right-aligned refresh countdown |

---

## 10. Priority Order

1. **Layout → 3-column** (`mod.rs`) — the single biggest visual change
2. **Gauge helper + bar rendering** (`theme.rs` + all panels) — replaces flat `Gauge` with btop-textured `█░` bars
3. **Sparkline** (`session.rs`) — adds the signature btop graph, now has full column height
4. **Title padding** (all panels) — quick fix, immediately noticeable
5. **Bottom-anchor reset text** (`weekly.rs`, `session.rs`) — cross-column alignment
6. **Status bar polish** — minor refinements

---

## 11. Testing Checklist

- [ ] Three panels render side-by-side at 80+ column width
- [ ] Sparkline fills remaining vertical space in session panel with braille characters
- [ ] All gauge bars use `█` (filled) + `░` (empty), no solid background fills
- [ ] Percentage values are colored green/yellow/red matching the bar
- [ ] Panel titles have space padding: ` ¹session ` not `¹session`
- [ ] All borders are rounded (`╭╮╰╯`)
- [ ] Reset timestamps in session and weekly are bottom-anchored at the same row
- [ ] Status bar fits on a single line with no wrapping at 80 columns
- [ ] Minimum terminal size check still works (may need to adjust from 40x12)
- [ ] Narrow terminals (<60 cols) degrade gracefully — consider a fallback vertical layout
- [ ] `--cookie` and OAuth paths both render correctly (spend shows fallback for OAuth)
- [ ] Spend panel readable at ~26 char inner width (test with 80-col terminal)