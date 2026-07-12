use anyhow::Result;
use colored::Colorize;

use super::CmdCtx;
use crate::{client::RoutraClient, config};

/// Verify the effective API key by hitting an authenticated endpoint.
/// Returns Ok(valid) or Err when no key is configured at all.
async fn verify_key(ctx: &CmdCtx) -> Result<bool> {
    let client = RoutraClient::new(&ctx.api_key, &ctx.base_url)?;
    Ok(client.get("/billing").await.is_ok())
}

pub async fn run(ctx: &CmdCtx) -> Result<()> {
    let cfg = config::load()?;
    let config_path = config::config_path()?;

    let has_key = ctx.api_key.is_some() || cfg.api_key.is_some();
    let verified = if has_key {
        Some(verify_key(ctx).await.unwrap_or(false))
    } else {
        None
    };

    if ctx.is_json() {
        let data = serde_json::json!({
            "config_path": config_path.to_string_lossy(),
            "base_url": ctx.base_url,
            "authenticated": has_key,
            "key_valid": verified,
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

    // Effective key = --api-key / ROUTRA_API_KEY override, else config file.
    match ctx.api_key.as_deref().or(cfg.api_key.as_deref()) {
        Some(key) => {
            let masked = if key.len() > 8 {
                format!("{}...", &key[..8])
            } else {
                "****".to_string()
            };
            let source = if ctx.api_key.is_some() { "env/flag" } else { "config" };
            let status = match verified {
                Some(true) => "valid".green(),
                Some(false) => "INVALID or unreachable".red(),
                None => "configured".normal(),
            };
            println!("  API Key:  {} ({}, from {})", status, masked, source);
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
