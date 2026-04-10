use anyhow::Result;
use colored::Colorize;

use crate::config;
use super::CmdCtx;

pub async fn run(ctx: &CmdCtx) -> Result<()> {
    let cfg = config::load()?;
    let config_path = config::config_path()?;

    if ctx.is_json() {
        let data = serde_json::json!({
            "config_path": config_path.to_string_lossy(),
            "base_url": ctx.base_url,
            "authenticated": cfg.api_key.is_some(),
            "key_prefix": cfg.api_key.as_deref().map(|k| {
                if k.len() > 8 { format!("{}...", &k[..8]) } else { "****".to_string() }
            }),
        });
        println!("{}", serde_json::to_string_pretty(&data)?);
        return Ok(());
    }

    println!("{}", "Routra CLI".bold());
    println!("  Config:   {}", config_path.display());
    println!("  Base URL: {}", ctx.base_url.as_deref().unwrap_or("https://api.routra.dev/v1"));

    match &cfg.api_key {
        Some(key) => {
            let masked = if key.len() > 8 {
                format!("{}...", &key[..8])
            } else {
                "****".to_string()
            };
            println!("  API Key:  {} ({})", "configured".green(), masked);
        }
        None => {
            println!(
                "  API Key:  {} (run `routra login`)",
                "not set".yellow()
            );
        }
    }

    Ok(())
}
