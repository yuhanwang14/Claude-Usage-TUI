use anyhow::{anyhow, Result};
use std::process::Command;

/// Run the login flow by delegating to Claude Code's own login.
/// This refreshes ~/.claude/.credentials.json which our app reads.
pub fn run_login() -> Result<()> {
    // Check if claude is installed
    let claude_path = which_claude();

    match claude_path {
        Some(path) => {
            println!("Launching Claude Code login...");
            println!("This will open your browser to authenticate.\n");

            let status = Command::new(&path)
                .arg("/login")
                .status()?;

            if status.success() {
                // Verify credentials exist after login
                let creds_path = dirs::home_dir()
                    .unwrap_or_default()
                    .join(".claude")
                    .join(".credentials.json");

                if creds_path.exists() {
                    println!("\nLogin successful! Credentials saved.");
                    println!("Run `claude-usage-tui` to start monitoring.");
                } else {
                    println!("\nLogin completed but credentials file not found.");
                    println!("You can also use `claude-usage-tui --cookie <sessionKey>`");
                }
            } else {
                return Err(anyhow!("Claude Code login failed. Try running `claude /login` manually."));
            }
        }
        None => {
            println!("Claude Code is not installed.");
            println!();
            println!("Options:");
            println!("  1. Install Claude Code: npm install -g @anthropic-ai/claude-code");
            println!("     Then run: claude-usage-tui login");
            println!();
            println!("  2. Use cookie auth directly:");
            println!("     - Open https://claude.ai in your browser");
            println!("     - Open DevTools (F12) → Application → Cookies");
            println!("     - Copy the `sessionKey` value");
            println!("     - Run: claude-usage-tui --cookie \"sk-ant-...\"");
        }
    }

    Ok(())
}

fn which_claude() -> Option<String> {
    // Try common locations
    for name in &["claude"] {
        if let Ok(output) = Command::new("which").arg(name).output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return Some(path);
                }
            }
        }
    }
    None
}
