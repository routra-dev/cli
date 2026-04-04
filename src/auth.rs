use anyhow::Result;
use colored::Colorize;

use crate::config;

pub async fn login() -> Result<()> {
    // TODO Phase 4: browser OAuth flow via pkce + local callback server
    println!("{}", "routra login".bold());
    println!();
    println!("Enter your Routra API key (from https://app.routra.dev/keys):");
    print!("  API key: ");
    std::io::Write::flush(&mut std::io::stdout())?;

    let mut key = String::new();
    std::io::BufRead::read_line(&mut std::io::stdin().lock(), &mut key)?;
    let key = key.trim().to_string();

    if key.is_empty() {
        anyhow::bail!("API key cannot be empty");
    }

    let mut cfg = config::load().unwrap_or_default();
    cfg.api_key = Some(key);
    config::save(&cfg)?;

    println!("{} Credentials saved to ~/.routra/config.json", "OK".green().bold());
    Ok(())
}

pub fn logout() -> Result<()> {
    let mut cfg = config::load().unwrap_or_default();
    cfg.api_key = None;
    config::save(&cfg)?;
    println!("{} Logged out.", "OK".green().bold());
    Ok(())
}
