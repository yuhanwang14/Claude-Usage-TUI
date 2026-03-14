mod api;
mod app;
mod auth;
mod browser;
mod config;
mod ui;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, Event, EventStream, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::StreamExt;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use tokio::time::{interval, Duration};

use api::ClaudeClient;
use app::App;
use auth::resolve_auth;
use config::Config;

/// Claude.ai usage TUI
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Session key for cookie-based auth
    #[arg(long)]
    cookie: Option<String>,

    /// Organization ID override
    #[arg(long)]
    org: Option<String>,

    /// Refresh interval in seconds (default: 5)
    #[arg(long, short)]
    interval: Option<u64>,

    /// Save cookie to config for future use
    #[arg(long)]
    save: bool,

    /// Extract session cookie from browser ("chrome")
    #[arg(long)]
    browser: Option<String>,
}

#[derive(clap::Subcommand, Debug)]
enum Command {
    /// Authenticate with Claude.ai via browser OAuth
    Login,
}

fn is_network_error(e: &anyhow::Error) -> bool {
    // Check if the underlying error is a reqwest error caused by a network issue
    if let Some(re) = e.downcast_ref::<reqwest::Error>() {
        return re.is_connect() || re.is_timeout() || re.is_request();
    }
    false
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, crossterm::event::EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) {
    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen, crossterm::event::DisableMouseCapture);
    let _ = terminal.show_cursor();
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle login subcommand
    if let Some(Command::Login) = cli.command {
        return auth::login::run_login();
    }

    // Load config
    let mut config = Config::load().unwrap_or_default();

    // CLI overrides
    if let Some(interval) = cli.interval {
        config.refresh_interval = interval;
    }
    if let Some(ref org) = cli.org {
        config.org_id = Some(org.clone());
    }

    // Extract cookie from browser if requested
    let browser_cookie = if let Some(ref browser_name) = cli.browser {
        match browser_name.as_str() {
            "chrome" => {
                eprintln!("Extracting session cookie from Chrome...");
                match browser::extract_chrome_cookie() {
                    Ok(cookie) => {
                        eprintln!("Cookie extracted successfully.");
                        Some(cookie)
                    }
                    Err(e) => {
                        return Err(e.context("Failed to extract cookie from Chrome"));
                    }
                }
            }
            other => {
                return Err(anyhow::anyhow!("Unsupported browser: {}. Supported: chrome", other));
            }
        }
    } else {
        None
    };

    // Use browser cookie if available, otherwise CLI cookie
    let effective_cookie = browser_cookie.as_deref().or(cli.cookie.as_deref());

    // Resolve auth
    let auth = resolve_auth(&config, effective_cookie)?;
    let plan_name = auth.plan_name();

    // Save cookie if --save was passed
    if cli.save {
        if let Some(key) = effective_cookie {
            Config::save_session_key(key)?;
        }
    }

    // Create API client (fetches org_id if not provided)
    let client = ClaudeClient::new(&auth, config.org_id.as_deref()).await?;

    // Build app state
    let mut app = App::new(config.refresh_interval, plan_name);

    // Set up terminal with panic hook to restore on panic
    let mut terminal = setup_terminal()?;
    let orig_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        orig_hook(info);
    }));

    // Initial fetch
    match client.fetch_all().await {
        Ok(data) => app.update_data(data),
        Err(e) => app.set_error(is_network_error(&e)),
    }

    // Event loop
    let mut event_stream = EventStream::new();
    let mut refresh_ticker = interval(Duration::from_secs(app.refresh_interval));
    refresh_ticker.tick().await; // consume the immediate first tick

    loop {
        // Check minimum terminal size
        let size = terminal.size()?;
        if size.width < 40 || size.height < 12 {
            // Too small — show a message and wait
            terminal.draw(|f| {
                let msg = ratatui::widgets::Paragraph::new("Terminal too small (min 40x12)")
                    .style(ratatui::style::Style::default().fg(ratatui::style::Color::Red));
                f.render_widget(msg, f.area());
            })?;
        } else {
            terminal.draw(|f| ui::draw(f, &app))?;
        }

        tokio::select! {
            // Keyboard / resize events
            maybe_event = event_stream.next() => {
                match maybe_event {
                    Some(Ok(Event::Key(key))) if key.kind == KeyEventKind::Press => {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                                app.running = false;
                            }
                            KeyCode::Char('r') | KeyCode::Char('R') => {
                                match client.fetch_all().await {
                                    Ok(data) => app.update_data(data),
                                    Err(e) => app.set_error(is_network_error(&e)),
                                }
                                // Reset the ticker so we don't double-refresh
                                refresh_ticker = interval(Duration::from_secs(app.refresh_interval));
                                refresh_ticker.tick().await;
                            }
                            KeyCode::Char('+') | KeyCode::Char('=') => {
                                app.increase_interval();
                                refresh_ticker = interval(Duration::from_secs(app.refresh_interval));
                                refresh_ticker.tick().await;
                            }
                            KeyCode::Char('-') => {
                                app.decrease_interval();
                                refresh_ticker = interval(Duration::from_secs(app.refresh_interval));
                                refresh_ticker.tick().await;
                            }
                            _ => {}
                        }
                    }
                    Some(Ok(Event::Mouse(mouse))) => {
                        if mouse.kind == crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left) {
                            // Check if click is on the status bar - / + buttons
                            let size = terminal.size()?;
                            let status_bar_area = ratatui::layout::Rect::new(0, size.height.saturating_sub(1), size.width, 1);
                            if let Some(increase) = ui::status_bar::check_interval_click(status_bar_area, mouse.column, mouse.row, &app) {
                                if increase {
                                    app.increase_interval();
                                } else {
                                    app.decrease_interval();
                                }
                                refresh_ticker = interval(Duration::from_secs(app.refresh_interval));
                                refresh_ticker.tick().await;
                            }
                        }
                    }
                    Some(Ok(Event::Resize(_, _))) => {
                        // Will redraw on next loop iteration
                    }
                    None => { app.running = false; }
                    _ => {}
                }
            }

            // Periodic refresh
            _ = refresh_ticker.tick() => {
                match client.fetch_all().await {
                    Ok(data) => app.update_data(data),
                    Err(e) => app.set_error(is_network_error(&e)),
                }
            }
        }

        if !app.running {
            break;
        }
    }

    restore_terminal(&mut terminal);
    Ok(())
}
