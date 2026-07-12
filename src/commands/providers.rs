use anyhow::Result;

use crate::client::RoutraClient;
use super::CmdCtx;

pub async fn run(ctx: &CmdCtx) -> Result<()> {
    let client = RoutraClient::new(&ctx.api_key, &ctx.base_url)?;

    let resp = client.get("/providers").await?;
    let data: serde_json::Value = resp.json().await?;

    if ctx.is_json() {
        println!("{}", serde_json::to_string_pretty(&data)?);
        return Ok(());
    }

    // Server contract: ProvidersResponse { providers: [ProviderInfo] } with
    // display_name, health ("healthy"|"degraded"|"down"|"unknown"), etc.
    let items = data["providers"]
        .as_array()
        .map(|a| a.as_slice())
        .unwrap_or(&[]);
    if items.is_empty() {
        println!("No providers found.");
        return Ok(());
    }

    println!(
        "{:<20}  {:<24}  {:<10}  {:<8}  {:>8}  KEY",
        "SLUG", "NAME", "HEALTH", "ACTIVE", "MODELS"
    );
    for p in items {
        let slug = p["slug"].as_str().unwrap_or("-");
        let name = p["display_name"].as_str().unwrap_or("-");
        let health = p["health"].as_str().unwrap_or("unknown");
        let active = if p["is_active"].as_bool().unwrap_or(false) {
            "yes"
        } else {
            "no"
        };
        let models = p["models_served"].as_i64().unwrap_or(0);
        let has_key = if p["has_api_key"].as_bool().unwrap_or(false) {
            "yes"
        } else {
            "no"
        };
        println!(
            "{:<20}  {:<24}  {:<10}  {:<8}  {:>8}  {}",
            slug, name, health, active, models, has_key
        );
    }

    Ok(())
}
